use dioxus::prelude::*;

#[inline_props]
pub fn CenterContainer<'a>(cx: Scope<'a>, children: Element<'a>) -> Element<'a> {
    cx.render(rsx!(div {
        class: "h-full flex justify-center items-center",
        children
    }))
}

#[derive(Props)]
pub struct FormProps<'a> {
    onsubmit: EventHandler<'a, FormEvent>,
    children: Element<'a>,
}

pub fn Form<'a>(cx: Scope<'a, FormProps<'a>>) -> Element<'a> {
    cx.render(rsx!(
        form {
            class: "bg-zinc-800 rounded-xl shadow-lg w-80 max-w-[80%]",
            prevent_default: "onsubmit",
            onsubmit: move |evt| {
                cx.props.onsubmit.call(evt);
            },
            &cx.props.children
        }
    ))
}

#[inline_props]
pub fn FormTopBar<'a>(cx: Scope<'a>, children: Element<'a>) -> Element<'a> {
    cx.render(rsx!(div {
        class: "bg-zinc-700 rounded-t-xl p-2 flex justify-between items-center",
        children
    }))
}
