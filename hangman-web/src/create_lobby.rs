use crate::components::{CenterContainer, Form, FormTopBar, MaterialButton, MaterialLinkButton};
use dioxus::prelude::*;
use dioxus_router::use_router;

pub fn CreateLobby(cx: Scope) -> Element {
    let router = use_router(cx);

    cx.render(rsx!(
        CenterContainer {
            Form {
                onsubmit: move |_| {
                    router.navigate_to("/game");
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
