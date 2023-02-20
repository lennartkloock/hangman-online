use crate::components::{CenterContainer, Form, FormTopBar, MaterialButton, MaterialLinkButton};
use dioxus::prelude::*;
use dioxus_material_icons::{MaterialIcon, MaterialIconColor};
use dioxus_router::{use_route, use_router};

pub fn JoinLobby(cx: Scope) -> Element {
    let route = use_route(cx);
    let router = use_router(cx);

    let code = route
        .parse_segment::<String>("code")
        .and_then(|r| r.ok())
        .unwrap_or_default();

    cx.render(rsx!(
        CenterContainer {
            Form {
                onsubmit: move |e: FormEvent| {
                    // TODO: Provide feedback to the user
                    if let Some(nickname) = e.data.values.get("nickname") {
                        if let Some(local_storage) = web_sys::window().and_then(|w| w.local_storage().ok()).flatten() {
                            local_storage.set_item("hangman_user", nickname).unwrap();
                        }
                    }
                    router.navigate_to("/game");
                },
                FormTopBar {
                    MaterialLinkButton { name: "arrow_back", to: "/" }
                    span {
                        class: "font-light",
                        "Join Lobby"
                    }
                    MaterialButton { name: "done" }
                }
                div {
                    class: "m-8 flex flex-col gap-2",
                    label {
                        class: "flex items-center gap-2",
                        MaterialIcon { name: "numbers", color: MaterialIconColor::Light, size: 42 }
                        input {
                            class: "input p-1 w-full rounded font-mono",
                            placeholder: "Code",
                            minlength: 4,
                            maxlength: 4,
                            required: true,
                            name: "code",
                            value: "{code}",
                        }
                    }
                    label {
                        class: "flex items-center gap-2",
                        MaterialIcon { name: "account_circle", color: MaterialIconColor::Light, size: 42 }
                        input {
                            class: "input p-1 w-full rounded",
                            placeholder: "Enter your name",
                            required: true,
                            name: "nickname",
                        }
                    }
                }
            }
        }
    ))
}
