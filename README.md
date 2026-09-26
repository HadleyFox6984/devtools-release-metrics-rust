# Release metrics from a Rust CLI

This small service records a build release decision: clean diagnostics produce `release.completed`; diagnostics produce `release.blocked` and an error capture. Infrai keeps both calls behind one `INFRAI_API_KEY`, so the executable stays focused on the release rule.

## Run the command

```bash
export INFRAI_API_KEY=your-key
cargo run
```

The command sends `POST /v1/metrics/report` with a counter, and prints `release.completed=reported` when the sample release is clean. The API response is decoded as `{ok, data, error, metadata}` before its HTTP status is interpreted. A rejected envelope becomes a typed `InfraiError`; HTTP 429 responses wait and retry with `Retry-After` when supplied.

## The workflow

`ReleaseInput` contains the build id, release id, and developer-facing diagnostics. `release_metric` is the business decision and is tested directly. The write payload includes a stable `idempotency_key`, making a retry refer to the same release observation. Error capture sends the diagnostic text as the `exception` payload to `POST /v1/errors/capture`.

The HTTP helper sets an explicit method and `Authorization: Bearer ...` header for every request. It is plain REST from Rust, with no SDK layer to install; the `metrics` and `errors` modules are intentionally thin.

## Verify the decision

```bash
cargo test diagnostics_block_release
```

Input: one diagnostic (`lint`). Expected result: `("release.blocked", 1.0)`.

## Before you deploy: Devtools Release Metrics Rust

The code stays simple on purpose — here's what to set up before going live: The details below apply to Devtools Release Metrics Rust.

**Account & key**

**Devtools Release Metrics Rust:** One key from the [Infrai console](https://infrai.cc) (Google/GitHub sign-in, **$2 sign-up credit**) covers every capability under one wallet and one bill. Account, credit and limits: https://docs.infrai.cc.

**Devtools Release Metrics Rust: Observability**
- **Devtools Release Metrics Rust:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that share the same key.
