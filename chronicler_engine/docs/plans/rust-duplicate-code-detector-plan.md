# Plan: Rust Duplicate Code Detector (Skill-Based)

**Date:** 2026-06-25
**Status:** Planned
**Goal:** Surface duplicate/duplicate-ish Rust function bodies in `chronicler_engine/`
so reviewer agents can decide whether to extract them into `fixtures.rs` / test_support,
keep as legitimate domain overlap, or ignore as intentional.

---

## Overview

Duplicate function bodies creep in across handlers, tests, and adapters. Example from
this session: two `sqlite_storage()` setup functions with identical bodies, one real
overlapping context, one accidental duplicate. Detecting them manually is grep roulette.

This plan evaluates three implementation approaches and defers the pick to a baseline
run — which option actually works is empirical, not theoretical.

---

## Background

**Problem source:** Pain investigation re duplicate rust code. Exact-match duplicates
across files (esp. test setup, adapter boilerplate) recur and are easy to miss in review.

**Three candidate approaches:**

- Option A — Custom Python script (mirrors `comment_finder.py`).
- Option B — Wrap external duplicate-code detector (jscpd / duplo / PMD CPD).
- Option C — pi-lens ast-grep rules (`write-ast-grep-rule` skill).

Tradeoffs of each below. No selection until baseline run proved which is worth keeping.

---

## Option A — Custom Python Script

**Shape:** Same as `comment_finder.py`. Script extracts candidates, agent reviews.

**How:**

- Brace-match `fn name(...) {...}` blocks in `*.rs`.
- Normalize body: strip whitespace, comments, optionally normalize identifiers.
- Hash normalized body. Group by hash. Report groups with N≥2 members.
- Discovery modes: `--uncommitted`, `--branch`, `--all`, `--pattern` (reuse from
  `comment_finder`).

**Pros:**

- Mirrors `chronicler-comment-fixer` pattern — fits existing skill ecosystem.
- Zero new deps, pure Python.
- Agent filters FPs (same as comment-fixer does for "What" comments).
- Can run in `--uncommitted` / `--branch` / `--all` modes (reuse from `comment_finder`).

**Cons:**

- Regex/brace-matching for Rust is approximate. Edge cases: fn in macros/strings,
  nested braces, raw strings. Some noise — fine since agent reviews.
- Maintenance on us if Rust syntax evolves (rare).

**Effort:** ~100 LOC. Reuses `comment_finder.py` file-discovery scaffolding.

---

## Option B — Wrap External Duplicate-Code Detector

Existing tools that support Rust:

- **jscpd** (npm) — supports Rust via tree-sitter. Tokenized, robust. Adds Node dep.
- **duplo** — C++ tool, Rust-agnostic line-based, decent. Adds binary dep.
- **PMD CPD** — Java tool, supports Rust. Heavy.

**How:**

- Run external tool on `chronicler_engine/src/` + `tests/`.
- Parse its output (JSON / XML / text) into our skill's group format.
- Re-emit in the group-with-members format the reviewer skill expects.

**Pros:**

- Tokenizer-based, far fewer FPs than regex brace-matching.
- Handles macros / strings / nested braces natively (jscpd via tree-sitter).
- Mature tools, maintainers handle syntax evolution.

**Cons:**

- New dependency for `chronicler_engine` toolchain (Node, C++, or Java runtime).
- Output format doesn't match our skill pattern — needs adapter/wrapper layer.
- PMD/jscpd need runtime install → friction for fresh contributors.
- Different tool's heuristics not tunable for our review workflow.

**Effort:** Medium. Wrapper + output adapter, ~150 LOC, plus tool install docs.

---

## Option C — pi-lens ast-grep Rules

`write-ast-grep-rule` skill exists. ast-grep has proper Rust tree-sitter parser. In
principle could write a rule that finds duplicate fn bodies.

**How:**

- Write ast-grep rule(s) that match `fn $NAME(...) { $$$BODY }`.
- Output matches, post-process to group identical bodies across files.
- Or: explore whether pi-lens custom rule DSL supports similarity grouping.

**Pros:**

- Native tree-sitter parsing, no FPs from macros/strings.
- Fits pi-lens ecosystem already in project (`.pi-lens/` dir exists).
- Already integrated with editor diagnostics.

**Cons:**

- ast-grep matching is pattern-based, not similarity-based. Matching "any two functions
  with identical body" is awkward — ast-grep rules match one pattern, not "two things
  equal to each other". Would need a different approach (custom pi-lens rule?).
- Duplicate detection (across files) typically needs post-processing ast-grep output.
- **Verdict: ast-grep is the wrong tool** — it matches given patterns, doesn't compute
  similarity groups.

**Effort:** Unclear. Pattern-match path inadequate; custom pi-lens rule path unproven —
needs to check `write-ast-grep-rule` SKILL before feasibility confirmed.

---

## Comparison Matrix

| Criterion             | Option A (Python)      | Option B (External)        | Option C (ast-grep)      |
| --------------------- | ---------------------- | -------------------------- | ------------------------ |
| New deps              | None                   | Node/C++/Java runtime      | None (pi-lens already in)|
| FP tolerance          | High (agent reviews)   | Low (tokenizer)            | Low (tree-sitter)        |
| Fits skill ecosystem  | Yes                    | Partial (needs adapter)    | Yes if rule fits         |
| Cross-file grouping   | Native (we build it)   | Need output adapter        | Needs post-processing    |
| Maintenance burden   | Ours (Rust syntax)    | Upstream tool maintainers  | Ours (rule + post-proc)  |
| Effort                | ~100 LOC               | ~150 LOC + install docs    | Unproven feasibility     |
| Risk                  | Parser noise           | Toolchain friction         | Wrong tool for similarity|

---

## Scope (Applies to All Options)

### In scope (Phase 1)

- Extract `fn` blocks (top-level + `impl` methods).
- Skip fns inside strings/macros (best effort heuristic for Option A; native for B/C).
- Normalize body:
  - Collapse whitespace
  - Drop line + block comments
  - Optional identifier normalization for structural-dupes second pass
- Group by body hash, print groups with N≥2 members.
- Output format (canonical across options):

  ```
  GROUP (N=2, ~45 bytes each):
    chronicler_engine/src/storage/sqlite.rs:120  fn sqlite_storage() -> Storage
    chronicler_engine/src/storage/memory.rs:88  fn sqlite_storage() -> Storage
  ```

- Discovery modes: `--uncommitted`, `--branch`, `--all`, `--pattern`.

### Out of scope (Phase 1)

- Similarity scoring (near-dupes with small diffs). Exact body hash only first.
- Cross-crate dedup (only `chronicler_engine/`).
- Type-aware dedup (treating `let x: Foo` same as `let x: Bar` if structures match).
- Integration into `build.py` (non-blocking).

### Special handling

- `#[cfg(test)]` modules included — test duplication often points at `fixtures.rs`
  opportunities, value high here.
- `tests/` integration tests included.
- Macro-generated code: skip bodies inside `macro_rules!` invocation targets.

---

## Architecture Decisions (Option-Independent)

1. **Non-blocking.** NEVER in `build.py`. Optional pre-commit or agent-triggered via
   skill. Same blast radius as comment-fixer.
2. **Skill provides action templates, not auto-fix.** Reviewer decides: (a) promote to
   `fixtures.rs`/`test_support`, (b) keep (legitimate domain overlap), or (c) ignore
   (intentional).
3. **Baseline pass gates the whole effort.** If baseline (whichever option) shows 50
   legit 2-member groups → weak signal → kill. If shows few groups with 4+ members →
   worth keeping. Decision point after baseline run.
4. **Option selection DEFERRED to baseline.** No pick in this plan. Each option is
   prototyped or at minimum feasibility-checked, then the option that produces the best
   signal-to-noise on actual `chronicler_engine/` code wins.

---

## Phase 1: Prototype All Three → Baseline Gate → Pick Winner

### Task 1.1: Feasibility Check Option C (ast-grep)

- Read `.agents/skills/write-ast-grep-rule/SKILL.md` (or pi-lens equivalent).
- Determine if ast-grep / pi-lens rule DSL can compute cross-file similarity groups, or
   only single-pattern matches.
- **Decision gate:**
  - If similarity grouping feasible → prototype Option C too in Task 1.2.
  - If only pattern-match (expected) → record verdict "ast-grep wrong tool", drop Option
    C from baseline comparison.
- **Deliverable:** Feasibility note in this plan + updated task list.

### Task 1.2: Prototype Each Surviving Option

- Build minimal version of each surviving option (A always; B always; C if Task 1.1
   greenlights):
  - **Option A:** `duplicate_finder.py` (~100 LOC, reuses `comment_finder.py` scaffolding).
  - **Option B:** Pick one external tool (lean: jscpd for tree-sitter support), write
    output adapter (~100-150 LOC).
  - **Option C (if feasible):** ast-grep rule + post-processing script.
- Each must emit the canonical output format.
- **Deliverable:** Three (or two) runnable scripts.

### Task 1.3: Baseline Run + Comparison

- For each surviving option, run on full `chronicler_engine/` tree.
- Score each option on:
  - Number of real-dup groups (4+ members) found
  - FP rate (groups that are macro-noise / legitimate overlap / intentional)
  - Runtime
  - Setup friction
- **Decision gate:** Pick ONE option going forward, OR kill plan if none show useful
  signal (no 4+ member real-dup groups).
- **Deliverable:** Baseline comparison table + winner decision recorded in this plan.

### Task 1.4: Write SKILL.md for Winning Option

- File: `.agents/skills/chronicler-code-duplication/SKILL.md`
- Mirror `chronicler-comment-fixer/SKILL.md` structure:
  - Trigger keywords (e.g., "find duplicates", "duplicate code", "dedup rust")
  - Script invocation examples
  - Action templates per group type (promote / keep / ignore)
  - FP guidance (macro-noise, cfg(test) variants, adapter boilerplate)
- The winning script lives under
  `.agents/skills/chronicler-code-duplication/scripts/`.
- **Deliverable:** Skill file + script in its final location.

### Task 1.5: Iterate on Heuristic

- Run winning skill end-to-end on a real feature branch with duplicates.
- Tune `--min-bytes`, `--min-group-size`, macro-skip / tokenizer thresholds based on noise.
- Add identifier-normalization second pass if first pass misses structural dupes the
  reviewer expects to catch.
- **Deliverable:** Tuned defaults documented in SKILL.md.

---

## Phase 2 (Deferred — Wait for Phase 1.3 Baseline Gate)

- Similarity scoring (near-dupes with small diffs via tokenized diff).
- `--promote` action template that auto-generates `fixtures.rs` extraction patch.
- CI integration (optional pre-commit hook).
- Cross-crate dedup (if workspace grows).

---

## Success Criteria

1. Winning script runs on full `chronicler_engine/` in < 10 seconds.
2. Baseline (Task 1.3) decision made: winner has ≥3 real-dup groups of 4+ members, OR
   plan killed with documented baseline.
3. SKILL.md mirrors `chronicler-comment-fixer` structure and reviewer agent can follow
   it without external context.
4. Non-blocking: zero changes to `build.py`, zero UNEXPECTED runtime deps for
   contributors using the default toolchain (Option B must document install steps
   clearly if selected).
5. End-to-end skill run on a real feature branch produces actionable groups, not just
   noise (FP rate < 50% of groups marked real-dup).

---

## Risks

- **Option A macro/string false positives** → mitigate with macro-skip heuristic + agent
  review. Acceptable noise threshold; matches comment-fixer tolerance.
- **Option B toolchain friction** → contributors must install runtime (Node for jscpd,
  etc.). Mitigated if Option A wins baseline; if B wins, clear install docs required.
- **Option C infeasibility** → expected from prior analysis; Task 1.1 confirms. Risk
  absorbed: plan doesn't commit to Option C.
- **Baseline shows no real dupes** → kill plan at Task 1.3 gate. That itself is valuable
  signal (proves absence of duplicate-code problem currently).
- **Identifier normalization over-groups** → catch unrelated fns with similar structure.
  Mitigated: first pass keeps identifiers, normalization opt-in.
- **Skill maintenance burden** → mitigate by keeping winning script simple Phase 1
  (no similarity scoring), small LOC.

---

## Open Questions

1. Should `--min-group-size` default to 2 or 3? (Lean: 2 for visibility first, raise
   default if noise high after baseline.)
2. Include `impl` block method duplicates, or top-level fns only Phase 1? (Lean: include
   methods — adapter boilerplate often duplicates across `impl`. Confirm during Task 1.2.)
3. Run baseline against `chronicler_engine/` only, or `mrn-general/` full tree including
   `scripts/`? (Lean: engine only first, scope creep otherwise.)
4. Output format: plain text groups (lean) vs JSON for machine consumption? (Lean: plain
   text Phase 1, add `--json` flag only if reviewer skill needs structured input.)
5. Should `fixtures.rs` extraction action template commit a patch file or print a
   suggestion? (Lean: print suggestion only Phase 1 — agent owns the edit, script does
   not write code.)
6. If Option B wins baseline, which tool is default? (Lean: jscpd for tree-sitter Rust
   support + npm ubiquity. Defer to Task 1.3 result.)
7. If Option C feasible via custom pi-lens rule not ast-grep, does that change the
   comparison? (Lean: yes — pi-lens already integrated, lower friction than B. Confirm
   in Task 1.1 before ruling Option C out.)
