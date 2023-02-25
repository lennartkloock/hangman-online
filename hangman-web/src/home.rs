use crate::components::{CenterContainer, LinkButton};
use dioxus::prelude::*;
use dioxus_material_icons::{MaterialIcon, MaterialIconColor};
use dioxus_router::{use_router, Link};
use std::time::Duration;

pub fn Home(cx: Scope) -> Element {
    let title = use_state(cx, || animate_title(0).unwrap());

    use_future(cx, (), |_| {
        let title = title.clone();
        async move {
            let mut i = 0;
            while let Some(new_title) = animate_title(i) {
                gloo_timers::future::sleep(Duration::from_millis(300)).await;
                title.set(new_title);
                i += 1;
            }
        }
    });

    cx.render(rsx!(
        CenterContainer {
            div {
                class: "flex flex-col gap-8 items-center",
                span {
                    class: "font-mono font-bold drop-shadow-2xl text-7xl tracking-widest",
                    "{title}"
                }
                div {
                    class: "flex flex-col gap-4",
                    LinkButton { to: "/create", "Create Lobby" }
                    JoinButton {}
                }
            }
        }
        Link {
            class: "absolute bottom-1 right-1 font-extralight underline hover:no-underline text-xs",
            to: "https://github.com/lennartkloock/hangman-online",
            external: true,
            "Open-source software by Lennart Kloock"
        }
    ))
}

fn animate_title(step: u32) -> Option<String> {
    match step {
        0 => Some("_______".to_string()),
        1 => Some("_a___a_".to_string()),
        2 => Some("_an__an".to_string()),
        3 => Some("Han__an".to_string()),
        4 => Some("Hang_an".to_string()),
        5 => Some("Hangman".to_string()),
        _ => None,
    }
}

fn JoinButton(cx: Scope) -> Element {
    let router = use_router(cx);

    let focused = use_state(cx, || false);
    let len = use_state(cx, || 0);

    let active = *focused.get() || *len.get() > 0;

    let classes = format!(
        "button w-full {}",
        if active {
            // "!" to mark text-left as important to override button class
            "!text-left font-mono"
        } else {
            "placeholder:text-white"
        }
    );

    cx.render(rsx!(
        form {
            class: "relative",
            prevent_default: "onsubmit",
            onsubmit: move |e| {
                if let Some(code) = e.values.get("code") {
                    router.navigate_to(&format!("/game/{code}"));
                }
            },
            input {
                class: "{classes}",
                r#type: "text",
                name: "code",
                placeholder: if active { "Code" } else { "Join Lobby" },
                value: "",
                minlength: 4,
                maxlength: 4,
                size: 4,
                pattern: "[a-fA-F\\d]{{4}}",
                onfocusin: move |_| {
                    focused.set(true);
                },
                onfocusout: move |_| {
                    focused.set(false);
                },
                oninput: move |e| {
                    len.set(e.value.len());
                }
            }
            if active {
                rsx!(
                    button {
                        class: "material-button absolute top-1 bottom-1 right-0.5",
                        r#type: "submit",
                        MaterialIcon {
                            name: "arrow_forward",
                            color: MaterialIconColor::Light,
                            size: 35,
                        }
                    }
                )
            }
        }
    ))
}
