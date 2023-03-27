use crate::{
    components::{CenterContainer, Error, Form, MaterialButton, MaterialLinkButton, TopBar},
    create_user::CreateUser,
    global_state::USER,
    urls,
    urls::UrlError,
};
use dioxus::prelude::*;
use dioxus_material_icons::{MaterialIcon, MaterialIconColor};
use dioxus_router::use_router;
use fermi::use_read;
use hangman_data::{CreateGameBody, Difficulty, GameCode, GameLanguage, GameMode, GameSettings};
use log::{error, info};
use thiserror::Error;

#[derive(Debug, Error)]
enum CreateGameError {
    #[error("failed to parse form fields")]
    FormParseError,
    #[error("failed to retrieve url: {0}")]
    UrlError(#[from] UrlError),
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
}

pub fn CreateGame(cx: Scope) -> Element {
    let router = use_router(cx);
    let client = cx.use_hook(reqwest::Client::new);
    let error = use_state(cx, || Option::<CreateGameError>::None);
    let user = use_read(cx, USER);

    match (user, error.get()) {
        (Ok(Some(user)), None) => {
            cx.render(rsx!(
                CenterContainer {
                    Form {
                        onsubmit: |e: FormEvent| {
                            let mode = e.data.values.get("mode").and_then(|s| serde_json::from_str::<GameMode>(s).ok());
                            let lang = e.data.values.get("language").and_then(|s| serde_json::from_str::<GameLanguage>(s).ok());
                            let diff = e.data.values.get("difficulty").and_then(|s| serde_json::from_str::<Difficulty>(s).ok());
                            if let (Some(mode), Some(language), Some(difficulty)) = (mode, lang, diff) {
                                match urls::http_url_origin() {
                                    Ok(origin) => {
                                        let token = user.token; // Copies token
                                        to_owned![router, client, error]; // Clones states
                                        cx.spawn(async move {
                                            let body = CreateGameBody { token, settings: GameSettings { mode, language, difficulty } };
                                            match client.post(format!("{origin}/api/game"))
                                                .json(&body)
                                                .send()
                                                .await {
                                                Ok(res) => {
                                                    match res.json::<GameCode>().await {
                                                        Ok(code) => {
                                                            info!("created game {code}");
                                                            router.navigate_to(&format!("/game/{code}"));
                                                        },
                                                        Err(e) => error.set(Some(e.into())),
                                                    }
                                                },
                                                Err(e) => error.set(Some(e.into())),
                                            }
                                        });
                                    },
                                    Err(e) => error.set(Some(e.into())),
                                }
                            } else {
                                error!("failed to parse language from form");
                                error.set(Some(CreateGameError::FormParseError));
                            }
                        },
                        TopBar {
                            MaterialLinkButton { name: "arrow_back", to: "/" }
                            span {
                                class: "font-light",
                                "Create Game"
                            }
                            MaterialButton { name: "done" }
                        }
                        div {
                            class: "p-6 flex flex-col gap-1",
                            label {
                                class: "flex items-center gap-2",
                                MaterialIcon { name: "people", color: MaterialIconColor::Light, size: 42 },
                                select {
                                    class: "input p-1 w-full rounded",
                                    required: true,
                                    name: "mode",
                                    GameMode::all().iter().map(|m| {
                                        let value = serde_json::to_string(&m).expect("failed to serialize game mode");
                                        rsx!(option { value: "{value}", "{m}" })
                                    })
                                }
                            }
                            label {
                                class: "flex items-center gap-2",
                                MaterialIcon { name: "language", color: MaterialIconColor::Light, size: 42 },
                                select {
                                    class: "input p-1 w-full rounded",
                                    required: true,
                                    name: "language",
                                    GameLanguage::all().iter().map(|l| {
                                        let value = serde_json::to_string(&l).expect("failed to serialize language");
                                        rsx!(option { value: "{value}", "{l}" })
                                    })
                                }
                            }
                            label {
                                class: "flex items-center gap-2",
                                MaterialIcon { name: "star", color: MaterialIconColor::Light, size: 42 },
                                select {
                                    class: "input p-1 w-full rounded",
                                    required: true,
                                    name: "difficulty",
                                    Difficulty::all().iter().map(|d| {
                                        let is_default = *d == Difficulty::default();
                                        let value = serde_json::to_string(&d).expect("failed to serialize difficulty");
                                        rsx!(option { value: "{value}", selected: is_default, "{d}" })
                                    })
                                }
                            }
                        }
                    }
                }
            ))
        }
        (Ok(None), _) => cx.render(rsx!(CreateUser {})),
        (Err(e), _) => cx.render(rsx!(Error {
            title: "Failed to load user",
            error: e,
        })),
        (_, Some(e)) => cx.render(rsx!(Error {
            title: "Failed to create game",
            error: e,
        })),
    }
}
