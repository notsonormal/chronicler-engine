# Make the mutation check unconditional for state-repair tests and fixtures

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Two dead tests survived a green gate because the masking was structural, not suspicious. `tests/http/games_switch.rs` asserted `fetch_generating_status(&app) == "idle"`, but `/status/generating` answers from the live registry and reports `idle` whatever the handler does. A fixture swallowed its setup result with `let _ = try_claim(...)`, so a rejected claim left two channels silently disagreeing.

The `tdd` skill gated the mutation check on a test "looking suspicious" (`.agents/skills/tdd/SKILL.md:51`, deleted 2026-11-01). A structurally masked test does not look suspicious. No written rule now asks for the check.

Should the check be unconditional in some cases?

- **Yes — state-repair tests and swallow-fixtures.** Any test asserting a repaired state, and any fixture that discards a setup result, must be shown to fail before the change.
- **Keep the suspicion trigger**, and add the two masking shapes to the anti-pattern list.
- **A lint instead.** A guardrail flags `let _ = <result>` in `tests/` and assertions on recomputed projections.

## Context

- Source: `/reflect` 2026-10-07, session `01a1175e` (standards axis) and its fix session.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 16).
- `tests/http/games_switch.rs` — scenario 18.3; the fix replaced `let _ = try_claim(...)` with `assert!(...)`.
- `.agents/skills/tdd/SKILL.md:51` — "when a test looks suspicious". Deleted with the skill on 2026-11-01; the quoted rule is gone from the tree.
- `tests/AGENTS.md` rule 5 — "Investigate pre-existing and flaky failures too".

## Comments

- 2026-10-21, triage pass 2: Ticket 55 (closed; see the map) folded in: widen the trigger to "a test that turns green in a step that changes its subject".
- 2026-10-25, audit: the user deferred this ticket. Do not implement it yet; it stays open.
- 2026-11-01: the `tdd` skill was deleted at the user's request. This ticket now decides which *check* replaces the prose rule (option 3, a guardrail, or the mutation tool in `../unclebob-quality-tools/`), not how to reword it.

## Further evidence (2026-11-08, statechart-models tickets 08–16, coordinator `01a1bd22`)

- **known** t12 (`549a889a`) copied `src/application/generation/gate.rs` to `tmp/gate.rs.bak`, ran two scratch mutants with `test-pattern` and restored the file. t11 (`6f1e3893`) wrote `/tmp/mut.py`, which wrote 5 mutants into `gate.rs` while t16 built in the same checkout.
- **known** Neither agent held the build slot for the whole mutate, test and restore cycle. A build by another agent in that time compiles the mutant, and a killed shell leaves it in place. Both agents wrote the same harness again.
- **Requirement for a mutation tool:** it takes the old text, the new text, the test pattern and the target. It holds the build slot for the cycle, restores the file in `finally`, and reads its own log.
