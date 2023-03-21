use hangman_data::ChatColor;
use std::fmt::{Display, Formatter};
use unicode_segmentation::UnicodeSegmentation;

pub struct Word {
    target: Vec<String>,
    current: Vec<Character>,
}

#[derive(Clone, PartialEq)]
pub enum GuessResult {
    Hit,
    Miss,
    Solved,
}

impl From<GuessResult> for ChatColor {
    fn from(value: GuessResult) -> Self {
        match value {
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
    pub fn new(target: String) -> Self {
        let target: Vec<String> = target.graphemes(true).map(|s| s.to_string()).collect();
        Self {
            current: vec![Character::Unknown; target.len()],
            target,
        }
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
