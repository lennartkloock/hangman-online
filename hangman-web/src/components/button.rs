#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_material_icons::{MaterialIcon, MaterialIconColor};
use dioxus_router::Link;

const BUTTON_CLASSES: &str =
    "font-light text-lg text-center bg-zinc-700 shadow-lg rounded-md px-2 py-1 transition-all hover:ring ring-zinc-500";

#[inline_props]
pub fn Button<'a>(cx: Scope<'a>, children: Element<'a>) -> Element<'a> {
    cx.render(rsx!(button {
        class: BUTTON_CLASSES,
        children
    }))
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

const MATERIAL_BUTTON_CLASSES: &str =
    "flex items-center rounded-full p-1 transition-colors hover:bg-zinc-600";

#[derive(Props)]
pub struct MaterialButtonProps<'a> {
    name: &'a str,
    onclick: EventHandler<'a, MouseEvent>,
}

pub fn MaterialButton<'a>(cx: Scope<'a, MaterialButtonProps<'a>>) -> Element<'a> {
    cx.render(rsx!(
        button {
            class: MATERIAL_BUTTON_CLASSES,
            onclick: move |evt| cx.props.onclick.call(evt),
            MaterialIcon {
                name: cx.props.name,
                color: MaterialIconColor::Light,
                size: 35,
            }
        }
    ))
}

#[derive(Props)]
pub struct MaterialLinkButtonProps<'a> {
    to: &'a str,
    name: &'a str,
}

pub fn MaterialLinkButton<'a>(cx: Scope<'a, MaterialLinkButtonProps<'a>>) -> Element<'a> {
    cx.render(rsx!(
        Link {
            class: MATERIAL_BUTTON_CLASSES,
            to: cx.props.to,
            MaterialIcon {
                name: cx.props.name,
                color: MaterialIconColor::Light,
                size: 35,
            }
        }
    ))
}
