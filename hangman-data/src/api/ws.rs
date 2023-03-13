use crate::{Game, GameState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum ClientMessage {
    ChatMessage(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ChatMessage {
    pub from: Option<String>,
    pub content: String,
    pub color: ChatColor,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatColor {
    Neutral,
    Green,
    Red,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum ServerMessage {
    Init(Game),
    UpdatePlayers(Vec<String>),
    UpdateGame { word: String, tries_used: u32 },
    ChatMessage(ChatMessage),
    UpdateGameState(GameState),
}
