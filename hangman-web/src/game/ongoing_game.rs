use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_material_icons::{MaterialIcon, MaterialIconColor};
use dioxus_router::use_router;
use gloo_net::websocket::WebSocketError;
use gloo_utils::errors::JsError;
use hangman_data::ChatColor;
use thiserror::Error;

use hangman_data::{ChatMessage, ClientMessage, Game, GameSettings, User};

use crate::{
    components::{CenterContainer, MaterialButton, RcError},
    game::{ongoing_game::ws_logic::connect, GameCode},
};

mod game_logic;
mod ws_logic;

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("failed to establish a connection due to syntax error: {0}")]
    SyntaxError(JsError),
    #[error("failed to serialize message: {0}")]
    SerializeError(serde_json::Error),
    #[error("failed to send message: {0}")]
    SendError(JsError),

    #[error("failed to deserialize message: {0}")]
    DeserializeError(serde_json::Error),
    #[error("failed to deserialize message due to wrong data type")]
    DeserializeWrongDataTypeError,

    #[error("websocket error: {0}")]
    WsError(#[from] WebSocketError),

    #[error("this game doesn't exist")]
    GameNotFound,
    #[error("this game closed")]
    GameClosed,
}

impl ConnectionError {
    pub fn rc(self) -> Rc<Self> {
        Rc::new(self)
    }
}

pub enum GameState {
    /// waiting for connection and init message
    Loading,
    Joined(Game),
    /// Rc to make it cloneable
    Error(Rc<ConnectionError>),
}

#[inline_props]
pub fn OngoingGame<'a>(cx: Scope<'a>, code: GameCode, user: &'a User) -> Element<'a> {
    let state = use_ref(cx, || GameState::Loading);

    let (ws_tx, ws_rx) = cx.use_hook(|| {
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("nickname", &user.nickname)
            .append_pair("token", &format!("{}", user.token))
            .finish();
        connect(
            state,
            format!("ws://localhost:8000/api/game/{code}/ws?{query}"),
        )
    });
    let _ws_read: &Coroutine<()> = use_coroutine(cx, |_| {
        to_owned![state];
        ws_logic::ws_read(ws_rx.take(), state)
    });
    let ws_write: &Coroutine<ClientMessage> = use_coroutine(cx, |rx| {
        to_owned![state];
        ws_logic::ws_write(rx, ws_tx.take(), state)
    });

    state.with(|s| match s {
        GameState::Loading => cx.render(rsx!(
            CenterContainer {
                div {
                    class: "flex flex-col gap-2",
                    div { class: "race-by" }
                    p {
                        class: "text-2xl",
                        "Joining..."
                    }
                }
            }
        )),
        GameState::Error(e) => {
            let title = match **e {
                ConnectionError::GameNotFound => "Game not found",
                ConnectionError::GameClosed => "The game was closed",
                _ => "Connection error",
            };
            cx.render(rsx!(RcError {
                title: title,
                error: Rc::clone(e),
            }))
        }
        GameState::Joined(Game {
            settings,
            players,
            chat,
            tries_used,
            word,
        }) => cx.render(rsx!(
            Header { code: code, settings: settings.clone() }
            div {
                class: "h-full flex items-center",
                div {
                    class: "grid game-container gap-y-2 w-full",

                    // Players
                    ul {
                        style: "grid-area: players",
                        class: "justify-self-start bg-zinc-800 p-2 rounded-r-lg flex flex-col gap-2",
                        players
                            .iter()
                            .map(|p| rsx!(
                                li {
                                    class: "flex items-center gap-1",
                                    MaterialIcon { name: "account_circle", color: MaterialIconColor::Light, size: 30 }
                                    "{p}"
                                }
                            ))
                    }

                    // Word
                    h1 {
                        class: "text-xl font-light text-center",
                        style: "grid-area: title",
                        "GUESS THE WORD"
                    }
                    pre {
                        class: "text-6xl font-mono tracking-[.25em] mr-[-.25em] text-center px-2",
                        style: "grid-area: word",
                        "{word}"
                    }

                    Chat { chat: chat.clone(), ws_write: ws_write }

                    // Hangman
                    div {
                        style: "grid-area: hangman",
                        class: "flex justify-center items-center",
                        "{tries_used}/10"
                    }
                }
            }
        )),
    })
}

#[inline_props]
fn Header<'a>(cx: Scope<'a>, code: &'a GameCode, settings: GameSettings) -> Element<'a> {
    let router = use_router(cx);

    let on_copy = move |_| {
        // TODO: Provide feedback to the user
        if let Some(c) = web_sys::window().and_then(|w| w.navigator().clipboard()) {
            let mut url = router.current_location().url.clone();
            url.set_path(&format!("/game/{code}"));
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

    let lang = &settings.language;

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
                span { "{lang}" }
            }
            MaterialButton { name: "settings" }
        }
    ))
}

#[inline_props]
fn Chat<'a>(
    cx: Scope<'a>,
    chat: Vec<ChatMessage>,
    ws_write: &'a Coroutine<ClientMessage>,
) -> Element<'a> {
    let value = use_state(cx, || "");
    let on_letter_submit = move |evt: FormEvent| {
        if let Some(msg) = evt.values.get("letter") {
            ws_write.send(ClientMessage::ChatMessage(msg.to_string()));
            value.set("");
        }
    };

    cx.render(rsx!(
        div {
            class: "flex flex-col gap-0",
            style: "grid-area: chat",
            ul {
                class: "bg-zinc-800 rounded-t-lg overflow-y-auto font-light flex flex-col-reverse h-64",
                chat.iter()
                    .rev()
                    .map(|ChatMessage { from, content, color }| {
                        let color_class = match color {
                            ChatColor::Neutral => "",
                            ChatColor::Green => "bg-green-900/30",
                            ChatColor::Red => "bg-red-900/30",
                        };
                        let text = match from {
                            Some(from) => format!("{from}: {content}"),
                            None => format!("{content}"),
                        };
                        rsx!(li {
                            class: "{color_class} px-2 py-0.5",
                            "{text}"
                        })
                    })
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
