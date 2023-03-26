use crate::{
    components::{CenterContainer, MaterialButton, RcError},
    game::{
        ongoing_game::{hangman::Hangman, ws_logic::connect},
        GameCode,
    },
    urls,
    urls::UrlError,
};
use chrono::Utc;
use dioxus::prelude::*;
use dioxus_material_icons::{MaterialIcon, MaterialIconColor};
use dioxus_router::use_router;
use gloo_net::websocket::WebSocketError;
use gloo_utils::errors::JsError;
use hangman_data::{ChatColor, ChatMessage, ClientMessage, Game, GameSettings, GameState, User};
use log::error;
use std::{rc::Rc, time::Duration};
use thiserror::Error;

mod game_logic;
mod hangman;
mod ws_logic;

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("failed to retrieve url: {0}")]
    UrlError(#[from] UrlError),

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
    #[error("this game was closed")]
    GameClosed,
}

impl ConnectionError {
    pub fn rc(self) -> Rc<Self> {
        Rc::new(self)
    }
}

pub enum ClientState {
    /// waiting for connection and init message
    Loading,
    Joined(Game),
    GameResult(Vec<(String, u32)>),
    /// Rc to make it cloneable
    Error(Rc<ConnectionError>),
}

#[inline_props]
pub fn OngoingGame<'a>(cx: Scope<'a>, code: GameCode, user: &'a User) -> Element<'a> {
    let router = use_router(cx);

    let state = use_ref(cx, || ClientState::Loading);

    let (ws_tx, ws_rx) = cx.use_hook(|| match urls::game_ws_url(code, user) {
        Ok(url) => connect(state, url),
        Err(e) => {
            state.set(ClientState::Error(ConnectionError::UrlError(e).rc()));
            (None, None)
        }
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
        ClientState::Loading => cx.render(rsx!(
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
        ClientState::Error(e) => {
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
        ClientState::Joined(Game {
            settings,
            state,
            players,
            chat,
            tries_used,
            word,
            countdown,
        }) => cx.render(rsx!(
            Header { code: *code, settings: settings.clone(), countdown: *countdown }
            div {
                class: "h-full flex items-center",
                div {
                    class: "grid game-container gap-y-2 w-full",

                    // Players
                    div {
                        style: "grid-area: players",
                        class: "justify-self-start bg-zinc-800 p-2 rounded-r-lg flex flex-col",
                        ul {
                            class: "flex flex-col gap-2 grow",
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
                        button {
                            class: "base-button hover:bg-red-700/70 ring-zinc-700/50",
                            onclick: move |_| router.navigate_to("/"),
                            "Leave"
                        }
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

                    Chat {
                        game_state: state.clone(),
                        chat: chat.clone(),
                        ws_write: ws_write
                    }

                    // Hangman
                    Hangman { tries_used: *tries_used }
                }
            }
            Footer { game_state: state.clone(), ws_write: ws_write }
        )),
        ClientState::GameResult(results) => cx.render(rsx!(
            p { "{results:?}" }
        )),
    })
}

#[derive(PartialEq, Props)]
struct HeaderProps {
    code: GameCode,
    settings: GameSettings,
    #[props(!optional)]
    countdown: Option<chrono::DateTime<Utc>>,
}

fn Header(cx: Scope<HeaderProps>) -> Element {
    let router = use_router(cx);

    let on_copy = move |_| {
        // TODO: Provide feedback to the user
        // Fixme: Doesn't work in other browsers than FF
        match web_sys::window().and_then(|w| w.navigator().clipboard()) {
            Some(c) => {
                let mut url = router.current_location().url.clone();
                url.set_path(&format!("/game/{}", cx.props.code));
                cx.spawn(async move {
                    if let Err(e) =
                        wasm_bindgen_futures::JsFuture::from(c.write_text(url.as_str())).await
                    {
                        error!("failed to write to clipboard, no permission: {e:?}");
                    }
                });
            }
            None => {
                error!("failed to retrieve clipboard");
            }
        }
    };

    let countdown_text = use_state(cx, || "".to_string());

    use_coroutine(cx, |_: UnboundedReceiver<()>| {
        to_owned![countdown_text];
        let countdown = cx.props.countdown;
        async move {
            if let Some(date_time) = countdown {
                while let Some(dur) = {
                    let dur = date_time - Utc::now();
                    (dur > chrono::Duration::zero()).then_some(dur)
                } {
                    countdown_text.set(format!(
                        "{:0>2}:{:0>2}", //Keep leading zeros
                        dur.num_minutes(),
                        dur.num_seconds() % 60
                    ));
                    gloo_timers::future::sleep(Duration::from_millis(100)).await;
                }
                countdown_text.set("Time is up!".to_string());
            }
        }
    });

    let lang = &cx.props.settings.language;

    cx.render(rsx!(
        div {
            class: "absolute top-2 left-2 right-2 flex justify-between",
            div {
                class: "flex items-center gap-1 p-1",
                span { class: "font-mono text-xl", "{cx.props.code}" }
                MaterialButton { name: "content_copy", onclick: on_copy }
            }
            div {
                class: "font-mono text-2xl p-1",
                "{countdown_text}"
            }
            div {
                class: "flex items-center gap-1",
                button {
                    class: "material-button gap-1 bg-zinc-700",
                    MaterialIcon { name: "language", color: MaterialIconColor::Light, size: 35 }
                    span { "{lang}" }
                }
                MaterialButton { name: "settings" }
            }
        }
    ))
}

#[inline_props]
fn Footer<'a>(
    cx: Scope<'a>,
    game_state: GameState,
    ws_write: &'a Coroutine<ClientMessage>,
) -> Element<'a> {
    let button = (*game_state == GameState::RoundFinished).then(|| {
        cx.render(rsx!(
            button {
                class: "base-button ring-zinc-500 py-1",
                onclick: move |_| ws_write.send(ClientMessage::NextRound),
                "Next Round →"
            }
        ))
    });

    cx.render(rsx!(div {
        class: "absolute bottom-2 right-2 flex items-center gap-1 p-1",
        button
    }))
}

#[inline_props]
fn Chat<'a>(
    cx: Scope<'a>,
    game_state: GameState,
    chat: Vec<ChatMessage>,
    ws_write: &'a Coroutine<ClientMessage>,
) -> Element<'a> {
    let value = use_state(cx, String::new);
    let on_letter_submit = move |evt: FormEvent| {
        if let Some(msg) = evt.values.get("letter") {
            if !msg.is_empty() {
                ws_write.send(ClientMessage::ChatMessage(msg.to_string()));
                value.set(String::new());
            }
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
                            None => content.to_string(),
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
                    disabled: *game_state != GameState::Playing,
                    value: "{value}",
                    oninput: move |e| value.set(e.data.value.to_string()),
                }
            }
        }
    ))
}
