use std::io::{self, BufRead, Write};
use std::thread;

use serde::{Deserialize, Serialize};
use tracing::info;

pub mod message;

use message::NodeId;

use crate::maelstrom::message::{Message, InitBody, EchoBody};

#[derive(Debug)]
pub struct ClusterInfo {
    self_node_id: NodeId,
    all_node_ids: Vec<NodeId>,
}

pub struct MaelstromRx {
    receiver: tokio::sync::mpsc::Receiver<Message>,
}

impl MaelstromRx {
    fn start() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(16);

        thread::spawn(move || {
            let mut stdin = io::stdin().lock();
            let mut buffer = String::new();

            while stdin.read_line(&mut buffer).unwrap() > 0 {
                let msg: Message = serde_json::from_str(&buffer).unwrap();
                tx.blocking_send(msg).unwrap();
            }
        });

        MaelstromRx { receiver: rx }
    }

    pub async fn recv_message(&mut self) -> Result<Message, anyhow::Error> {
        Ok(self
            .receiver
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Sender disconnected for MaelstromRx"))?)
    }
}

#[derive(Debug, Clone)]
pub struct MaelstromTx {
    sender: tokio::sync::mpsc::Sender<Message>,
}

impl MaelstromTx {
    fn start() -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel(16);

        thread::spawn(move || {
            let mut stdout = io::stdout().lock();

            while let Some(msg) = rx.blocking_recv() {
                info!("Sending: {:?}", &msg);

                let res: Result<(), anyhow::Error> = (|| {
                    serde_json::to_writer(&mut stdout, &msg)?;
                    stdout.write_all(b"\n")?;
                    stdout.flush()?;
                    Ok(())
                })();
                res.unwrap();
            }
        });

        MaelstromTx { sender: tx }
    }

    pub async fn send_message(&self, value: Message) -> Result<(), anyhow::Error> {
        Ok(self.sender.send(value).await?)
    }
}

pub struct Maelstrom {
    pub cluster_info: ClusterInfo,
    pub tx: MaelstromTx
}


pub struct MaelstromServer {
    pub cluster_info: ClusterInfo,
    pub tx: MaelstromTx,
    pub rx: MaelstromRx,
}

impl MaelstromServer {
    pub async fn start() -> Result<MaelstromServer, anyhow::Error> {
        let tx = MaelstromTx::start();
        let mut rx = MaelstromRx::start();

        let first_msg = rx.recv_message().await?;

        let init_msg: Message<InitBody> = first_msg.try_into_message()?;

        let cluster_info = ClusterInfo {
            self_node_id: init_msg.body.node_id,
            all_node_ids: init_msg.body.node_ids,
        };

        info!(cluster_info = ?cluster_info, "Server initialized");

        Ok(MaelstromServer {
            cluster_info,
            tx,
            rx,
        })
    }
}

// type DispatchHandler = Box<dyn Fn()>

// struct Dispatcher {
//     handlers: Vec<DispatchHandler>
// }
