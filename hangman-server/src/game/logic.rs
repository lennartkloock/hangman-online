//! Game logic

use crate::{
    game::{logic::word::Word, ServerGame},
    sender_utils::{LogSend, SendToAll},
};
use hangman_data::{ChatColor, ChatMessage, ClientMessage, Game, ServerMessage, User, UserToken};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};
use crate::game::logic::word::GuessResult;

mod word;

#[derive(Debug)]
pub enum GameMessage {
    Join {
        user: User,
        sender: mpsc::Sender<ServerMessage>,
    },
    Leave(UserToken),
    ClientMessage {
        token: UserToken,
        message: ClientMessage,
    },
}

type Players = HashMap<UserToken, (User, mpsc::Sender<ServerMessage>)>;

pub async fn game_logic(game: ServerGame, mut rx: mpsc::Receiver<GameMessage>) {
    // Game logic
    let code = game.code;
    let mut players = Players::new();
    let mut chat: Vec<ChatMessage> = vec![];
    let mut tries_used = 0;
    let mut word = Word::generate(&game.settings.language, 10000).await.unwrap();

    while let Some(msg) = rx.recv().await {
        debug!("[{code}] received {msg:?}");
        match msg {
            GameMessage::Join { user, sender } => {
                info!("[{code}] {:?} joins the game", user);
                let sender_c = sender.clone();
                let token = user.token;
                players.insert(user.token, (user, sender));
                let settings = game.settings.clone();
                let player_names = player_names(&players);
                sender_c
                    .log_send(ServerMessage::Init(Game {
                        settings,
                        players: player_names.clone(),
                        chat: chat.clone(),
                        tries_used,
                        word: word.word(),
                    }))
                    .await;
                // Send update to all clients
                players
                    .iter()
                    .filter(|(t, _)| **t != token)
                    .map(|(_, (_, s))| s)
                    .send_to_all(ServerMessage::UpdatePlayers(player_names))
                    .await;
            }
            GameMessage::Leave(token) => {
                if let Some((user, _)) = players.remove(&token) {
                    info!("[{code}] {:?} left the game", user);
                } else {
                    warn!("there was no user in this game with this token");
                }
                // Send update to all clients
                players
                    .values()
                    .map(|(_, s)| s)
                    .send_to_all(ServerMessage::UpdatePlayers(player_names(&players)))
                    .await;
            }
            GameMessage::ClientMessage {
                message: ClientMessage::ChatMessage(msg),
                token,
            } => {
                if let Some((user, _)) = &players.get(&token) {
                    let guess = word.guess(msg.clone());
                    let color = match guess {
                        GuessResult::Miss => {
                            info!("[{code}] {} guessed wrong", user.nickname);
                            tries_used += 1;
                            ChatColor::Red
                        },
                        GuessResult::Hit => {
                            info!("[{code}] {} guessed right", user.nickname);
                            ChatColor::Green
                        }
                        GuessResult::Solved => {
                            info!("[{code}] {} solved the word", user.nickname);
                            ChatColor::Green
                        }
                    };
                    let message = ChatMessage {
                        from: user.nickname.clone(),
                        content: msg,
                        color,
                    };
                    chat.push(message.clone());
                    players
                        .values()
                        .map(|(_, s)| s)
                        .send_to_all(ServerMessage::Guess {
                            message,
                            word: word.word(),
                            tries_used,
                        })
                        .await;
                    if guess == GuessResult::Solved {
                        players
                            .values()
                            .map(|(_, s)| s)
                            .send_to_all(ServerMessage::Solved)
                            .await;
                    }
                }
            }
        }
    }
}

fn player_names(players: &Players) -> Vec<String> {
    players.values().map(|(u, _)| u.nickname.clone()).collect()
}
