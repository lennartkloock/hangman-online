use crate::ChatMessage;
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{
    fmt::{Display, Formatter},
    num::ParseIntError,
    str::FromStr,
};
use thiserror::Error;

/// Two bytes that represent a game code
///
/// 4 characters encoded in hex
#[derive(Copy, Clone, Debug, DeserializeFromStr, Eq, Hash, SerializeDisplay, PartialEq)]
pub struct GameCode(u16);

impl GameCode {
    pub fn random() -> Self {
        Self(rand::thread_rng().gen())
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseGameCodeError {
    #[error("game code must be 4 characters long")]
    InvalidLength,
    #[error("invalid game code: {0}")]
    ParseIntError(#[from] ParseIntError),
}

impl FromStr for GameCode {
    type Err = ParseGameCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(ParseGameCodeError::InvalidLength);
        }
        u16::from_str_radix(s, 16)
            .map(Self)
            .map_err(ParseGameCodeError::from)
    }
}

impl Display for GameCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04X}", self.0)
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GameMode {
    #[default]
    Team,
    Competitive,
}

impl GameMode {
    pub fn all() -> Vec<Self> {
        vec![GameMode::Team, GameMode::Competitive]
    }
}

impl Display for GameMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mode = match self {
            GameMode::Team => "Team",
            GameMode::Competitive => "Competitive",
        };
        write!(f, "{}", mode)
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum GameLanguage {
    #[default]
    English,
    Spanish,
    French,
    German,
    Russian,
    Turkish,
}

impl GameLanguage {
    pub fn all() -> Vec<Self> {
        vec![
            Self::English,
            Self::Spanish,
            Self::French,
            Self::German,
            Self::Russian,
            Self::Turkish,
        ]
    }
}

impl Display for GameLanguage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lang = match self {
            GameLanguage::English => "English",
            GameLanguage::French => "Français",
            GameLanguage::Spanish => "Español",
            GameLanguage::German => "Deutsch",
            GameLanguage::Russian => "Русский",
            GameLanguage::Turkish => "Türkçe",
        };
        write!(f, "{}", lang)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Random,
    Easy,
    #[default]
    Medium,
    Hard,
    Insane,
}

impl Difficulty {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Random,
            Self::Easy,
            Self::Medium,
            Self::Hard,
            Self::Insane,
        ]
    }
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lang = match self {
            Difficulty::Random => "Random",
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
            Difficulty::Insane => "Insane",
        };
        write!(f, "{}", lang)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GameSettings {
    pub mode: GameMode,
    pub language: GameLanguage,
    pub difficulty: Difficulty,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GameState {
    Playing,
    RoundFinished,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Score {
    pub rank: u32,
    pub nickname: String,
    pub score: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum Game {
    InProgress {
        settings: GameSettings,
        state: GameState,
        players: Vec<String>,
        chat: Vec<ChatMessage>,
        tries_used: u32,
        word: String,
        countdown: Option<chrono::DateTime<Utc>>,
    },
    Results(Vec<Score>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_code() {
        let code = GameCode::from_str("1337").unwrap();
        assert_eq!(code.0, 0x1337);
        assert_eq!(format!("{}", code), "1337");

        assert_eq!(
            GameCode::from_str("error"),
            Err(ParseGameCodeError::InvalidLength)
        );
        assert!(GameCode::from_str("error").is_err());
    }
}
