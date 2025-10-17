use arraydeque::ArrayDeque;
use arraydeque::Wrapping;
use arrayvec::ArrayString;
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct NewsBoxProps {
    pub model: String,
    pub article_text: ArrayDeque<ArrayString<300>, 15, Wrapping>,
    pub article_links: ArrayDeque<ArrayString<30>, 15, Wrapping>,
    pub dates: ArrayDeque<ArrayString<30>, 15, Wrapping>,
}

impl NewsBoxProps {
    pub fn empty_news(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            article_text: ArrayDeque::new(),
            article_links: ArrayDeque::new(),
            dates: ArrayDeque::new(),
        }
    }
}
#[component]
pub fn NewsFetch(props: NewsBoxProps) -> Element {
    // render up to the shortest deque length (capped at 15)
    let n = props
        .article_text
        .len()
        .min(props.article_links.len())
        .min(props.dates.len())
        .min(15);

    // build a simple view model of (&str, &str, &str)
    let rows: Vec<(&str, &str, &str)> = (0..n)
        .map(|i| {
            let text = props.article_text.get(i).map(|s| s.as_str()).unwrap_or("");
            let link = props.article_links.get(i).map(|s| s.as_str()).unwrap_or("");
            let date = props.dates.get(i).map(|s| s.as_str()).unwrap_or("");
            (text, link, date)
        })
        .collect();

    rsx! {
        table {class: "trade-table",
            thead {
                tr {
                    th { "Article" }
                    th { "Link" }
                    th { "Date" }
                }
            }
            tbody {
                for (text, link, date) in rows {
                    tr {
                        td { "{text}" }
                        td {
                            a { href: "{link}", target: "_blank", rel: "noopener", "link" }
                        }
                        td { "{date}" }
                    }
                }
            }
        }
    }
}
