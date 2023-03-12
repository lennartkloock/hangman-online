use crate::game::ongoing_game::GameState;
use dioxus::prelude::*;
use hangman_data::{Game, ServerMessage};
use log::{info, warn};

pub fn handle_message(msg: ServerMessage, state: &UseRef<GameState>) {
    match msg {
        ServerMessage::Init(data) => state.set(GameState::Joined(data)),
        ServerMessage::UpdatePlayers(players) => state.with_mut(|s| {
            info!("updating player list: {players:?}");
            modify_game(s, |game| game.players = players);
        }),
        ServerMessage::UpdateGame {
            word,
            tries_used,
        } => state.with_mut(|s| {
            info!("new guess: {word}");
            modify_game(s, |game| {
                game.word = word;
                game.tries_used = tries_used;
            });
        }),
        ServerMessage::ChatMessage(message) => state.with_mut(|s| {
            info!("new chat message: {message:?}");
            modify_game(s, |game| game.chat.push(message));
        }),
        ServerMessage::Solved => {
            info!("game was solved");
        }
        ServerMessage::GameOver => {
            info!("game was lost");
        }
    }
}

fn modify_game<F: FnOnce(&mut Game)>(state: &mut GameState, f: F) {
    if let GameState::Joined(game) = state {
        f(game)
    } else {
        warn!("received update message before init");
    }
}
