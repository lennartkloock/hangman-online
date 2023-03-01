use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum ClientMessage {
    Ping
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum ServerMessage {
    Pong
}
