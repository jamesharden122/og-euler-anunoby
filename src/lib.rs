pub mod charts;
pub mod data_structures;
pub mod model_request;
pub mod news;
pub mod ops;
pub mod prompting;
pub mod surr_queries;
pub mod tables;
pub mod views;

use views::{multi_assets::MultiAsset, portfolio::Portfolio, single_assets::SingleAsset};

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Routable, Debug, PartialEq, Serialize, Deserialize)]
enum Route {
    #[route("/")]
    Home,
}

#[component]
fn Home() -> Element {
    let mut selected_tab = use_signal(|| 0);
    rsx! {
        style { { include_str!("./../src/css_files/home_style.css") } }

        div {
            h1 { "Quant Streaming Demo" }
        }
        div { class: "analysis-root",
            div { class: "analysis-selector",
                button { class: "analysis-button", onclick: move |_| selected_tab.set(1), "Single Asset Analysis" }
                button { class: "analysis-button", onclick: move |_| selected_tab.set(2), "Multi-Asset Analysis" }
                button { class: "analysis-button", onclick: move |_| selected_tab.set(3), "Portfolio Analysis" }
            }
        }
            match selected_tab() {
            1 => rsx! { SingleAsset {} },
            2 => rsx! { MultiAsset {} },
            3 => rsx! { Portfolio {} },
            _ => rsx! { "Select analysis toolkit" },
        }
    }
}

pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
