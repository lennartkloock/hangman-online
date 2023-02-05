use crate::components::{CenterContainer, MaterialButton};
use dioxus::prelude::*;
use dioxus_material_icons::{MaterialIcon, MaterialIconColor};
use fermi::prelude::*;
use log::debug;

static LETTERS: AtomRef<Vec<char>> = |_| vec![];

pub fn game(cx: Scope) -> Element {
    let letters = use_atom_ref(cx, LETTERS);

    let value = use_state(cx, || "");

    let read = letters.read();
    let chat_messages = read.iter().rev().map(|l| {
        cx.render(rsx!(
            p { "{l}" }
        ))
    });

    cx.render(rsx!(
        div {
            class: "absolute top-2 left-2 flex items-center gap-1 p-1",
            span { class: "font-mono text-xl", "0XUA" }
            MaterialButton { name: "content_copy" }
        }
        div {
            class: "absolute top-2 right-2 flex items-center gap-1",
            button {
                class: "material-button gap-1 bg-zinc-700",
                MaterialIcon { name: "language", color: MaterialIconColor::Light, size: 35 }
                span { "German" }
            }
            MaterialButton { name: "settings" }
        }
        CenterContainer {
            div {
                class: "flex flex-col gap-8 items-center w-full",
                h1 {
                    class: "text-xl font-light",
                    "GUESS THE WORD"
                }
                Word { word: "Hangman" }
                div {
                    class: "flex flex-col gap-0 max-w-sm w-4/5",
                    div {
                        class: "bg-zinc-800 rounded-t-lg overflow-y-scroll px-2 py-1 font-light flex flex-col-reverse h-64",
                        chat_messages
                    }
                    form {
                        class: "w-full",
                        prevent_default: "onsubmit",
                        onsubmit: move |evt| {
                            debug!("On submit: {:?}", evt);
                            if let Some(c) = evt.values.get("letter").and_then(|s| s.chars().next()) {
                                letters.write().push(c.to_ascii_uppercase());
                                value.set("");
                            }
                        },
                        input {
                            class: "input w-full px-2 py-1 rounded-b-lg font-light",
                            r#type: "text",
                            name: "letter",
                            placeholder: "Guess something...",
                            value: "{value}",
                        }
                    }
                }
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
            if letters.read().contains(&c.to_ascii_uppercase()) {
                c
            } else {
                '_'
            }
        })
        .collect();

    cx.render(rsx!(p {
        class: "text-6xl font-mono tracking-[.25em]",
        rendered_word
    }))
}
