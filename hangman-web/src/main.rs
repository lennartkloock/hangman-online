use dioxus::prelude::*;

fn main() {
    dioxus_web::launch(root);
}

fn root(cx: Scope) -> Element {
    cx.render(rsx!(
        h1 { "Hangman" }
        Word { word: "Test" }
    ))
}

#[derive(Props)]
struct WordProps<'a> {
    word: &'a str,
}

fn Word(cx: Scope<WordProps>) -> Element {
    cx.render(rsx!(
        p { cx.props.word }
    ))
}
