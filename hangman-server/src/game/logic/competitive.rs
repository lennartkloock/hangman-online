use crate::{
    game::{
        logic::word::{GuessResult, Word},
    },
    sender_utils::{LogSend},
    GENERATOR,
};
use hangman_data::{
    ChatColor, ChatMessage, ClientMessage, Game, GameCode, GameSettings, GameState, ServerMessage,
    User, UserToken,
};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use tokio::sync::{mpsc, mpsc::Sender, RwLock};
use tracing::{debug, info, warn};
use crate::game::logic::{GameMessage, Players, ToName};

struct UserState {
    user: User,
    state: GameState,
    tries_used: u32,
    chat: Vec<ChatMessage>,
}

impl ToName for UserState {
    fn to_name(&self) -> &str {
        &self.user.nickname
    }
}

pub async fn game_loop(mut rx: mpsc::Receiver<GameMessage>, code: GameCode, settings: GameSettings, owner: UserToken) {
    // let mut players = Players::new();
    // let mut global_chat = Vec::new();
    //
    // while let Some(msg) = rx.recv().await {
    //     debug!("[{code}] received {msg:?}");
    //     match msg {
    //         GameMessage::Join { user, sender } => {
    //             info!("[{code}] {user:?} joins the game");
    //             players.add_player(user.token, sender, UserState {
    //                 user,
    //                 state: GameState::Playing,
    //                 tries_used: 0,
    //                 chat: vec![],
    //             });
    //
    //             sender
    //                 .log_send(ServerMessage::Init(Game {
    //                     settings: settings.clone(),
    //                     state: GameState::Playing,
    //                     players: players.player_names(),
    //                     chat: global_chat.clone(),
    //                     tries_used: 0,
    //                     word: first_word,
    //                 }))
    //                 .await;
    //             self.send_global_chat_message(ChatMessage {
    //                 content: format!("→ {} joined the game", user.nickname),
    //                 ..Default::default()
    //             })
    //             .await;
    //         }
    //         GameMessage::Leave(token) => {
    //             let Some((user, sender)) = players.remove(&token) else {
    //                 warn!(
    //                     "[{}] there was no user in this game with this token",
    //                     self.code
    //                 );
    //                 return;
    //             };
    //             info!("[{}] {user:?} left the game", self.code);
    //             // Send update to all clients
    //             players
    //                 .send_to_all(ServerMessage::UpdatePlayers(
    //                     players.player_names(),
    //                 ))
    //                 .await;
    //
    //             // ...
    //
    //             if players.is_empty() {
    //                 info!("[{}] all players left the game, closing", self.code);
    //                 break;
    //             } else if token == owner {
    //                 info!("[{}] the game owner left the game, closing", self.code);
    //                 break;
    //             }
    //         }
    //         GameMessage::ClientMessage { message, token } => {
    //             if let Some((_, user_state)) = players.get(&token) {
    //                 // ...
    //             }
    //         }
    //     }
    // }
    todo!()
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
