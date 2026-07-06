//! Official Rust client for the [Pulsight](https://pulsight.xyz) API.
//!
//! The protocol core is generated at compile time from the in-crate
//! `spec/public.json` by [progenitor](https://github.com/oxidecomputer/progenitor)
//! (never hand-edited). This crate adds api-token auth and typed errors
//! ([`Error`]).
//!
//! The spec lives *inside* the crate (`spec/public.json`, a copy of the
//! canonical `sdks/openapi/public.json` kept in sync by `make sdk-public-spec`)
//! so the crate is self-contained: `progenitor::generate_api!` resolves the
//! path relative to `CARGO_MANIFEST_DIR`, and `cargo publish` only packages
//! files under the crate root — a `../openapi/...` path would escape both.

mod errors;

pub use errors::{credits_remaining, map_response, Error};

/// Generated protocol core (progenitor). Rebuilt from the committed spec on
/// every `cargo build` — there is no separate generate step.
pub mod generated {
    progenitor::generate_api!("spec/public.json");
}

/// The production Pulsight API root.
pub const DEFAULT_BASE_URL: &str = "https://pulsight.xyz";

/// Build an authenticated client for the given api token (`pk_live_…`)
/// against the production API.
pub fn new(api_token: &str) -> Result<generated::Client, Error> {
    new_with_base_url(api_token, DEFAULT_BASE_URL)
}

/// Like [`new`] but against a custom base URL (e.g. staging).
///
/// The token rides the `Authorization` header on every request; product
/// docs call it an "api token", never a "Bearer token".
pub fn new_with_base_url(api_token: &str, base_url: &str) -> Result<generated::Client, Error> {
    use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

    let mut headers = HeaderMap::new();
    let mut value = HeaderValue::from_str(&format!("Bearer {api_token}"))
        .map_err(|_| Error::Api { status: 0, body: "invalid api token".into() })?;
    value.set_sensitive(true);
    headers.insert(AUTHORIZATION, value);

    let http = reqwest::Client::builder().default_headers(headers).build()?;
    Ok(generated::Client::new_with_client(base_url, http))
}
