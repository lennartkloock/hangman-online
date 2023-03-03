use crate::{
    components::{CenterContainer, Error, Form, FormTopBar, MaterialButton, MaterialLinkButton},
    create_user::CreateUser,
    global_state::USER,
    storage,
    storage::StorageError,
};
use dioxus::prelude::*;
use dioxus_router::use_router;
use fermi::{use_read, Atom};
use hangman_data::{CreateGameBody, GameCode, GameLanguage, GameSettings, User, UserToken};
use log::info;
use reqwest::Error;

pub fn CreateGame(cx: Scope) -> Element {
    let router = use_router(cx);
    let client = cx.use_hook(|| reqwest::Client::new());
    let error = use_state(cx, || Option::<reqwest::Error>::None);
    let user = use_read(cx, USER);

    match (user, error.get()) {
        (Ok(Some(user)), None) => {
            cx.render(rsx!(
                CenterContainer {
                    Form {
                        onsubmit: |_| {
                            let token = user.token; // Copies token
                            to_owned![router, client, error]; // Clones states
                            cx.spawn(async move {
                                let body = CreateGameBody { token, settings: GameSettings { language: GameLanguage::German } };
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
