use crate::components::CenterContainer;
use dioxus::prelude::*;
use dioxus_material_icons::{MaterialIcon, MaterialIconColor};
use log::error;
use std::{convert::Infallible, error::Error, fmt::Display, rc::Rc};

#[derive(Props)]
pub struct ErrorProps<'a, E: Error> {
    title: &'a str,
    error: Option<E>,
}

pub fn Error<'a, E: Error>(cx: Scope<'a, ErrorProps<'a, E>>) -> Element<'a> {
    let details = cx.props.error.as_ref().map(|e| render_details(cx.scope, e));
    cx.render(rsx!(
        CenterContainer {
            MaterialIcon { name: "warning", color: MaterialIconColor::Light, size: 200 }
            h1 {
                class: "text-3xl",
                "{cx.props.title}"
            }
            details
        }
    ))
}

#[derive(Props)]
pub struct RcErrorProps<'a, E: Error> {
    title: &'a str,
    error: Option<Rc<E>>,
}

pub fn RcError<'a, E: Error>(cx: Scope<'a, RcErrorProps<'a, E>>) -> Element<'a> {
    let details = cx.props.error.as_ref().map(|e| render_details(cx.scope, e));
    cx.render(rsx!(
        CenterContainer {
            MaterialIcon { name: "warning", color: MaterialIconColor::Light, size: 200 }
            h1 {
                class: "text-3xl",
                "{cx.props.title}"
            }
            details
        }
    ))
}

fn render_details<'a, D: Display + 'a>(cx: &'a ScopeState, error: D) -> Element {
    cx.render(rsx!(
        button {
            class: "text-xs hover:underline",
            onclick: move |_| {
                if let Some(w) = web_sys::window() {
                    if let Err(e) = w.alert_with_message(&format!("Error: {error}")) {
                        error!("Error calling window.alert: {e:?}");
                    }
                }
            },
            "Show details"
        }
    ))
}
