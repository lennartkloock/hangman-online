use dioxus::prelude::*;

#[inline_props]
pub fn Hangman(cx: Scope, tries_used: u32) -> Element {
    cx.render(rsx!(
        div {
            style: "grid-area: hangman",
            class: "flex flex-col gap-4 justify-center items-center",
            svg {
                width: 277,
                height: 317,
                view_box: "0 0 277 317",
                fill: "none",
                xmlns: "http://www.w3.org/2000/svg",
                // Bottom
                if *tries_used >= 1 {
                    rsx!(line {
                        x2: 123,
                        y1: 314.5,
                        y2: 314.5,
                        stroke: "white",
                        stroke_width: 5,
                    })
                }
                // Left
                if *tries_used >= 2 {
                    rsx!(line {
                        x1: 64.5,
                        x2: 64.5,
                        y1: 6,
                        y2: 317,
                        stroke: "white",
                        stroke_width: 5,
                    })
                }
                // Top
                if *tries_used >= 3 {
                    rsx!(line {
                        x1: 62,
                        x2: 235,
                        y1: 3.5,
                        y2: 3.5,
                        stroke: "white",
                        stroke_width: 5,
                    })
                }
                // Right
                if *tries_used >= 4 {
                    rsx!(line {
                        x1: 237.5,
                        x2: 237.5,
                        y1: 1,
                        y2: 56,
                        stroke: "white",
                        stroke_width: 5,
                    })
                }
                // Cross
                if *tries_used >= 5 {
                    rsx!(line {
                        x1: 118.5,
                        x2: 64,
                        y1: 2.5,
                        y2: 57,
                        stroke: "white",
                        stroke_width: 5,
                    })
                }
                // Head
                if *tries_used >= 6 {
                    rsx!(circle {
                        cx: 238,
                        cy: 85,
                        r: 27.5,
                        stroke: "white",
                        stroke_width: 5,
                    })
                }
                // Body
                if *tries_used >= 7 {
                    rsx!(line {
                        x1: 237.5,
                        x2: 237.5,
                        y1: 115,
                        y2: 199,
                        stroke: "white",
                        stroke_width: 5,
                    })
                }
                // Left arm
                if *tries_used >= 8 {
                    rsx!(line {
                        x1: 205,
                        x2: 237,
                        y1: 127.5,
                        y2: 166.5,
                        stroke: "white",
                        stroke_width: 5,
                    })
                }
                // Right arm
                if *tries_used >= 8 {
                    rsx!(line {
                        x1: 275,
                        x2: 237.5,
                        y1: 127,
                        y2: 166,
                        stroke: "white",
                        stroke_width: 5,
                    })
                }
                // Left leg
                if *tries_used >= 9 {
                    rsx!(line {
                        x1: 237,
                        x2: 200,
                        y1: 197.5,
                        y2: 237,
                        stroke: "white",
                        stroke_width: 5,
                    })
                }
                // Right leg
                if *tries_used >= 9 {
                    rsx!(line {
                        x1: 237,
                        x2: 270,
                        y1: 198,
                        y2: 237,
                        stroke: "white",
                        stroke_width: 5,
                    })
                }
            }
        }
    ))
}
