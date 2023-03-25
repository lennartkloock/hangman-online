use crate::{
    game::logic::{
        join_message, leave_message,
        word::{GuessResult, Word},
        Chat, GameMessage, Players,
    },
    sender_utils::LogSend,
    word_generator,
};
use hangman_data::{
    ChatColor, ChatMessage, ClientMessage, Game, GameCode, GameSettings, GameState, ServerMessage,
    UserToken,
};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, log::warn};

pub async fn game_loop(
    mut rx: mpsc::Receiver<GameMessage>,
    code: GameCode,
    settings: GameSettings,
    owner: UserToken,
) {
    let players = Arc::new(RwLock::new(Players::new()));
    let mut chat = Chat::new(Arc::clone(&players));
    let mut state = GameState::Playing;
    let mut tries_used = 0;
    let mut word = Word::new(word_generator::generate_word(&settings).await);

    while let Some(msg) = rx.recv().await {
        debug!("[{code}] received {msg:?}");
        match msg {
            GameMessage::Join { user, sender } => {
                info!("[{code}] {} joins the game", user.nickname);
                let nickname = user.nickname.clone();
                players.write().await.add_player(sender.clone(), user).await;

                sender
                    .log_send(ServerMessage::Init(Game {
                        settings: settings.clone(),
                        state: state.clone(),
                        players: players.read().await.player_names(),
                        chat: chat.clone(),
                        tries_used,
                        word: word.word(),
                        countdown: None,
                    }))
                    .await;

                chat.send_message(join_message(&nickname)).await;
            }
            GameMessage::Leave(token) => {
                let Some((_, user)) = players.write().await.remove_player(&token).await else {
                    warn!("[{code}] there was no user in this game with this token");
                    return;
                };
                info!("[{code}] {} left the game", user.nickname);

                chat.send_message(leave_message(&user.nickname)).await;

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
                                .read()
                                .await
                                .send_to_all(ServerMessage::UpdateGame {
                                    word: word.word(),
                                    tries_used,
                                })
                                .await;

                            chat.send_message(ChatMessage {
                                from: Some(user.nickname.clone()),
                                content: message,
                                color: guess.clone().into(),
                            })
                            .await;

                            if guess == GuessResult::Solved {
                                state = GameState::Solved;
                                chat.send_message(ChatMessage {
                                    content: "You guessed the word!".to_string(),
                                    color: ChatColor::Green,
                                    ..Default::default()
                                })
                                .await;
                            } else if tries_used == 9 {
                                state = GameState::OutOfTries;
                                chat.send_message(ChatMessage {
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
                                .read()
                                .await
                                .send_to_all(ServerMessage::UpdateGameState(state.clone()))
                                .await;
                        }
                        ClientMessage::NextRound => {
                            state = GameState::Playing;
                            chat.retain(|m| m.from.is_none());
                            tries_used = 0;
                            word = Word::new(word_generator::generate_word(&settings).await);
                            players
                                .read()
                                .await
                                .send_to_all(ServerMessage::Init(Game {
                                    settings: settings.clone(),
                                    state: state.clone(),
                                    players: players.read().await.player_names(),
                                    chat: chat.clone(),
                                    tries_used,
                                    word: word.word(),
                                    countdown: None,
                                }))
                                .await;
                            chat.send_message(ChatMessage {
                                content: format!("{} started a new round", user.nickname),
                                ..Default::default()
                            })
                            .await;
                            info!("[{code}] {} started next round", user.nickname);
                        }
                    }
                } else {
                    warn!("[{code}] there was no user in this game with this token");
                }
            }
        }
    }
}
