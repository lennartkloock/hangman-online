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
                onsubmit: move |_| {
                    log::debug!("Done");
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
                            class: "input-mono",
                            placeholder: "Code",
                            minlength: 4,
                            maxlength: 4,
                            required: true,
                            value: "{code}",
                        }
                    }
                    label {
                        class: "flex items-center gap-2",
                        MaterialIcon { name: "account_circle", color: MaterialIconColor::Light, size: 42 }
                        input {
                            class: "input",
                            placeholder: "Enter your name",
                            required: true,
                        }
                    }
                }
            }
        }
    ))
}
