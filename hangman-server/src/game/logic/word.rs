use hangman_data::{ChatColor, GameLanguage};
use rand::Rng;
use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
    str::FromStr,
};
use thiserror::Error;
use tokio::{fs, io};
use tracing::info;
use unicode_segmentation::UnicodeSegmentation;

pub struct Word {
    target: Vec<String>,
    current: Vec<Character>,
}

#[derive(PartialEq)]
pub enum GuessResult {
    Hit,
    Miss,
    Solved,
}

impl Into<ChatColor> for GuessResult {
    fn into(self) -> ChatColor {
        match self {
            GuessResult::Hit => ChatColor::Green,
            GuessResult::Miss => ChatColor::Red,
            GuessResult::Solved => ChatColor::Green,
        }
    }
}

#[derive(Clone)]
enum Character {
    Unknown,
    Guessed(String),
}

impl Display for Character {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Character::Unknown => write!(f, "_"),
            Character::Guessed(s) => write!(f, "{}", s),
        }
    }
}

impl Word {
    pub async fn generate(language: &GameLanguage, limit: u32) -> Result<Self, RandomWordError> {
        let random = random_word_for_language(language, limit).await?;
        info!("generated random word for {language}: {random}");
        let target: Vec<String> = random.graphemes(true).map(|s| s.to_string()).collect();
        Ok(Self {
            current: vec![Character::Unknown; target.len()],
            target,
        })
    }

    pub fn target(&self) -> String {
        self.target.join("")
    }

    pub fn word(&self) -> String {
        self.current
            .iter()
            .fold(String::new(), |a, b| format!("{a}{b}"))
    }

    pub fn guess(&mut self, s: String) -> GuessResult {
        // TODO: Feels a bit messy
        let graphemes: Vec<String> = s.graphemes(true).map(|s| s.to_lowercase()).collect();
        if self
            .target
            .iter()
            .map(|s| s.to_lowercase())
            .collect::<Vec<String>>()
            == graphemes
        {
            self.current = self
                .target
                .iter()
                .map(|s| Character::Guessed(s.clone()))
                .collect();
            GuessResult::Solved
        } else if graphemes.len() == 1 {
            if let Some(g) = graphemes.get(0) {
                let mut found = false;
                for (i, _) in self
                    .target
                    .iter()
                    .enumerate()
                    .filter(|t| t.1.to_lowercase() == *g)
                {
                    self.current[i] = Character::Guessed(self.target[i].clone());
                    found = true;
                }
                if !found {
                    GuessResult::Miss
                } else if self
                    .current
                    .iter()
                    .all(|c| matches!(c, Character::Guessed(_)))
                {
                    GuessResult::Solved
                } else {
                    GuessResult::Hit
                }
            } else {
                GuessResult::Miss
            }
        } else {
            GuessResult::Miss
        }
    }
}

fn wordlist_path_for_language(lang: &GameLanguage) -> PathBuf {
    let mut path = PathBuf::new();
    path.push("wordlists");
    match lang {
        GameLanguage::English => path.push("eng-com_web-public_2018_1M-words.txt"),
        GameLanguage::Spanish => path.push("spa_web_2016_1M-words.txt"),
        GameLanguage::French => path.push("fra_mixed_2009_1M-words.txt"),
        GameLanguage::German => path.push("deu-de_web_2021_1M-words.txt"),
        GameLanguage::Russian => path.push("rus-ru_web-public_2019_1M-words.txt"),
        GameLanguage::Turkish => path.push("tur-tr_web_2019_1M-words.txt"),
    }
    path
}

#[derive(Debug, Error)]
pub enum RandomWordError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("limit is too high")]
    LimitTooHigh,
    #[error("failed to parse wordlist")]
    ParseError,
}

async fn random_word_for_language(
    lang: &GameLanguage,
    limit: u32,
) -> Result<String, RandomWordError> {
    let file = fs::read_to_string(wordlist_path_for_language(lang))
        .await
        .map_err(RandomWordError::Io)?;
    let n = rand::thread_rng().gen_range(0..limit);
    let mut line = file
        .lines()
        .skip_while(|s| {
            if let Some(id) = s
                .split_ascii_whitespace()
                .next()
                .and_then(|i| u32::from_str(i).ok())
            {
                id <= 100
            } else {
                // TODO: Skips word if parse error occurs, but should return error
                true
            }
        })
        .nth(n as usize)
        .ok_or(RandomWordError::LimitTooHigh)?
        .split_ascii_whitespace();
    line.nth(1)
        .map(|s| s.to_string())
        .ok_or(RandomWordError::ParseError)
}
