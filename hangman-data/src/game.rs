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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GameLanguage {
    English,
    German,
}

impl Display for GameLanguage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lang = match self {
            GameLanguage::English => "English",
            GameLanguage::German => "Deutsch",
        };
        write!(f, "{}", lang)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameSettings {
    pub language: GameLanguage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Game {
    pub settings: GameSettings,
    pub players: Vec<String>,
    pub chat: Vec<(String, String)>,
    pub tries_used: u32,
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
