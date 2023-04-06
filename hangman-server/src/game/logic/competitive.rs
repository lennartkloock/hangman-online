use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use once_cell::sync::Lazy;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, info, warn};

use hangman_data::{
    ChatColor, ChatMessage, ClientMessage, CompetitiveState, Game, GameCode, GameSettings, Score,
    ServerMessage, ServerMessageInner, User, UserToken,
};

use crate::{
    game::logic::{
        join_message, leave_message,
        word::{GuessResult, Word},
        GameMessage, Players,
    },
    sender_utils::LogSend,
    word_generator,
};

static GAME_DURATION: Lazy<chrono::Duration> = Lazy::new(|| chrono::Duration::minutes(3));

struct PlayerState {
    pub tries_used: u32,
    pub chat: Vec<ChatMessage>,
    pub countdown: chrono::DateTime<Utc>,
    pub word: Word,
    pub word_index: usize,
    pub score: u32,
}

impl PlayerState {
    fn to_state(&self) -> CompetitiveState {
        CompetitiveState {
            chat: self.chat.clone(),
            tries_used: self.tries_used,
            word: self.word.word(),
            countdown: self.countdown,
        }
    }
}

async fn round_countdown(
    code: GameCode,
    players: Arc<RwLock<Players>>,
    player_states: Arc<RwLock<HashMap<UserToken, PlayerState>>>,
    results: Arc<Mutex<Option<Vec<Score>>>>,
) {
    tokio::time::sleep(
        GAME_DURATION
            .to_std()
            .expect("failed to convert chrono duration to std duration"),
    )
    .await;
    info!("[{code}] game round finished");

    let players_guard = players.read().await;
    let states_guard = player_states.read().await;
    let mut sorted_states: Vec<(&User, &PlayerState)> = states_guard
        .iter()
        .filter_map(|(token, state)| players_guard.get(token).map(|(_, user)| (user, state)))
        .collect();
    sorted_states.sort_by_key(|(_, s)| s.score);

    let mut scores = Vec::with_capacity(states_guard.len());
    let mut rank = 0;
    let mut current_score = None;
    for (user, state) in sorted_states.iter().rev() {
        if current_score.map_or(true, |cs| state.score < cs) {
            rank += 1;
        }
        scores.push(Score {
            rank,
            nickname: user.nickname.clone(),
            score: state.score,
        });
        current_score = Some(state.score);
    }
    *results.lock().await = Some(scores.clone());

    for (token, _) in states_guard.iter() {
        if let Some((sender, _)) = players_guard.get(token) {
            sender
                .log_send(ServerMessage::Competitive(ServerMessageInner::Results(
                    scores.clone(),
                )))
                .await;
        }
    }
}

pub async fn game_loop(
    mut rx: mpsc::Receiver<GameMessage>,
    code: GameCode,
    settings: GameSettings,
    owner: UserToken,
) {
    let players = Arc::new(RwLock::new(Players::new()));
    let mut game = Game {
        owner_hash: owner.hashed(),
        settings: settings.clone(),
        players: vec![],
        state: None,
    };
    let player_states: Arc<RwLock<HashMap<UserToken, PlayerState>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let mut global_chat = vec![];
    let mut words = vec![Word::new(word_generator::generate_word(&settings).await)];
    let mut countdown = None;
    let results = Arc::new(Mutex::new(None::<Vec<Score>>));

    while let Some(msg) = rx.recv().await {
        debug!("[{code}] received {msg:?}");
        match msg {
            GameMessage::Join { user, sender } => {
                info!("[{code}] {} joins the game", user.nickname);
                let user_token = user.token;
                let nickname = user.nickname.clone();

                players.write().await.add_player(sender.clone(), user).await;
                game.players = players.read().await.player_names();

                let mut states_guard = player_states.write().await;
                match states_guard.get(&user_token) {
                    None => {
                        states_guard.insert(
                            user_token,
                            PlayerState {
                                tries_used: 0,
                                chat: global_chat.clone(),
                                countdown: countdown.unwrap_or(Utc::now()),
                                word: words[0].clone(),
                                word_index: 0,
                                score: 0,
                            },
                        );
                    }
                    Some(_) => {
                        debug!("{nickname} rejoined, using previous session");
                    }
                }

                match *results.lock().await {
                    None => {
                        let join_msg = join_message(&nickname);
                        global_chat.push(join_msg.clone());
                        let players_guard = players.read().await;
                        for (token, state) in states_guard.iter_mut() {
                            state.chat.push(join_msg.clone());
                            if token != &user_token {
                                if let Some((sender, _)) = players_guard.get(token) {
                                    let message = ServerMessage::Competitive(
                                        ServerMessageInner::UpdateGame(Game {
                                            state: countdown.map(|_| state.to_state()),
                                            ..game.clone()
                                        }),
                                    );
                                    sender.log_send(message).await;
                                }
                            }
                        }
                    }
                    Some(ref r) => {
                        sender
                            .log_send(ServerMessage::Competitive(ServerMessageInner::Results(
                                r.clone(),
                            )))
                            .await;
                    }
                }
            }
            GameMessage::Leave(token) => {
                let Some((_, user)) = players.write().await.remove_player(&token).await else {
                    warn!("[{code}] there was no user in this game with this token");
                    return;
                };
                info!("[{code}] {} left the game", user.nickname);

                game.players = players.read().await.player_names();

                let leave_msg = leave_message(&user.nickname);
                global_chat.push(leave_msg.clone());
                let guard = players.read().await;
                for (token, state) in player_states.write().await.iter_mut() {
                    state.chat.push(leave_msg.clone());
                    if let Some((sender, _)) = guard.get(token) {
                        let message =
                            ServerMessage::Competitive(ServerMessageInner::UpdateGame(Game {
                                state: countdown.map(|_| state.to_state()),
                                ..game.clone()
                            }));
                        sender.log_send(message).await;
                    }
                }

                if guard.is_empty() {
                    info!("[{code}] all players left the game, closing");
                    break;
                } else if token == owner {
                    info!("[{code}] the game owner left the game, closing");
                    break;
                }
            }
            GameMessage::ClientMessage { message, token } => {
                if let Some((sender, user)) = players.read().await.get(&token) {
                    match message {
                        ClientMessage::ChatMessage(msg) => {
                            if countdown.is_none() {
                                return;
                            }
                            let mut lock = player_states.write().await;
                            let Some(player_state) = lock.get_mut(&token) else {
                                warn!("failed to find player state for {token}");
                                return;
                            };
                            let guess = player_state.word.guess(msg.clone());
                            match guess {
                                GuessResult::Hit => {
                                    info!("[{code}] {} guessed right", user.nickname)
                                }
                                GuessResult::Miss => {
                                    info!("[{code}] {} guessed wrong", user.nickname);
                                    player_state.tries_used += 1;
                                }
                                GuessResult::Solved => {
                                    info!("[{code}] {} solved the word", user.nickname);
                                    player_state.score += 1;
                                }
                            }

                            player_state.chat.push(ChatMessage {
                                from: Some(user.nickname.clone()),
                                content: msg,
                                color: guess.clone().into(),
                            });
                            if guess == GuessResult::Solved || player_state.tries_used == 9 {
                                let chat_msg = if guess == GuessResult::Solved {
                                    ChatMessage {
                                        content: format!(
                                            "You guessed \"{}\"",
                                            player_state.word.target()
                                        ),
                                        color: ChatColor::Green,
                                        ..Default::default()
                                    }
                                } else {
                                    ChatMessage {
                                        content: format!(
                                            "No tries left! The word was \"{}\"",
                                            player_state.word.target()
                                        ),
                                        color: ChatColor::Red,
                                        ..Default::default()
                                    }
                                };
                                player_state.chat.push(chat_msg.clone());

                                // New word
                                player_state.chat.retain(|m| m.from.is_none());
                                player_state.tries_used = 0;
                                player_state.word_index += 1;
                                if let Some(new_word) = words.get(player_state.word_index) {
                                    player_state.word = new_word.clone();
                                } else {
                                    let new_word =
                                        Word::new(word_generator::generate_word(&settings).await);
                                    player_state.word = new_word.clone();
                                    words.push(new_word);
                                }
                            }
                            sender
                                .log_send(ServerMessage::Competitive(
                                    ServerMessageInner::UpdateGame(Game {
                                        state: Some(player_state.to_state()),
                                        ..game.clone()
                                    }),
                                ))
                                .await;
                        }
                        ClientMessage::NextRound => {
                            if countdown.is_none() {
                                // Starting game
                                if user.token == owner {
                                    info!("[{code}] {} started the game", user.nickname);
                                    let guard = players.read().await;
                                    let msg = ChatMessage {
                                        content: format!("{} started the game", user.nickname),
                                        ..Default::default()
                                    };
                                    global_chat.push(msg.clone());
                                    let ctdwn = Utc::now() + *GAME_DURATION;
                                    countdown = Some(ctdwn);
                                    for (token, state) in player_states.write().await.iter_mut() {
                                        state.chat.push(msg.clone());
                                        state.countdown = ctdwn;
                                        if let Some((sender, _)) = guard.get(token) {
                                            sender
                                                .log_send(ServerMessage::Competitive(
                                                    ServerMessageInner::UpdateGame(Game {
                                                        state: Some(state.to_state()),
                                                        ..game.clone()
                                                    }),
                                                ))
                                                .await;
                                        }
                                    }
                                    tokio::spawn(round_countdown(
                                        code,
                                        Arc::clone(&players),
                                        Arc::clone(&player_states),
                                        Arc::clone(&results),
                                    ));
                                } else {
                                    warn!(
                                        "{} tried to start the game, but is not owner",
                                        user.nickname
                                    );
                                }
                            } else {
                                // New round
                                info!("[{code}] {} started a new round", user.nickname);
                                let new_round_msg = ChatMessage {
                                    content: format!("{} started a new round", user.nickname),
                                    ..Default::default()
                                };
                                global_chat = vec![new_round_msg];
                                words =
                                    vec![Word::new(word_generator::generate_word(&settings).await)];
                                let ctdwn = Utc::now() + *GAME_DURATION;
                                countdown = Some(ctdwn);
                                *results.lock().await = None;
                                for p in player_states.write().await.values_mut() {
                                    *p = PlayerState {
                                        tries_used: 0,
                                        chat: global_chat.clone(),
                                        countdown: ctdwn,
                                        word: words[0].clone(),
                                        word_index: 0,
                                        score: 0,
                                    };
                                }
                                let guard = players.read().await;
                                guard
                                    .send_to_all(ServerMessage::Competitive(
                                        ServerMessageInner::UpdateGame(Game {
                                            state: Some(CompetitiveState {
                                                chat: global_chat.clone(),
                                                countdown: ctdwn,
                                                tries_used: 0,
                                                word: words[0].word(),
                                            }),
                                            ..game.clone()
                                        }),
                                    ))
                                    .await;
                                tokio::spawn(round_countdown(
                                    code,
                                    Arc::clone(&players),
                                    Arc::clone(&player_states),
                                    Arc::clone(&results),
                                ));
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
