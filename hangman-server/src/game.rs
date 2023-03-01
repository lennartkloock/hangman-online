use hangman_data::{GameCode, GameSettings, ServerMessage, UserToken};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tracing::info;

mod logic;

pub use logic::GameMessage;

pub type GameManagerState = Arc<Mutex<GameManager>>;

#[derive(Debug, Default)]
pub struct GameManager {
    games: HashMap<GameCode, mpsc::Sender<GameMessage>>,
    clients: HashMap<UserToken, mpsc::Sender<ServerMessage>>,
}

impl GameManager {
    pub fn add_game(&mut self, game: Game) {
        info!("new game: {}", game.code);
        let code = game.code;
        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(async move { logic::game_logic(game, rx).await });
        self.games.insert(code, tx);
    }

    pub fn add_client(&mut self, client: UserToken, tx: mpsc::Sender<ServerMessage>) {
        info!("new client with token: {}", client);
        self.clients.insert(client, tx);
    }

    pub fn get_game(&self, code: GameCode) -> Option<mpsc::Sender<GameMessage>> {
        self.games.get(&code).map(mpsc::Sender::clone)
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
