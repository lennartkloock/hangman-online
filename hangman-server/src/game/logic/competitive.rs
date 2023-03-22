use crate::{
    game::logic::{join_message, leave_message, Chat, GameMessage, Players},
    sender_utils::LogSend,
    word_generator,
};
use hangman_data::{
    ChatMessage, ClientMessage, Game, GameCode, GameSettings, GameState, ServerMessage, UserToken,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

struct PlayerState {
    pub state: GameState,
    pub tries_used: u32,
    pub chat: Vec<ChatMessage>,
}

pub async fn game_loop(
    mut rx: mpsc::Receiver<GameMessage>,
    code: GameCode,
    settings: GameSettings,
    owner: UserToken,
) {
    let players = Arc::new(RwLock::new(Players::new()));
    let mut player_states = HashMap::new();
    let mut global_chat = Chat::new(Arc::clone(&players));

    while let Some(msg) = rx.recv().await {
        debug!("[{code}] received {msg:?}");
        match msg {
            GameMessage::Join { user, sender } => {
                info!("[{code}] {} joins the game", user.nickname);
                let token = user.token;
                let nickname = user.nickname.clone();
                players.write().await.add_player(sender.clone(), user).await;
                let player_state = match player_states.get(&token) {
                    None => {
                        player_states.insert(
                            token,
                            PlayerState {
                                state: GameState::Playing,
                                tries_used: 0,
                                chat: global_chat.clone(),
                            },
                        );
                        player_states.get(&token).unwrap()
                    }
                    Some(s) => {
                        debug!("{nickname} rejoined, using previous session");
                        s
                    }
                };

                sender
                    .log_send(ServerMessage::Init(Game {
                        settings: settings.clone(),
                        state: player_state.state.clone(),
                        players: players.read().await.player_names(),
                        chat: player_state.chat.clone(),
                        tries_used: player_state.tries_used,
                        word: word_generator::generate_word(&settings).await,
                    }))
                    .await;

                let join_msg = join_message(&nickname);
                for state in player_states.values_mut() {
                    state.chat.push(join_msg.clone());
                }
                global_chat.send_message(join_msg).await;
            }
            GameMessage::Leave(token) => {
                let Some((_, user)) = players.write().await.remove_player(&token).await else {
                    warn!("[{code}] there was no user in this game with this token");
                    return;
                };
                info!("[{code}] {} left the game", user.nickname);

                let leave_msg = leave_message(&user.nickname);
                for state in player_states.values_mut() {
                    state.chat.push(leave_msg.clone());
                }
                global_chat.send_message(leave_msg).await;

                if players.read().await.is_empty() {
                    info!("[{code}] all players left the game, closing");
                    break;
                } else if token == owner {
                    info!("[{code}] the game owner left the game, closing");
                    break;
                }
            }
            GameMessage::ClientMessage { message, token } => {
                if let Some((_, user)) = players.read().await.get(&token) {
                    match message {
                        ClientMessage::ChatMessage(_) => {}
                        ClientMessage::NextRound => {}
                    }
                } else {
                    warn!("[{code}] there was no user in this game with this token");
                }
            }
        }
    }
}

// pub struct CompetitiveGameLogic {
//     players: Arc<RwLock<Players>>,
//     settings: GameSettings,
//     user_states: HashMap<UserToken, UserState>,
//     global_chat: Vec<ChatMessage>,
//     words: VecDeque<Word>,
// }
//
// impl CompetitiveGameLogic {
//     async fn send_global_chat_message(&mut self, msg: ChatMessage) {
//         self.global_chat.push(msg.clone());
//         self.players
//             .read()
//             .await
//             .player_txs()
//             .send_to_all(ServerMessage::ChatMessage(msg));
//     }
// }
//
// #[async_trait]
// impl GameLogic for CompetitiveGameLogic {
//     async fn new(settings: GameSettings, players: Arc<RwLock<Players>>) -> Self {
//         let word = Word::new(
//             GENERATOR
//                 .get()
//                 .expect("generator not initialized")
//                 .generate(&settings.language, &settings.difficulty)
//                 .await
//                 .expect("failed to generate word"),
//         );
//         let mut words = VecDeque::new();
//         words.push_back(word);
//         Self {
//             players,
//             settings,
//             user_states: HashMap::new(),
//             global_chat: vec![],
//             words,
//         }
//     }
//
//     async fn handle_message(
//         &mut self,
//         code: GameCode,
//         (user, sender): (&User, Sender<ServerMessage>),
//         msg: ClientMessage,
//     ) {
//         match msg {
//             ClientMessage::ChatMessage(message) => {
//                 let user_state = self.user_states.get_mut(&user.token).unwrap();
//                 let guess = user_state.words.get_mut(0).unwrap().guess(message.clone());
//                 match guess {
//                     GuessResult::Hit => info!("[{}] {} guessed right", code, user.nickname),
//                     GuessResult::Miss => {
//                         info!("[{}] {} guessed wrong", code, user.nickname);
//                         user_state.tries_used += 1;
//                     }
//                     GuessResult::Solved => info!("[{}] {} solved the word", code, user.nickname),
//                 }
//
//                 sender
//                     .log_send(ServerMessage::UpdateGame {
//                         word: user_state.words.front().unwrap().word(),
//                         tries_used: user_state.tries_used,
//                     })
//                     .await;
//                 sender
//                     .log_send(ServerMessage::ChatMessage(ChatMessage {
//                         from: Some(user.nickname.clone()),
//                         content: message,
//                         color: guess.clone().into(),
//                     }))
//                     .await;
//                 if guess == GuessResult::Solved || user_state.tries_used == 9 {
//                     let msg = if guess == GuessResult::Solved {
//                         ChatMessage {
//                             content: "You guessed the word!".to_string(),
//                             color: ChatColor::Green,
//                             ..Default::default()
//                         }
//                     } else {
//                         ChatMessage {
//                             content: format!(
//                                 "No tries left! The word was \"{}\"",
//                                 user_state.words.front().unwrap().target()
//                             ),
//                             color: ChatColor::Red,
//                             ..Default::default()
//                         }
//                     };
//                     user_state.chat.push(msg.clone());
//                     sender.log_send(ServerMessage::ChatMessage(msg));
//
//                     // New word
//                     user_state.words.pop_front();
//                     if user_state.words.front().is_none() {
//                         let new_word = Word::new(
//                             GENERATOR
//                                 .get()
//                                 .expect("generator not initialized")
//                                 .generate(&self.settings.language, &self.settings.difficulty)
//                                 .await
//                                 .expect("failed to generate word"),
//                         );
//                         for s in self.user_states.values_mut() {
//                             s.words.push_back(new_word.clone());
//                         }
//                         self.words.push_back(new_word);
//                     }
//                 }
//
//                 self.players
//                     .read()
//                     .await
//                     .player_txs()
//                     .send_to_all(ServerMessage::UpdateGame {
//                         word: user_state.words.front().unwrap().word(),
//                         tries_used: user_state.tries_used,
//                     })
//                     .await;
//             }
//             ClientMessage::NextRound => {
//                 warn!("not supported in this game mode");
//             }
//         }
//     }
//
//     async fn on_user_join(&mut self, (user, sender): (&User, Sender<ServerMessage>)) {

//     }
//
//     async fn on_user_leave(&mut self, user: (&User, Sender<ServerMessage>)) {
//         todo!()
//     }
// }
