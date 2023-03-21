use crate::config::HangmanConfig;
use hangman_data::{Difficulty, GameLanguage};
use rand::Rng;
use std::{collections::HashMap, path::PathBuf};
use thiserror::Error;
use tokio::{fs, io};
use tracing::{debug, info};

#[derive(Debug)]
pub struct WordGenerator {
    wordlists_dir: String,
    limits: HashMap<GameLanguage, usize>,
}

#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("this language was not preprocessed")]
    LanguageNotPreprocessed,
}

impl WordGenerator {
    pub async fn preprocess(
        config: &HangmanConfig,
        languages: &[GameLanguage],
    ) -> Result<Self, GeneratorError> {
        let mut limits = HashMap::new();

        debug!("preprocessing wordlists");
        for lang in languages {
            let file = fs::read_to_string(wordlist_path_for_language(&config.wordlists_dir, lang))
                .await
                .map_err(GeneratorError::Io)?;
            let n = file.lines().count();

            debug!("finished preprocessing for {lang}: {n}");
            limits.insert(lang.clone(), n);
        }

        debug!("preprocessing finished");
        Ok(Self { wordlists_dir: config.wordlists_dir.clone(), limits })
    }

    pub async fn generate(
        &self,
        lang: &GameLanguage,
        difficulty: &Difficulty,
    ) -> Result<String, GeneratorError> {
        let words = self
            .limits
            .get(lang)
            .ok_or(GeneratorError::LanguageNotPreprocessed)?;
        let range = match difficulty {
            Difficulty::Random => 0..*words,
            _ => {
                let diffs = vec![
                    Difficulty::Easy,
                    Difficulty::Medium,
                    Difficulty::Hard,
                    Difficulty::Insane,
                ];
                let n = diffs
                .iter()
                .enumerate()
                .find(|(_, d)| *d == difficulty)
                .unwrap()
                .0;
                let frac = words / diffs.len();
                (n * frac)..((n + 1) * frac)
            }
        };

        debug!("choosing random word in range {range:?} for {lang}, {difficulty}");

        let s = fs::read_to_string(wordlist_path_for_language(&self.wordlists_dir, lang))
            .await
            .map_err(GeneratorError::Io)?
            .lines()
            .nth(rand::thread_rng().gen_range(range))
            .expect("random number too high")
            .to_string();
        info!("generated random word for {lang}: {s}");
        Ok(s)
    }
}

fn wordlist_path_for_language(wordlists_dir: &str, lang: &GameLanguage) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(wordlists_dir);
    match lang {
        GameLanguage::English => path.push("eng-com_web-public_2018_1M-words.pre.txt"),
        GameLanguage::Spanish => path.push("spa_web_2016_1M-words.pre.txt"),
        GameLanguage::French => path.push("fra_mixed_2009_1M-words.pre.txt"),
        GameLanguage::German => path.push("deu-de_web_2021_1M-words.pre.txt"),
        GameLanguage::Russian => path.push("rus-ru_web-public_2019_1M-words.pre.txt"),
        GameLanguage::Turkish => path.push("tur-tr_web_2019_1M-words.pre.txt"),
    }
    path
}
