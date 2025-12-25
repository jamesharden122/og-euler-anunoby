use dioxus::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize)]
pub struct AiResp {
    pub text: String,
}

fn extract_output_text(resp: &Value) -> String {
    let mut out = String::new();
    if let Some(items) = resp.get("output").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(content) = item.get("content").and_then(|v| v.as_array()) {
                for block in content {
                    if block.get("type").and_then(|t| t.as_str()) == Some("output_text") {
                        if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                            if !out.is_empty() {
                                out.push('\n');
                            }
                            out.push_str(text);
                        }
                    }
                }
            }
        }
    }
    out
}

#[post("/api/ai")]
pub async fn llm_ai(input: String) -> Result<AiResp, ServerFnError> {
    let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| ServerFnError::ServerError {
        message: "OPENAI_API_KEY missing".into(),
        code: 500,
        details: None,
    })?;

    let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-5-mini".to_string());

    let body = json!({
        "model": model,
        "input": input,
    });

    let resp: Value = reqwest::Client::new()
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();

    Ok(AiResp {
        text: extract_output_text(&resp),
    })
}

#[derive(Serialize)]
pub struct AiReq {
    pub input: String,
}

pub async fn post_ai(input: impl Into<String>) -> Result<String, gloo_net::Error> {
    let resp: AiResp = Request::post("/api/ai")
        .json(&AiReq {
            input: input.into(),
        })?
        .send()
        .await?
        .json()
        .await?;

    Ok(resp.text)
}
