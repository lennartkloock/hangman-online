use dioxus::prelude::*;

#[derive(Props)]
pub struct CenterContainerProps<'a> {
    children: Element<'a>,
}

pub fn CenterContainer<'a>(cx: Scope<'a, CenterContainerProps<'a>>) -> Element<'a> {
    cx.render(rsx!(
        div {
            class: "h-full flex justify-center items-center",
            &cx.props.children
        }
    ))
}
