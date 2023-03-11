use crate::{
    components::{CenterContainer, Error, Form, FormTopBar, MaterialButton, MaterialLinkButton},
    create_user::CreateUser,
    global_state::USER,
};
use dioxus::prelude::*;
use dioxus_material_icons::{MaterialIcon, MaterialIconColor};
use dioxus_router::use_router;
use fermi::use_read;
use hangman_data::{CreateGameBody, GameCode, GameLanguage, GameSettings};
use log::{error, info};

pub fn CreateGame(cx: Scope) -> Element {
    let router = use_router(cx);
    let client = cx.use_hook(reqwest::Client::new);
    let error = use_state(cx, || Option::<reqwest::Error>::None);
    let user = use_read(cx, USER);

    match (user, error.get()) {
        (Ok(Some(user)), None) => {
            cx.render(rsx!(
                CenterContainer {
                    Form {
                        onsubmit: |e: FormEvent| {
                            if let Some(Ok(language)) = e.data.values.get("language").map(|s| serde_json::from_str::<GameLanguage>(s)) {
                                let token = user.token; // Copies token
                                to_owned![router, client, error]; // Clones states
                                cx.spawn(async move {
                                    let body = CreateGameBody { token, settings: GameSettings { language } };
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
                            } else {
                                error!("failed to parse language from form");
                            }
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
