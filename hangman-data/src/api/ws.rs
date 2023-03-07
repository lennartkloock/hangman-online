use serde::{Deserialize, Serialize};
use crate::Game;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum ClientMessage {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum ServerMessage {
    Init(Game),
    UpdatePlayers(Vec<String>),
    NewMessage((String, String)),
    UpdateTriesUsed(u32),
}
