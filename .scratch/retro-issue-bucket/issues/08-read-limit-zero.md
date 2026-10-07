# Decide whether `read` with `limit=0` needs a note

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Both implementation sessions opened with `read(offset=0, limit=0)` calls. Each returned only a pagination notice (`[N more lines in file]`), so the agent re-read the file. The t68 session spent two of its first four reads on the retry.

`limit=0` is a valid parameter value that yields no content. Is a repo-side note the right fix?

- **Note it in `AGENTS.md`.** "Pass `offset=1`; `limit=0` returns no content."
- **Fix it in the harness.** Treat `limit=0` as the default limit.
- **Drop it.** Two wasted reads per session is within noise.

## Context

- t68 session entry idx 7–9; files `68-split-settings-sub-tabs.md`, `map.md`.
- t70 session entry idx 7–9; files `70-remove-story-log-check.md`, `map.md`.
- The pagination notice read: `[34 more lines in file. Use offset=1 to continue.]`.
