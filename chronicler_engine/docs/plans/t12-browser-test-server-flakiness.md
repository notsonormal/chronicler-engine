# T12: Browser Test Server Flakiness Fix

**Status:** Implementing — design confirmed 2026-07-13 via plan-grilling review
**Original date:** 2026-07-11
**Depends on:** none
**Blocks:** none
**Priority:** P2 (pre-existing flakiness masked by `--retries 2`; cost is ~10s wasted runtime + false signal)

## Final design (amended from original)

After plan-grilling review (2026-07-13), the following simplifications were applied:

- **Phase 1 dropped entirely** — no diagnostic instrumentation. Empirical fix-and-measure only.
- **Phase 2 ships as a single atomic commit** in `tests/test_utils/server.rs`, not three separate edits. Internal sequential test gates during dev work (H1 → H2 → H4), but commit history shows one change.
- **H1 fix = PID registry** — static `OnceLock<Mutex<HashMap<u16, u32>>>` keyed by port. Spawn records `(port, pid)`; kill looks up by port + `libc::kill(pid, SIGTERM)`. Surgical, no collateral risk to unrelated dev servers.
- **H2 fix = HTTP readiness probe** — `reqwest::get("/")` (already in dev-deps, no new dep) with per-attempt timeout, retries on non-200.
- **H4 / Task 2.3 dropped** — no `CE_TEST_SERVER_TIMEOUT_SECS` env var. Literal 30s timeout retained (300 attempts × 100ms). If CI proves this too tight, future fix.
- **Phase 3 / Task 3.1 replaced** — no parallel-10-server stress test (didn't match observed failure mode). Regression gate = `cargo test editing::test_delete_removes_message --retries 0 --test-threads=1` × 50 runs + `python build.py` × 3.
- **No documentation update**, no production code changes. Pure test-infra fix.

### Stop condition
Plan closes when: 50 consecutive reruns of `test_delete_removes_message` all pass without retries, AND `python build.py` passes 3 times consecutively. If all three Phase-2 fixes are in place and flake rate still exceeds 5%, escalate to a fresh diagnostic round (this plan does not pre-commit).

### Attribution caveat
Single-commit deliverable means we cannot isolate which of H1/H2/H4 actually mattered. Acceptable tradeoff for cleaner history.


## Summary

`tests/browser/editing.rs::test_delete_removes_message` intermittently fails on first attempt, passes on retry. test-police reviews 1 + 2 both observed flakiness at the ~9-30s mark. Root cause lives in `tests/test_utils/server.rs` test server bootstrap, not in T9-01 changes. Plan: investigate + harden server readiness probe + fix platform-specific process-cleanup bug.

## Evidence

	```
FLAKY 2/3 [   9.595s] ( 853/1241) chronicler_engine::browser editing::test_delete_removes_message
     Summary [ 274.353s] 1241 tests run: 1241 passed, 2 skipped
	```

(test-police review 2; review 1 saw `Server failed to start on port 30xx` at the 30s mark)

## Scope

### In scope
- `tests/test_utils/server.rs` — `kill_existing_server`, `wait_for_server`, `start_server_with_env`
- `tests/test_utils/browser.rs` — `goto_with_connection_check`, `with_test_page` (if readiness probe needs to move)
- `tests/browser/editing.rs::test_delete_removes_message` — diagnosis only, not the test body itself

### Out of scope
- Browser test bodies (assertion logic unchanged)
- Port allocation algorithm (`get_available_port` — already robust with file locks + PID tracking + stale-lock cleanup)
- Server.rs's stdout/stderr capture (already implemented — buffers surfaced on panic via `eprintln!`)
- Other flaky tests (only `test_delete_removes_message` flagged consistently across both reviews)

## Root cause hypotheses (to verify during impl)

### H1 — `kill_existing_server` is a no-op on Linux (PLATFORM BUG)
**File:** `tests/test_utils/server.rs:22`

	```rust
let _ = Command::new("taskkill")
    .args(["/F", "/IM", "chronicler_engine.exe"])
    .output();
	```

`taskkill` is a Windows builtin. On Linux/macOS this `Command::new` returns `Err` (binary not found); `let _ =` swallows it. Result: previous test's server process (if still alive) is never killed; port stays bound; new child spawn can't bind; 30s panic.

The `SERVER_MANAGED` AtomicBool is process-global, so within one test process this is mostly fine — but across parallel test processes or zombie survivors from previous runs, this fires.

Note: the test-police reviews claimed stdout/stderr was "discarded" at `server.rs:333` — that was WRONG. Current code captures stdout/stderr into `Arc<Mutex<Vec<u8>>>` buffers and surfaces them via `eprintln!` on panic. The real bug is the platform-incompatible kill command.

**Verify:** `cargo nextest run -j 4 --test browser` on Linux with `strace`/`lsof` attached — confirm `taskkill` returns `ENOENT` while a port is genuinely held.

### H2 — `wait_for_server` readiness check doesn't actually probe HTTP
**File:** `tests/test_utils/server.rs:131-145`

	```rust
pub async fn wait_for_server(port: u16, max_attempts: usize) -> bool {
    for _ in 0..max_attempts {
        if port_in_use(port) {
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            if std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                return true;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    false
}
	```

Returns `true` when TCP accepts a connection. Axum bind first, route table second — the server can accept TCP but 500 on HTTP for ~100-500ms after bind. Browser test then fires `page.goto(url)` → empty/error response → test flake.

**Fix direction:** probe `GET http://127.0.0.1:{port}/` (returns `index_handler` HTML 200 — verified route exists at `src/adapters/driving/http/router.rs:21`) instead of raw TCP connect. Use `reqwest::get` with short timeout. Retry until 200.

### H3 — TIME_WAIT port reuse race (POSIX)
**File:** `tests/test_utils/server.rs:111` `get_available_port`
`TcpListener::bind` succeeds during allocation probe → `drop(listener)` → port enters TIME_WAIT on Linux → spawned binary tries to bind same port → EADDRINUSE → server crashes → 30s panic.

The allocation listener is closed before spawn, but Linux default `TIME_WAIT=60s`. Without `SO_REUSEADDR` on the binary's listen socket (which axum does set by default — verify), this race can fire.

**Verify:** check `lsof -i :3010` after a failed test run; if `TIME_WAIT` entries exist, this is confirmed.

### H4 — Fixed 30s budget too tight under CI load
**File:** `tests/test_utils/server.rs:170`

	```rust
let started = wait_for_server(port, 300).await; // 300 * 100ms = 30s total
	```

On a cold CI worker or disk-thrashing local machine, server binary spawn + sqlite init + template load can exceed 30s. Configurable via env var: `CE_TEST_SERVER_TIMEOUT_SECS` (default 30, CI override 90).

**Verify:** instrument actual startup time in a debug build; check if any observed failure hit the 30s mark exactly.

## Implementation

### Phase 1: Diagnosis (verify which hypothesis is active)

- [ ] #### Task 1.1: Add startup timing instrumentation (2 SP)
  - Add `tracing` log line at spawn + ready with elapsed ms
  - Run `python build.py` 5x locally, 1x on CI, capture distribution
  - Verify: produces a histogram of startup times

- [ ] #### Task 1.2: Confirm Linux `kill_existing_server` no-op (1 SP)
  - Run a test that leaves a server running (insert artificial sleep), confirm taskkill fails
  - Use `strace -f -e trace=execve cargo nextest run` or `ps aux | grep chronicler_engine` mid-test
  - Verify: documented evidence (log paste) that taskkill returned `ENOENT` on Linux

### Phase 2: Hardening (apply fixes based on confirmed diagnosis)

- [ ] #### Task 2.1: Fix `kill_existing_server` cross-platform (3 SP)
  - Replace `Command::new("taskkill")` with:
    - Windows: keep `taskkill /F /IM chronicler_engine.exe`
    - Unix: `Command::new("pkill").args(["-f", &format!("chronicler_engine --port {port}")])` OR (preferred) track child PIDs via a temp file / static registry and `libc::kill(pid, SIGTERM)`
  - Verify: `kill_existing_server` test (spawn a fake server, call kill, confirm port released within 500ms)

- [ ] #### Task 2.2: Replace TCP probe with HTTP readiness probe (3 SP)
  - Add `async fn probe_http_ready(port: u16, timeout: Duration) -> bool` using `reqwest` (already in deps? if not, use raw `tokio::net::TcpStream` + manual `GET / HTTP/1.0\r\n\r\n` to avoid new dep)
  - Replace the `port_in_use(port) + sleep + TcpStream::connect` block with `probe_http_ready(port, Duration::from_millis(500)).await`
  - Retry budget stays at 300 attempts but now each attempt confirms HTTP 200, not just TCP accept
  - Verify: test that injects 200ms HTTP-accept-but-no-route delay proves probe waits correctly

- [ ] #### Task 2.3: Make timeout configurable via env var (1 SP)
  - Add `pub fn default_timeout_secs() -> u64 { 30 }` helper
  - `std::env::var("CE_TEST_SERVER_TIMEOUT_SECS").map(|s| s.parse().unwrap_or(30)).unwrap_or(30)`
  - Replace literal `300` in `wait_for_server(port, 300)` with `timeout_secs * 10`
  - Document in `docs/reference/testing.md`
  - Verify: `CE_TEST_SERVER_TIMEOUT_SECS=90 cargo nextest run` overrides default

### Phase 3: Regression guard

- [ ] #### Task 3.1: Add startup-stress test (2 SP)
  - New test `tests/test_utils/server_stress_tests.rs` — spawns 10 servers in parallel across the port range, asserts all come up within timeout, all release ports cleanly on Drop
  - Reuses `TestServer::new_with_mock`
  - Verify: stress test passes 5x consecutively without `--retries`

## Test Plan

1. **Unit**: each Phase 2 task has its own verify step (above)
2. **Stress**: Phase 3.1 stress test
3. **Observational**: run `python build.py` 10x locally; previously flaky on ~1/3 of runs; post-fix should be 10/10 green without `--retries` adjustment
4. **Regression**: confirm `tests/browser/` suite still passes end-to-end (nothing broken by readiness probe change)

## Per Task/Sub Task Validation Steps

- After Task 1.x: `cargo build --tests` GREEN; instrumentation logs land in build output
- After Task 2.1: `cargo test --test browser editing::test_delete_removes_message -- --retries 0` 5x all pass; unit test for kill_existing_server passes
- After Task 2.2: `cargo test --test browser` 3x all pass with `--retries 0`; HTTP probe unit test passes
- After Task 2.3: `CE_TEST_SERVER_TIMEOUT_SECS=90 cargo test --test browser` passes; env var absent → 30s default works
- After Task 3.1: `cargo test --test test_utils` (or wherever stress test lands) passes 5x consecutively
- Final: `python build.py` 5x consecutively — 0 browser-test retries needed

## Assumptions

- Flakiness is test infrastructure (server.rs readiness probe + kill semantics), not the browser test body or the engine HTTP router
- `reqwest` is preferred for HTTP probe; if adding it as a dev-dependency is undesirable, manual raw-HTTP-over-TcpStream is 5 lines and avoids new deps
- No new public API in production code — all changes confined to `tests/`
- Existing stdout/stderr capture + `eprintln!` on panic in `TestServer::start` is sufficient diagnostic; no need to switch to tempfile logging
- Port range 3010-3050 (40 ports) is wide enough for parallel test execution; only `test_delete_removes_message` has been observed flaky, suggesting the others use lower-contention ports
- Reviews' earlier claims about "discarded stdout/stderr" at `server.rs:333` are stale — verified stdout/stderr IS captured and surfaced via `eprintln!` on panic

## Decisions left to implementer (LOW RISK — ask user only if blocked)

- `reqwest` vs manual raw-HTTP-over-TcpStream for the readiness probe
- Whether to add a `/health` endpoint on the production server (out of scope — use `/` route which already exists via `index_handler`)
- Whether to log startup-time histogram to stdout or to a metrics file (implementation-detail)

## References

- `tests/test_utils/server.rs:22` — `kill_existing_server` (taskkill no-op on Linux)
- `tests/test_utils/server.rs:131-145` — `wait_for_server` (TCP connect-based readiness)
- `tests/test_utils/server.rs:170` — `wait_for_server(port, 300)` call site (fixed 30s budget)
- `tests/test_utils/browser.rs:42-58` — `goto_with_connection_check` (consumer of wait_for_server, uses 100 attempts instead of 300)
- `tests/browser/editing.rs:163` — `test_delete_removes_message` (flaky test)
- `tests/test_config.json` — port range 3010-3050
- `src/adapters/driving/http/router.rs:21` — `.route("/", get(index_handler))` (HTTP probe target)
- test-police review 2 Finding 1 (`tmp/test-police-review-2.md`)
- test-police review 1 Findings 1+2 (same flakiness signature, `tmp/test-police-review.md`)
