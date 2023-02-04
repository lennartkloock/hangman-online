#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_material_icons::{MaterialIcon, MaterialIconColor};
use dioxus_router::Link;

#[inline_props]
pub fn Button<'a>(cx: Scope<'a>, children: Element<'a>) -> Element<'a> {
    cx.render(rsx!(button {
        class: "button",
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
            class: "button",
            &cx.props.children
        }
    ))
}

#[derive(Props)]
pub struct MaterialButtonProps<'a> {
    name: &'a str,
    onclick: EventHandler<'a, MouseEvent>,
}

pub fn MaterialButton<'a>(cx: Scope<'a, MaterialButtonProps<'a>>) -> Element<'a> {
    cx.render(rsx!(
        button {
            class: "material-button",
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
            class: "material-button",
            to: cx.props.to,
            MaterialIcon {
                name: cx.props.name,
                color: MaterialIconColor::Light,
                size: 35,
            }
        }
    ))
}
