#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::Link;

const BUTTON_CLASSES: &str =
    "font-light text-lg text-center bg-zinc-700 shadow-lg rounded-md px-2 py-1 hover:ring ring-zinc-500";

#[inline_props]
pub fn Button<'a>(cx: Scope<'a>, children: Element<'a>) -> Element<'a> {
    cx.render(rsx!(
        button {
            class: BUTTON_CLASSES,
            children
        }
    ))
}

#[derive(Props)]
pub struct LinkButtonProps<'a> {
    to: &'a str,
    children: Element<'a>,
}

pub fn LinkButton<'a>(cx: Scope<'a, LinkButtonProps<'a>>) -> Element<'a> {
    cx.render(rsx!(
        Link {
            to: cx.props.to,
            class: BUTTON_CLASSES,
            &cx.props.children
        }
    ))
}
