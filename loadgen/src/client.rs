use crate::workloads::{Req, ReqMethod};
use anyhow::Context;
use reqwest::{Client, Response, StatusCode, Url};
use serde_json::json;
use std::{sync::Arc, time::Duration};

#[derive(Debug, Clone)]
pub struct HttpClient {
    base: Url,
    inner: Arc<Client>,
}

async fn drain_response(res: Response) -> anyhow::Result<StatusCode> {
    let status = res.status();
    res.bytes().await?;
    Ok(status)
}

impl HttpClient {
    pub fn new(base: &str, max_pool_idle_cons: usize) -> anyhow::Result<Self> {
        let url = Url::parse(base).context("invalid base URL")?;
        let client = Client::builder()
            .pool_max_idle_per_host(max_pool_idle_cons)
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(3))
            .tcp_keepalive(Some(Duration::from_secs(60)))
            .build()?;

        Ok(Self {
            base: url,
            inner: Arc::new(client),
        })
    }

    pub async fn send_request(&self, req: Req) -> anyhow::Result<StatusCode> {
        match req.method {
            ReqMethod::POST => {
                let url = self.base.clone();
                let res = self
                    .inner
                    .post(url)
                    .json(&json!({
                        "key": req.key,
                        "value": req.value.unwrap_or_default(),
                    }))
                    .send()
                    .await?;
                drain_response(res).await
            }

            ReqMethod::GET => {
                let url = self.base.join(&req.key)?;
                let res = self.inner.get(url).send().await?;
                drain_response(res).await
            }

            ReqMethod::DELETE => {
                let url = self.base.join(&req.key)?;
                let res = self.inner.delete(url).send().await?;
                drain_response(res).await
            }
        }
    }
}
