use crate::config::Config;
use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

pub trait RemoteModel {
    fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String>;
}

pub fn build_remote(cfg: &Config) -> Result<Box<dyn RemoteModel>> {
    if let Some(url) = cfg.remote_endpoint.clone() {
        Ok(Box::new(HttpRemote {
            url,
            api_key: cfg.api_key.clone(),
            model: cfg.model.clone().unwrap_or_else(|| "default".into()),
            http: Client::builder().build()?,
        }))
    } else {
        Ok(Box::new(EchoRemote))
    }
}

struct EchoRemote;

impl RemoteModel for EchoRemote {
    fn generate(&self, prompt: &str, _max_tokens: usize) -> Result<String> {
        Ok(format!("[no remote configured]\nEchoing prompt:\n{}", prompt))
    }
}

struct HttpRemote {
    url: String,
    api_key: Option<String>,
    model: String,
    http: Client,
}

#[derive(Serialize)]
struct InferenceRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    max_tokens: usize,
}

#[derive(Deserialize)]
struct InferenceResponse {
    output: String,
}

impl RemoteModel for HttpRemote {
    fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        let body = InferenceRequest { model: &self.model, prompt, max_tokens };
        let mut req = self.http.post(&self.url).json(&body);
        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }
        let res = req.send().context("sending remote inference request")?;
        if !res.status().is_success() {
            let status = res.status();
            let txt = res.text().unwrap_or_default();
            anyhow::bail!("remote error: {} - {}", status, txt);
        }
        // Allow either our schema or a plain string for flexibility
        let text = res.text()?;
        if let Ok(parsed) = serde_json::from_str::<InferenceResponse>(&text) {
            Ok(parsed.output)
        } else {
            Ok(text)
        }
    }
}
