use dioxus::prelude::*;
use dioxus_material_icons::{MaterialIcon, MaterialIconColor};
use log::error;
use std::error::Error;

#[derive(Props)]
pub struct ErrorProps<'a, E: Error> {
    title: &'a str,
    error: Option<E>,
}

pub fn Error<'a, E: Error>(cx: Scope<'a, ErrorProps<'a, E>>) -> Element<'a> {
    let details = cx.props.error.as_ref().map(|e| {
        cx.render(rsx!(
            button {
                class: "text-xs hover:underline",
                onclick: move |_| {
                    if let Some(w) = web_sys::window() {
                        if let Err(e) = w.alert_with_message(&format!("Error: {e}")) {
                            error!("Error calling window.alert: {:?}", e);
                        }
                    }
                },
                "Show details"
            }
        ))
    });

    cx.render(rsx!(
        MaterialIcon { name: "warning", color: MaterialIconColor::Light, size: 200 }
        h1 {
            class: "text-4xl",
            "{cx.props.title}"
        }
        details
    ))
}
