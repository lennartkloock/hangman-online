use std::collections::HashMap;
use hangman_data::{GameCode, GameSettings};
use std::sync::Arc;
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
}

#[derive(Debug)]
pub struct Game {
    pub code: GameCode,
    pub settings: GameSettings,
}

impl Game {
    pub fn new(settings: GameSettings) -> Self {
        Self {
            code: GameCode::random(),
            settings,
        }
    }
}
