use crate::{
    components::{CenterContainer, Error, Form, FormTopBar, MaterialButton, MaterialLinkButton},
    game::USER,
    storage,
    storage::User,
};
use dioxus::prelude::*;
use dioxus_material_icons::{MaterialIcon, MaterialIconColor};
use fermi::use_set;
use log::error;

pub fn CreateUser(cx: Scope) -> Element {
    let set_user = use_set(cx, USER);
    let error = use_state(cx, || None);

    if let Some(err) = error.get() {
        cx.render(rsx!(Error {
            title: "Failed to create user",
            error: err
        }))
    } else {
        cx.render(rsx!(
        CenterContainer {
            Form {
                onsubmit: move |e: FormEvent| {
                    if let Some(nickname) = e.data.values.get("nickname") {
                        let new_user = User::new(nickname);
                        match storage::store("hangman_user", &new_user) {
                            Ok(_) => set_user(Ok(Some(new_user))),
                            Err(e) => error.set(Some(e))
                        }
                    } else {
                        error!("Failed to parse nickname form field");
                    }
                },
                FormTopBar {
                    MaterialLinkButton { name: "arrow_back", to: "/" }
                    span {
                        class: "font-light",
                        "Create user"
                    }
                    MaterialButton { name: "done" }
                }
                div {
                    class: "p-6 flex flex-col",
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
}
