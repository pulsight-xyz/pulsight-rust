# pulsight (Rust)

Official Rust client for the [Pulsight](https://pulsight.xyz) public API.

📦 **[crates.io/crates/pulsight](https://crates.io/crates/pulsight)** — `cargo add pulsight`

## How generation works

Unlike the Go/Python SDKs, the Rust core is generated **at compile time** by
the `progenitor::generate_api!("spec/public.json")` macro in `src/lib.rs` —
there is no committed `generated/` directory and no separate generate step.
`cargo build` reads the committed spec and emits the `generated::Client`
in-memory.

The spec lives **inside the crate** at `spec/public.json` (a copy of the
canonical `sdks/openapi/public.json`, kept in sync by `make sdk-public-spec`).
Keeping it in-crate is what lets a standalone `pulsight-xyz/pulsight-rust`
checkout build and `cargo publish` succeed — the macro path resolves against
`CARGO_MANIFEST_DIR`, and crates.io only packages files under the crate root.

```sh
make sdk-rust      # = cargo build in sdks/rust (regenerates the spec first)
# or:
cd sdks/rust && cargo build
```

The handwritten ergonomic layer (`src/errors.rs`, the `new*` constructors in
`src/lib.rs`) adds api-token auth and typed errors; it never touches the
generated core.

## Usage

```rust
let client = pulsight::new("pk_live_…")?;          // api token from settings UI
let resp = client.get_traders().send().await?;      // generated operation (operationId getTraders)
// progenitor decodes 2xx; for raw responses, map non-2xx with
// pulsight::map_response(resp).await? and read pulsight::credits_remaining(headers).
```

## Error mapping (`pulsight::Error`)

| HTTP | Variant |
|---|---|
| 402 `CREDIT_EXHAUSTED` | `Error::CreditExhausted { pool }` |
| 429 | `Error::RateLimited { retry_after }` |
| 403 (missing scope) | `Error::MissingScope { message }` |
| other non-2xx | `Error::Api { status, body }` |
| transport | `Error::Transport(reqwest::Error)` |

## Spec normalization (why the build is green)

progenitor is stricter than the other generators, so the swaggo-emitted spec is
run through `sdks/scripts/normalize-public-spec.py` (by `make sdk-public-spec` and
in CI) before generation:

- **operationIds** — progenitor requires one on every operation (`missing
  operation ID` otherwise); swaggo emits none. The normalizer injects a
  deterministic id from method+path, which also gives Go/Python/JS stable,
  readable method names instead of per-generator synthesized ones.
- **duplicate enum values** — swaggo emits a value twice when two Go consts alias
  it (`VenueID` had `solana` twice), so progenitor emits a duplicate variant
  (`E0428`). The normalizer drops the duplicate.

Version pinning matters: **progenitor `0.14`** (0.8 mis-generated `.to_string()`
on array query params like `/api/mints?dex=`), and because progenitor-client 0.14
pulls **reqwest `0.13`**, this crate's `reqwest` must stay on 0.13 too — a skew
pulls two reqwests and won't compile (see the `Cargo.toml` note; `rustls-tls`
became `rustls` in reqwest 0.13).

## If progenitor chokes

If a future spec change hits a schema progenitor still can't model (per
`docs/sdks-design.md` §2, an accepted risk), fall back to `openapi-generator`
(rust-reqwest) into a `generated/` dir and point `lib.rs` at it instead of the
macro:

```sh
openapi-generator generate -i sdks/openapi/public.json -g rust \
  -o sdks/rust/generated --additional-properties=library=reqwest,packageName=pulsight_api
```
