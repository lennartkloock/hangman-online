use std::{collections::HashMap, future::Future};

use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{
    debug, info,
    log::{warn, Level::Debug},
};

use hangman_data::{
    ChatColor, ChatMessage, ClientMessage, Game, GameCode, GameSettings, GameState, ServerMessage,
    User, UserToken,
};

use crate::{
    game::{
        logic::word::{GuessResult, Word},
        GameLogic, GameMessage, ServerGame,
    },
    sender_utils::{LogSend, SendToAll},
};

pub struct TeamGameLogic {
    state: GameState,
    chat: Vec<ChatMessage>,
    tries_used: u32,
    word: Word,
}

impl TeamGameLogic {
    async fn send_chat_message(&mut self, msg: ChatMessage) {
        self.chat.push(msg.clone());
        self.player_txs()
            .send_to_all(ServerMessage::ChatMessage(msg))
            .await;
    }
}

#[async_trait]
impl GameLogic for TeamGameLogic {
    async fn new(settings: &GameSettings) -> Self {
        let word = Word::generate(&settings.language, 10000).await.unwrap();
        Self {
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
                // TODO: Solve send_to_all not available here
                send_to_all(ServerMessage::UpdateGame {
                    word: self.word.word(),
                    tries_used: self.tries_used,
                })
                .await;

                self.send_chat_message(ChatMessage {
                    from: Some(user.nickname.clone()),
                    content: message,
                    color: guess.into(),
                })
                .await;

                if guess == GuessResult::Solved {
                    self.state = GameState::Solved;
                    send_to_all(ServerMessage::ChatMessage(ChatMessage {
                        from: None,
                        content: "You guessed the word!".to_string(),
                        color: ChatColor::Green,
                    }))
                    .await;
                } else if self.tries_used == 9 {
                    self.state = GameState::OutOfTries;
                    send_to_all(ServerMessage::ChatMessage(ChatMessage {
                        from: None,
                        content: format!("No tries left! The word was \"{}\"", self.word.target()),
                        color: ChatColor::Red,
                    }))
                    .await;
                }
                send_to_all(ServerMessage::UpdateGameState(self.state.clone())).await;
            }
        }
    }

    async fn on_user_join(
        &mut self,
        user: (&User, mpsc::Sender<ServerMessage>),
        init_game: &mut Game,
    ) {
        // TODO: Needs improvement
        init_game.state = self.state.clone();
        init_game.chat = self.chat.clone();
        init_game.tries_used = self.tries_used;
        init_game.word = self.word.word();
        self.send_chat_message(ChatMessage {
            content: format!("→ {} joined the game", user.0.nickname),
            ..Default::default()
        })
        .await;
    }

    async fn on_user_leave(&mut self, user: (&User, mpsc::Sender<ServerMessage>)) {
        self.send_chat_message(ChatMessage {
            content: format!("← {} left the game", user.0.nickname),
            ..Default::default()
        })
        .await;
    }
}
