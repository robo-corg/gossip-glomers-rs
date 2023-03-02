use std::io::{self, BufRead, Write};
use std::thread;

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::maelstrom::{EchoOkBody, InitBody, InitOkBody, Message, MessageBody, NodeId};

mod maelstrom;

#[derive(Debug)]
struct ClusterInfo {
    self_node_id: NodeId,
    all_node_ids: Vec<NodeId>,
}

struct MaelstromRx {
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

    async fn recv_message(&mut self) -> Result<Message, anyhow::Error> {
        Ok(self
            .receiver
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Sender disconnected for MaelstromRx"))?)
    }
}

#[derive(Debug, Clone)]
struct MaelstromTx {
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

    async fn send_message(&self, value: Message) -> Result<(), anyhow::Error> {
        Ok(self.sender.send(value).await?)
    }
}

struct Maelstrom {
    cluster_info: ClusterInfo,
    tx: MaelstromTx,
    rx: MaelstromRx,
}

impl Maelstrom {
    async fn start() -> Result<Maelstrom, anyhow::Error> {
        let tx = MaelstromTx::start();
        let mut rx = MaelstromRx::start();

        let first_msg = rx.recv_message().await?;

        let init_msg: Message<InitBody> = first_msg.try_into_message()?;

        let cluster_info = ClusterInfo {
            self_node_id: init_msg.body.node_id,
            all_node_ids: init_msg.body.node_ids,
        };

        info!(cluster_info = ?cluster_info, "Server initialized");

        Ok(Maelstrom {
            cluster_info,
            tx,
            rx,
        })
    }
}

// enum MaelstromServer {
//     WaitingForInit,
//     Initialized(ClusterInfo),
// }

// impl MaelstromServer {
//     async fn handle_message(&mut self, request: Message) -> Result<Option<Message>, anyhow::Error> {
//         match request.body {
//             MessageBody::Init(init_body) => Ok(Some(Message {
//                 src: init_body.node_id,
//                 dest: request.src,
//                 body: MessageBody::InitOk(InitOkBody {
//                     in_reply_to: init_body.msg_id,
//                 }),
//             })),
//             MessageBody::Echo(echo_body) => Ok(Some(Message {
//                 src: request.dest,
//                 dest: request.src,
//                 body: MessageBody::EchoOk(EchoOkBody {
//                     msg_id: echo_body.msg_id,
//                     in_reply_to: echo_body.msg_id,
//                     echo: echo_body.echo,
//                 }),
//             })),
//             _other => unimplemented!(),
//         }
//     }
// }

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().with_writer(io::stderr).init();

    let maelstrom = Maelstrom::start().await?;

    // let stdin = io::stdin().lock();
    // let mut stdout = io::stdout().lock();

    // for maybe_line in stdin.lines() {
    //     let line = maybe_line?;

    //     info!("Received raw {:?}", &line);

    //     let msg: Message = serde_json::from_str(&line)?;

    //     info!("Received {:?}", &msg);

    //     if let Some(response) = handle_message(msg)? {
    //         info!("Sending: {:?}", &response);

    //         serde_json::to_writer(&mut stdout, &response)?;
    //         stdout.write_all(b"\n")?;
    //         stdout.flush()?;
    //     }
    // }

    Ok(())
    //println!("Hello, world!");
}
