# Name the tier-1 wait helpers in `tests/AGENTS.md`

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`tests/AGENTS.md` lists `http_requests.rs` as "generation-idle polling" but does not name `wait_idle`, `wait_settled` or `wait_for_jobs`. Three scouts each searched two to four times to find them. Should the index line name them?

- **Name them in the module summary of `http_requests.rs`** (the index is generated from it).
- **Leave it.**

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- `tests/AGENTS.md` line 118; scouts `b42e8412` entry 105, `699961ee` entries 232 to 241, `e3690c86` entries 232 to 238.
