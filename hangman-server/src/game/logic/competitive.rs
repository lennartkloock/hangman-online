use crate::{
    game::{
        logic::word::{GuessResult, Word},
        GameLogic, Players,
    },
    sender_utils::{LogSend, SendToAll},
    GENERATOR,
};
use async_trait::async_trait;
use hangman_data::{
    ChatColor, ChatMessage, ClientMessage, Game, GameCode, GameSettings, GameState, ServerMessage,
    User, UserToken,
};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use tokio::sync::{mpsc::Sender, RwLock};
use tracing::{info, warn};

struct UserState {
    state: GameState,
    tries_used: u32,
    chat: Vec<ChatMessage>,
    words: VecDeque<Word>,
}

pub struct CompetitiveGameLogic {
    players: Arc<RwLock<Players>>,
    settings: GameSettings,
    user_states: HashMap<UserToken, UserState>,
    global_chat: Vec<ChatMessage>,
    words: VecDeque<Word>,
}

// impl CompetitiveGameLogic {
//     async fn add_new_word(&mut self) {
//         let word = Word::new(
//             GENERATOR
//                 .get()
//                 .expect("generator not initialized")
//                 .generate(&self.settings.language, &self.settings.difficulty)
//                 .await
//                 .expect("failed to generate word"),
//         );
//         self.words.push_back(word);
//     }
// }

impl CompetitiveGameLogic {
    async fn send_global_chat_message(&mut self, msg: ChatMessage) {
        self.global_chat.push(msg.clone());
        self.players
            .read()
            .await
            .player_txs()
            .send_to_all(ServerMessage::ChatMessage(msg));
    }
}

#[async_trait]
impl GameLogic for CompetitiveGameLogic {
    async fn new(settings: GameSettings, players: Arc<RwLock<Players>>) -> Self {
        let word = Word::new(
            GENERATOR
                .get()
                .expect("generator not initialized")
                .generate(&settings.language, &settings.difficulty)
                .await
                .expect("failed to generate word"),
        );
        let mut words = VecDeque::new();
        words.push_back(word);
        Self {
            players,
            settings,
            user_states: HashMap::new(),
            global_chat: vec![],
            words,
        }
    }

    async fn handle_message(
        &mut self,
        code: GameCode,
        (user, sender): (&User, Sender<ServerMessage>),
        msg: ClientMessage,
    ) {
        match msg {
            ClientMessage::ChatMessage(message) => {
                let user_state = self.user_states.get_mut(&user.token).unwrap();
                let mut word = user_state.words.get_mut(0).unwrap();
                let guess = word.guess(message.clone());
                match guess {
                    GuessResult::Hit => info!("[{}] {} guessed right", code, user.nickname),
                    GuessResult::Miss => {
                        info!("[{}] {} guessed wrong", code, user.nickname);
                        user_state.tries_used += 1;
                    }
                    GuessResult::Solved => info!("[{}] {} solved the word", code, user.nickname),
                }

                sender
                    .log_send(ServerMessage::UpdateGame {
                        word: word.word(),
                        tries_used: user_state.tries_used,
                    })
                    .await;
                sender
                    .log_send(ServerMessage::ChatMessage(ChatMessage {
                        from: Some(user.nickname.clone()),
                        content: message,
                        color: guess.clone().into(),
                    }))
                    .await;

                if guess == GuessResult::Solved || self.tries_used == 9 {
                    let msg = if guess == GuessResult::Solved {
                        ChatMessage {
                            content: "You guessed the word!".to_string(),
                            color: ChatColor::Green,
                            ..Default::default()
                        }
                    } else if user_state.tries_used == 9 {
                        ChatMessage {
                            content: format!("No tries left! The word was \"{}\"", word.target()),
                            color: ChatColor::Red,
                            ..Default::default()
                        }
                    };
                    user_state.chat.push(msg.clone());
                    sender.log_send(ServerMessage::ChatMessage(msg));
                    // New word
                }

                self.players
                    .read()
                    .await
                    .player_txs()
                    .send_to_all(ServerMessage::UpdateGame {
                        word: word.word(),
                        tries_used: user_state.tries_used,
                    })
                    .await;
            }
            ClientMessage::NextRound => {
                warn!("not supported in this game mode");
            }
        }
    }

    async fn on_user_join(&mut self, (user, sender): (&User, Sender<ServerMessage>)) {
        let mut words = VecDeque::new();
        words.append(&mut self.words.clone());
        let first_word = words.front().unwrap().word();
        self.user_states.insert(
            user.token,
            UserState {
                state: GameState::Playing,
                tries_used: 0,
                chat: vec![],
                words,
            },
        );
        sender
            .log_send(ServerMessage::Init(Game {
                settings: self.settings.clone(),
                state: self.state.clone(),
                players: self.players.read().await.player_names(),
                chat: self.global_chat.clone(),
                tries_used: 0,
                word: first_word,
            }))
            .await;
        self.send_global_chat_message(ChatMessage {
            content: format!("→ {} joined the game", user.nickname),
            ..Default::default()
        })
        .await;
    }

    async fn on_user_leave(&mut self, user: (&User, Sender<ServerMessage>)) {
        todo!()
    }
}
