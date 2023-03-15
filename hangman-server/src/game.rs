use std::{collections::HashMap, future::Future, sync::Arc};

use async_trait::async_trait;
use futures::FutureExt;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info, log::warn};

use hangman_data::{
    ChatMessage, ClientMessage, Game, GameCode, GameSettings, GameState, ServerMessage, User,
    UserToken,
};
pub use logic::GameMessage;

use crate::sender_utils::{LogSend, SendToAll};

pub mod logic;

pub type GameManagerState = Arc<Mutex<GameManager>>;

#[derive(Debug, Default)]
pub struct GameManager {
    games: HashMap<GameCode, mpsc::Sender<GameMessage>>,
}

impl GameManager {
    pub fn add_game<L: GameLogic>(&mut self, mut game: ServerGame<L>) {
        info!("new game: {}", game.code);
        let code = game.code;
        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(game.game_loop(rx).then(|_| async {
            debug!("[{code}] game loop finished, removing game");
            self.games.remove(&code);
        }));
        self.games.insert(code, tx);
    }

    pub fn get_game(&self, code: GameCode) -> Option<mpsc::Sender<GameMessage>> {
        self.games.get(&code).map(mpsc::Sender::clone)
    }
}

#[async_trait]
pub trait GameLogic {
    async fn new(settings: &GameSettings) -> Self;
    async fn handle_message(
        &mut self,
        code: GameCode,
        user: (&User, mpsc::Sender<ServerMessage>),
        msg: ClientMessage,
    );
    // async fn handle_message(&mut self, game: &ServerGame<Self>, user: (&User, mpsc::Sender<ServerMessage>), msg: ClientMessage);
    async fn on_user_join(
        &mut self,
        user: (&User, mpsc::Sender<ServerMessage>),
        init_game: &mut Game,
    );
    async fn on_user_leave(&mut self, user: (&User, mpsc::Sender<ServerMessage>));
}

#[derive(Debug)]
pub struct ServerGame<L: GameLogic> {
    pub code: GameCode,
    pub settings: GameSettings,
    pub owner: UserToken,
    pub players: HashMap<UserToken, (User, mpsc::Sender<ServerMessage>)>,
    pub logic: L,
}

impl<L: GameLogic> ServerGame<L> {
    pub async fn new(owner: UserToken, settings: GameSettings) -> Self {
        Self {
            code: GameCode::random(),
            owner,
            players: HashMap::new(),
            logic: L::new(&settings).await,
            settings,
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
                        .insert(user.token, (user.clone(), sender.clone()));
                    // Send update to all clients
                    self.players
                        .iter()
                        .filter(|(t, _)| **t != user.token)
                        .map(|(_, (_, s))| s)
                        .send_to_all(ServerMessage::UpdatePlayers(self.player_names()))
                        .await;

                    // TODO: Needs improvement
                    let mut init_game = Game {
                        settings: self.settings.clone(),
                        players: self.player_names(),

                        state: GameState::Playing,
                        chat: vec![],
                        tries_used: 0,
                        word: String::new(),
                    };
                    self.logic
                        .on_user_join((&user, sender.clone()), &mut init_game)
                        .await;
                    sender.log_send(ServerMessage::Init(init_game)).await;
                }
                GameMessage::Leave(token) => {
                    if let Some((user, sender)) = self.players.remove(&token) {
                        info!("[{}] {user:?} left the game", self.code);
                        // Send update to all clients
                        self.player_txs()
                            .send_to_all(ServerMessage::UpdatePlayers(self.player_names()))
                            .await;

                        self.logic.on_user_leave((&user, sender)).await;

                        if self.players.is_empty() {
                            break;
                        } else {
                            if token == self.owner {
                                info!("[{}] the game owner left the game, closing", self.code);
                                break;
                            }
                        }
                    } else {
                        warn!(
                            "[{}] there was no user in this game with this token",
                            self.code
                        );
                    }
                }
                GameMessage::ClientMessage { message, token } => {
                    if let Some((user, sender)) = self.players.get(&token) {
                        self.logic
                            .handle_message(self.code, (user, sender.clone()), message)
                            .await;
                    }
                }
            }
        }
    }

    pub fn player_names(&self) -> Vec<String> {
        self.players
            .values()
            .map(|(u, _)| u.nickname.clone())
            .collect()
    }

    pub fn player_txs(&self) -> impl Iterator<Item = &mpsc::Sender<ServerMessage>> {
        self.players.iter().map(|(_, (_, s))| s)
    }
}
