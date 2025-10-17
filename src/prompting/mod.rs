use crate::news::fetch::NewsBoxProps;
use arraydeque::{ArrayDeque, Wrapping};
use arrayvec::ArrayString;
use dioxus::prelude::*; // <- no `behavior::`

#[component]
pub fn PromptBox() -> Element {
    let mut new_article = use_signal(|| ArrayString::<4096>::new);
    let mut new_prompt = use_signal(|| ArrayString::<4096>::new);
    let mut results = use_signal(|| ArrayString::<4096>::new);
    rsx! {
        div {
            class:"prompt-box",
            label { "Prompt"}
            textarea {

            }
            button {
                "news update"
            }
            p {"What do you need help with"}
        }

    }
}
