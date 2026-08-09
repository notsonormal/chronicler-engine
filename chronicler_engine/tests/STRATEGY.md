# Test Strategy

The normative tier-placement rules for `chronicler_engine`. Referenced from
`tests/AGENTS.md`. Decisions settled via the wayfinder map
(`.scratch/test-strategy/`); this doc is the codified output.

## The four tiers

The tier is defined by **what's faked**, not by sync vs async.

| Tier | Driven ports | Real | Location | Purpose |
|---|---|---|---|---|
| **Unit** | both (`MockBackend` + in-memory `Storage`) | nothing | `src/`, `*_tests.rs` | branch coverage — every branch in the code gets a test |
| **HTTP E2E** | LLM (`MockBackend` via pipeline override) | real axum router, real or in-memory storage | `tests/http/` | spec validation — all spec scenarios validated end-to-end through the real driving adapter |
| **Browser** | LLM | real browser, real server | `tests/browser/` | presentation only — DOM, CSS, JS interaction |
| **Driven-adapter** | nothing | real SQLite | `tests/storage/` | the storage seam — CRUD, error handling, referential integrity, query correctness |

`#[tokio::test]` with fakes is a unit test. The unit tier includes async
scenarios (cancellation, mid-flight timing) that need in-process seams — these
are unit tests doing their job, not exceptions.

## What was dissolved

The pipeline-level component tier (`tests/integration/application/` and
`tests/integration/flow/`) — tests that called pipeline methods directly on
`AppState` with real SQLite and asserted on `GameState` — is dissolved. Those
tests duplicated the unit tier's job (same methods, same `GameState`
assertions) at a heavier harness cost. They port down to unit (branch
coverage) or up to HTTP E2E (spec validation).

## Spec scenarios and HTTP E2E

Specs (`docs/specs/`) are the behavioural authority. Every spec scenario maps
to at least one HTTP E2E test that validates it end-to-end through the real
driving adapter. Single-call scenarios ("POST /action with empty input → one
continuation narration") and multi-call sequence scenarios ("POST /action →
POST /retry → POST /retry → assert swipe=2") both live at the HTTP tier. The
`flow/` tests are not a separate tier — they are multi-call spec scenarios at
HTTP E2E.

Scenarios that can't be expressed through HTTP surfaces — because their Givens
or Thens touch seams that only exist in-process — live at the unit or
driven-adapter tier instead:

- **Cancellation** — needs `CancellationToken` → unit (`#[tokio::test]` in `src/` with fakes)
- **Internal state** (e.g. `last_trigger`, phase transitions) — assert on `GameState` fields → unit
- **Mid-flight observation** — needs sync flags → unit
- **Call sequencing** — direct call-count assertion → unit

These aren't exceptions to the model — they're the unit and driven-adapter
tiers doing their job. A scenario that can't be expressed through HTTP simply
doesn't get an HTTP E2E test.

**Spec completeness is load-bearing.** The model only prevents drift if specs
are complete — every failure mode, every edge case. A half-written spec +
HTTP E2E + comprehensive unit tests is less safe than the old component tier,
because the component tier was catching unspecified behaviour the new model
doesn't cover end-to-end.

## Overlap rule

**Each tier asserts what it can see.** Cross-tier overlap is expected and
correct — unit tests the branch (internal state), HTTP E2E validates the spec
scenario (client-observable behaviour), driven-adapter tests the storage seam
(persistence integrity). They cover the same behaviour from different angles
for different reasons.

The violation is **same-tier duplication**: two tests at the same tier
asserting the same thing about the same behaviour. If a test's assertions are
fully covered by another test at the same tier, delete the weaker one.

## Browser placement test

Browser tests must assert something **only a browser can see** — DOM
structure, CSS layout, JS interaction, visual rendering. If the assertion can
be expressed through HTTP, the test is at the wrong tier. No fixed numeric cap;
the placement rule is the guardrail.

## SCENARIO tags

`SCENARIO:` tags (format: `// [spec-path] SCENARIO: N.N`) go on HTTP E2E tests
in `tests/http/` and browser behaviour tests in
`tests/browser/behaviour.rs`. Scenarios that live at the unit or
driven-adapter tier don't carry SCENARIO tags — they're covered by tests
whose names describe the behaviour. Tags must not appear in
`tests/storage/` or `tests/browser/invariants.rs`.

`tests/browser/invariants.rs` carries a named exemption: no tags, no
spec link, test code is the definition (same shape as unit branch tests
— STRATEGY.md's "every branch needs a unit test" rule doesn't produce a
per-branch doc; the test is the definition).

Mechanical enforcement lives in `scripts/validate_feature_spec.py` via
the `TEST_DIRS` list (scans `tests/http/` + `tests/browser/` for
`// SCENARIO:` comments). The "tags only in `tests/http/` +
`tests/browser/behaviour.rs`" rule is a social convention layered on top;
the validator scans the whole browser dir but `invariants.rs` has no
tags to contribute.

## Placement test

The question is **which tier's purpose does this test serve?**

- Branch coverage → **unit** (`src/`)
- Spec validation through the driving adapter → **HTTP E2E** (`tests/http/`)
- Persistence integrity → **driven-adapter** (`tests/integration/storage/`)
- DOM/CSS/JS → **browser** (`tests/browser/`)
