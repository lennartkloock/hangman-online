use hangman_data::{GameCode, GameSettings, UserToken};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::info;

pub type GameManagerState = Arc<Mutex<GameManager>>;

#[derive(Debug, Default)]
pub struct GameManager {
    games: HashMap<GameCode, Game>,
}

impl GameManager {
    pub fn add_game(&mut self, game: Game) {
        info!("new game: {}", game.code);
        self.games.insert(game.code, game);
    }

    pub fn get_game(&self, code: GameCode) -> Option<&Game> {
        self.games.get(&code)
    }
}

#[derive(Debug)]
pub struct Game {
    pub code: GameCode,
    pub settings: GameSettings,
    pub owner: UserToken,
}

impl Game {
    pub fn new(owner: UserToken, settings: GameSettings) -> Self {
        Self {
            code: GameCode::random(),
            settings,
            owner,
        }
    }
}
