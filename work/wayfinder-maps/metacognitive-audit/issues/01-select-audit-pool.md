# Select the audit session pool

Type: task
Status: resolved
Blocked by:

## Question

From the chronicler-engine session files dated 2026-07-21 through 2026-08-20, which contain substantive software-engineering problem-solving by the user?

Define "substantive software-problem-solving turn": a user turn that states a hypothesis, makes a design or architecture choice, pushes back on an AI suggestion, asks a clarifying question about the problem (not about tooling), reports a bug or observation about behavior, or drives a refactor. Exclude pure skill/subagent auto-runs and session-management chatter.

Apply the definition to all chronicler-engine sessions in the window. Exclude the two auditor sessions (this one, and prior run id `01a02084-3749-7047-9e4b-9df531a609d5`) — they are meta, not software problem-solving.

Produce a manifest asset listing each qualifying session: file path, date, count of substantive user turns, one-line reason for inclusion. Link the manifest from this ticket.

This ticket defines the pool every downstream ticket reads from. If the pool looks wrong (too large, too small, or visibly missing reasoning sessions), revisit the definition before extraction.

## Answer

Audit pool selected: **30 qualifying sessions** (~161 substantive user turns) out of 83 top-level chronicler-engine session files. Manifest asset: [`assets/audit-pool-manifest.md`](../assets/audit-pool-manifest.md).

**Scope finding (important):** the map's 30-day window (2026-07-21 → 2026-08-20) is only 10 days of data. The chronicler-engine project's earliest session is 2026-08-11 — the repo was moved out of `mrn-general` into its own project that day, so pre-08-11 work lives under `mrn-general` (out of scope, Q5=B). The pool is inherently limited to 2026-08-11 → 2026-08-20. This is data availability, not a scoping choice; flagged for the user.

**Selection applied:** substantive software-problem-solving turn = hypothesis / design choice / pushback / problem-clarifying question / bug-or-behavior observation / refactor drive. Excluded pure skill/subagent auto-runs (incl. all 21 `tasks/` subdirs), tooling/environment chatter, session-management chatter, and the two auditor sessions (`01a02084` prior run, `01a020c2` this session). Included design/planning/reasoning sessions per Q9=B — wayfinder/grilling sessions with real design decisions are in.

**Pool shape:** design/planning-heavy. Richest sessions: 11 (guardrails/invariants, ~15 turns), 17 (guided-generations, ~10), 20 (after-plan review, ~10). Thinnest: 18, 19, 25, 26, 27 (1–2 substantive turns).

**Fog cleared:** pool size is 30 sessions, ~161 substantive turns — fits one extraction session reading the flattened transcripts, no batching. The map's "Extraction overflow" fog is resolved. The "Substantive-turn definition tuning" fog held: the definition produced a reasonable pool (not too large/small, reasoning sessions present), no revisit needed.

**Downstream:** extraction (03) reads `tmp/flattened_pool/*.md`; verification (04) substring-checks against those files; assessability (05) has a design-heavy pool, so Achievement (band-aid vs refactor) may still need diffs for the impl sessions.
