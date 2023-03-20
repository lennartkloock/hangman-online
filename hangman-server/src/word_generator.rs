use hangman_data::{Difficulty, GameLanguage};
use rand::Rng;
use std::{collections::HashMap, path::PathBuf, str::FromStr};
use tokio::{fs, io};
use tracing::{debug, trace};
use thiserror::Error;
use crate::config::HangmanConfig;

pub struct WordGenerator {
    limits: HashMap<GameLanguage, HashMap<Difficulty, u32>>,
}

#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("failed to parse wordlist")]
    ParseError,
}

impl WordGenerator {
    pub async fn preprocess(config: &HangmanConfig, languages: &[GameLanguage]) -> Result<Self, GeneratorError> {
        let mut limits = HashMap::new();

        debug!("preprocessing wordlists");
        let diffs = Difficulty::all();
        for lang in languages {
            let file = fs::read_to_string(wordlist_path_for_language(config, lang))
                .await
                .map_err(GeneratorError::Io)?;
            let iter = file
                .lines()
                .map(|s| s.split_ascii_whitespace().map(|s| s.to_string()).collect::<Vec<String>>())
                .skip_while(|l| {
                    if let Some(id) = l.get(0).and_then(|i| u32::from_str(i).ok()) {
                        id <= 100
                    } else {
                        // TODO: Skips word if parse error occurs, but should return error
                        true
                    }
                })
                .filter(|l| {
                    if let Some(occurrences) = l.get(2).and_then(|i| u32::from_str(i).ok()) {
                        occurrences > 1
                    } else {
                        // TODO: Skips word if parse error occurs, but should return error
                        false
                    }
                });

            let total_occ: u32 = iter
                .clone()
                .filter_map(|l| l.get(2).and_then(|i| u32::from_str(i).ok()))
                .sum();
            trace!("[{lang}] calculated total occurrences: {total_occ}");

            let occ_per_diff = total_occ / diffs.len() as u32;
            trace!("[{lang}] calculated occurrences per difficulty: {occ_per_diff}");

            let mut limits_for_lang = HashMap::new();

            let mut difficulties = diffs.iter();
            let mut d = difficulties.next().expect("found no difficulties");
            let mut occ = 0;
            for (i, l) in iter.enumerate() {
                occ += l
                    .get(2)
                    .and_then(|i| u32::from_str(i).ok())
                    .ok_or(GeneratorError::ParseError)?;
                if occ > occ_per_diff {
                    trace!("[{lang}] limit for {d} is {i}: \"{}\"", l[1]);
                    limits_for_lang.insert(d.clone(), i as u32);
                    occ = 0;
                    if let Some(diff) = difficulties.next() {
                        d = diff;
                    } else {
                        trace!("[{lang}] finished");
                        break;
                    }
                }
            }

            limits.insert(lang.clone(), limits_for_lang);
        }

        debug!("preprocessing finished");
        Ok(Self { limits })
    }

    // pub async fn generate(
    //     lang: &GameLanguage,
    //     difficulty: &Difficulty,
    // ) -> Result<String, GeneratorError> {
    // }
}

fn wordlist_path_for_language(config: &HangmanConfig, lang: &GameLanguage) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(&config.wordlists_dir);
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

// async fn random_word_for_language(
//     lang: &GameLanguage,
//     limit: u32,
// ) -> Result<String, GeneratorError> {
//     let file = fs::read_to_string(wordlist_path_for_language(lang))
//         .await
//         .map_err(GeneratorError::Io)?;
//     let n = rand::thread_rng().gen_range(0..limit);
//     let mut line = file
//         .lines()
//         .skip_while(|s| {
//             if let Some(id) = s
//                 .split_ascii_whitespace()
//                 .next()
//                 .and_then(|i| u32::from_str(i).ok())
//             {
//                 id <= 100
//             } else {
//                 // TODO: Skips word if parse error occurs, but should return error
//                 true
//             }
//         })
//         .nth(n as usize)
//         .ok_or(GeneratorError::LimitTooHigh)?
//         .split_ascii_whitespace();
//     line.nth(1)
//         .map(|s| s.to_string())
//         .ok_or(GeneratorError::ParseError)
// }
