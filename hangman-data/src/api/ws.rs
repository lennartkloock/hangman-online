use crate::Game;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum ClientMessage {
    ChatMessage(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum ServerMessage {
    Init(Game),
    UpdatePlayers(Vec<String>),
    Guess {
        message: (String, String),
        word: String,
        tries_used: u32,
        solved: bool,
    },
}
