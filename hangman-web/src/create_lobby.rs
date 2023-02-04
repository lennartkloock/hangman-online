use crate::components::{CenterContainer, MaterialButton, MaterialLinkButton};
use dioxus::prelude::*;

pub fn CreateLobby(cx: Scope) -> Element {
    cx.render(rsx!(
        CenterContainer {
            div {
                class: "bg-zinc-800 rounded-xl w-80 max-w-[80%]",
                div {
                    class: "bg-zinc-700 rounded-t-xl p-2 flex justify-between items-center",
                    MaterialLinkButton { name: "arrow_back", to: "/" }
                    span {
                        class: "font-light",
                        "Create Lobby"
                    }
                    MaterialButton { name: "done", onclick: move |_| log::debug!("Done") }
                }
                div {
                    class: "m-2",
                    "TESTT"
                }
            }
        }
    ))
}
