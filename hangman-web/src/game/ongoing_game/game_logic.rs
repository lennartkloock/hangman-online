use crate::game::ongoing_game::GameState;
use dioxus::prelude::*;
use hangman_data::{Game, ServerMessage};
use log::{info, warn};

pub fn handle_message(msg: ServerMessage, state: &UseRef<GameState>) {
    match msg {
        ServerMessage::Init(data) => state.set(GameState::Joined(data)),
        ServerMessage::UpdatePlayers(players) => state.with_mut(|s| {
            modify_game(s, |game| {
                info!("updating player list: {players:?}");
                game.players = players;
            });
        }),
        ServerMessage::Guess {
            message,
            word,
            tries_used,
            solved,
        } => state.with_mut(|s| {
            modify_game(s, |game| {
                info!("new guess: {message:?}");
                game.chat.push(message);
                game.word = word;
                game.tries_used = tries_used;
            });
        }),
    }
}

fn modify_game<F: FnOnce(&mut Game)>(state: &mut GameState, f: F) {
    if let GameState::Joined(game) = state {
        f(game)
    } else {
        warn!("received update message before init");
    }
}
