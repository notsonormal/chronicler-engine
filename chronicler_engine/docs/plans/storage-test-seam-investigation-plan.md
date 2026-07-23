# Storage Test-Seam Investigation Plan

**Status:** Draft (investigation only — no implementation)
**Date:** 2026-07-15
**Scope:** Investigation only. Produces findings + ADR amendment proposal. **No code changes.**
**Depends on:** none
**Blocks:** potentially a follow-up implementation plan, if investigation recommends refactor
**Story points:** 3 SP (pure investigation, no production diff)

---

## Why investigate

User flagged: `#[cfg(feature = "testing")]` shows up 17× across 5 files. Most of it (9 of 17) is fault-injection plumbing on the production `Storage` type — `LayeredBackend::Test`, `with_test_failures`, `with_failure`, `add_failure`, `set_override`, plus types in `test_support.rs`.

ADR-020 explicitly chose this pattern as a deliberate trade-off (no custom mock structs; dynamic toggling mid-test; exhaustive matching). So the question is **not** "is this wrong" but:

1. Is the trade-off in ADR-020 still right given today's code volume?
2. Are the `cfg(feature = "testing")` gates doing what they claim to do?
3. Is there a cheaper pattern we should switch to?

This plan produces findings. Implementation (if justified) goes in a follow-up plan after sign-off.

---

## Investigation Questions

### Q1. Maintenance tax of `LayeredBackend::Test` in the production enum

Every backend op pays for the `Test` arm in `with_backend` and the `is_*` helpers. Quantify:

- Count of `BackendKind::Test {` / `LayeredBackend::Test {` match arms across the codebase (outside `core.rs` itself, where it's the "owner" file).
- Count of distinct backend methods that route through `with_backend` (the chokepoint).
- Trend: count of backend methods added in the last year via `git log --since=... -- src/adapters/driven/storage/backend/`. If growing rapidly, the tax compounds.
- Miss-risk surface: how many backend methods exist where forgetting the `Test` arm would compile silently? (i.e., methods that go through `with_backend` vs methods that don't, and whether the compiler enforces anything.)

### Q2. Is `feature = "testing"` doing what it claims?

- All known callers of `TestOverride` / `TestFailureHandle` / `with_test_failures` are inside this crate. Confirm: no dev-tool, no test-helper crate, no benchmark binary in this workspace depends on `--features testing`.
- The `default = ["testing"]` setting means the gates never restrict anything in any real build. Confirm whether removing from default would break any documented build path (`build.py`, CI, README quickstart).
- Audit the 8 non-storage `cfg(feature = "testing")` sites — are they all eligible for `cfg(test)` simplification, or do some genuinely need the feature? Specifically: `bootstrap/wiring.rs`, `agents/quantifier/agent.rs`, `lib.rs`. Read each to confirm.

### Q3. Are there cheaper alternatives?

Sketch three alternatives. **No code changes** — produce 1-page writeups each, with rough diff size and risk per option:

- **Option A: `TestStorage` wrapper struct** (my earlier suggestion). Keep `LayeredBackend` clean; test-only wrapper holds `Storage` + overrides. Update all call sites.
- **Option B: Trait-object `DynStorage`** with test impl. More indirection but no in-place variant. ADR-020 already rejected this; restate why with today's numbers.
- **Option C: Keep current, but move `Test` variant behind `cfg(test)`** — minimum diff. Tests would need `cfg(test)` access to the storage private API. Check whether the existing `pub(crate)` boundaries support this.

Each option writeup includes: file count touched, test-call-site count changed, binary-size win (if any), risk to behavior, lines added vs removed.

### Q4. ADR-020 re-engagement

ADR-020 was written when the codebase had 6 storage traits and 12 mock structs. Since then the calculus may have shifted — read ADR-020 plus CHANGELOG entries that mention `TestOverride`, `TestFailureHandle`, or `test_support` to reconstruct the recent evolution in this area. Produce:

- A 1-paragraph summary of what ADR-020's `LayeredBackend::Test` decision replaced (the "before" state — mock structs).
- An updated consequences table reflecting 2026-07 conditions: file sizes, method counts, call-site counts.
- An explicit list of conditions under which the ADR should be **superseded** (not just amended). E.g., if maintaining the Test arm is ≥ N minutes per new backend method, recommend supersede.

---

## Key Outputs

1. **`docs/plans/storage-test-seam-findings.md`** — investigation report with:
   - Q1 numbers (match-arm tax, method count, trend, miss-risk surface)
   - Q2 answer (cfg simplification scope confirmed or refuted)
   - Q3 writeups (1 page each, 3 options)
   - Q4 ADR re-engagement + draft amendment

2. **Draft ADR-020 amendment** appended to `docs/plans/storage-test-seam-findings.md` (NOT yet merged into `adr-020-storage-consolidation.md`). Amendment format follows the existing History-section pattern in the ADR.

3. **Go/No-Go recommendation** at the top of `storage-test-seam-findings.md`:
   - **GO** (recommend refactor) → write a follow-up implementation plan with chosen option
   - **NO-GO** (keep as-is) → file a one-line ADR-020 amendment noting the review and decision; close This plan
   - **DEFER** → file the amendment with a re-evaluate-when trigger (e.g., "re-evaluate when backend method count crosses N")

---

## Phases

### Phase 1: Data collection (Q1 + Q2)

- [ ] #### Task 1.1: Quantify `LayeredBackend::Test` tax
  - [ ] ##### SubTask 1.1.1: Count match arms on `BackendKind::Test` / `LayeredBackend::Test` across `src/`, excluding `core.rs`. Record file:line for each.
  - [ ] ##### SubTask 1.1.2: Count the `with_backend` chokepoint methods (i.e., backend operations funneled through that single `match`). Get the count + a representative sample (5 file:line refs).
  - [ ] ##### SubTask 1.1.3: Count backend methods added in the last 12 months (`git log --since=2025-07-15 --oneline -- src/adapters/driven/storage/backend/` → enumerate method additions). Record as "X new methods since ADR-020's `Test` decision."
  - [ ] ##### SubTask 1.1.4: Identify backend methods that DO NOT route through `with_backend` (potential miss-risk surface). List file:line for each. If zero, the compiler enforces Test arm coverage everywhere.
- [ ] #### Task 1.2: Verify `feature = "testing"` consumption scope
  - [ ] ##### SubTask 1.2.1: `rg -l 'TestOverride|TestFailureHandle|with_test_failures|with_failure\(|add_failure' chronicler_engine/` — confirm all consumers are inside this crate's `*_tests.rs` / `#[cfg(test)]` mods.
  - [ ] ##### SubTask 1.2.2: Check for any external consumer via `Cargo.toml` `[workspace]` members + `[dev-dependencies]` that pull the feature. If none, "no external consumer" confirmed.
  - [ ] ##### SubTask 1.2.3: Read each of the 8 non-storage `#[cfg(feature = "testing")]` sites (`lib.rs:38,40,43`, `bootstrap/wiring.rs:103`, `agents/quantifier/agent.rs:16,52`). For each, decide: `cfg(test)` eligible, or genuinely feature-flag-required? Record per-site verdict.
- [ ] #### Task 1.3: Build & baseline
  - [ ] ##### SubTask 1.3.1: `python build.py` — confirm green baseline before any investigation work (investigation is no-op, but record the green state).

### Phase 2: Alternative writeups (Q3)

- [ ] #### Task 2.1: Sketch Option A (`TestStorage` wrapper)
  - [ ] ##### SubTask 2.1.1: Write 1-page sketch: signature, 1 example test rewrite, file diff count, call-site rewrite count (~30?).
  - [ ] ##### SubTask 2.1.2: Estimate lines added vs removed. Compare against the maintain-tax from Q1.
- [ ] #### Task 2.2: Sketch Option B (`DynStorage` trait)
  - [ ] ##### SubTask 2.2.1: Restate ADR-020's rejection in 2026 terms. 1 paragraph.
- [ ] #### Task 2.3: Sketch Option C (keep, but `cfg(test)`-gate the Test variant)
  - [ ] ##### SubTask 2.3.1: Verify the existing `pub(crate)` boundaries in `core.rs` allow tests (in the same crate) to access a `cfg(test)`-only Test variant. Read `mod.rs` `#[cfg(test)] mod core_tests;` — same crate, so `cfg(test)` works without `pub`.
  - [ ] ##### SubTask 2.3.2: Sketch the diff (1-page) and call-site impact (zero call-site changes if `with_test_failures` keeps its name on `cfg(test)`-only method).

### Phase 3: ADR re-engagement + report (Q4)

- [ ] #### Task 3.1: Read ADR-020 + recent history
  - [ ] ##### SubTask 3.1.1: Read `docs/adr/adr-020-storage-consolidation.md` (full).
  - [ ] ##### SubTask 3.1.2: Read CHANGELOG entries that mention `TestOverride` / `TestFailureHandle` / `test_support` to reconstruct the recent evolution.
- [ ] #### Task 3.2: Write `storage-test-seam-findings.md`
  - [ ] ##### SubTask 3.2.1: Top section: Go/No-Go recommendation (3 bullets max).
  - [ ] ##### SubTask 3.2.2: Q1 numbers table.
  - [ ] ##### SubTask 3.2.3: Q2 verdict (cfg simplification scope) with per-site table.
  - [ ] ##### SubTask 3.2.4: Q3 three option writeups (1 page each).
  - [ ] ##### SubTask 3.2.5: Q4 ADR amendment draft (append-only — explicitly NOT merged yet).

### Phase 4: Validation

- [ ] #### Task 4.1: Validate investigation artifacts
  - [ ] ##### SubTask 4.1.1: Confirm `python build.py` still green (investigation produces no code change; build must be identical).
  - [ ] ##### SubTask 4.1.2: Diff `git status` — must show only the 2 new docs files (`this plan-investigate-storage-test-seam.md`, `storage-test-seam-findings.md`).
  - [ ] ##### SubTask 4.1.3: If Go verdict: hand off to a follow-up implementation plan (separate file, separate scope).
  - [ ] ##### SubTask 4.1.4: If No-Go verdict: file 1-line ADR-020 amendment noting review + decision; close This plan.

---

## Test Plan

- `python build.py` — green before, green after (investigation produces no diff).
- `git status` — only `docs/plans/this plan-*.md` files added.
- `cargo doc --no-deps` — docs build clean (no broken intra-doc links from new files).

## Out of Scope (explicit)

- **Refactoring code in this investigation.** This plan reads; it does not write `src/`. If investigation recommends a fix, it goes in the follow-up plan.
- **Modifying ADR-020.** The amendment lives in `storage-test-seam-findings.md` until the user signs off on the Go verdict. Then it gets ported into ADR-020 History section.
- **The cfg simplification.** Q2 produces a per-site verdict; the actual simplification is a separate task (its own follow-up plan, or a sub-track of the refactor implementation plan).
- **Cross-crate consumers.** Already verified absent in earlier work; Q2.1.2 is a re-confirmation, not a new survey.

## Assumptions

- ADR-020's design rationale still holds *unless* Q1 numbers refute it. The plan's job is to surface refuting evidence, not to assume the design is broken.
- User's concern was "too many test-only functions in production code." This is the question This plan answers. If the answer is "yes, here's the tax and 3 alternatives," the user can then decide.
- Investigation is non-disruptive: zero `src/` changes, only docs added.

## Risks

- **Scope creep**: This plan reads a lot of code (entire `storage/backend/`, `application/action_pipeline/pipeline_tests.rs`, etc.). Stay disciplined: produce numbers + writeups, do not fix.
- **False confidence**: "the code works today" can hide a slow-burn maintenance cost. Q1.1.3 trend number is the key counter to that.
- **Premature refactor**: GO verdict + rushing into implementation within This plan would defeat the plan's "investigate first" structure.

## Hand-off

- **If GO**: produce follow-up implementation plan following the same pattern (Status / Phases / Test Plan / Story Points). Copy the chosen option's sketch as the starting point for the follow-up plan's starting point.
- **If NO-GO**: ADR-020 amendment moves from `storage-test-seam-findings.md` into `docs/adr/adr-020-storage-consolidation.md` History section. This plan closes.
- **If DEFER**: ADR-020 amendment with the re-evaluate-when trigger. This plan closes; reopen if/when trigger fires.
