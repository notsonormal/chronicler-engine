# Plan: Antipattern-Checker Agent Skill

**Date:** 2026-06-26
**Status:** Planned
**Goal:** Create a project-scoped, standalone, explicitly-invoked agent skill that detects abstraction anti-patterns in Chronicler Engine Rust code via LLM semantic review.

**Prior work:** `reports/abstraction-antipatterns-summary.md` (47 findings across 5 categories) + per-zone reports (`reports/zone-{a,b,c,d}-*.md`) serve as validation corpus.

---

## Overview

Static rules (arch-lint, clippy, guardrails.rs) catch syntactic anti-patterns: single-variant enums, `too_many_arguments`, banned filenames. They cannot catch **semantic** anti-patterns that require reasoning about intent, cohesion, or root cause.

Of 47 findings in the validation corpus:

- ~10 are syntactic — coverable by rules (Rules 1, 3, 4, 5 from prevention strategy).
- ~37 are semantic — require an LLM reading code in context.

This skill provides the semantic layer. It is **not** a replacement for static rules — it is complementary. Rules prevent regression; skill catches new and semantic smells.

**Key design choices (settled with user):**

- **Project-scoped** — lives in `.agents/skills/antipattern-checker/`, uses Chronicler Engine findings as in-context examples.
- **Standalone** — main agent loads skill and executes review itself using its own tools (`read`, `bash`, `grep`). No subagent integration.
- **Explicitly invoked** — does not auto-trigger on every code review. User invokes via `/antipattern-check` or natural language.
- **Categorized framework** — 5 categories × sub-types with crisp definitions + signals.
- **Trimmed examples** — 1-2 examples per sub-type, not the full 47.

---

## Background

**Validation corpus:** `reports/abstraction-antipatterns-summary.md` + 4 zone reports produced by a parallel subagent investigation of `chronicler_engine/src/` (232 files, ~32k LOC). The methodology, output format, and finding structure in those reports are the model for the skill.

**Why LLM, not rules:**

- **Coincidental cohesion** — module groups items by "used together" not shared concept. Requires reading module + reasoning about whether items share a concept.
- **False deduplication** — same shape, different intent. Requires comparing 2+ callers' intent.
- **Refactor-be-damned extraction** — extract-and-relieve-symptom. Requires understanding root cause to see that the extraction avoided the real fix.
- **Premature generalization (semantic cases)** — `PromptAssembler` trait with 1 impl, `narrate_continuation` method with 0 prod callers. Requires cross-file analysis.
- **Helper smell** — god-functions, identity wrappers. Requires reading function body + callers.

These cannot be done by grep, tree-sitter, or static analysis alone.

**Existing skills overlap check:**

- `code-review-and-quality` — broader code review, overlaps but less specialized.
- `m15-anti-pattern` — general Rust anti-patterns (beginner mistakes like `clone` everywhere).
- Neither covers abstraction-specific semantic patterns at the depth validated in the corpus.

The new skill is narrower (abstraction anti-patterns only) and deeper (categorized framework + validated examples from this codebase).

---

## Architecture Decisions

1. **Project-scoped placement.** Skill lives in `.agents/skills/antipattern-checker/`. Chronicler Engine findings are embedded as in-context examples. Tighter fit than user-scope; avoids premature generalization to other projects.

2. **Standalone execution model.** Main agent loads SKILL.md, reads files with its own `read` / `bash` tools, produces report inline. No `subagent(...)` calls, no pi-subagents integration. Simpler, no async coordination needed.

3. **Explicit invocation only.** No auto-trigger keywords beyond direct request. Prevents noise on every code review. Triggers:
   - `/antipattern-check`
   - `/anti-pattern-check`
   - "check anti-patterns", "audit abstractions", "review for abstraction smells"

4. **Categorized framework, not flat list.** 5 categories × ~22 sub-types. Each sub-type has definition + detection signal. Prevents LLM from generating generic "this could be cleaner" findings. Forces every finding to map to a specific sub-type.

5. **Anti-slop methodology enforced.** Every finding must have: file:line, verbatim quote, why-smell, severity, proposed fix. No quote → no finding. No fix → no finding. Cap ~12 per zone.

6. **Trimmed examples.** 1-2 examples per sub-type, drawn from the 47-finding corpus. Total ~30 lines per examples file. Purpose: pattern recognition, not exhaustive inventory.

7. **Output format matches validation corpus.** Markdown report with summary, findings (categorized), cross-cutting notes, positive patterns. Same structure as `reports/zone-{a,b,c,d}-*.md` — already proven LLM-consumable.

---

## Skill Structure

```
.agents/skills/antipattern-checker/
├── SKILL.md              # trigger, scope, invocation, output contract
├── framework.md          # 5 categories × sub-types (definitions + signals)
├── methodology.md        # anti-slop rules + severity rubric
├── report-template.md    # markdown output format
└── examples/
    ├── premature-generalization.md   # 1-2 trimmed examples per sub-type
    ├── coincidental-cohesion.md
    ├── false-deduplication.md
    ├── helper-smell.md
    └── refactor-be-damned.md
```

---

## Framework (content of `framework.md`)

5 categories, ~22 sub-types:

### 1. Premature Generalization (over-abstraction)

Abstraction introduced for a second consumer that never appears.

| Sub-type | Signal |
|----------|--------|
| Single-variant enum | `enum Foo` with 1 variant; match sites have 1 arm |
| Single-impl trait | `trait Foo` with 1 `impl` block (excluding tests) |
| Dead trait method | Trait method with zero production callers; one impl returns `not_implemented()` |
| Dead enum variant | Variant `#[allow(dead_code)]` or never constructed |
| Single-consumer hook | Trait method overridden by only 1 of N impls |

### 2. Coincidental Cohesion (grab-bag modules)

Module groups items by "used together" not by shared concept.

| Sub-type | Signal |
|----------|--------|
| Generic filename | `misc.rs`, `util.rs`, `helpers.rs`, `common.rs` |
| Unrelated types in one module | Module has N types spanning M subsystems |
| Module named for use, not concept | "Fragment utilities" instead of `text_check.rs` |

### 3. False Deduplication (same shape, different intent)

Code looks similar; semantics differ. Merging couples unrelated callers.

| Sub-type | Signal |
|----------|--------|
| Mirror types across modules | Two enums/structs with same shape + bidirectional `From` impls |
| Global behavior forced on all impls | Trait default applies model-specific formatting to all backends |
| Generic function carrying provider-specific params | `Option<&str>` flags act as hidden mode switches |
| Copy-paste impl forced by empty trait default | Identical impls across N backends because trait default is empty |

### 4. Helper Smell / Utility Abuse

Helpers that grow flags, params, or become god-functions.

| Sub-type | Signal |
|----------|--------|
| God-function | 60+ line method threading mutable state through many calls |
| Identity-wrapper service | Service method forwards to inner service, no added logic |
| `too_many_arguments` suppression | `#[allow(clippy::too_many_arguments)]` on fn with 6+ params |
| Parameter accumulation | Same trio of args threaded through every phase method |
| Error burying | Function named `error_*` returns `Ok` while stuffing failure in state |
| Test double flag-bag | Mock struct with N `AtomicBool` fields + per-call Vecs |

### 5. Refactor-be-damned Extraction

Extract-and-relieve-symptom instead of fix root cause.

| Sub-type | Signal |
|----------|--------|
| Invalid domain object from constructor | `from_db` / `new` returns object violating documented invariant |
| Orphaned param / dummy arg | Function param prefixed `_`, never read |
| Pipeline re-implementation | New module re-runs pipeline steps by hand instead of parameterizing |
| Single-caller helper extracted for "clarity" | Private fn called from 1 site; inlining loses nothing |
| Mirrored fields needing manual sync | Type A duplicates Type B's fields; sync methods maintain them |
| DTO mirroring source with no behavior | Struct exists solely to flatten another type for serialization |
| Test-only enum leaking into prod | `Operation` enum threaded through every prod method, used only by `Test` branch |

---

## Methodology (content of `methodology.md`)

Anti-slop rules — prevent generic, low-value, or hallucinated findings:

1. **Read files fully.** No grep-only findings. Findings based on snippets are rejected. If a file is in scope, read it in full before reporting on it.

2. **Evidence required.** Every finding must include:
   - `file:line` reference
   - Verbatim quote of the offending code
   - Why it's a smell (semantic reasoning, not restating the pattern name)
   - Severity (high / med / low)
   - Proposed fix direction (concrete, not "improve this")
   - No quote, no finding. No fix direction, no finding.

3. **Severity rubric.**
   - **high** — produces bugs, maintenance burden, or architectural drag
   - **med** — increases cost of change, couples unrelated concerns
   - **low** — cosmetic, mechanical cleanup

4. **Cap findings.** ~12 per zone / file group. Forces prioritization. Drop low-value findings rather than padding the count.

5. **Map to sub-type.** Every finding must map to a specific sub-type from `framework.md`. If it doesn't fit, either the framework has a gap (note it) or the finding is too vague (drop it).

6. **Note positive patterns.** Include "X is OK because..." observations. Prevents doomsday tone and trains on acceptable tradeoffs. Cap 3-5 positive notes per report.

7. **Distinguish fundamental vs mechanical.**
   - Fundamental — architectural issue, requires plan + multi-file change
   - Mechanical — trivial rename, inline, delete
   Tag each finding accordingly.

8. **Stop when scope exhausted.** Do not extend review beyond requested scope. Do not pre-emptively audit adjacent files.

---

## SKILL.md (content)

- **Trigger:** `/antipattern-check`, `/anti-pattern-check`, "check anti-patterns", "audit abstractions", "review for abstraction smells"
- **Scope options (user picks at invocation):**
  - Single file
  - Module / directory
  - Diff (changed files only)
  - Full codebase (warn: slow, expensive — may take 10-30 minutes)
- **Output:** Markdown report to stdout. Format per `report-template.md`.
- **Resources loaded:**
  - `framework.md` (always)
  - `methodology.md` (always)
  - `report-template.md` (always)
  - `examples/*.md` (always — for pattern recognition)
- **Execution:** Main agent reads files in scope, applies framework, produces report. No subagent delegation.

---

## Scope Options

| Scope | When to use | Cost |
|-------|-------------|------|
| Single file | Pre-commit check | Low (~1 min) |
| Module/diff | Code review | Medium (~5 min) |
| Zone (architectural layer) | Periodic audit | High (~15 min) |
| Full codebase | Quarterly audit | Very high (~30 min) |

Skill describes each; user picks by stating scope. Default to diff if in a feature branch, else ask.

---

## Output Format (`report-template.md`)

```markdown
# Anti-Pattern Report: [Scope]
## Summary
- N findings, severity distribution
- Files reviewed, LOC

## Findings
### 1. [Category / Sub-type] Short title
- **File:** path:line
- **Evidence:** `verbatim quote`
- **Why smell:** ... (semantic reasoning, not restating pattern name)
- **Severity:** high/med/low
- **Type:** fundamental / mechanical
- **Proposed fix:** ... (concrete direction)

## Cross-cutting notes
- Patterns recurring across multiple files
- Architecture-level observations

## Positive patterns (what's working)
- Prevents doomsday tone
- Trains on acceptable tradeoffs
```

Matches the format used in `reports/zone-{a,b,c,d}-*.md` — already validated.

---

## Implementation Phases

### Phase 1: Skill skeleton

**Task 1.1: Create directory structure**
Create `.agents/skills/antipattern-checker/` with empty files:

- `SKILL.md`
- `framework.md`
- `methodology.md`
- `report-template.md`
- `examples/premature-generalization.md`
- `examples/coincidental-cohesion.md`
- `examples/false-deduplication.md`
- `examples/helper-smell.md`
- `examples/refactor-be-damned.md`

**Acceptance criteria:**

- [ ] Directory + files exist.
- [ ] Each file has YAML frontmatter placeholder (for SKILL.md only).

### Task 1.2: Write `framework.md`

Content: the 5-category × sub-type table above. For each sub-type:

- Definition (1-2 sentences)
- Detection signal (concrete, grep-able or readable)
- Legitimate exceptions

**Acceptance criteria:**

- [ ] All 5 categories present.
- [ ] All ~22 sub-types covered.
- [ ] Each sub-type has definition + signal + exceptions.
- [ ] Total length < 300 lines.

### Task 1.3: Write `methodology.md`

Content: the 8 anti-slop rules above + severity rubric.

**Acceptance criteria:**

- [ ] All 8 rules present.
- [ ] Severity rubric present.
- [ ] Total length < 150 lines.

### Task 1.4: Write `report-template.md`

Content: the markdown output format above with placeholders.

**Acceptance criteria:**

- [ ] Template matches the format used in `reports/zone-*.md`.
- [ ] Includes all required fields (file:line, evidence, why, severity, fix).
- [ ] Total length < 80 lines.

---

### Phase 2: Examples

**Task 2.1: Write `examples/premature-generalization.md`**
1-2 trimmed examples per sub-type, drawn from the 47-finding corpus:

- Single-variant enum → `StatePatch` (A1)
- Single-impl trait → `PromptAssembler` (C4)
- Dead trait method → `narrate_continuation` (C1)
- Dead enum variant → `ActionOutcome::Error` (B2)
- Single-consumer hook → `preprocess_user_text` (C8)

Each example: ~15-20 lines (file:line, quote, why smell, fix).

**Acceptance criteria:**

- [ ] All 5 sub-types represented.
- [ ] Each example < 25 lines.
- [ ] Total file < 150 lines.

### Task 2.2: Write `examples/coincidental-cohesion.md`

- Generic filename → `fragments/misc.rs` (D1)
- Unrelated types in one module → `model/state.rs` (A7)
- Module named for use, not concept → `fragments/renderers.rs` (D8)

**Acceptance criteria:**

- [ ] All 3 sub-types represented.
- [ ] Total file < 100 lines.

### Task 2.3: Write `examples/false-deduplication.md`

- Mirror types → `Confidence` vs `QuantifierConfidence` (A3)
- Global behavior forced on all impls → `sanitize_llm_output` (C2)
- Generic function with provider-specific params → `configure_request` OpenRouter headers (C5)
- Copy-paste forced by empty trait default → `save_message` in 4 backends (C11)

**Acceptance criteria:**

- [ ] All 4 sub-types represented.
- [ ] Total file < 150 lines.

### Task 2.4: Write `examples/helper-smell.md`

- God-function → `run_from_input` (B3)
- Identity-wrapper service → `DefaultApplicationService` 14 wrappers (B4)
- `too_many_arguments` suppression → `phase_narrate` (B5)
- Parameter accumulation → `build_trigger_request` (B6)
- Error burying → `error_return` (B9)
- Test double flag-bag → `MockBackend` (C6)

**Acceptance criteria:**

- [ ] All 6 sub-types represented.
- [ ] Total file < 200 lines.

### Task 2.5: Write `examples/refactor-be-damned.md`

- Invalid domain object → `Message::from_db` (A10)
- Orphaned param → `_player_name` (B1)
- Pipeline re-implementation → `ArrivalTaskContext` (B8)
- Single-caller helper → `push_section` (A9)
- Mirrored fields → `Message` mirrors `Swipe` (A4)
- DTO mirroring source → `MessageEntry` (A11)
- Test-only enum leaking into prod → `Operation` enum (D4)

**Acceptance criteria:**

- [ ] All 7 sub-types represented.
- [ ] Total file < 200 lines.

---

### Phase 3: SKILL.md

**Task 3.1: Write `SKILL.md`**
Frontmatter + body:

- Trigger keywords
- Scope options (single file / module / diff / full codebase)
- Output contract (markdown to stdout)
- Resources loaded (always: framework, methodology, report-template, examples)
- Execution model (standalone, no subagents)
- What to do when scope exhausted
- What to do when framework doesn't fit a finding (note gap, drop finding)

**Acceptance criteria:**

- [ ] Trigger keywords listed.
- [ ] Scope options clear.
- [ ] Resources enumerated.
- [ ] Total length < 150 lines.

---

## Validation Phase

### Task 4.1: Dry-run against existing reports

Invoke skill scope=`src/model/state.rs` (subset of Zone A). Compare output to `reports/zone-a-model.md` findings A7 (state.rs grab-bag), A11 (MessageEntry DTO).

**Acceptance criteria:**

- [ ] Skill output includes finding about state.rs cohesion.
- [ ] Skill output does NOT include findings outside scope.
- [ ] Each finding has all required fields (file:line, quote, why, severity, fix).
- [ ] Execution completes in < 5 minutes.

### Task 4.2: Dry-run against a clean file

Invoke skill on a file known to be clean (e.g. `src/error.rs` — small, well-structured).

**Acceptance criteria:**

- [ ] Skill outputs "0 findings" or only low-severity findings.
- [ ] Skill does not hallucinate problems to pad the count.
- [ ] Positive patterns section notes what's working.

### Task 4.3: Dry-run against full zone

Invoke skill scope=`src/narrative/` (Zone C from corpus).

**Acceptance criteria:**

- [ ] Skill output overlaps with `reports/zone-c-narrative.md` on at least 5 of the 11 findings.
- [ ] Skill output does not produce generic "this could be cleaner" findings.
- [ ] Total findings ≤ 15 (cap respected).

---

## Dependencies

| Task | Depends on | Blocks |
|------|-----------|--------|
| 1.1 Directory structure | None | 1.2-1.4, 2.x, 3.1 |
| 1.2 framework.md | 1.1 | 2.x, 3.1, 4.x |
| 1.3 methodology.md | 1.1 | 2.x, 3.1, 4.x |
| 1.4 report-template.md | 1.1 | 3.1, 4.x |
| 2.1-2.5 examples | 1.2, 1.3 | 3.1, 4.x |
| 3.1 SKILL.md | 1.1, 1.2, 1.3, 1.4, 2.x | 4.x |
| 4.1 dry-run subset | 3.1 | — |
| 4.2 dry-run clean | 3.1 | — |
| 4.3 dry-run zone | 3.1 | — |

---

## Risks

| Risk | Mitigation |
|------|-----------|
| Skill produces slop findings (generic "could be cleaner") | Anti-slop methodology rules + evidence-required rule + cap |
| Skill hallucinates file:line refs | Required verbatim quote forces agent to read file first |
| Skill overfits to chronicler_engine patterns | Acceptable — project-scoped by design. Generalize later if needed. |
| Framework has gaps (finding doesn't fit any sub-type) | Methodology rule 5: note gap, drop finding. Don't force fit. |
| Inconsistent output across runs | `report-template.md` enforces structure. Methodology enforces required fields. |
| Skill too slow on full codebase | Scope options + warn user about full-codebase cost. Default to zone scope. |
| Examples file too long → LLM context bloat | Cap each examples file < 200 lines. Trim aggressively. |
| Skill triggers on every code review (noise) | Explicit invocation only, no auto-trigger. |
| Overlap with existing `code-review-and-quality` / `m15-anti-pattern` skills | Document in SKILL.md that this is narrower (abstraction anti-patterns only) and deeper (categorized framework). |

---

## Success Criteria

1. Skill exists at `.agents/skills/antipattern-checker/` with all files present.
2. Dry-run on `src/model/state.rs` produces findings overlapping with Zone A corpus.
3. Dry-run on clean file produces 0 or low-severity findings (no hallucinated padding).
4. Dry-run on `src/narrative/` produces findings overlapping with Zone C corpus (≥5 of 11).
5. Every finding has all required fields (file:line, quote, why, severity, fix).
6. Total skill size < 1500 lines across all files.

---

## Out of Scope

- **Static rule implementation** — `arch-lint.toml`, `tests/guardrails.rs`, `clippy.toml` changes. Those are covered by a separate plan (`abstraction-antipattern-healthcheck-plan.md`).
- **Subagent integration** — skill is standalone. No pi-subagents wiring, no parallel fan-out, no async orchestration. Future extension if needed.
- **User-scope generalization** — skill is project-scoped to Chronicler Engine. Generalizing to other Rust projects is a future task after methodology is validated.
- **CI integration** — skill is dev-time only. LLM cannot run in CI.
- **Automatic remediation** — skill reports findings only. Applying fixes is a separate task (Tier 1 surgical deletes from the summary report).
- **Auto-trigger on code review** — skill is explicitly invoked only.
- **Architecture doc update** — per `chronicler_engine/AGENTS.md`, runtime behavior unchanged; this skill is tooling, not a spec change. No `docs/system/*.md` update needed.

---

## Future Work (Not in This Plan)

- **Tier 2 prevention rules** — single-variant enum / single-impl trait blocking rules in `arch-lint.toml` or `tests/guardrails.rs`. Defer until static rule tooling is extended.
- **Tier 1 surgical deletes** — fix the 47 findings from the validation corpus. Separate cleanup task.
- **User-scope generalization** — promote skill to `~/.agents/skills/` after 2-3 successful runs validate methodology.
- **Subagent orchestration integration** — optional future extension for full-codebase audits using the parallel-zone pattern from the validation investigation.
