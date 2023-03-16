use crate::{
    game::{
        logic::word::{GuessResult, Word},
        GameLogic, Players,
    },
    sender_utils::{LogSend, SendToAll},
};
use async_trait::async_trait;
use hangman_data::{
    ChatColor, ChatMessage, ClientMessage, Game, GameCode, GameSettings, GameState, ServerMessage,
    User,
};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::info;

pub struct TeamGameLogic {
    players: Arc<RwLock<Players>>,
    settings: GameSettings,
    state: GameState,
    chat: Vec<ChatMessage>,
    tries_used: u32,
    word: Word,
}

impl TeamGameLogic {
    async fn send_chat_message(&mut self, msg: ChatMessage) {
        self.chat.push(msg.clone());
        self.players
            .read()
            .await
            .player_txs()
            .send_to_all(ServerMessage::ChatMessage(msg))
            .await;
    }
}

#[async_trait]
impl GameLogic for TeamGameLogic {
    async fn new(settings: GameSettings, players: Arc<RwLock<Players>>) -> Self {
        let word = Word::generate(&settings.language, 10000).await.unwrap();
        Self {
            players,
            settings,
            state: GameState::Playing,
            chat: vec![],
            tries_used: 0,
            word,
        }
    }

    async fn handle_message(
        &mut self,
        code: GameCode,
        (user, _): (&User, mpsc::Sender<ServerMessage>),
        msg: ClientMessage,
    ) {
        match msg {
            ClientMessage::ChatMessage(message) => {
                let guess = self.word.guess(message.clone());
                match guess {
                    GuessResult::Miss => {
                        info!("[{}] {} guessed wrong", code, user.nickname);
                        self.tries_used += 1;
                    }
                    GuessResult::Hit => {
                        info!("[{}] {} guessed right", code, user.nickname)
                    }
                    GuessResult::Solved => {
                        info!("[{}] {} solved the word", code, user.nickname)
                    }
                };
                self.players
                    .read()
                    .await
                    .player_txs()
                    .send_to_all(ServerMessage::UpdateGame {
                        word: self.word.word(),
                        tries_used: self.tries_used,
                    })
                    .await;

                self.send_chat_message(ChatMessage {
                    from: Some(user.nickname.clone()),
                    content: message,
                    color: guess.clone().into(),
                })
                .await;

                if guess == GuessResult::Solved {
                    self.state = GameState::Solved;
                    self.players
                        .read()
                        .await
                        .player_txs()
                        .send_to_all(ServerMessage::ChatMessage(ChatMessage {
                            from: None,
                            content: "You guessed the word!".to_string(),
                            color: ChatColor::Green,
                        }))
                        .await;
                } else if self.tries_used == 9 {
                    self.state = GameState::OutOfTries;
                    self.players
                        .read()
                        .await
                        .player_txs()
                        .send_to_all(ServerMessage::ChatMessage(ChatMessage {
                            from: None,
                            content: format!(
                                "No tries left! The word was \"{}\"",
                                self.word.target()
                            ),
                            color: ChatColor::Red,
                        }))
                        .await;
                }
                self.players
                    .read()
                    .await
                    .player_txs()
                    .send_to_all(ServerMessage::UpdateGameState(self.state.clone()))
                    .await;
            }
        }
    }

    async fn on_user_join(&mut self, (user, sender): (&User, mpsc::Sender<ServerMessage>)) {
        sender
            .log_send(ServerMessage::Init(Game {
                settings: self.settings.clone(),
                state: self.state.clone(),
                players: self.players.read().await.player_names(),
                chat: self.chat.clone(),
                tries_used: self.tries_used,
                word: self.word.word(),
            }))
            .await;
        self.send_chat_message(ChatMessage {
            content: format!("→ {} joined the game", user.nickname),
            ..Default::default()
        })
        .await;
    }

    async fn on_user_leave(&mut self, (user, _): (&User, mpsc::Sender<ServerMessage>)) {
        self.send_chat_message(ChatMessage {
            content: format!("← {} left the game", user.nickname),
            ..Default::default()
        })
        .await;
    }
}
