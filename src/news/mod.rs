use dioxus::prelude::*;

pub mod fetch;
pub mod polygon_req;
pub mod helpers;
use fetch::{NewsFetch,NewsBoxProps};
use polygon_req::{
    fetch_polygon_news, NewsQuery, PolygonInsight, PolygonNewsItem, PolygonNewsResponse, PolygonPublisher,
};

#[component]
pub fn Fetch() -> Element {
    // controls
    let api_key = use_signal(|| "ATu3EI3R9YubjAXM48t1UYrm0dkMoWyL".to_string());
    let mut ticker = use_signal(|| {"AAPL".to_string()});
    let mut gte = use_signal(|| "".to_string()); // YYYY-MM-DD or RFC3339
    let mut lte = use_signal(|| "".to_string());
    let mut order = use_signal(|| {"desc".to_string()});
    let mut limit = use_signal(|| 25u32);

    // data/state
    let mut loading = use_signal(|| false);
    let mut error_msg = use_signal(|| None::<String>);
    let mut news_box = use_signal(|| { 
        NewsBoxProps::empty_news("gpt".to_string())
    });
    let do_fetch = move |_| async move {
            loading.set(true);
            error_msg.set(None);
            let query = NewsQuery { 
                api_key: api_key(),
                ticker: Some(ticker().to_string()),
                published_utc_gte: Some(gte().to_string()),
                published_utc_lte: Some(lte().to_string()),
                limit: Some(limit()),
                sort: None,
                order: None,
                 };
                 match fetch_polygon_news(&query).await {
                    Ok(resp) => {
                        let props = helpers::polygon_to_props("polygon-news", resp);
                        news_box.set(props);
                    },
                    Err(e) => error_msg.set(Some(e)),
                 }

        };
    rsx! {
        div { class: "card-news",
            // ticker
            div {
                label { "Ticker" }
                input {
                    value: "{ticker()}",
                    oninput: move |ev| ticker.set(ev.value())
                }
            }

            // published_utc_gte
            div {
                label { "From (gte)" }
                input {
                    r#type: "date",
                    value: "{gte()}",
                    oninput: move |ev| gte.set(ev.value())
                }
            }

            // published_utc_lte
            div {
                label { "To (lte)" }
                input {
                    r#type: "date",
                    value: "{lte()}",
                    oninput: move |ev| lte.set(ev.value())
                }
            }

            // order
            div {
                label { "Order" }
                select {
                    value: "{order()}",
                    oninput: move |ev| order.set(ev.value()),
                    option { value: "desc", "desc" }
                    option { value: "asc", "asc" }
                }
            }

            // limit
            div {

                label { "Limit" }
                input {
                    r#type: "number",
                    min: "1", max: "1000",
                    value: "{limit()}",
                    oninput: move |ev| if let Ok(v) = ev.value().parse::<u32>() { limit.set(v) }
                }
            }
            button { onclick: do_fetch,"Fetch"}
        }
        NewsFetch{
            model: news_box().model,
            article_text: news_box().article_text,
            article_links: news_box().article_links,
            dates: news_box().dates}
    }
}
