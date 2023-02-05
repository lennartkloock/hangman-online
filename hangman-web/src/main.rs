#![allow(non_snake_case)]

use crate::components::{CenterContainer, LinkButton};
use dioxus::prelude::*;
use dioxus_material_icons::MaterialIconStylesheet;
use dioxus_router::{Route, Router};
use fermi::use_init_atom_root;

mod components;
mod create_lobby;
mod game;
mod home;
mod join_lobby;

fn main() {
    console_log::init_with_level(log::Level::Trace).expect("Error initializing logger");
    log::info!("Starting app...");
    dioxus_web::launch(App);
}

fn App(cx: Scope) -> Element {
    use_init_atom_root(cx);

    cx.render(rsx!(
        Router {
            MaterialIconStylesheet {}
            Route { to: "/", home::Home {} }
            Route { to: "/create", create_lobby::CreateLobby {} }
            Route { to: "/join", join_lobby::JoinLobby {} }
            Route { to: "/join/:code", join_lobby::JoinLobby {} }
            Route { to: "/game", game::game {} }
            Route { to: "", NotFound {} }
        }
    ))
}

fn NotFound(cx: Scope) -> Element {
    cx.render(rsx!(
        CenterContainer {
            p {
                h1 { "Not Found" }
                p { "This page was not found" }
                LinkButton { to: "/", "Return home" }
            }
        }
    ))
}
