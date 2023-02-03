#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::{Route, Router};
use fermi::{use_atom_ref, use_init_atom_root, AtomRef};
use log::{debug, Level};

fn main() {
    console_log::init_with_level(Level::Trace).expect("Error initializing logger");
    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    use_init_atom_root(cx);

    cx.render(rsx!(
        Router {
            style { include_str!("../out/output.css") }
            h1 { "Hangman" }
            Route { to: "/game", self::game {} }
            Route { to: "", h1 { "Not Found" } }
        }
    ))
}

static LETTERS: AtomRef<Vec<char>> = |_| vec![];

fn game(cx: Scope) -> Element {
    let letters = use_atom_ref(cx, LETTERS);

    let value = use_state(cx, || "");

    cx.render(rsx!(
        Word { word: "Hangman" }
        form {
            prevent_default: "onsubmit",
            onsubmit: move |evt| {
                debug!("On submit: {:?}", evt);
                if let Some(c) = evt.values.get("letter").and_then(|s| s.chars().next()) {
                    letters.write().push(c.to_ascii_lowercase());
                    value.set("");
                }
            },
            input {
                name: "letter",
                value: "{value}",
                r#type: "text",
                maxlength: 1,
            }
        }
    ))
}

#[derive(Props)]
struct WordProps<'a> {
    word: &'a str,
}

fn Word<'a>(cx: Scope<'a, WordProps<'a>>) -> Element<'a> {
    let letters = use_atom_ref(cx, LETTERS);

    let rendered_word: String = cx
        .props
        .word
        .chars()
        .map(|c| {
            if letters.read().contains(&c.to_ascii_lowercase()) {
                c
            } else {
                '_'
            }
        })
        .collect();

    cx.render(rsx!(
        p {
            b {
                style: "letter-spacing: .2rem;",
                rendered_word
            }
        }
    ))
}
