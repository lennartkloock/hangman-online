use crate::components::TopBar;
use dioxus::prelude::*;
use hangman_data::Score;

#[inline_props]
pub fn Scoreboard(cx: Scope, scores: Vec<Score>) -> Element {
    cx.render(rsx!(
        div {
            class: "bg-zinc-800 rounded-xl shadow-lg",
            TopBar {
                p {
                    class: "mx-auto text-xl font-light",
                    "Results"
                }
            }
            div {
                class: "p-8",
                div {
                    class: "flex gap-8 items-end",
                    scores.get(1).map(|score| rsx!(Podium { score: score }))
                    scores.get(0).map(|score| rsx!(Podium { score: score }))
                    scores.get(2).map(|score| rsx!(Podium { score: score }))
                }
                div {
                    class: "flex justify-evenly",
                    scores.get(3).map(|score| rsx!(ShortPodium { score: score }))
                    scores.get(4).map(|score| rsx!(ShortPodium { score: score }))
                }
            }
        }
    ))
}

#[inline_props]
fn Podium<'a>(cx: Scope<'a>, score: &'a Score) -> Element<'a> {
    let height = match score.rank {
        1 => "h-64",
        2 => "h-44",
        _ => "h-20",
    };

    cx.render(rsx!(
        div {
            class: "text-center text-3xl font-light",
            p {
                class: "max-w-[12rem] overflow-hidden text-ellipsis",
                "{score.nickname}"
            }
            div {
                class: "rounded-t-xl w-28 {height} bg-zinc-700 flex flex-col justify-between py-6 mt-1 mx-auto",
                p {
                    class: "leading-6",
                    span { "{score.score}" }
                    br {}
                    span {
                        class: "text-xl",
                        if score.score == 1 { "word" } else { "words" }
                    }
                }
                p {
                    class: "font-bold",
                    "{score.rank}."
                }
            }
        }
    ))
}

#[inline_props]
fn ShortPodium<'a>(cx: Scope<'a>, score: &'a Score) -> Element<'a> {
    cx.render(rsx!(
        p {
            class: "text-xl mt-4",
            span {
                class: "font-bold",
                "{score.rank}. "
            }
            span { "{score.nickname}" }
        }
    ))
}
