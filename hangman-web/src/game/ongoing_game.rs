use crate::{
    components::{CenterContainer, MaterialButton},
    game::GameCode,
};
use dioxus::prelude::*;
use dioxus_material_icons::{MaterialIcon, MaterialIconColor};
use dioxus_router::use_router;
use fermi::prelude::*;
use log::debug;

static LETTERS: AtomRef<Vec<char>> = |_| vec![];

#[inline_props]
pub fn OngoingGame(cx: Scope<OngoingGameProps>, code: GameCode) -> Element {
    cx.render(rsx!(
        Header { code: code }
        div {
            class: "h-full flex items-center",
            div {
                class: "grid game-container gap-y-2 w-full",
                Players {}
                h1 {
                    class: "text-xl font-light text-center",
                    style: "grid-area: title",
                    "GUESS THE WORD"
                }
                Word { word: "Hangman" }
                Chat {}
                Hangman {}
            }
        }
    ))
}

#[inline_props]
fn Header<'a>(cx: Scope<'a>, code: &'a GameCode) -> Element<'a> {
    let router = use_router(cx);

    let on_copy = move |_| {
        // TODO: Provide feedback to the user
        if let Some(c) = web_sys::window().and_then(|w| w.navigator().clipboard()) {
            let mut url = router.current_location().url.clone();
            url.set_path(&format!("/game/{}", code));
            cx.spawn(async move {
                if wasm_bindgen_futures::JsFuture::from(c.write_text(url.as_str()))
                    .await
                    .is_err()
                {
                    // Write failed, no permission
                    todo!();
                }
            });
        } else {
            todo!();
        }
    };

    cx.render(rsx!(
        div {
            class: "absolute top-2 left-2 flex items-center gap-1 p-1",
            span { class: "font-mono text-xl", "{code}" }
            MaterialButton { name: "content_copy", onclick: on_copy }
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
    ))
}

fn Players(cx: Scope) -> Element {
    cx.render(rsx!(
        div {
            style: "grid-area: players",
            class: "justify-self-start bg-zinc-800 p-2 rounded-r-lg",
            "Players"
        }
    ))
}

fn Chat(cx: Scope) -> Element {
    let letters = use_atom_ref(cx, LETTERS);
    let read = letters.read();
    let chat_messages = read.iter().rev().map(|l| {
        cx.render(rsx!(
            p { "{l}" }
        ))
    });

    let value = use_state(cx, || "");
    let on_letter_submit = move |evt: FormEvent| {
        if let Some(c) = evt.values.get("letter").and_then(|s| s.chars().next()) {
            letters.write().push(c.to_ascii_uppercase());
            value.set("");
        }
    };

    cx.render(rsx!(
        div {
            class: "flex flex-col gap-0",
            style: "grid-area: chat",
            div {
                class: "bg-zinc-800 rounded-t-lg overflow-y-auto px-2 py-1 font-light flex flex-col-reverse h-64",
                chat_messages
            }
            form {
                class: "w-full",
                prevent_default: "onsubmit",
                onsubmit: on_letter_submit,
                input {
                    class: "input w-full px-2 py-1 rounded-b-lg font-light",
                    r#type: "text",
                    name: "letter",
                    placeholder: "Guess something...",
                    value: "{value}",
                }
            }
        }
    ))
}

#[inline_props]
fn Word<'a>(cx: Scope<'a>, word: &'a str) -> Element<'a> {
    let letters = use_atom_ref(cx, LETTERS);

    let rendered_word: String = word
        .chars()
        .map(|c| {
            if letters.read().contains(&c.to_ascii_uppercase()) {
                c
            } else {
                '_'
            }
        })
        .collect();

    cx.render(rsx!(pre {
        class: "text-6xl font-mono tracking-[.25em] mr-[-.25em] text-center px-2",
        style: "grid-area: word",
        rendered_word
    }))
}

fn Hangman(cx: Scope) -> Element {
    cx.render(rsx!(
        div {
            style: "grid-area: hangman",
            class: "flex justify-center items-center",
            "0/10"
        }
    ))
}
