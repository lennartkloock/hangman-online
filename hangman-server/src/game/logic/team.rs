use crate::{
    game::logic::{
        join_message, leave_message,
        word::{GuessResult, Word},
        GameMessage, Players,
    },
    word_generator,
};
use hangman_data::{ChatColor, ChatMessage, ClientMessage, Game, GameCode, GameResults, GameSettings, GameState, ServerMessage, UserToken};
use tokio::sync::mpsc;
use tracing::{debug, info, log::warn};

// TODO: This code is shit, too much duplication and too similar to competitive code

pub async fn game_loop(
    mut rx: mpsc::Receiver<GameMessage>,
    code: GameCode,
    settings: GameSettings,
    owner: UserToken,
) {
    let mut players = Players::new();
    let mut chat = vec![];
    let mut tries_used = 0;
    let mut word = Word::new(word_generator::generate_word(&settings).await);
    let mut game = Game::Waiting { owner_hash: owner.hashed(), settings: settings.clone() };

    while let Some(msg) = rx.recv().await {
        debug!("[{code}] received {msg:?}");
        match msg {
            GameMessage::Join { user, sender } => {
                info!("[{code}] {} joins the game", user.nickname);
                let nickname = user.nickname.clone();
                players.add_player(sender.clone(), user).await;
                chat.push(join_message(&nickname));
                if matches!(game, Game::Started { .. }) {
                    game = Game::Started {
                        settings: settings.clone(),
                        owner_hash: owner.hashed(),
                        state: GameState::Team {
                            players: players.player_names(),
                            chat: chat.clone(),
                            tries_used,
                            word: word.word(),
                        },
                    };
                }
                players.send_to_all(ServerMessage::UpdateGame(game.clone())).await;
            }
            GameMessage::Leave(token) => {
                let Some((_, user)) = players.remove_player(&token).await else {
                    warn!("[{code}] there was no user in this game with this token");
                    return;
                };
                info!("[{code}] {} left the game", user.nickname);

                chat.push(leave_message(&user.nickname));

                if matches!(game, Game::Started { .. }) {
                    game = Game::Started {
                        settings: settings.clone(),
                        owner_hash: owner.hashed(),
                        state: GameState::Team {
                            players: players.player_names(),
                            chat: chat.clone(),
                            tries_used,
                            word: word.word(),
                        },
                    };
                }
                players.send_to_all(ServerMessage::UpdateGame(game.clone())).await;

                if players.is_empty() {
                    info!("[{code}] all players left the game, closing");
                    break;
                } else if token == owner {
                    info!("[{code}] the game owner left the game, closing");
                    break;
                }
            }
            GameMessage::ClientMessage { message, token } => {
                if let Some((_, user)) = players.get(&token) {
                    match message {
                        ClientMessage::ChatMessage(message) => {
                            if matches!(game, Game::Started { .. }) {
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

                                chat.push(ChatMessage {
                                    from: Some(user.nickname.clone()),
                                    content: message,
                                    color: guess.clone().into(),
                                });

                                if guess == GuessResult::Solved || tries_used == 9 {
                                    if guess == GuessResult::Solved {
                                        chat.push(ChatMessage {
                                            content: "You guessed the word!".to_string(),
                                            color: ChatColor::Green,
                                            ..Default::default()
                                        });
                                    } else {
                                        chat.push(ChatMessage {
                                            content: format!(
                                                "No tries left! The word was \"{}\"",
                                                word.target()
                                            ),
                                            color: ChatColor::Red,
                                            ..Default::default()
                                        });
                                    }
                                    game = Game::Finished {
                                        settings: settings.clone(),
                                        owner_hash: owner.hashed(),
                                        state: GameState::Team {
                                            players: players.player_names(),
                                            chat: chat.clone(),
                                            tries_used,
                                            word: word.word(),
                                        },
                                        results: GameResults::Team,
                                    };
                                } else {
                                    game = Game::Started {
                                        settings: settings.clone(),
                                        owner_hash: owner.hashed(),
                                        state: GameState::Team {
                                            players: players.player_names(),
                                            chat: chat.clone(),
                                            tries_used,
                                            word: word.word(),
                                        },
                                    };
                                }
                                players.send_to_all(ServerMessage::UpdateGame(game.clone())).await;
                            }
                        }
                        ClientMessage::NextRound => {
                            match game {
                                Game::Waiting { .. } => {
                                    if user.token == owner {
                                        info!("[{code}] {} started the game", user.nickname);
                                        chat.push(ChatMessage {
                                            content: format!("{} started the game", user.nickname),
                                            ..Default::default()
                                        });
                                        game = Game::Started {
                                            settings: settings.clone(),
                                            owner_hash: owner.hashed(),
                                            state: GameState::Team {
                                                players: players.player_names(),
                                                chat: chat.clone(),
                                                tries_used,
                                                word: word.word(),
                                            },
                                        };
                                        players.send_to_all(ServerMessage::UpdateGame(game.clone())).await;
                                    } else {
                                        warn!("{} tried to start the game, but is not owner", user.nickname);
                                    }
                                }
                                Game::Started { .. } => {
                                    warn!("can't start a new round when game is still `Started`");
                                }
                                Game::Finished { .. } => {
                                    chat.retain(|m| m.from.is_none());
                                    tries_used = 0;
                                    word = Word::new(word_generator::generate_word(&settings).await);
                                    chat.push(ChatMessage {
                                        content: format!("{} started a new round", user.nickname),
                                        ..Default::default()
                                    });
                                    game = Game::Started {
                                        settings: settings.clone(),
                                        owner_hash: owner.hashed(),
                                        state: GameState::Team {
                                            players: players.player_names(),
                                            chat: chat.clone(),
                                            tries_used,
                                            word: word.word(),
                                        },
                                    };
                                    players.send_to_all(ServerMessage::UpdateGame(game.clone())).await;
                                    info!("[{code}] {} started next round", user.nickname);
                                }
                            }
                        }
                    }
                } else {
                    warn!("[{code}] there was no user in this game with this token");
                }
            }
        }
    }
}
