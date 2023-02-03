use crate::components::{CenterContainer, LinkButton};
use dioxus::prelude::*;
use std::time::Duration;

pub fn home(cx: Scope) -> Element {
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
                    class: "font-mono font-bold drop-shadow-2xl text-6xl tracking-widest",
                    "{title}"
                }
                div {
                    class: "flex flex-col gap-4",
                    LinkButton { to: "/create", "Create Lobby" }
                    LinkButton { to: "/join", "Join Lobby" }
                }
            }
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
