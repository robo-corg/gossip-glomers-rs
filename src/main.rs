use std::io::{self, BufRead, Write};
use std::thread;

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::maelstrom::MaelstromServer;

mod maelstrom;


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

    let maelstrom = MaelstromServer::start().await?;

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
