# 01: Convert three `*_impl` orchestrators to methods on DefaultApplicationService

Status: open
Type: task
Assignee: (unassigned)
Blocked by: (none)

## Question

Convert `execute_action_impl`, `retry_last_response_impl`, `retrigger_event_impl` from free fns to methods on `impl DefaultApplicationService`, removing the parked-orchestrator category from the scanner's residual list before the new module-allowlist rules land.

## Context

- `execute_action_impl(app: &DefaultApplicationService, input: String)` in `src/application/action_pipeline/actions.rs:9` — reads `app.load_or_fresh()`, calls `app.game_service().pipeline().run_from_input(app, state, input)`, handles `PhaseError::Cancelled`.
- `retry_last_response_impl(app: &DefaultApplicationService)` in `src/application/action_pipeline/retry.rs:41`.
- `retrigger_event_impl(app: &DefaultApplicationService)` in `src/application/action_pipeline/retry.rs:169`.
- All three take `&DefaultApplicationService` as first arg and orchestrate its collaborators. Textbook method candidates — NOT honest free fns.
- They were parked as free fns during the T3 cleanup to keep the service's API surface slim (the cleanup deleted 14 identity-passthrough methods). The "spawn-blocking needs owned Arc" doctrine in `plan-eliminate-free-function-smells-final.md` does NOT apply — verified: no `spawn_blocking` / `tokio::spawn` in `actions.rs` or `retry.rs`. The spawn lives in `bootstrap/init_game.rs::ArrivalTaskContext`, a different concern.

## Target shape

```rust
impl DefaultApplicationService {
    pub fn execute_action(&self, input: String) { ... }
    pub fn retry_last_response(&self) { ... }
    pub fn retrigger_event(&self) { ... }
}
```

Names drop the `_impl` suffix — they become the public entry points on the service, not internal helpers. (If a reviewer objects that `_impl` was signalling "entry point called from spawn context", keep the suffix; but the spawn context is in `ArrivalTaskContext`, not here, so the suffix is currently misleading.)

## Call sites

- `src/application/action_pipeline/mod.rs:17,20,21` — re-exports `execute_action_impl`, `retrigger_event_impl`, `retry_last_response_impl`. These become `DefaultApplicationService` methods; the re-exports are deleted (callers use `app.execute_action(...)` directly).
- `src/application/action_pipeline/actions_tests.rs` — 15+ call sites `execute_action_impl(&app, ...)` → `app.execute_action(...)`.
- `src/application/action_pipeline/retry_tests.rs` — 20+ call sites `retry_last_response_impl(&app)` / `retrigger_event_impl(&app)` → `app.retry_last_response()` / `app.retrigger_event()`.
- `src/application/message_editing.rs` and `src/application/generation_gate/gate.rs` — internal callers; verify and migrate.

## Verification

- `cd chronicler_engine && cargo fmt && cargo clippy --all-targets -- -D warnings`
- `cargo nextest run --lib` + `cargo nextest run --tests` (retry/actions integration tests)
- `python scripts/find_free_fn_smells.py` — smell count drops by 3 (the three parked-orchestrator suppressions can be removed from `SUPPRESSED_FREE_FNS` once converted).
- `python build.py` — full gate green.

## Story points

2 SP. Mostly mechanical `sed` rename across tests + a few internal callers. No logic change.

## Notes

- This ticket is execution work, not a decision. It earns its place by unblocking the module-allowlist scanner: once these three convert, the scanner's residual suppression list contains zero parked-orchestrator entries — only the 18 honest free fns remain, all of which fit categorical buckets.
- TODO from this ticket, if any: update the Free fn Doctrine section in `system.md` to remove the "Arc-self spawn-blocking orchestrator" category row (no longer applies).
