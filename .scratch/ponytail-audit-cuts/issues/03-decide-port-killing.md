# 06: Decide: remove Windows port-killing bootstrap?

Type: grilling
Status: open

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
