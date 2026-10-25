# Decide how to partition the codebase-wide spec/tier audit

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

The codebase has **1177 tests** across all tiers and **9 specs**
(`actions.md`, `browser.md`, `games_create.md`, `games_delete.md`,
`games_switch.md`, `reset.md`, `retrigger.md`, `story_log.md`,
`swipe_new.md`) after tickets 05/08/11/13. The consolidation (tickets
01–06) handled the component tier. But every other tested component
also needs a spec and correct tier placement — that work is
investigation only on this map (no implementation), but the landscape
needs to be mapped.

A single research ticket covering the entire codebase would be too large
(~160 test files, ~1177 tests). This ticket decides **how to partition**
the audit so it breaks into research tickets sized for one session each.

### The partition question

The natural seams are by tier (domain, application, HTTP adapter, storage
adapter, LLM adapter, browser, infrastructure) or by component (lifecycle,
action pipeline, storage, prompt-building, settings, etc.). But specs are
per-component, not per-tier — a single component's tests may span
multiple tiers (unit in `src/`, E2E in `tests/http/`, browser in
`tests/browser/`). Auditing by tier would split one component's story
across multiple audit tickets.

This ticket grills out:

- What is the right partition — by tier, by component, or some hybrid?
- What does each audit ticket produce? (A markdown artifact mapping
  components → tests → spec coverage → tier placement → gaps?)
- How do findings feed back into the map? (Fog in Not-yet-specified?
  Direct tickets if sharp enough?)
- Should the audit cover only spec gaps, or also tier misplacement (tests
  in the wrong tier per the strategy)?

### Current landscape (for context)

| Tier | Location | Tests | Files | Specs? |
|---|---|---|---|---|
| Unit (domain) | `src/domain/model/*_tests.rs` | 158 | 18 | none |
| Unit (application) | `src/application/**/*_tests.rs` | 229 | 28 | none |
| Unit (HTTP adapter) | `src/adapters/driving/http/**/*_tests.rs` | 124 | 23 | none |
| Unit (storage adapter) | `src/adapters/driven/storage/**/*_tests.rs` | 206 | 16 | none |
| Unit (LLM adapter) | `src/adapters/driven/llm/**/*_tests.rs` | 49 | 6 | none |
| Unit (bootstrap+utils) | `src/bootstrap/*_tests.rs`, `src/utils/*_tests.rs` | 50 | 6 | none |
| HTTP E2E | `tests/http/` | 137 | 20 | `actions.md`, `reset.md`, `story_log.md`, `swipe_new.md`, `retrigger.md`, `games_create.md`, `games_switch.md`, `games_delete.md` |
| Browser | `tests/browser/` | 15 | 3 | `browser.md` |
| LLM E2E | `tests/llm/` | 2 | 2 | none |
| Infrastructure | `tests/infrastructure/` | 114 | 12 | none |
| Integration | `tests/integration/{adapters,bootstrap,model,storage}` | 93 | 14 | none |

Total: 1177 tests, ~160 files. `retry.md` was deleted by ticket 05
(split into `swipe_new.md` + `retrigger.md`); the old `R1.x`–`R5.x`
IDs ride inside the two new specs. `docs/specs/action_pipeline.md` +
`flow.md` deleted by ticket 11. `tests/integration/application/`
(the dissolved component tier) is gone; the remaining `tests/integration/`
subdirs are a separate integration tier the audit must cover.

### What this ticket does NOT do

- Does not execute the audit — that's the research tickets that graduate
  from this ticket's resolution.
- Does not implement any fixes — the audit is investigation only on this
  map.

## Answer

Grilled 3 rounds. Partition decided: **by component cluster, by intent**.
The user partitioned the audit by intent rather than by tier, which
collapses the partition question — the graduates are pre-sliced by
what kind of gap each covers.

**4 graduating tickets** (all children of this map, all investigation
only — no fixes on this map):

1. [Storage integration spec decision](16-storage-spec-decision.md) —
   grilling (HITL). Decides whether the 6 `tests/integration/storage/*`
   files get specs or stay spec-less per STRATEGY.md. Blocks ticket 17
   (storage rows of the disposition table depend on the answer).
2. [Migrate existing integration tests to the right tier](17-integration-migration.md) —
   research (AFK). Disposition table for 14 `tests/integration/` files +
   2 `tests/llm/` files → port-to-unit / port-to-http-e2e / stay / delete.
   Blocked by 16 (storage spec decision shapes the storage rows).
3. [Audit missing specs across HTTP E2E + browser](18-missing-spec-audit.md) —
   research (AFK). Spec gap/orphan/drift scan across the spec-owning
   surface only (HTTP E2E: 23 driving-adapter unit files + 20 HTTP
   test files + 9 specs; browser: 3 test files + 1 spec). Domain /
   application / storage units have no spec by STRATEGY.md — covered
   implicitly when HTTP E2E covers its spec scenario.
4. [Audit branch coverage across src unit tests](19-branch-coverage-audit.md) —
   research (AFK). Maps uncovered branches across `src/` (100 files,
   816 tests); answers whether `ArrivalTaskContext::run` is isolated
   or a pattern. In scope per user decision (Q2 → B) despite the map's
   destination being component-tier dissolution.

**Blocking edges:** 16 → 17. Tickets 18 and 19 are independent.

**Cluster boundaries** (no separate artifact — the scope lives in each
graduate's body, per the "map is an index" rule):

- Integration migration (17) covers `tests/integration/{adapters,
  bootstrap, model, storage}/*` + `tests/llm/*` — the non-dissolved
  integration tier.
- Missing-spec audit (18) covers `src/adapters/driving/http/` +
  `tests/http/` + 9 HTTP specs + `tests/browser/` + `browser.md`.
- Branch-coverage audit (19) covers `src/{domain, application,
  adapters/driving/http, adapters/driven/storage, adapters/driven/llm,
  bootstrap, utils}/*_tests.rs`.

**No artifact of its own** — grilling ticket, resolution is the
graduates (Q3 → A).
