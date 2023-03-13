use config::Config;
use serde::Deserialize;
use std::net::IpAddr;

#[derive(Deserialize)]
pub struct HangmanConfig {
    pub address: IpAddr,
    pub port: u16,
    pub public_dir: String,
    pub wordlists_dir: String,
}

pub fn load_config() -> HangmanConfig {
    let config = Config::builder()
        .add_source(config::File::with_name("Server"))
        .add_source(config::Environment::with_prefix("HANGMAN"))
        .set_default("address", "0.0.0.0")
        .unwrap()
        .set_default("public_dir", "public")
        .unwrap()
        .set_default("wordlists_dir", "wordlists")
        .unwrap()
        .build()
        .expect("failed to read config");
    config
        .try_deserialize()
        .expect("failed to deserialize config")
}
