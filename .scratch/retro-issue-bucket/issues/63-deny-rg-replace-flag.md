# Deny `rg -r` and `rg -rn` with a reason in the permission config

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

In ripgrep, `-r` is `--replace`, so `rg -rn "pat"` prints each match with the match text replaced by `n`. A scout in this run used it seven times, decided that the harness redacts matches, and moved to `sed`. The permission config already denies a repo-root `grep` with a teaching reason. Should it deny `rg -r` the same way?

- **Add a deny rule** for `*rg -r*` and `*rg -rn*` with the reason "in ripgrep `-r` means --replace; use `rg -n`". This needs the owner's approval.
- **One line in `AGENTS.md`** instead (always in context, no enforcement).

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- Scout `b42e8412` entries 74, 77, 80, 83, 91, 101, 105; thinking at 86 and 103 ("the display filter is mangling matched text").
- `.pi/extensions/pi-permission-system/config.json`: the `grep * /workspace/chronicler-engine *` rule and its reason.
