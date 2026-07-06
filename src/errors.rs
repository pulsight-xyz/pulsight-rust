//! Typed errors for the Pulsight API wire contract.

use std::time::Duration;

use thiserror::Error;

/// Errors returned by the Pulsight client.
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP 402 — the api credit pool is empty for this billing cycle.
    #[error("credit pool {pool:?} exhausted (HTTP 402)")]
    CreditExhausted { pool: String },

    /// HTTP 429 — too many requests. `retry_after` is the server hint.
    #[error("rate limited, retry after {retry_after:?} (HTTP 429)")]
    RateLimited { retry_after: Option<Duration> },

    /// HTTP 403 — the api token lacks a required scope.
    #[error("{message} (HTTP 403)")]
    MissingScope { message: String },

    /// Any other non-2xx response.
    #[error("unexpected status {status}: {body}")]
    Api { status: u16, body: String },

    /// Transport-level failure.
    #[error(transparent)]
    Transport(#[from] reqwest::Error),
}

/// Remaining api credits from the `X-Credits-Remaining` header, if present.
pub fn credits_remaining(headers: &reqwest::header::HeaderMap) -> Option<i64> {
    headers
        .get("X-Credits-Remaining")?
        .to_str()
        .ok()?
        .parse()
        .ok()
}

/// Map a non-2xx [`reqwest::Response`] to a typed [`Error`]; returns the
/// response unchanged on 2xx so it composes in a `?` chain. Consumes the
/// body on error.
pub async fn map_response(response: reqwest::Response) -> Result<reqwest::Response, Error> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    match status.as_u16() {
        402 => {
            let pool = response
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|v| v.get("pool").and_then(|p| p.as_str()).map(String::from))
                .unwrap_or_default();
            Err(Error::CreditExhausted { pool })
        }
        429 => {
            let retry_after = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(Duration::from_secs);
            Err(Error::RateLimited { retry_after })
        }
        403 => {
            let message = response
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(String::from))
                .unwrap_or_else(|| "forbidden".to_string());
            Err(Error::MissingScope { message })
        }
        _ => {
            let body = response.text().await.unwrap_or_default();
            Err(Error::Api { status: status.as_u16(), body })
        }
    }
}
