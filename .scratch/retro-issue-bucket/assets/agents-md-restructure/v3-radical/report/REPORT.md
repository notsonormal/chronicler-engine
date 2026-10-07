# VARIANT 3 — AGENTS.md as a routing table (radical restructure)

Worktree: `/workspace/chronicler-engine/tmp/agents-restructure/v3/wt` (detached HEAD `5fa46615`).
No commit, no `git add`, no `git stash`, no `python build.py`, no `src/` or `tests/*.rs` edits, no DOC-anchor edits.
Edited files are left in the worktree (all unstaged). This report lives outside the worktree.

---

## 1. Exact diff of the two scripts

### `scripts/generate_structure_index.py`

```diff
diff --git a/scripts/generate_structure_index.py b/scripts/generate_structure_index.py
index 1933429f..d1aa70ef 100644
--- a/scripts/generate_structure_index.py
+++ b/scripts/generate_structure_index.py
@@ -1,4 +1,4 @@
-"""Generate AGENTS.md structure index from module summaries."""
+"""Generate the STRUCTURE.md source index from module summaries."""
 
 from __future__ import annotations
 
@@ -134,13 +134,13 @@ def main() -> int:
     engine_dir = Path(__file__).parent.parent
     src_dir = engine_dir / "src"
     scripts_dir = engine_dir / "scripts"
-    agents_md = engine_dir / "AGENTS.md"
+    structure_md = engine_dir / "STRUCTURE.md"
 
-    if not agents_md.exists():
-        print(f"AGENTS.md not found: {agents_md}")
+    if not structure_md.exists():
+        print(f"STRUCTURE.md not found: {structure_md}")
         return 1
 
-    content = agents_md.read_text(encoding="utf-8")
+    content = structure_md.read_text(encoding="utf-8")
 
     rust_bullets = build_bullet_structure(src_dir)
     python_bullets = build_python_bullets(scripts_dir, engine_dir)
@@ -166,10 +166,10 @@ def main() -> int:
     if structure_match:
         old_structure = structure_match.group(0)
         new_content = content.replace(old_structure, new_structure)
-        agents_md.write_text(new_content, encoding="utf-8")
-        print("  Updated STRUCTURE section in AGENTS.md")
+        structure_md.write_text(new_content, encoding="utf-8")
+        print("  Updated STRUCTURE section in STRUCTURE.md")
     else:
-        print("  Warning: Could not find STRUCTURE section in AGENTS.md")
+        print("  Warning: Could not find STRUCTURE section in STRUCTURE.md")
         return 1
 
     print("  Structure index regenerated successfully")
```

Also changed (inside `extract_module_info`'s docstring, not shown by the hunk above because it is a comment-only edit in the same function; it appears in `git diff --stat` as +19/-?):

```diff
-      - `//! summary` on line 1 (anchor-less, e.g. src/test_support/ per AGENTS.md)
+      - `//! summary` on line 1 (anchor-less, e.g. src/test_support/, which the
+        structure guardrail exempts)
```

### `scripts/precommit_regenerate.py`

```diff
diff --git a/scripts/precommit_regenerate.py b/scripts/precommit_regenerate.py
index d4f24155..392e2405 100644
--- a/scripts/precommit_regenerate.py
+++ b/scripts/precommit_regenerate.py
@@ -20,7 +20,7 @@ from typing import Mapping, Sequence
 # (path, expected marker keys in file order). The keys mirror the four generators;
 # a file without exactly these pairs is treated as foreign and left alone.
 GENERATED_FILES: tuple[tuple[str, tuple[str, ...]], ...] = (
-    ("AGENTS.md", ("AUTO-STRUCTURE",)),
+    ("STRUCTURE.md", ("AUTO-STRUCTURE",)),
     ("tests/AGENTS.md", ("AUTO-STRUCTURE-TESTS",)),
     ("docs/AGENTS.md", ("AUTO-INDEX",)),
     (
```

Verified `GENERATED_FILES` still has four entries and the marker contract is unchanged:

```
(('STRUCTURE.md', ('AUTO-STRUCTURE',)),
 ('tests/AGENTS.md', ('AUTO-STRUCTURE-TESTS',)),
 ('docs/AGENTS.md', ('AUTO-INDEX',)),
 ('docs/diataxis/reference/coding_standards/guardrails.md',
  ('AUTO-GUARDRAILS: clippy', 'AUTO-GUARDRAILS: arch-lint', 'AUTO-GUARDRAILS: syn')))
```

`precommit_regenerate.split_prose` parses `STRUCTURE.md`'s `AUTO-STRUCTURE` pair successfully (checked directly). The existing unit tests for both scripts encode no old path, so they were left unchanged; both still pass (see §6).

The `test_generate_structure_index.py` and `test_precommit_regenerate.py` files were inspected: neither hard-codes `AGENTS.md`. `test_precommit_regenerate.py` uses its own `TARGET.md` fixture; `test_generate_structure_index.py` only tests `build_bullet_structure`/`extract_module_info`. No update required.

---

## 2. Full new `AGENTS.md` (26 lines ≤ 40)

```markdown
# Chronicler Engine Knowledge Base

## Overview
Interactive fiction/text adventure engine in Rust. HTTP/WebSocket server with HTMX dashboard, LLM-powered narrative generation, data-driven game state from JSON configs.

## Every-run guardrails
- Repository health outranks the current task: a green build matters more than this task succeeding.
- Never delete or revert unknown or untracked files, even when they block you.
- During code reviews, change no code; report findings only.
- Never commit without explicit approval.
- Never circumvent the permission system; never edit its config without approval.

## Where to read next

| Doc | Holds | Reach it when |
| --- | --- | --- |
| `docs/agents/README.md` | Agent-process router. | Orient yourself on how to work here. |
| `docs/agents/development-loop.md` | Working norms, build loop, commands, concurrent builds. | Building, testing, or iterating. |
| `docs/agents/permissions.md` | Permission rules and approval boundaries. | Touching git, config, or permission-gated actions. |
| `docs/agents/delegation.md` | Subagent choice and sizing. | Delegating to a subagent. |
| `docs/agents/documentation.md` | Doc index and generated-index rules. | Writing docs or committing. |
| `tests/AGENTS.md` | Test structure, tiers, failure handling. | Writing, running, or debugging tests. |
| `CODING_STANDARDS.md` | Coding-rules router. | Writing or reviewing code. |
| `CONTEXT.md` | Domain glossary. | Naming a domain concept. |
| `ENVIRONMENT.md` | Build-environment limits and diagnostics. | Builds slow, blocked, or out of memory. |
| `STRUCTURE.md` | Generated source-tree index. | Finding a module's file or summary. |
```

Constraint 1 holds: the five every-run guardrails (a)–(e) are inline. The 300-line `## Structure` heading + marker block is gone; the heading and `AUTO-STRUCTURE` markers moved verbatim to `STRUCTURE.md` (which keeps the `## Structure` heading so `precommit_regenerate.py`'s marker match still works).

---

## 3. New doc tree and what each doc holds

```
AGENTS.md                26 lines — router (this file)
STRUCTURE.md            308 lines — generated source-tree index (AUTO-STRUCTURE block)
CODING_STANDARDS.md      14 lines — router to the coding-rule docs
docs/agents/
  README.md              21 lines — router mirror (process docs + neighbouring docs)
  development-loop.md    81 lines — communication, decision-making, test-first, loop, commands, concurrent builds
  permissions.md          7 lines — permission system + approval boundaries
  delegation.md           5 lines — subagent sizing + scout/model rule
  documentation.md       19 lines — doc catalogue + generated-index/pre-commit rules
docs/diataxis/reference/coding_standards/
  implementation.md      16 lines — backward compat, naming, HTTP form structs
  code_comments.md       19 lines — file headers (DOC anchor + summary), what comments are for
  code_reviews.md        12 lines — no-compile-during-review, actionable findings
  testing.md            (+4 lines) — "Test Quality: Tautological tests are considered harmful."
```

| Doc | Holds | Reached by |
|---|---|---|
| `AGENTS.md` | Overview + five guardrails + pointer table. | Always loaded. |
| `STRUCTURE.md` | Generated `src/`, `scripts/`, root-script index. | "Finding a module's file or summary." |
| `docs/agents/README.md` | Process-doc router with more detail than the AGENTS.md table. | "Orient yourself on how to work here." |
| `docs/agents/development-loop.md` | Communication; decision-making + progress updates; test-first stance; the loop; commands; final validation; concurrent builds. Env "why" deferred to `ENVIRONMENT.md`. | "Building, testing, or iterating." |
| `docs/agents/permissions.md` | Config location, no-circumvention, recommend-not-apply, git norms. | "Touching git, config, or permission-gated actions." |
| `docs/agents/delegation.md` | Prefer one long subagent; scout restricted to Anthropic models. | "Delegating to a subagent." |
| `docs/agents/documentation.md` | `docs/AGENTS.md` + `tests/AGENTS.md` catalogues; the four generated files; never hand-edit; regenerate via `scripts/`. | "Writing docs or committing." |
| `tests/AGENTS.md` (existing) | Test structure, tiers, failure handling. No duplication — AGENTS.md points here for testing. | "Writing, running, or debugging tests." |
| `CODING_STANDARDS.md` | Thin branch table into the three new STANDARD docs + the test standards. | "Writing or reviewing code." |
| `CONTEXT.md` (existing) | Domain glossary. | "Naming a domain concept." |
| `ENVIRONMENT.md` (existing) | Build slot, memory, sccache, seeding, lld, diagnostics (already owned there). | "Builds slow, blocked, or out of memory." |

New STANDARD docs carry the YAML front-matter shape copied from `testing.md` (`diataxis: reference`, `title: …`). `/workspace/.../tmp/agents-restructure/v3/wt/scripts/validate_docs.py` passes with **0 errors, 0 warnings** (all standard-doc rules: front-matter keys/mode, link integrity, doc-internal body references confined to `## Document References`, line citations, source-path existence).

### `CODING_STANDARDS.md` (new content, in full)

```markdown
# Coding Standards

Coding rules are split by branch. Read the rule for the branch you are on:

| Branch | Rule doc | Reach it when |
| --- | --- | --- |
| Implementation | `docs/diataxis/reference/coding_standards/implementation.md` | Writing production code. |
| Code comments | `docs/diataxis/reference/coding_standards/code_comments.md` | Adding or reviewing file comments or DOC anchors. |
| Code reviews | `docs/diataxis/reference/coding_standards/code_reviews.md` | Reviewing a change. |
| Tests | `docs/diataxis/reference/coding_standards/testing.md` | Writing or reviewing tests. |
| Unit test standards | `docs/diataxis/reference/coding_standards/unit_test_standards.md` | Shaping a `*_tests.rs` unit test. |
| Integration test standards | `docs/diataxis/reference/coding_standards/integration_test_standards.md` | Shaping a test under `tests/`. |

Read `tests/AGENTS.md` and `tests/STRATEGY.md` before writing or reviewing tests. Read `docs/AGENTS.md` before writing or reviewing documentation.
```

The four existing files in `docs/diataxis/reference/coding_standards/` stay in place; only `testing.md` gained a 4-line `## Test Quality` section (below).

### `docs/AGENTS.md` (generated index; regenerated so it is not stale)

`python scripts/generate_docs_index.py` was run. Its diff is the timestamp plus three new entries under `docs/diataxis/reference/coding_standards/` (`Code Comments`, `Code Reviews`, `Implementation`). This was not in the mandated validation list but keeps the generated catalogue consistent with the new files; without it the index would be stale until the pre-commit hook ran.

---

## 4. "What would break if this landed" — consumer list (F)

No build- or test-breaking consumer was found. `python build.py`'s steps do not read the root `AGENTS.md` beyond it being present; `validate-docs` is unaffected; the git hook calls `precommit_regenerate.py`, which was updated. Two real landing actions and several stale pointers remain:

**Must be staged on the first commit**
1. `STRUCTURE.md` is a **new untracked file**. `precommit_regenerate._diagnose` requires each generated file to be committed (`git show :<rel>`); with `STRUCTURE.md` untracked the hook aborts with *"STRUCTURE.md is not committed yet, so this hook cannot regenerate it safely. `git add STRUCTURE.md`"*. Needs: `git add STRUCTURE.md` (and the other new docs) in the landing commit. This is a required first-commit step, not a break.

**Actively wrong after the move (need a fix; not made here — F asks only to list)**
2. `.agents/skills/commit-and-push/SKILL.md:28` — the "four generated files" list still names `AGENTS.md`. Needs: `- \`STRUCTURE.md\`` in place of `- \`AGENTS.md\`` (it already lists `tests/AGENTS.md`, `docs/AGENTS.md`, guardrails). Without this the skill tells reviewers a file is generated when it no longer is.
3. `.agents/skills/chronicler-comment-fixer/SKILL.md:103` — "The anchor and module-summary rules live in `CODING_STANDARDS.md` (`## Code comments`) …". That heading moved. Needs: point at `docs/diataxis/reference/coding_standards/code_comments.md`.
4. `.agents/skills/test-police/SKILL.md:108,114` — points at `AGENTS.md` for "commands, development loop, concurrent-build flags" / "Full command reference". Needs: `docs/agents/development-loop.md`.
5. `ENVIRONMENT.md:3` — "`AGENTS.md` says how to run builds." Needs: `docs/agents/development-loop.md` (ENVIRONMENT.md owns only the environment facts).
6. `.cargo/config.toml:4` — comment references "(`AGENTS.md, "Concurrent Builds")`". Needs: `docs/agents/development-loop.md`.

**Stale/ambiguous but not newly broken**
7. `docs/diataxis/explanation/diataxis.md:67` — links `../../AGENTS.md` as "the writing-convention layer for this docs tree". That layer is actually `docs/AGENTS.md`, so the pointer is already inaccurate; the radical cut makes the mismatch sharper. Needs: retarget to `docs/AGENTS.md`.
8. `.agents/skills/code-review/SKILL.md`, `antipattern-checker/SKILL.md`, `thermo-nuclear-code-quality-review/SKILL.md`, `code-consistency-check/SKILL.md` — "Always read `CODING_STANDARDS.md` before reviewing." Still valid (it is now a router); could optionally retarget to `code_reviews.md`.
9. `.agents/skills/chronicler-docs-hygiene/SKILL.md` and `diataxis-doc-review/SKILL.md` — "AGENTS.md §anchor" wording, but contextually `docs/AGENTS.md ## Writing Conventions`, which is untouched. Fine; worth de-ambiguating.
10. `README.md:24` ("Working in this repo … AGENTS.md"), `.backpassrc.json memoryFiles: ["AGENTS.md"]`, `docs/agents/domain.md:7,20` — all remain correct: `AGENTS.md` still exists and routes.
11. `scripts/tests/*` — no old-path encoding; unchanged.
12. `.scratch/**`, `docs/plans/**` — transient/historical references to AGENTS.md sections; no action, not canonical.

**Verified non-consumers**
- `scripts/git-hooks/pre-commit` — generic; calls `scripts/precommit_regenerate.py`, no file names.
- `build.py` — `validate-docs` runs `scripts/validate_docs.py`; no reference to the structure block. (No full build was run, per instructions.)
- `scripts/generate_docs_index.py` — indexes `docs/diataxis/` only, unaffected by `STRUCTURE.md`.
- `scripts/generate_guardrails_doc.py` — sources are `src/lib.rs`, `arch-lint.toml`, `tests/infrastructure/guardrails/*.rs`, not `CODING_STANDARDS.md`, so thinning the router is safe.

---

## 5. No-ops deleted

- **Timing prose in `AGENTS.md`** — "A standard build takes about 2 minutes once the target dir is warm. A cold one takes far longer." and the `(~2 min warm)` comment on the final-validation command. `ENVIRONMENT.md`'s "Typical gate cost" table is the single home for gate timings; the relocated `development-loop.md` owns only the commands.
- **"never rationalize a failure away"** — dropped from the relocated test-first paragraph because `tests/AGENTS.md` already owns the failure-handling protocol and the same instruction; `development-loop.md` now points there instead of restating it.
- **"used for auto-generating Structure section"** → "used for auto-generating the source structure index" in `code_comments.md` (the Section no longer exists; this is now true and shorter).
- **The pre-commit generated-file list** was not repeated in `AGENTS.md`/`CODING_STANDARDS.md`; it has one in-scope home in `docs/agents/documentation.md` (`commit-and-push`'s copy is a skill, outside the no-duplication set, and is listed in §4 as needing an update).
- **Duplicate `subagents`/`permissions`/`development loop` bodies** are gone from `AGENTS.md`, which keeps only the terse pointer rows.

---

## 6. Exact validation output

All run from `/workspace/chronicler-engine/tmp/agents-restructure/v3/wt`.

### `git diff --stat`

```
 AGENTS.md                                          | 447 +--------------------
 CODING_STANDARDS.md                                |  46 +--
 docs/AGENTS.md                                     |   5 +-
 .../diataxis/reference/coding_standards/testing.md |   4 +
 scripts/generate_structure_index.py                |  19 +-
 scripts/precommit_regenerate.py                    |   2 +-
 6 files changed, 52 insertions(+), 471 deletions(-)
```

`git status --short` (the generated/edited files are all left unstaged; `STRUCTURE.md` and the new docs are untracked by design — no `git add` allowed):

```
 M AGENTS.md
 M CODING_STANDARDS.md
 M docs/AGENTS.md
 M docs/diataxis/reference/coding_standards/testing.md
 M scripts/generate_structure_index.py
 M scripts/precommit_regenerate.py
?? STRUCTURE.md
?? docs/agents/README.md
?? docs/agents/delegation.md
?? docs/agents/development-loop.md
?? docs/agents/documentation.md
?? docs/agents/permissions.md
?? docs/diataxis/reference/coding_standards/code_comments.md
?? docs/diataxis/reference/coding_standards/code_reviews.md
?? docs/diataxis/reference/coding_standards/implementation.md
```

### `python scripts/generate_structure_index.py`

```
  Updated STRUCTURE section in STRUCTURE.md
  Structure index regenerated successfully
```

Run a second time, `STRUCTURE.md` is byte-identical (idempotent); the `AUTO-STRUCTURE` marker pair parses under `precommit_regenerate.split_prose`.

### `python scripts/validate_docs.py`

```

============================================================
Scanned 28 file(s)
Summary: 0 error(s), 0 warning(s) across 0 file(s)

============================================================
Scanned 420 file(s)
Summary: 0 error(s), 0 warning(s) across 0 file(s)

PASS
```

### `python -m pytest scripts/tests/ -q`

**Cannot be run — pytest is not installed in this environment** (`/home/node/.local/bin/python: No module named pytest`; `pip` is also absent, so I did not install it — installs are outside the task's authority). The repo's own canonical Python-test step is `python -m unittest discover scripts/tests -v` (`build.py` line 496–498), which is the equivalent. Its output:

```
----------------------------------------------------------------------
Ran 285 tests in 5.505s

OK
```

285 tests, 0 failures — same count as the pre-change baseline. That baseline was taken before any edit with the same command; the two structure-related test modules are included in the 285.

---

## 7. Judgement — is the radical cut worth its cost?

**Verdict: yes for `AGENTS.md`, with two reservations.**

The radical cut is the right shape. The old `AGENTS.md` was 430 lines, ~300 of them a generated tree that renders as pure reference — never consulted in the order it is written, and re-paid in context every turn. Moving it to `STRUCTURE.md` behind a one-line pointer is exactly the "disclosed reference" move: the always-loaded context drops to 26 lines while the structure stays one hop away. The `CODING_STANDARDS.md` thinning is the same move for coding rules and is unambiguous — it was already a hybrid of rules and pointers, and the new STANDARD docs put the rules beside the existing test standards where the diataxis front-matter/validator discipline applies.

Reservations:

1. **Double-router duplication.** The task explicitly asks for both the `AGENTS.md` pointer table and a "mirror with more detail" in `docs/agents/README.md`, which conflicts with hard constraint 2 (one home per meaning). I resolved it by making `AGENTS.md` rows terse and the README the detailed branch description, but the same "this doc holds X" meaning genuinely lives in two tables. If the constraint is read strictly, the README should instead be a *subset-free* deeper router (e.g. describe what's inside each doc, not repeat its purpose). This is the one place where I knowingly carried a little duplication because the task requested it. State the trade-off: terse + detailed routers buy discoverability from two entry points at the cost of one extra place to edit.

2. **`development-loop.md` carries more than its name.** Communication, decision-making, progress-update and test-first norms are not "loop" material. I co-located them there because the task enumerated only five docs and none of the others (permissions/delegation/documentation) fit, and putting them in the router would defeat the router's single job. It works, but a cleaner cut would be a separate `working-norms.md` (a sixth doc) or naming this one `how-to-work.md`. Flagging rather than silently inventing a sixth file.

Cost check: the two script edits are 2 logical changes (a path and a docstring); the hook's marker contract is untouched; `validate_docs` is clean; the Python suite is unchanged. The only real landing friction is that `STRUCTURE.md` must be `git add`ed in the landing commit before the hook will regenerate it (§4.1). The consumer list in §4 is short and mechanical. The cut is worth its cost.
