# Make the mutation check unconditional for state-repair tests and fixtures

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Two dead tests survived a green gate because the masking was structural, not suspicious. `tests/http/games_switch.rs` asserted `fetch_generating_status(&app) == "idle"`, but `/status/generating` answers from the live registry and reports `idle` whatever the handler does. A fixture swallowed its setup result with `let _ = try_claim(...)`, so a rejected claim left two channels silently disagreeing.

`tdd/SKILL.md:51` gates the mutation check on a test "looking suspicious". A structurally masked test does not look suspicious.

Should the check be unconditional in some cases?

- **Yes — state-repair tests and swallow-fixtures.** Any test asserting a repaired state, and any fixture that discards a setup result, must be shown to fail before the change.
- **Keep the suspicion trigger**, and add the two masking shapes to the anti-pattern list.
- **A lint instead.** A guardrail flags `let _ = <result>` in `tests/` and assertions on recomputed projections.

## Context

- Source: `/reflect` 2026-10-07, session `01a1175e` (standards axis) and its fix session.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 16).
- `tests/http/games_switch.rs` — scenario 18.3; the fix replaced `let _ = try_claim(...)` with `assert!(...)`.
- `.agents/skills/tdd/SKILL.md:51` — "when a test looks suspicious".
- `tests/AGENTS.md` rule 5 — "Investigate pre-existing and flaky failures too".
