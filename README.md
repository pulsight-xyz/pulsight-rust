# pulsight (Rust)

Official Rust client for the [Pulsight](https://pulsight.xyz) public API.

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
let resp = client.list_traders().send().await?;     // generated operation
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

## If progenitor chokes

progenitor is strict about some OpenAPI constructs. **Known blocker (2026-07):**
the current public spec omits `operationId`s, so `cargo build` fails with
`missing operation ID` — progenitor needs them to name methods, whereas
oapi-codegen (Go) and openapi-ts (JS) synthesize names from the method+path.
So `publish-<VERSION>` will fail the `publish-rust` job until either the backend
emits `operationId`s on public operations (swaggo annotations / the
`cmd/openapi-public` doc) **or** you switch this crate to the `openapi-generator`
fallback below. Go/Python/JS are unaffected.

If `cargo build` fails on a schema it can't model (per `docs/sdks-design.md` §2,
an accepted risk), fall back to `openapi-generator` (rust-reqwest) into a
`generated/` dir and point `lib.rs` at it instead of the macro:

```sh
openapi-generator generate -i sdks/openapi/public.json -g rust \
  -o sdks/rust/generated --additional-properties=library=reqwest,packageName=pulsight_api
```
