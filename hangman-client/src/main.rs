use dioxus::prelude::*;

fn main() {
    dioxus_web::launch(root);
}

fn root(cx: Scope) -> Element {
    cx.render(rsx!(
        h1 { "Test" }
    ))
}
