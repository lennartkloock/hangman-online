use crate::{
    game::logic::{
        word::{GuessResult, Word},
        GameMessage,
    },
    sender_utils::{LogSend, SendToAll},
    GENERATOR,
};
use hangman_data::{
    ChatColor, ChatMessage, ClientMessage, Game, GameCode, GameSettings, GameState, ServerMessage,
    User, UserToken,
};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use tokio::sync::mpsc;
use tracing::{debug, info, log::warn};

#[derive(Debug)]
pub struct Players(HashMap<UserToken, (User, mpsc::Sender<ServerMessage>)>);

impl Players {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub async fn add_player(&mut self, user: User, tx: mpsc::Sender<ServerMessage>) {
        self.player_txs()
            .send_to_all(ServerMessage::UpdatePlayers(self.player_names()))
            .await;
        self.insert(user.token, (user, tx));
    }

    pub async fn remove_player(
        &mut self,
        token: &UserToken,
    ) -> Option<(User, mpsc::Sender<ServerMessage>)> {
        let res = self.remove(token);
        if res.is_some() {
            self.player_txs()
                .send_to_all(ServerMessage::UpdatePlayers(self.player_names()))
                .await;
        }
        res
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

pub async fn game_loop(
    mut rx: mpsc::Receiver<GameMessage>,
    code: GameCode,
    settings: GameSettings,
    owner: UserToken,
) {
    let mut players = Players::new();
    let mut chat = Vec::new();
    let mut state = GameState::Playing;
    let mut tries_used = 0;
    let mut word = generate_word(&settings).await;

    let send_message = |m: ChatMessage| async {
        chat.push(m.clone());
        players
            .player_txs()
            .send_to_all(ServerMessage::ChatMessage(m))
            .await;
    };

    while let Some(msg) = rx.recv().await {
        debug!("[{code}] received {msg:?}");
        match msg {
            GameMessage::Join { user, sender } => {
                info!("[{code}] {user:?} joins the game");
                let nickname = user.nickname.clone();
                players.add_player(user, sender.clone()).await;

                sender
                    .log_send(ServerMessage::Init(Game {
                        settings: settings.clone(),
                        state: state.clone(),
                        players: players.player_names(),
                        chat: chat.clone(),
                        tries_used,
                        word: word.word(),
                    }))
                    .await;

                send_message(ChatMessage {
                    content: format!("→ {} joined the game", nickname),
                    ..Default::default()
                });
                let m = ChatMessage {
                    content: format!("→ {} joined the game", nickname),
                    ..Default::default()
                };
                chat.push(m.clone());
                players
                    .player_txs()
                    .send_to_all(ServerMessage::ChatMessage(m))
                    .await;
            }
            GameMessage::Leave(token) => {
                let Some((user, _)) = players.remove_player(&token).await else {
                    warn!("[{code}] there was no user in this game with this token");
                    return;
                };
                info!("[{code}] {user:?} left the game");

                send_message(ChatMessage {
                    content: format!("← {} left the game", user.nickname),
                    ..Default::default()
                })
                .await;

                if players.is_empty() {
                    info!("[{code}] all players left the game, closing");
                    break;
                } else if token == owner {
                    info!("[{code}] the game owner left the game, closing");
                    break;
                }
            }
            GameMessage::ClientMessage { message, token } => {
                if let Some((user, _)) = players.get(&token) {
                    match message {
                        ClientMessage::ChatMessage(message) => {
                            let guess = word.guess(message.clone());
                            match guess {
                                GuessResult::Miss => {
                                    info!("[{code}] {} guessed wrong", user.nickname);
                                    tries_used += 1;
                                }
                                GuessResult::Hit => {
                                    info!("[{code}] {} guessed right", user.nickname);
                                }
                                GuessResult::Solved => {
                                    info!("[{code}] {} solved the word", user.nickname);
                                }
                            };
                            players
                                .player_txs()
                                .send_to_all(ServerMessage::UpdateGame {
                                    word: word.word(),
                                    tries_used,
                                })
                                .await;

                            send_message(ChatMessage {
                                from: Some(user.nickname.clone()),
                                content: message,
                                color: guess.clone().into(),
                            })
                            .await;

                            if guess == GuessResult::Solved {
                                state = GameState::Solved;
                                send_message(ChatMessage {
                                    content: "You guessed the word!".to_string(),
                                    color: ChatColor::Green,
                                    ..Default::default()
                                })
                                .await;
                            } else if tries_used == 9 {
                                state = GameState::OutOfTries;
                                send_message(ChatMessage {
                                    content: format!(
                                        "No tries left! The word was \"{}\"",
                                        word.target()
                                    ),
                                    color: ChatColor::Red,
                                    ..Default::default()
                                })
                                .await;
                            }
                            players
                                .player_txs()
                                .send_to_all(ServerMessage::UpdateGameState(state.clone()))
                                .await;
                        }
                        ClientMessage::NextRound => {
                            state = GameState::Playing;
                            chat.retain(|m| m.from.is_none());
                            tries_used = 0;
                            word = generate_word(&settings).await;
                            players
                                .player_txs()
                                .send_to_all(ServerMessage::Init(Game {
                                    settings: settings.clone(),
                                    state: state.clone(),
                                    players: players.player_names(),
                                    chat: chat.clone(),
                                    tries_used,
                                    word: word.word(),
                                }))
                                .await;
                            send_message(ChatMessage {
                                content: format!("{} started a new round", user.nickname),
                                ..Default::default()
                            })
                            .await;
                            info!("[{code}] {} started next round", user.nickname);
                        }
                    }
                }
            }
        }
    }
}

async fn generate_word(settings: &GameSettings) -> Word {
    Word::new(
        GENERATOR
            .get()
            .expect("generator not initialized")
            .generate(&settings.language, &settings.difficulty)
            .await
            .expect("failed to generate word"),
    )
}
