use crate::components::{
    CenterContainer, Error, Form, FormTopBar, MaterialButton, MaterialLinkButton,
};
use dioxus::prelude::*;
use dioxus_router::use_router;
use hangman_data::{CreateGameBody, GameCode, GameLanguage, GameSettings, UserToken};
use log::info;
use reqwest::Error;
use std::convert::Infallible;

pub fn CreateGame(cx: Scope) -> Element {
    let router = use_router(cx);
    let client = cx.use_hook(|| reqwest::Client::new());
    let error = use_state(cx, || Option::<reqwest::Error>::None);

    match error.get() {
        None => {
            cx.render(rsx!(
                CenterContainer {
                    Form {
                        onsubmit: |_| {
                            to_owned![router, client, error];
                            cx.spawn(async move {
                                let body = CreateGameBody { token: UserToken::random(), settings: GameSettings { language: GameLanguage::German } };
                                match client.post("http://localhost:8000/api/game")
                                    .json(&body)
                                    .send()
                                    .await {
                                    Ok(res) => {
                                        match res.json::<GameCode>().await {
                                            Ok(code) => {
                                                info!("created game {code}");
                                                router.navigate_to(&format!("/game/{code}"));
                                            },
                                            Err(e) => error.set(Some(e)),
                                        }
                                    },
                                    Err(e) => error.set(Some(e)),
                                }
                            });
                        },
                        FormTopBar {
                            MaterialLinkButton { name: "arrow_back", to: "/" }
                            span {
                                class: "font-light",
                                "Create Game"
                            }
                            MaterialButton { name: "done" }
                        }
                        div {
                            class: "m-2",
                            "TESTT"
                        }
                    }
                }
            ))
        }
        Some(e) => {
            cx.render(rsx!(Error {
                title: "Failed to create game",
                error: e,
            }))
        }
    }
}
