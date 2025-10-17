pub mod momentum_lstm;
use std::str::FromStr;
use tracing::{info,error,debug};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;

// ====================================
// 1) Reactive Context (UI state): UiCtx
// ====================================
#[derive(Clone)]
pub struct UiCtx {
    pub start_date: Signal<Option<String>>,
    pub end_date: Signal<Option<String>>,
    pub instrument_ids: Signal<Vec<i64>>,
    pub bin_size: Signal<String>,
    pub url: Signal<String>,
    pub user: Signal<String>,
    pub pass: Signal<String>,
    pub ns: Signal<String>,
    pub db: Signal<String>,
    // extras used by some actions
    pub feats: Signal<Option<String>>,
    pub model_path: Signal<Option<String>>,
    pub out_csv: Signal<Option<String>>,
    pub run_name: Signal<Option<String>>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct DbParams {
    pub url: String,
    pub user: String,
    pub pass: String,
    pub ns: String,
    pub db: String, // maps to "dbname"
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct UiParams {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub instrument_ids: Vec<i64>,
    pub bin_size: String,
    pub db: DbParams,
    pub feats: Option<String>,
    pub model_path: Option<String>,
    pub out_csv: Option<String>,
    pub run_name: Option<String>,
}

impl Snapshot for UiCtx {
    type Out = UiParams;
    fn snapshot(&self) -> UiParams {
        UiParams {
            start_date: (self.start_date)(),
            end_date: (self.end_date)(),
            instrument_ids: (self.instrument_ids)(),
            bin_size: (self.bin_size)(),
            db: DbParams {
                url: (self.url)(),
                user: (self.user)(),
                pass: (self.pass)(),
                ns: (self.ns)(),
                db: (self.db)(),
            },
            feats: (self.feats)(),
            model_path: (self.model_path)(),
            out_csv: (self.out_csv)(),
            run_name: (self.run_name)(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum RunStatus {
    Idle,
    Running,
    Success(String),
    Error(String),
}

pub trait BuildRequest {
    fn build_request(&self, action: &ModelAction) -> RequestSpec;
}

pub trait Snapshot {
    type Out;
    fn snapshot(&self) -> Self::Out;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpMethod { Get, Post }

#[derive(Clone, Debug)]
pub struct RequestSpec {
    pub method: HttpMethod,
    pub url: String,
    /// For GET we ignore body; for POST we send JSON string
    pub body_json: Option<serde_json::Value>,
    /// Optional headers (default adds Content-Type for POST)
    pub headers: Vec<(String, String)>,
}

fn http_err(err: serde_json::Error) -> gloo_net::Error {
    gloo_net::Error::from(err)
}



pub async fn execute_request(spec: RequestSpec) -> Result<String, gloo_net::Error> {
     match spec.method {
        HttpMethod::Get => {
            debug!("Made it into the get request");  
            let mut req = Request::get(&spec.url);
            for (k, v) in &spec.headers {
                debug!("hedaer {:?}",(k,v));
                req = req.header(k, v);
            }
            debug!("req {:?}",req);  
            let resp = match req.send().await {
                Ok(r) => r,
                Err(e) => {
                    // You'll see CORS/network/etc. here instead of “disappearing”
                    error!("send() failed: {:?}", e);
                    Err(gloo_net::Error::GlooError(e.to_string()))
                }?
            };

            info!("resp {:?}", resp);  
            let status = resp.status();
            info!("status {:?}",status);  
            let text = resp.text().await.unwrap_or_default();
            info!("text {:?}",text);    
            if (200..300).contains(&status) {
                Ok(text)
            } else {
            	Err(gloo_net::Error::GlooError(status.to_string()))
            }
        }

        HttpMethod::Post => {
            let mut req = Request::post(&spec.url);
            // Default JSON header if none provided
            let mut has_ct = false;
            for (k, v) in &spec.headers {
                if k.eq_ignore_ascii_case("content-type") {
                    has_ct = true;
                }
                req = req.header(k, v);
            }
            if !has_ct {
                req = req.header("Content-Type", "application/json");
            }

            let body = spec
                .body_json
                .map(|v| serde_json::to_string(&v))
                .transpose()? // Result<Option<String>, NetError>
                .unwrap_or_default();

            let resp = req.body(body)?.send().await?;
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();

            if (200..300).contains(&status) {
                Ok(text)
            } else {
                Err(gloo_net::Error::GlooError(status.to_string()))
            }
        }
    }
}

// One enum to rule them all (add varintas freely)
#[derive(Clone, Debug,PartialEq)]
pub enum ModelAction {
    MomTrain(momentum_lstm::TrainSpec),
    MomBacktest(momentum_lstm::BacktestSpec),
    // Future: Evaluate(EvalSpec), Export(ExportSpec), GridSearch(GridSpec), …
}

pub fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").chars().take(160).collect()
}

impl ModelAction {
    pub fn from_str_with_specs(
        s: &str,
        trn: &momentum_lstm::TrainSpec,
        bt:  &momentum_lstm::BacktestSpec,
    ) -> Option<Self> {
        match s {
            "train"    => Some(Self::MomTrain(trn.clone())),
            "backtest" => Some(Self::MomBacktest(bt.clone())),
            _ => Some(Self::MomBacktest(bt.clone())),
        }
    }
}