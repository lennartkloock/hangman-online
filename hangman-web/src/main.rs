#![allow(non_snake_case)]

use dioxus::prelude::*;
use fermi::{AtomRef, use_atom_ref, use_init_atom_root};
use log::{debug, Level};

fn main() {
    console_log::init_with_level(Level::Debug).expect("Error initializing logger");
    dioxus_web::launch(root);
}

static LETTERS: AtomRef<Vec<char>> = |_| vec![];

fn root(cx: Scope) -> Element {
    use_init_atom_root(cx);

    let letters = use_atom_ref(cx, LETTERS);

    let value = use_state(cx, || "");

    cx.render(rsx!(
        style { include_str!("../out/output.css") }
        h1 { "Hangman" }
        Word { word: "Hangman" }
        form {
            prevent_default: "onsubmit",
            onsubmit: move |evt| {
                debug!("On submit: {:?}", evt);
                if let Some(Some(c)) = evt.values.get("letter").map(|s| s.chars().nth(0)) {
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

    let rendered_word: String = cx.props.word.chars().map(|c| {
        if letters.read().contains(&c.to_ascii_lowercase()) {
            c
        } else {
            '_'
        }
    }).collect();

    cx.render(rsx!(
        p {
            b {
                style: "letter-spacing: .2rem;",
                rendered_word
            }
        }
    ))
}
