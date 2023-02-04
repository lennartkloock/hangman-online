use crate::components::{CenterContainer, MaterialButton, MaterialLinkButton};
use dioxus::prelude::*;

pub fn CreateLobby(cx: Scope) -> Element {
    cx.render(rsx!(
        CenterContainer {
            form {
                class: "form",
                prevent_default: "onsubmit",
                onsubmit: move |_| {
                    log::info!("Create");
                },
                div {
                    class: "form-top-bar",
                    MaterialLinkButton { name: "arrow_back", to: "/" }
                    span {
                        class: "font-light",
                        "Create Lobby"
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
