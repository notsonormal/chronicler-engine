---
diataxis: how-to
title: Debugging the Engine
---

## Quick Start

Run the checks in this order. Most failures resolve before step 7.

1. **Read the failing test's source.** The test body tells you what was expected. The `*_tests.rs` sibling-file convention is the layout; open the file the test name implies.
2. **Run the failing test directly.** `cargo nextest run -p chronicler_engine <test_path>` (project standard), or `cargo test -p chronicler_engine --test <name>`. Read the failure message and location.
3. **Run `cargo clippy -p chronicler_engine --all-targets -- -D warnings`.** Catches the build/lint class of failure that may be the actual cause.
4. **Run `python build.py`.** Standard full validation (fmt + clippy + tests + coverage). If green, the failure is logic, not build.
5. **Inspect the diff.** `git diff`, `git status --short`, `git log @{u}..HEAD`. Most test failures after a code change are diff-visible.
6. **For layer/import violations**, run `arch-lint` (or read the deny messages from `cargo build`).
7. **For runtime/server-startup hangs**, spawn the binary manually: `cargo run -p chronicler_engine -- --world <name> --persona <name>`. Check stdout/stderr and `ss -tlnp` for port bindings. Reach for `RUST_LOG=info` or `=trace` only when steps 1–6 don't surface the bug — see next section.

## Read Tracing Output

`RUST_LOG` is read by `bootstrap/logging.rs::init_logging()`; only `main.rs:15` calls it. Test binaries do not initialise the subscriber themselves.

For integration tests that spawn the engine as a subprocess, `tests/test_utils/server.rs:126,138` **hardcodes** `chronicler_engine=debug` on the child — it does not forward a user-set `RUST_LOG`. To get `=trace` output from the child, rebuild and rerun the failing test against the manually-spawned binary with `RUST_LOG=trace cargo run -p chronicler_engine -- --world <name> --persona <name>`.

Raw `RUST_LOG=info` and `RUST_LOG=trace` (no module filter) are the dominant patterns; module-filter patterns are rare. `cargo nextest run <name>` is the project standard (not `cargo test`); `--nocapture` works the same way under either runner.

Most debugging doesn't need tracing — the test failure message plus source reading usually suffices. Reach for `RUST_LOG=trace` only when the bug is in runtime behaviour the test output doesn't surface.

## Diagnose Common Failures

### Tracing output is empty despite `RUST_LOG=info`

**Cause.** The subscriber is only initialised through `main.rs`; test binaries do not call `init_logging()`. For test runs the engine-subprocess subscriber only fires when the fixture forwards `RUST_LOG` — which the project's fixture does not (it hardcodes `chronicler_engine=debug`, see above).

**Fix.** Re-run the failing test with the engine binary spawned manually under `RUST_LOG=trace`, or accept that tracing output will not appear in test logs.

### Test panicked before any state was visible

**Cause.** Panic in early init / DB migration / fixture setup, before the test body runs.

**Fix.** Run the test in isolation: `cargo nextest run -p chronicler_engine --test <name> -E 'test(<exact_name>)'`. Read the panic backtrace and the last log line before the panic.

## Diagnose by Error Variant

Variants with non-trivial First Checks. For variants not listed here, the variant name + payload in the error message is the diagnostic; see `src/error.rs`.

### `EngineError::Llm(LlmFailure::Http { status, body })`

**First Check.** The `status` code in the error payload: `401` = API key issue; `429` = rate limited; `5xx` = provider outage.

**Common Causes.** Invalid API key (note: keys live on `LlmProviderConfig.api_key`, not `AppSettings`; the `OPENROUTER_API_KEY` env-var fallback lives in `LlmProviderConfig::resolve_api_key()` at `src/domain/model/settings.rs:83-94`). Rate limiting. Model-routing failure. Provider maintenance. The response body is captured in `body` for forensics.

### `EngineError::Llm(LlmFailure::Network { url, detail })`

**First Check.** Reachability of `url` from the host: `curl -I <url>`.

**Common Causes.** Ollama not running. Network partition. DNS failure. Configured overall request timeout exceeded. Truncated gzip stream. Server closed connection. TLS handshake failure.

### `EngineError::Llm(LlmFailure::Timeout)`

**First Check.** `RUST_LOG=debug` logs for `[LLM][req:N] Request failed after ...`.

**Common Causes.** Configured overall timeout exceeded. Model too slow for the prompt size. Network congestion.

### `EngineError::Llm(LlmFailure::ParseError { raw_response, expected_format })`

**First Check.** The `raw_response` payload in logs. Is it valid JSON? Does it carry the `expected_format` shape?

**Common Causes.** Model returned non-JSON prose. Response missing `choices[0].message.content`. Streaming response when `stream: false` was requested.

### `EngineError::Narrative(NarrativeFailure::Generation { stage, reason })`

**First Check.** The `stage` field. The mock backend uses `stage: "mock"` for narration and `stage: "mock_trigger"` for trigger continuation. Cross-reference backend logs for the actual LLM call path.

**Common Causes.** LLM call failed after prompt built successfully. Backend misconfiguration (e.g. DeepSeek not implemented).

### `EngineError::ContextOverflow { requested, max }`

**First Check.** The token-budget calculation in `src/application/narrative_prompt/`.

**Common Causes.** History too long. System prompt too large. Combined context exceeds `max_context_tokens`.

### `EngineError::WorldHasGames { game_count }`

**First Check.** The `game_count` payload. List games whose `world_key` matches the offending world.

**Common Causes.** Games created against the world after the delete attempt began. Admin trying to delete a world with active sessions. The engine's default SQLite file is `chronicler.db` (relative to CWD), not `data/chronicler.db` (`src/bootstrap/run.rs:59`).

## Document References

- [`../reference/coding_standards/testing.md`](../reference/coding_standards/testing.md) — testing policy; the `*_tests.rs` sibling-file convention; test categories. The runtime `llm_messages` forensics table and the `RecordingForensics` test spy are documented there as test-writing helpers, not debugging tools.