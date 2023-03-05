use crate::game::ongoing_game::GameState;
use dioxus::prelude::*;
use hangman_data::ServerMessage;

pub fn handle_message(msg: ServerMessage, state: &UseState<GameState>) {
    match msg {
        ServerMessage::Pong => state.set(GameState::Joined { players: vec![] }),
    }
}
