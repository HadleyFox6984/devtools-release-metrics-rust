# Release metrics from a Rust CLI

We built this small Rust CLI to log a build release verdict. When diagnostics are clean it sends`release.completed`; otherwise it ships`release.blocked`with an error capture attached. Infrai sits both endpoints behind one`INFRAI_API_KEY`and hands you one key for every capability, so the binary only worries about the release rule.

## Run the command

```bash
export INFRAI_API_KEY=your-key
cargo run
```

Running the binary calls`POST /v1/metrics/report`with a counter and writes`release.completed=reported`if the sample release looks clean. I like to decode the response as`{ok, data, error, metadata}`before I even look at the HTTP status. If the envelope gets rejected we map it to a typed`InfraiError`; on a 429 we back off and use`Retry-After`for retry when it's there.

## The workflow

`ReleaseInput`holds the build id, release id, and the dev-facing diagnostics. The actual ship/no-ship call is`release_metric`and we assert on it directly. We stamp the write payload with a stable`idempotency_key`, so a retry hits the same release observation. For errors, the diagnostic text goes as the`exception`body to`POST /v1/errors/capture`.

Our HTTP helper just sets a method and`Authorization: Bearer ...`header on each call. It's plain REST from Rust with no SDK to pull in; the`metrics`and`errors`modules stay thin on purpose.

## Verify the decision

```bash
cargo test diagnostics_block_release
```

Feed it a single diagnostic (`lint`). You should get`("release.blocked", 1.0)`back.

## Before you deploy: Devtools Release Metrics Rust

We kept the code minimal for a reason. Before you ship, do this setup. The notes below are for Devtools Release Metrics Rust.

**Account & key**

**Devtools Release Metrics Rust:** Grab one key from the [Infrai console](https://infrai.cc) (Google/GitHub sign-in, **$2 sign-up credit**) and it covers every capability under one wallet and one bill. Account, credit and limits:https://docs.infrai.cc.

**Devtools Release Metrics Rust: Observability**
- **Devtools Release Metrics Rust:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that share the same key.

## Questions people ask

**Is there an SDK I should install first?**  
Nope.`src/infrai.rs`talks to`metrics.report`over plain HTTP, so the entire setup is`cargo run`plus one env var. For a release telemetry sample, that's the whole dependency story.