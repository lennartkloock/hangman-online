use crate::game::ongoing_game::GameState;
use dioxus::prelude::*;
use hangman_data::ServerMessage;
use log::{info, warn};

pub fn handle_message(msg: ServerMessage, state: &UseRef<GameState>) {
    match msg {
        ServerMessage::Init(data) => state.set(GameState::Joined(data)),
        ServerMessage::UpdatePlayers(players) => state.with_mut(|s| {
            if let GameState::Joined(game) = s {
                info!("updating player list: {players:?}");
                game.players = players;
            } else {
                warn!("received update message before init");
            }
        }),
        ServerMessage::NewMessage(msg) => state.with_mut(|s| {
            if let GameState::Joined(game) = s {
                info!("new chat message: {msg:?}");
                game.chat.push(msg);
            } else {
                warn!("received update message before init");
            }
        }),
        ServerMessage::UpdateTriesUsed(tries_used) => state.with_mut(|s| {
            if let GameState::Joined(game) = s {
                info!("updating tries used: {tries_used}");
                game.tries_used = tries_used;
            } else {
                warn!("received update message before init");
            }
        }),
    }
}
