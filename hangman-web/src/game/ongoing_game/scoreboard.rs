use dioxus::prelude::*;
use hangman_data::Score;
use crate::components::TopBar;

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
                class: "flex justify-between gap-8 p-8 items-end",
                scores.get(1).map(|score| rsx!(Podium { score: score }))
                scores.get(0).map(|score| rsx!(Podium { score: score }))
                scores.get(2).map(|score| rsx!(Podium { score: score }))
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
