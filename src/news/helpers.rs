use super::{
    fetch::NewsBoxProps,
    polygon_req::{
        fetch_polygon_news, NewsQuery, PolygonInsight, PolygonNewsItem, PolygonNewsResponse,
        PolygonPublisher,
    },
};
use arraydeque::{behavior::Wrapping, ArrayDeque};
use arrayvec::ArrayString;
use chrono::{DateTime, NaiveDate, Utc};
use std::cmp::min;
// ---- helper: truncate safely to ArrayString<N> (no panics; keeps valid UTF-8) ----
fn to_arraystring<const N: usize>(s: &str) -> ArrayString<N> {
    // if it already fits, fast path
    if s.len() <= N {
        return ArrayString::<N>::from(s).unwrap();
    }
    // otherwise, truncate on a char boundary
    let mut out = ArrayString::<N>::new();
    for ch in s.chars() {
        // stop before we’d exceed capacity
        if out.len() + ch.len_utf8() > N {
            break;
        }
        // ArrayString push is infallible when pre-checked
        out.push(ch);
    }
    out
}

// ---- helper: pick best link and trim ----
fn best_link(item: &PolygonNewsItem) -> &str {
    // prefer AMP if present, else article_url
    item.amp_url.as_deref().unwrap_or(&item.article_url)
}

// ---- helper: format RFC3339 -> YYYY-MM-DD ----
fn ymd(date_str: &str) -> String {
    // Try RFC3339 first; fall back to raw string on failure
    match DateTime::parse_from_rfc3339(date_str).map(|dt| dt.with_timezone(&Utc).date_naive()) {
        Ok(date) => date.to_string(),
        Err(_) => Utc::now().naive_utc().to_string(),
    }
}

// ---- transform ----
pub fn polygon_to_props(model: impl Into<String>, resp: PolygonNewsResponse) -> NewsBoxProps {
    // Collect results (handle None)
    let mut items = resp.results.unwrap_or_default();

    // Sort newest first by published_utc (parseable go first)
    items.sort_by(|a, b| {
        let pa = DateTime::parse_from_rfc3339(&a.published_utc).ok();
        let pb = DateTime::parse_from_rfc3339(&b.published_utc).ok();
        pb.cmp(&pa) // descending
    });

    // Cap to 15 newest
    let take_n = min(15, items.len());

    // Create empty deques (Wrapping: overwrite oldest when full)
    let mut article_text: ArrayDeque<ArrayString<300>, 15, Wrapping> = ArrayDeque::new();
    let mut article_links: ArrayDeque<ArrayString<30>, 15, Wrapping> = ArrayDeque::new();
    let mut dates: ArrayDeque<ArrayString<30>, 15, Wrapping> = ArrayDeque::new();

    for item in items.into_iter().take(take_n) {
        // Title (or fallback to description or id)
        let title = if !item.title.is_empty() {
            &item.title
        } else if let Some(desc) = &item.description {
            desc
        } else {
            &item.id
        };

        // Link (may be longer than 30 chars — see note below)
        let link = best_link(&item);

        // Date in YYYY-MM-DD
        let date = ymd(&item.published_utc);

        // Push with safe truncation to avoid panics
        article_text.push_back(to_arraystring::<300>(title));
        article_links.push_back(to_arraystring::<30>(link));
        dates.push_back(to_arraystring::<30>(&date));
    }

    NewsBoxProps {
        model: model.into(),
        article_text,
        article_links,
        dates,
    }
}
