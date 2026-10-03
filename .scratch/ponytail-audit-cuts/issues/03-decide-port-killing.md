# 06: Decide: remove Windows port-killing bootstrap?

Type: grilling
Status: resolved

## Question

Should `bind_with_retry` and the Windows-only `port_utils` (netstat/taskkill process killing) be removed? The dev server currently tries to kill any process on its port before binding. On non-Windows platforms this code is already dead; on Windows it is a fragile dev-environment convenience.

## Blocking

- A follow-up task ticket to remove the code will be created if the decision is "yes".

## Notes

Trade-offs to weigh:
- Delete: simpler bootstrap, one failed `bind` path, less platform-specific code. Port conflicts become the caller's problem.
- Keep: preserves the current Windows dev UX of auto-freeing the port.
- Alternative: keep retry but drop the kill, so the server just waits/reports the conflict.

Resolution records the decision in the map's Decisions-so-far and creates the execution ticket if needed.

## Answer

Delete. Decided 2026-10-03 in the grilling session for `docs/plans/build-py-run-step-plan.md`, which owns the execution (no separate ticket).

- Remove `port_utils.rs` (`find_process_on_port`, `kill_process`), `bind_with_retry()`, and `ServerConfig.bind_attempts`. The server binds once and fails with "Failed to bind to port N" on a busy port.
- Also remove the `build.py` equivalents: `kill_port()` (gate prelude) and `kill_by_name()` (`--cleanup`).
- Port conflicts move to the caller: `python build.py run` runs a bind test before the build and before the exec.
- Verified on Linux: every one of these helpers calls a tool that is missing here (`netstat`, `Select-String`, `tasklist`, `taskkill`), so the removal changes behaviour only on Windows. On Linux the current server retries forever on a busy port, and the change turns that hang into an error.
