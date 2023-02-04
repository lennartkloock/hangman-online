use crate::components::{CenterContainer, Form, FormTopBar, MaterialButton, MaterialLinkButton};
use dioxus::prelude::*;

pub fn CreateLobby(cx: Scope) -> Element {
    cx.render(rsx!(
        CenterContainer {
            Form {
                onsubmit: move |_| {
                    log::info!("Create");
                },
                FormTopBar {
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
