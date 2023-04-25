use crate::{
    game::logic::{
        join_message, leave_message,
        word::{GuessResult, Word},
        GameMessage, Players,
    },
    word_generator,
};
use hangman_data::{
    ChatColor, ChatMessage, ClientMessage, Game, GameCode, GameSettings, ServerMessage,
    ServerMessageInner, TeamState, UserToken,
};
use tokio::sync::mpsc;
use tracing::{debug, info, log::warn};

pub async fn game_loop(
    mut rx: mpsc::Receiver<GameMessage>,
    code: GameCode,
    settings: GameSettings,
    owner: UserToken,
) {
    let mut players = Players::new();
    let mut chat = vec![];
    let mut word = Word::new(word_generator::generate_word(&settings).await);
    let mut game = Game::<TeamState> {
        owner_hash: owner.hashed(),
        settings: settings.clone(),
        players: vec![],
        state: None,
    };
    let mut finished = false;

    'game_loop: while let Some(msg) = rx.recv().await {
        debug!("[{code}] received {msg:?}");
        match msg {
            GameMessage::Join { user, sender } => {
                info!("[{code}] {} joins the game", user.nickname);
                let nickname = user.nickname.clone();
                players.add_player(sender, user).await;
                chat.push(join_message(&nickname));
                // If game is started
                game.players = players.player_names();
                if let Some(state) = &mut game.state {
                    state.chat = chat.clone();
                }
                players
                    .send_to_all(ServerMessage::Team(ServerMessageInner::UpdateGame(
                        game.clone(),
                    )))
                    .await;
            }
            GameMessage::Leave(token) => {
                let Some((_, user)) = players.remove_player(&token).await else {
                    warn!("[{code}] there was no user in this game with this token");
                    return;
                };
                info!("[{code}] {} left the game", user.nickname);

                chat.push(leave_message(&user.nickname));

                // If game is started
                game.players = players.player_names();
                if let Some(state) = &mut game.state {
                    state.chat = chat.clone();
                }
                players
                    .send_to_all(ServerMessage::Team(ServerMessageInner::UpdateGame(
                        game.clone(),
                    )))
                    .await;

                if players.is_empty() {
                    info!("[{code}] all players left the game, closing");
                    break 'game_loop;
                } else if token == owner {
                    info!("[{code}] the game owner left the game, closing");
                    break 'game_loop;
                }
            }
            GameMessage::ClientMessage { message, token } => {
                if let Some((_, user)) = players.get(&token) {
                    match message {
                        ClientMessage::ChatMessage(message) => {
                            // If game is started
                            if let Some(state) = &mut game.state {
                                let guess = word.guess(message.clone());
                                state.word = word.word();
                                match guess {
                                    GuessResult::Miss => {
                                        info!("[{code}] {} guessed wrong", user.nickname);
                                        state.tries_used += 1;
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

                                state.round_finished = guess == GuessResult::Solved || state.tries_used == 9;
                                if state.round_finished {
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
                                }
                                state.chat = chat.clone();
                                players
                                    .send_to_all(ServerMessage::Team(
                                        ServerMessageInner::UpdateGame(game.clone()),
                                    ))
                                    .await;
                            }
                        }
                        ClientMessage::NextRound => match &mut game.state {
                            None => {
                                if user.token == owner {
                                    info!("[{code}] {} started the game", user.nickname);
                                    finished = true;
                                    chat.push(ChatMessage {
                                        content: format!("{} started the game", user.nickname),
                                        ..Default::default()
                                    });
                                    game.state = Some(TeamState {
                                        chat: chat.clone(),
                                        tries_used: 0,
                                        word: word.word(),
                                        round_finished: false,
                                    });
                                    players
                                        .send_to_all(ServerMessage::Team(
                                            ServerMessageInner::UpdateGame(game.clone()),
                                        ))
                                        .await;
                                } else {
                                    warn!(
                                        "{} tried to start the game, but is not owner",
                                        user.nickname
                                    );
                                }
                            }
                            Some(state) if finished => {
                                finished = false;
                                chat.retain(|m| m.from.is_none());
                                state.tries_used = 0;
                                word = Word::new(word_generator::generate_word(&settings).await);
                                chat.push(ChatMessage {
                                    content: format!("{} started a new round", user.nickname),
                                    ..Default::default()
                                });
                                state.chat = chat.clone();
                                state.word = word.word();
                                state.round_finished = false;
                                players
                                    .send_to_all(ServerMessage::Team(
                                        ServerMessageInner::UpdateGame(game.clone()),
                                    ))
                                    .await;
                                info!("[{code}] {} started next round", user.nickname);
                            }
                            Some(_) => {
                                warn!("can't start a new round when game is still `Started`");
                            }
                        },
                    }
                } else {
                    warn!("[{code}] there was no user in this game with this token");
                }
            }
        }
    }
}
