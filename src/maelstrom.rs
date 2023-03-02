use std::fmt;
use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

//type NodeId = String;

#[derive(Error, Debug)]
pub enum NodeIdParseError {
    #[error("Invalid node type `{0}`")]
    InvalidNodeType(String),
    #[error("Invalid id number `{0}`")]
    InvalidIdNumber(#[from] std::num::ParseIntError),
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub enum NodeId {
    Server(i64),
    Client(i64)
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeId::Server(id) => write!(f, "n{}", id),
            NodeId::Client(id) => write!(f, "c{}", id)
        }
    }
}

impl From<NodeId> for String {
    fn from(value: NodeId) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for NodeId {
    type Error = NodeIdParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        NodeId::from_str(&value)
    }
}

impl FromStr for NodeId {
    type Err = NodeIdParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s[0..1] {
            "n" => Ok(NodeId::Server(i64::from_str_radix(&s[1..], 10)?)),
            "c" => Ok(NodeId::Client(i64::from_str_radix(&s[1..], 10)?)),
            other => Err(NodeIdParseError::InvalidNodeType(other.to_string()))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EchoBody {
    pub msg_id: i64,
    pub echo: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EchoOkBody {
    pub msg_id: i64,
    pub in_reply_to: i64,
    pub echo: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum MessageBody {
    Init(InitBody),
    InitOk(InitOkBody),
    Echo(EchoBody),
    EchoOk(EchoOkBody),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InitBody {
    pub msg_id: i64,
    pub node_id: NodeId,
    pub node_ids: Vec<NodeId>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InitOkBody {
    pub in_reply_to: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message<Body=MessageBody>
    where Body: Clone + fmt::Debug + DeserializeOwned + Serialize
{
    pub src: NodeId,
    pub dest: NodeId,
    #[serde(bound(deserialize = "Body: DeserializeOwned"))]
    pub body: Body,
}

impl TryFrom<MessageBody> for InitBody {
    type Error = anyhow::Error;

    fn try_from(value: MessageBody) -> Result<Self, Self::Error> {
        match value {
            MessageBody::Init(body) => Ok(body),
            _ => Err(anyhow::anyhow!("Message not of type init"))
        }
    }
}

impl <B> Message<B>
    where
        B: Clone + fmt::Debug + DeserializeOwned + Serialize
{
    pub fn try_into_message<T>(self) -> Result<Message<T>, T::Error>
        where T: TryFrom<B> + Clone + fmt::Debug + DeserializeOwned + Serialize
    {
        Ok(Message {
            src: self.src,
            dest: self.dest,
            body: self.body.try_into()?
        })
    }
}


// impl <T> From<Message> for Message<T>
//     where T: From<MessageBody>
// {
//     fn from(m: Message) -> Self {
//         Message { src: m.src, dest: m.dest, body: m.body.into() }
//     }
// }