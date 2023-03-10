use hangman_data::GameLanguage;
use std::fmt::{Display, Formatter};
use unicode_segmentation::UnicodeSegmentation;

pub struct Word {
    target: Vec<String>,
    current: Vec<Character>,
}

pub enum GuessResult {
    Hit,
    Miss,
    Solved,
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
    pub fn generate(language: &GameLanguage) -> Self {
        Self {
            target: "Banane".graphemes(true).map(|s| s.to_string()).collect(),
            current: vec![Character::Unknown; 6],
        }
    }

    pub fn word(&self) -> String {
        self.current
            .iter()
            .fold(String::new(), |a, b| format!("{a}{b}"))
    }

    pub fn guess(&mut self, s: String) -> GuessResult {
        let graphemes: Vec<String> = s.graphemes(true).map(|s| s.to_string()).collect();
        if graphemes == self.target {
            self.current = graphemes
                .into_iter()
                .map(|s| Character::Guessed(s))
                .collect();
            GuessResult::Solved
        } else if graphemes.len() == 1 {
            if let Some(g) = graphemes.get(0) {
                let mut found = false;
                for (i, _) in self
                    .target
                    .iter()
                    .enumerate()
                    .filter(|t| t.1.to_lowercase() == g.to_lowercase())
                {
                    self.current[i] = Character::Guessed(self.target[i].clone());
                    found = true;
                }
                if !found {
                    GuessResult::Miss
                } else if self.current.iter().all(|c| {
                    if let Character::Guessed(_) = c {
                        true
                    } else {
                        false
                    }
                }) {
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
