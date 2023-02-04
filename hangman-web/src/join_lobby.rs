use crate::components::{CenterContainer, MaterialButton, MaterialLinkButton};
use dioxus::prelude::*;
use dioxus_material_icons::{MaterialIcon, MaterialIconColor};

pub fn JoinLobby(cx: Scope) -> Element {
    cx.render(rsx!(
        CenterContainer {
            div {
                class: "bg-zinc-800 rounded-xl w-80 max-w-[80%]",
                div {
                    class: "bg-zinc-700 rounded-t-xl p-2 flex justify-between items-center",
                    MaterialLinkButton { name: "arrow_back", to: "/" }
                    span {
                        class: "font-light",
                        "Join Lobby"
                    }
                    MaterialButton { name: "done", onclick: move |_| log::debug!("Done") }
                }
                div {
                    class: "m-8 flex flex-col gap-2",
                    label {
                        class: "flex items-center gap-2",
                        MaterialIcon { name: "numbers", color: MaterialIconColor::Light, size: 42 }
                        input {
                            class: "input-mono",
                            placeholder: "Code",
                            maxlength: 4,
                        }
                    }
                    label {
                        class: "flex items-center gap-2",
                        MaterialIcon { name: "account_circle", color: MaterialIconColor::Light, size: 42 }
                        input {
                            class: "input",
                            placeholder: "Enter your name",
                        }
                    }
                }
            }
        }
    ))
}