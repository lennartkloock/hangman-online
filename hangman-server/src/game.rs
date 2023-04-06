use crate::game::logic::GameMessage;
use futures::FutureExt;
use hangman_data::{GameCode, GameMode, GameSettings, UserToken};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info};

pub mod logic;

#[derive(Clone, Debug)]
pub struct GameManager {
    games: Arc<Mutex<HashMap<GameCode, (GameMode, mpsc::Sender<GameMessage>)>>>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            games: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl GameManager {
    pub async fn add_game(&self, owner: UserToken, settings: GameSettings) -> GameCode {
        let code = GameCode::random();
        info!("new game: {}", code);
        let (tx, rx) = mpsc::channel(10);
        let games = Arc::clone(&self.games);
        tokio::spawn(
            async move {
                match &settings.mode {
                    GameMode::Team => logic::team::game_loop(rx, code, settings, owner).await,
                    GameMode::Competitive => {
                        logic::competitive::game_loop(rx, code, settings, owner).await
                    }
                }
                debug!("[{code}] game loop finished, removing game");
                games.lock().await.remove(&code);
            }
        );
        self.games.lock().await.insert(code, (settings.mode.clone(), tx));
        code
    }

    pub async fn get_game(&self, code: GameCode) -> Option<(GameMode, mpsc::Sender<GameMessage>)> {
        self.games.lock().await.get(&code).map(mpsc::Sender::clone)
    }
}
