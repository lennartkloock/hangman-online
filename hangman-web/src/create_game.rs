use crate::components::{CenterContainer, Form, FormTopBar, MaterialButton, MaterialLinkButton};
use dioxus::prelude::*;
use dioxus_router::use_router;
use hangman_data::{CreateGameBody, GameCode, GameLanguage, GameSettings, UserToken};
use log::info;

pub fn CreateGame(cx: Scope) -> Element {
    let router = use_router(cx);

    cx.render(rsx!(
        CenterContainer {
            Form {
                onsubmit: |_| {
                    to_owned![router];
                    cx.spawn(async move {
                        let client = reqwest::Client::new();
                        let body = CreateGameBody { token: UserToken::random(), settings: GameSettings { language: GameLanguage::German } };
                        let code: GameCode = client.post("http://localhost:8000/api/game")
                            .json(&body)
                            .send()
                            .await
                            .unwrap()
                            .json()
                            .await
                            .unwrap();
                        info!("Created game {code}");
                        router.navigate_to(&format!("/game/{code}"));
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
