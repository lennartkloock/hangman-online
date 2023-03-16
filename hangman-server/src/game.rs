use crate::{game::logic::GameMessage, sender_utils::SendToAll};
use async_trait::async_trait;
use futures::FutureExt;
use hangman_data::{ClientMessage, GameCode, GameSettings, ServerMessage, User, UserToken};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, info, log::warn, trace};

pub mod logic;

#[derive(Clone, Debug)]
pub struct GameManager {
    games: Arc<Mutex<HashMap<GameCode, mpsc::Sender<GameMessage>>>>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            games: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_game<L: GameLogic + Send + 'static>(&self, game: ServerGame<L>) {
        info!("new game: {}", game.code);
        let code = game.code;
        let (tx, rx) = mpsc::channel(10);
        let games = Arc::clone(&self.games);
        tokio::spawn(game.game_loop(rx).then(move |_| async move {
            debug!("[{code}] game loop finished, removing game");
            games.lock().await.remove(&code);
        }));
        self.games.lock().await.insert(code, tx);
    }

    pub async fn get_game(&self, code: GameCode) -> Option<mpsc::Sender<GameMessage>> {
        self.games.lock().await.get(&code).map(mpsc::Sender::clone)
    }
}

#[async_trait]
pub trait GameLogic {
    async fn new(settings: GameSettings, players: Arc<RwLock<Players>>) -> Self;
    async fn handle_message(
        &mut self,
        code: GameCode,
        user: (&User, mpsc::Sender<ServerMessage>),
        msg: ClientMessage,
    );
    async fn on_user_join(&mut self, user: (&User, mpsc::Sender<ServerMessage>));
    async fn on_user_leave(&mut self, user: (&User, mpsc::Sender<ServerMessage>));
}

#[derive(Debug)]
pub struct Players(HashMap<UserToken, (User, mpsc::Sender<ServerMessage>)>);

impl Players {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn player_names(&self) -> Vec<String> {
        self.0.values().map(|(u, _)| u.nickname.clone()).collect()
    }

    pub fn player_txs(&self) -> impl Iterator<Item = &mpsc::Sender<ServerMessage>> {
        self.0.iter().map(|(_, (_, s))| s)
    }
}

impl Deref for Players {
    type Target = HashMap<UserToken, (User, mpsc::Sender<ServerMessage>)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Players {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct ServerGame<L: GameLogic> {
    pub code: GameCode,
    pub owner: UserToken,
    pub players: Arc<RwLock<Players>>,
    pub logic: L,
}

impl<L: GameLogic> ServerGame<L> {
    pub async fn new(owner: UserToken, settings: GameSettings) -> Self {
        let players = Arc::new(RwLock::new(Players::new()));
        Self {
            code: GameCode::random(),
            owner,
            logic: L::new(settings, Arc::clone(&players)).await,
            players,
        }
    }

    pub async fn game_loop(mut self, mut rx: mpsc::Receiver<GameMessage>) {
        while let Some(msg) = rx.recv().await {
            debug!("[{}] received {msg:?}", self.code);
            match msg {
                GameMessage::Join { user, sender } => {
                    info!("[{}] {user:?} joins the game", self.code);
                    let sender = sender.clone();
                    self.players
                        .write()
                        .await
                        .insert(user.token, (user.clone(), sender.clone()));

                    let player_names = self.players.read().await.player_names();

                    // Send update to all clients
                    self.players
                        .read()
                        .await
                        .iter()
                        .filter(|(t, _)| **t != user.token)
                        .map(|(_, (_, s))| s)
                        .send_to_all(ServerMessage::UpdatePlayers(player_names.clone()))
                        .await;

                    trace!("calling on_user_join");
                    self.logic.on_user_join((&user, sender.clone())).await;
                }
                GameMessage::Leave(token) => {
                    let Some((user, sender)) = self.players.write().await.remove(&token) else {
                        warn!(
                            "[{}] there was no user in this game with this token",
                            self.code
                        );
                        return;
                    };
                    info!("[{}] {user:?} left the game", self.code);
                    // Send update to all clients
                    self.players
                        .read()
                        .await
                        .player_txs()
                        .send_to_all(ServerMessage::UpdatePlayers(
                            self.players.read().await.player_names(),
                        ))
                        .await;

                    trace!("calling on_user_leave");
                    self.logic.on_user_leave((&user, sender)).await;

                    if self.players.read().await.is_empty() {
                        info!("[{}] all players left the game, closing", self.code);
                        break;
                    } else if token == self.owner {
                        info!("[{}] the game owner left the game, closing", self.code);
                        break;
                    }
                }
                GameMessage::ClientMessage { message, token } => {
                    if let Some((user, sender)) = self.players.read().await.get(&token) {
                        trace!("calling handle_message");
                        self.logic
                            .handle_message(self.code, (user, sender.clone()), message)
                            .await;
                    }
                }
            }
        }
    }
}
