//! TODO: This protocol is pretty shit when supporting multiple game modes

use crate::{Game, GameState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum ClientMessage {
    ChatMessage(String),
    NextRound,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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

impl Default for ChatColor {
    fn default() -> Self {
        ChatColor::Neutral
    }
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
