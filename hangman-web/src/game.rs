use crate::{
    components::Error, create_user::CreateUser, game::ongoing_game::OngoingGame,
    global_state::USER,
};
use dioxus::prelude::*;
use dioxus_router::use_route;
use fermi::use_read;
use hangman_data::GameCode;
use std::convert::Infallible;

mod ongoing_game;

pub fn Game(cx: Scope) -> Element {
    let route = use_route(cx);

    let code = route.parse_segment::<GameCode>("code");
    let user = use_read(cx, USER);

    match (code, user) {
        // Render game
        (Some(Ok(code)), Ok(Some(_user))) => cx.render(rsx!(OngoingGame { code: code })),

        // Invalid game code
        (Some(Err(e)), _) => cx.render(rsx!(Error {
            title: "Invalid code",
            error: e
        })),
        // No game code found
        (None, _) => cx.render(rsx!(
            // Any type that implements Error
            Error::<Infallible> { title: "No code" }
        )),

        // No saved user
        (_, Ok(None)) => cx.render(rsx!(CreateUser {})),
        // Erroneous user
        (_, Err(e)) => cx.render(rsx!(Error {
            title: "Failed to load user",
            error: e
        })),
    }
}
