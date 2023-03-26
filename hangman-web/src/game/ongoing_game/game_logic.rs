use crate::game::ongoing_game::ClientState;
use dioxus::prelude::*;
use hangman_data::{Game, ServerMessage};
use log::{info, warn};

pub fn handle_message(msg: ServerMessage, state: &UseRef<ClientState>) {
    match msg {
        ServerMessage::Init(data) => state.set(ClientState::Joined(data)),
        ServerMessage::UpdatePlayers(players) => state.with_mut(|s| {
            info!("updating player list: {players:?}");
            modify_game(s, |game| game.players = players);
        }),
        ServerMessage::UpdateGame { word, tries_used } => state.with_mut(|s| {
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
        ServerMessage::UpdateGameState(game_state) => state.with_mut(|s| {
            info!("new game state: {game_state:?}");
            modify_game(s, |game| game.state = game_state);
        }),
        ServerMessage::GameResult(r) => state.set(ClientState::GameResult(r)),
    }
}

fn modify_game<F: FnOnce(&mut Game)>(state: &mut ClientState, f: F) {
    if let ClientState::Joined(game) = state {
        f(game)
    } else {
        warn!("received update message before init");
    }
}
