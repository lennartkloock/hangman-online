use std::fmt::Debug;
use crate::{Game, Score};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum ClientMessage {
    ChatMessage(String),
    NextRound,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ChatMessage {
    pub from: Option<String>,
    pub content: String,
    pub color: ChatColor,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
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
pub enum ServerMessage<State> {
    UpdateGame(Game<State>),
    Results(Vec<Score>),
}
