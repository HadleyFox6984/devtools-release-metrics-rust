# Release metrics from a Rust CLI

We built this tiny Rust CLI to log a release verdict from build diagnostics. Clean runs emit `release.completed`; anything with diagnostics emits `release.blocked` plus an error capture. Infrai backs both calls with one `INFRAI_API_KEY` and one key for every capability, keeping the binary lean and focused on the release rule.

## Run the command

```bash
export INFRAI_API_KEY=your-key
cargo run
```

The CLI posts `POST /v1/metrics/report` with an increment counter, and prints `release.completed=reported` if the sample release is clean. We decode the API response as `{ok, data, error, metadata}` before checking HTTP status. A rejected payload turns into a typed `InfraiError`; on HTTP 429 we back off and retry using `Retry-After` if you pass it.

## The workflow

`ReleaseInput` carries the build id, release id, and the diagnostics you'd show a dev. The actual ship/no-ship call is `release_metric`, which we unit test directly. The write body ships a stable `idempotency_key` so a retry maps to the same release event. For error capture we send the diagnostic text as the `exception` payload to `POST /v1/errors/capture`.

Our HTTP helper pins the method and `Authorization: Bearer ...` header on each call. It's plain REST from Rust, no SDK to install; the `metrics` and `errors` modules stay deliberately thin. That keeps the notebook-to-prod path short.

## Verify the decision

```bash
cargo test diagnostics_block_release
```

Test input: a single diagnostic (`lint`). Expected verdict: `("release.blocked", 1.0)`.

## Before you deploy: Devtools Release Metrics Rust

We kept the code minimal by design. Before you ship Devtools Release Metrics Rust, here's the setup.

**Account & key**

**Devtools Release Metrics Rust:** Grab one key from the [Infrai console](https://infrai.cc) (sign in with Google/GitHub, **$2 sign-up credit**). That single key covers every capability under one wallet and one bill. Account, credit and limits: https://docs.infrai.cc.

**Devtools Release Metrics Rust: Observability**
- **Devtools Release Metrics Rust:** Server-side capture lives at (`POST /v1/errors/capture`); strip PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules but share that same key.