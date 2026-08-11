# Plan: Abstraction Anti-Pattern Prevention via Advisory Healthcheck

**Date:** 2026-06-26
**Status:** Planned
**Goal:** Surface abstraction anti-pattern smells (`too_many_arguments`, `dead_code`) as advisory warnings in `scripts/healthcheck.py` to catch them before they ship, without blocking CI.

**Investigation source:** `reports/abstraction-antipatterns-summary.md` + per-zone reports (`reports/zone-{a,b,c,d}-*.md`).

---

## Overview

Investigation of `chronicler_engine` for abstraction anti-patterns surfaced 47 findings across 5 categories (premature generalization, coincidental cohesion, false deduplication, helper smell, refactor-be-damned extraction). Of these, 2 categories — already acknowledged by clippy — can be surfaced cheaply through advisory tooling rather than blocking lints:

- **`clippy::too_many_arguments`** — found at `application/action_pipeline/phases.rs:44, 199` (B5, B6), both suppressed via `#[allow]`.
- **`clippy::dead_code`** on enum variants — found at `application/action_pipeline/pipeline.rs:28` (B2 `ActionOutcome::Error`), suppressed via `#[allow(dead_code)]`.

Currently `clippy.toml` sets `too-many-arguments-threshold = 7`, but neither lint is in `lib.rs` `#![deny(...)]`. Both already emit warnings at clippy level but are hidden in build noise. Surfacing them in the existing `healthcheck.py` dispatcher makes them LLM-consumable summarizable for code review.

**Scope of this plan:** Advisory detection only. Does NOT touch `arch-lint.toml`, `tests/guardrails.rs`, or `lib.rs`. Future phases may promote to blocking rules.

---

## Background

**Existing tooling:**

- `scripts/healthcheck.py` — dispatcher with `@register` decorator + `CHECKS` dict. Currently has `duplicates` check (jscpd-based). Adding a check is ~60 LOC.
- `clippy.toml` — `too-many-arguments-threshold = 7`.
- `src/lib.rs` — `#![deny(clippy::unwrap_used, expect_used, dbg_macro, todo, unimplemented, print_stdout, print_stderr, panic)]`. Does NOT deny `too_many_arguments` or `dead_code`.
- `docs/architecture/guardrails.md` — documents three layers (compile-time clippy, test-time arch-lint, test-time syn-based guardrails.rs).

**Why advisory, not blocking:**

- `too_many_arguments` has legitimate cases (axum handler extractors in `src/server/`, `extern "C"` signatures if ever added).
- `dead_code` items often mid-refactor — blocking would prevent incremental work.
- Goal is awareness for code review, not enforcement.

---

## Architecture Decisions

1. **Advisory only, exit 0 always.** The check returns `ok=True` regardless of findings. Matches user intent — warnings, not build failures. Safe to add to `build.py` pre-commit without breaking CI.

2. **Reuse existing clippy output.** No new analysis tooling. `cargo clippy --all-targets --message-format=short` already emits both lints. Parse, filter, summarize.

3. **Two summary sections, grouped by lint.** Output mimics the `duplicates` report format — markdown, LLM-consumable, with file:line refs.

4. **Filter already-`#[allow]`-ed dead_code findings.** A `dead_code` finding with an adjacent `#[allow(dead_code)]` is acknowledged — exclude. Only un-allowed dead code surfaces. (Note: `too_many_arguments` surfaces all hits regardless of `#[allow]`.)

5. **Optional `--out` flag** for file output, mirroring `duplicates` check.

6. **Automatic inclusion in `all` subcommand** via `CHECKS` dict — no extra wiring.

---

## Phase 1: Implementation

### Task 1.1: Add `clippy_smells` check to `healthcheck.py`

Add new `@register("clippy_smells")` function that:

1. Runs `cargo clippy --all-targets --message-format=short` with ~300s timeout.
2. Parses stdout lines, filtering for:
   - `too_many_arguments` — keep all hits.
   - `dead_code` — exclude hits where the line above contains `#[allow(dead_code)]`.
3. Returns `CheckResult("clippy_smells", True, "<N> findings", output_path=out)`.

**Files:**

- `scripts/healthcheck.py` (extend)

**Implementation notes:**

- Use `subprocess.run` with `capture_output=True, text=True`.
- `--message-format=short` produces `<file>:<line>:<col>: <level>: <msg> [-<lint>]`.
- For `dead_code` filter: parse the lint name from `[-clippy::dead_code]` suffix, then for each finding, read the source file line above the finding's line and check for `#[allow(dead_code)]`. Skip if present.
- Build error / clippy failure: return `CheckResult("clippy_smells", False, "<error>")` — clippy itself failed, not advisory signal.

**Acceptance criteria:**

- [ ] `python scripts/healthcheck.py clippy_smells` runs without error.
- [ ] Output is markdown with sections for `too_many_arguments` and `dead_code`.
- [ ] Each finding has file:line reference + lint name.
- [ ] Exit code is 0 even when findings exist.
- [ ] Exit code is 1 only when clippy itself fails to run.
- [ ] `python scripts/healthcheck.py all` includes `clippy_smells` automatically.

### Task 1.2: Add CLI subparser

Add `clippy_smells` subparser with flags:

- `--out` — write summary to file
- `--verbose` — emit stderr progress
- `--no-dead-code` — skip dead_code findings (optional escape hatch)
- `--no-too-many-arguments` — skip too_many_arguments findings (optional escape hatch)

**Files:**

- `scripts/healthcheck.py` (extend `build_parser()`)

**Acceptance criteria:**

- [ ] `python scripts/healthcheck.py clippy_smells --out report/smells.md` writes file.
- [ ] `python scripts/healthcheck.py clippy_smells --no-dead-code` filters dead_code out.
- [ ] Help text mimics `duplicates` subparser style.

### Task 1.3: Verify against current findings

Run healthcheck and confirm output matches known smells:

- `src/application/action_pipeline/phases.rs:44` — `too_many_arguments` (B5)
- `src/application/action_pipeline/phases.rs:199` — `too_many_arguments` (B6)
- `src/application/action_pipeline/pipeline.rs:28` — `dead_code` on `ActionOutcome::Error` (B2) — should be filtered out due to `#[allow(dead_code)]`

**Acceptance criteria:**

- [ ] `clippy_smells` report lists exactly the 2 `too_many_arguments` findings.
- [ ] `dead_code` section is empty (the existing finding is `#[allow]`-ed).
- [ ] Manual verification of file:line refs against source.

---

## Phase 2: Documentation Update

### Task 2.1: Update `docs/architecture/guardrails.md`

Add new section "4. Advisory Healthcheck Checks" describing:

- `duplicates` (existing)
- `clippy_smells` (new) — what it surfaces, why advisory, how to use
- How to invoke: `python scripts/healthcheck.py clippy_smells` or `all`
- When to review: during code review of any pipeline / storage / service-layer code

**Files:**

- `docs/architecture/guardrails.md` (extend)

**Acceptance criteria:**

- [ ] Section added under existing "## 4. Coverage Exclusion Policy" or new "## 5." section.
- [ ] Cross-link from `AGENTS.md` "ESSENTIAL COMMANDS" section if applicable.

### Task 2.2: Add sample output to `scripts/healthcheck.py` docstring

Update module docstring `Usage:` block to include `clippy_smells` example.

**Files:**

- `scripts/healthcheck.py` (extend top-of-file docstring)

**Acceptance criteria:**

- [ ] Docstring lists `clippy_smells` alongside `duplicates` and `all`.

---

## Phase 3: Wire Into Developer Workflow (Optional, Defer)

### Task 3.1: Add to `build.py` as advisory step

If desired, `build.py` could invoke `python scripts/healthcheck.py clippy_smells --out report/smells-report.md` post-build for visibility. Mark as DEFERRED — only do this if developers request integration.

**Files:**

- `build.py` (extend)

**Acceptance criteria:**

- [ ] `python build.py` emits smells report to `report/`.
- [ ] Build does NOT fail on findings.

---

## Implementation Order

1. **Task 1.1** — core check function
2. **Task 1.2** — CLI subparser
3. **Task 1.3** — verify against current findings (gate — must match expected smells)
4. **Task 2.1** — docs update
5. **Task 2.2** — docstring update
6. *(Phase 3 deferred)*

---

## Dependencies

| Task | Depends on | Blocks |
|------|-----------|--------|
| 1.1 Core check | None | 1.3, 2.x |
| 1.2 CLI subparser | 1.1 | 1.3 |
| 1.3 Verify findings | 1.1, 1.2 | 2.x |
| 2.1 guardrails.md | 1.3 | — |
| 2.2 docstring | 1.1 | — |

---

## Risks

| Risk | Mitigation |
|------|-----------|
| `cargo clippy` slow (~10-15s for `--all-targets`) | Acceptable — advisory check run by developer, not every commit. Use 300s timeout. |
| `too_many_arguments` false positives for axum handlers in `src/server/` | Acknowledge in docs: axum handlers are known exception. Future work: --exclude-path flag. |
| `dead_code` filter misses multi-line `#[allow(...)]` attribute spanning multiple lines | Conservative — scan previous 5 lines for `#[allow(dead_code)]`. If attribute on separate line above variant, catches it. |
| Clippy lint naming changes across Rust versions | Parse lint name from `[-clippy::<name>]` suffix, not from message body. Stable. |
| Output format unstable across clippy versions | Pin `--message-format=short` (stable). If format breaks, check degrades to "clippy failed to parse" — visible, not silent. |
| Healthcheck dispatcher pattern changes | Follow existing `duplicates` check structure exactly. No new patterns introduced. |

---

## Success Criteria

1. `python scripts/healthcheck.py clippy_smells` produces a markdown report listing both `too_many_arguments` smells currently in the repo (B5, B6).
2. The `dead_code` on `ActionOutcome::Error` (B2) does not appear in the report (filtered — already `#[allow]`-ed).
3. Exit code is 0 even with findings present.
4. `python scripts/healthcheck.py all` includes `clippy_smells` automatically.
5. No regression to existing `duplicates` check.

---

## Out of Scope

- **Blocking rules** — `arch-lint.toml` and `tests/guardrails.rs` changes are explicitly deferred. See `reports/abstraction-antipatterns-summary.md` "Prevention" section for future tier-2 work.
- **Single-variant enum / single-impl trait / min-caller detection** — these require tree-sitter or LSP analysis not available via clippy. Needs separate toolA new script/`guardrails.rs` function. Defer to future plan.
- **Tier 1 surgical deletes** (47 findings) — separate cleanup task, not part of this plan.
- **Architecture doc updates** per `chronicler_engine/AGENTS.md` — no spec change required because this plan does not change runtime behavior. Only adds advisory tooling.
