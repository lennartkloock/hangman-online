#![allow(non_snake_case)]

use crate::components::{CenterContainer, LinkButton};
use dioxus::prelude::*;
use dioxus_router::{Route, Router};
use fermi::use_init_atom_root;

mod components;
mod game;
mod home;

fn main() {
    console_log::init_with_level(log::Level::Trace).expect("Error initializing logger");
    dioxus_web::launch(App);
}

fn App(cx: Scope) -> Element {
    use_init_atom_root(cx);

    cx.render(rsx!(
        Router {
            style { include_str!("../out/output.css") } // TailwindCSS styles
            Route { to: "/", home::home {} }
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
