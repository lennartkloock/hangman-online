use crate::{CompetitiveState, Game, Score, TeamState};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

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
pub enum ServerMessageInner<State> {
    UpdateGame(Game<State>),
    Results(Vec<Score>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum ServerMessage {
    Team(ServerMessageInner<TeamState>),
    Competitive(ServerMessageInner<CompetitiveState>),
}
