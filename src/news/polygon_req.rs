use std::os::raw::c_short;

use crate::model_request::HttpMethod;
use crate::model_request::execute_request;
use crate::model_request::RequestSpec;
use dioxus::html::u::order;
use dioxus::prelude::*;
use serde::Deserialize;
use chrono::{NaiveDate, NaiveDateTime, DateTime, Utc};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NewsQuery {
    pub ticker: Option<String>,
    pub published_utc_gte: Option<String>, // "YYYY-MM-DD" or RFC3339
    pub published_utc_lte: Option<String>,
    pub order: Option<String>,             // "asc" | "desc"
    pub limit: Option<u32>,                // 1..=1000
    pub sort: Option<String>,              // "published_utc"
    pub api_key: String,
}

impl NewsQuery {
    pub fn new_just_key(api_key: impl Into<String>) -> Self {
        Self { api_key: api_key.into(), ..Default::default() }
    }

     pub fn new<'a>(
        api_key: Option<impl Into<&'a str>>,
        ticker: Option<impl Into<String>>,
        published_gte: Option<impl Into<String>>,
        published_lte: Option<impl Into<String>>,
        limit: Option<u32>,
        sort: Option<impl Into<String>>,
        cursor: Option<impl Into<String>>,
    ) -> Self {
        Self {
            api_key: api_key.map(|v| v.into()).unwrap().to_string(),
            ticker: ticker.map(|v| v.into()),
            published_utc_gte: published_gte.map(|v| v.into()),
            published_utc_lte: published_lte.map(|v| v.into()),
            order: None,
            limit: limit.map(|v| v.into()),
            sort: sort.map(|v| v.into()),
        }
    }

    pub fn ticker(mut self, val: impl Into<String>) -> Self {
        self.ticker = Some(val.into());
        self
    }

    // --- published_utc.gte ---
    pub fn published_gte<S: Into<String>>(mut self, val: S) -> Self {
        self.published_utc_gte = Some(val.into());
        self
    }

    pub fn published_gte_naive_date(mut self, date: NaiveDate) -> Self {
        // Format as YYYY-MM-DD
        self.published_utc_gte = Some(date.format("%Y-%m-%d").to_string());
        self
    }

    pub fn published_gte_naive_datetime(mut self, dt: NaiveDateTime) -> Self {
        // RFC3339 with Z
        self.published_utc_gte = Some(format!("{}Z", dt.format("%Y-%m-%dT%H:%M:%S")));
        self
    }

    pub fn published_gte_datetime(mut self, dt: DateTime<Utc>) -> Self {
        // RFC3339 full format
        self.published_utc_gte = Some(dt.to_rfc3339());
        self
    }

    // --- published_utc.lte (same pattern) ---
    pub fn published_lte<S: Into<String>>(mut self, val: S) -> Self {
        self.published_utc_lte = Some(val.into());
        self
    }

    pub fn published_lte_naive_date(mut self, date: NaiveDate) -> Self {
        self.published_utc_lte = Some(date.format("%Y-%m-%d").to_string());
        self
    }

    pub fn published_lte_naive_datetime(mut self, dt: NaiveDateTime) -> Self {
        self.published_utc_lte = Some(format!("{}Z", dt.format("%Y-%m-%dT%H:%M:%S")));
        self
    }

    pub fn published_lte_datetime(mut self, dt: DateTime<Utc>) -> Self {
        self.published_utc_lte = Some(dt.to_rfc3339());
        self
    }

    pub fn build_url(&self) -> String {
        let mut url = "https://api.polygon.io/v2/reference/news".to_string();
        let mut params: Vec<(String, String)> = vec![];

        if let Some(t) = &self.ticker { params.push(("ticker".into(), t.clone())); }
        if let Some(v) = &self.published_utc_gte { params.push(("published_utc.gte".into(), v.clone())); }
        if let Some(v) = &self.published_utc_lte { params.push(("published_utc.lte".into(), v.clone())); }
        if let Some(v) = &self.order { params.push(("order".into(), v.clone())); }
        if let Some(v) = &self.limit { params.push(("limit".into(), v.to_string())); }
        if let Some(v) = &self.sort { params.push(("sort".into(), v.clone())); }

        params.push(("apiKey".into(), self.api_key.clone()));

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params
                .into_iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(&k), urlencoding::encode(&v)))
                .collect::<Vec<_>>()
                .join("&"));
        }
        url
    }

    pub fn to_spec(&self) -> RequestSpec {
        RequestSpec {
            method: HttpMethod::Get,
            url: self.build_url(),
            body_json: None,
            headers: vec![("Accept".into(), "application/json".into())],
        }
    }
}

// ----- Polygon response models -----
#[derive(Debug, Deserialize, Clone, Default)]
pub struct PolygonPublisher {
    pub name: Option<String>,
    pub homepage_url: Option<String>,
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
}
#[derive(Debug, Deserialize, Clone, Default)]
pub struct PolygonInsight {
    pub ticker: Option<String>,
    pub sentiment: Option<String>,
    pub sentiment_reasoning: Option<String>,
}
#[derive(Debug, Deserialize, Clone, Default)]
pub struct PolygonNewsItem {
    pub id: String,
    pub title: String,
    pub article_url: String,
    pub amp_url: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub published_utc: String,
    pub keywords: Option<Vec<String>>,
    pub publisher: Option<PolygonPublisher>,
    pub tickers: Option<Vec<String>>,
    pub insights: Option<Vec<PolygonInsight>>,
}
#[derive(Debug, Deserialize, Clone, Default)]
pub struct PolygonNewsResponse {
    pub status: Option<String>,
    pub request_id: Option<String>,
    pub count: Option<u64>,
    pub next_url: Option<String>,
    pub results: Option<Vec<PolygonNewsItem>>,
}


// ----- Fetch using your executor (returns typed response) -----
pub async fn fetch_polygon_news(q: &NewsQuery) -> Result<PolygonNewsResponse, String> {
    let spec = q.to_spec();
    let text = execute_request(spec).await.map_err(|e| e.to_string())?;
    serde_json::from_str::<PolygonNewsResponse>(&text).map_err(|e| e.to_string())
}
