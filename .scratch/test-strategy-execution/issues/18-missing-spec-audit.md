# Audit missing specs across HTTP E2E + browser

Type: research (AFK)
Status: open
Blocked by: 10

## Question

Graduates from [ticket 10](10-partition-codebase-audit.md).

Per STRATEGY.md, specs live at **HTTP E2E + browser only**. Domain,
application, and storage unit tests are branch coverage — no spec. This
ticket audits the spec-owning surface for gaps, orphans, and drift.

Scope:

- **HTTP E2E** — scan `src/adapters/driving/http/` (23 unit files, 124
  tests) for endpoints, cross-reference `tests/http/` (20 files, 137
  tests) and the 9 HTTP specs (`actions.md`, `reset.md`, `story_log.md`,
  `swipe_new.md`, `retrigger.md`, `games_create.md`, `games_switch.md`,
  `games_delete.md`) for:
  - **Gap** — endpoint with HTTP-observable behaviour but no spec scenario
  - **Orphan** — spec scenario with no HTTP E2E test
  - **Drift** — spec scenario whose HTTP E2E test asserts different behaviour
- **Browser** — scan `tests/browser/` (3 files, 15 tests) vs
  `browser.md` (6 scenarios + 1 invariant) for the same three failure
  modes.

## Output

A single markdown asset (linked from this ticket) with three sections:

1. HTTP E2E findings (one row per finding: endpoint, spec, test, mode)
2. Browser findings
3. Summary counts (gaps / orphans / drift per tier)

This ticket **maps only** — no spec edits, no new tests. Fixes execute
on a future effort (out of scope for this map).

## Out of scope

- Domain/application/storage unit branch coverage — separate ticket (19).
- `tests/integration/` + `tests/llm/` — ticket 17.
- Implementing any fix — investigation only on this map.
