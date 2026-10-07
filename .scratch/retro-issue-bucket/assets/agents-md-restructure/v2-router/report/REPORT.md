# Variant 2 REPORT — AGENTS.md as a router

Worktree: `/workspace/chronicler-engine/tmp/agents-restructure/v2/wt` (detached HEAD `5fa46615502ac1f98bace17f355364793455caf4`).
No `git add`, no `git stash`, no commit. No `src/` or `tests/*.rs` edits. `python build.py` was not run.

## 0. `docs/agents/` is exempt from `validate_docs.py` (evidence)

Two independent reasons, both read from `scripts/validate_docs.py`:

1. `classify_file` (lines 268–299) rule 5 returns `EXCLUDED` when the first path segment is not in
   `STANDARD_DIR_NAMES`. That set is `{reference, explanation, how-to, tutorials}` (lines ~192–198).
   `agents` is absent, so every `docs/agents/*.md` classifies as `EXCLUDED`. `scan_file` then returns
   without checks (`if role != "STANDARD": return report`, line ~1070).
   Verified by loading the function directly:

   ```
   docs/agents/development-loop.md -> EXCLUDED
   docs/agents/permissions.md -> EXCLUDED
   docs/agents/delegation.md -> EXCLUDED
   docs/agents/issue-tracker.md -> EXCLUDED
   STANDARD_DIR_NAMES: ['explanation', 'how-to', 'reference', 'tutorials']
   ```

2. Stronger: the script never even reaches `classify_file` for these paths. `scan_file` returns early
   unless `is_diataxis_tree_path` is true (path must start `docs/diataxis/`), and `main()` collects
   files only from `docs/diataxis/` (`collect_markdown_files([diataxis_root], ...)`). `docs/agents/`
   is never scanned. This matches the existing `docs/agents/issue-tracker.md`,
   `triage-labels.md`, `domain.md`, which carry no front-matter and still pass.

Conclusion: the three new docs under `docs/agents/` are safe to add; they need no front-matter.

## 1. The full new AGENTS.md prose below the Structure block

52 lines (lines 309–360, including the blank separator line after the marker). Target ≤ 60. The five
every-run guardrails are inline.

```

## Guardrails

- Repository health outranks the current task. A working repository — the build passing — matters more than your task succeeding.
- Keep every file you did not create. Leave unknown and untracked files in place.
- During a code review, report findings and leave the code unchanged.
- Commit only with explicit approval.
- Follow the permission system in `.pi/extensions/pi-permission-system/config.json`. Recommend a permission change at the end of a task; apply it only with explicit user approval.

## Pointers

| Doc | Holds | Reach it when |
|---|---|---|
| `CODING_STANDARDS.md` | Implementation, code-comment, code-review, and testing rules | You write or review production code |
| `tests/AGENTS.md` | Test placement, failure protocol, Test-First Philosophy, test seams | A test fails, or you write or review one |
| `docs/AGENTS.md` | Docs catalogue, Diátaxis conventions, generated-index and pre-commit rules | You write, edit, or review a doc |
| `docs/agents/development-loop.md` | Build commands, final validation, log tailing, target dirs | You build, format, lint, or start the dev server |
| `ENVIRONMENT.md` | Build limits, measured build cost, slow-build diagnosis | A build is slow, waits for the slot, or hits the memory limit |
| `docs/agents/permissions.md` | Permission scope and the rules for changing it | Your task needs git, an install, or a config change |
| `docs/agents/delegation.md` | Subagent count and model rules | You delegate work to a subagent |
| `docs/agents/issue-tracker.md` | `.scratch/` issue layout and wayfinding operations | You file, fetch, or resolve an issue |
| `docs/agents/triage-labels.md` | The five canonical triage `Status:` strings | You apply a triage label |
| `docs/agents/domain.md` | How an engineering skill consumes the domain docs | You explore the codebase with an engineering skill |
| `CONTEXT.md` | Engine glossary — the source of truth for term meanings | You name a domain concept in a title, proposal, or test |

## Communication

Label epistemic status when it matters: known, inferred, or guessed. Say "I don't know" instead of inventing an answer.

When you respond to feedback or analysis, say whether you agree or disagree first. Then say what you changed.

### Decision Making

State your assumptions explicitly. When something is unclear, stop and ask.

When an instruction contradicts the code you see, say so before you act.

When multiple interpretations exist, present all of them. Pick silently only when one is obvious.

Hold a reasoned position. Push back when a simpler approach exists. When the user pushes back and your reasoning still holds, say why.

When a change carries architectural implications the user did not ask about, name the trade-off in your response. Examples: a new dependency, an async pattern, a data structure with a different complexity.

These rules bias toward caution over speed. Use judgment on a trivial task.

### Progress updates

Before your first tool call, state in one sentence what you are about to do.

While you work, update only on an important find or a change of direction.

When you finish, lead with the outcome: the first sentence answers "what happened" or "what did you find".
```

## 2. Disclosure justification — branch, and why not every-branch

| Disclosed doc | Trigger branch (Pointers row) | Why it is not needed on every branch |
|---|---|---|
| `docs/agents/development-loop.md` | You build, format, lint, or start the dev server | Only a run that compiles or runs the engine needs the command list. A docs edit, a review, or a triage run never builds. |
| `tests/AGENTS.md` | A test fails, or you write or review one | Only test work reads the test tree. A non-test task never needs placement rules or the failure protocol. |
| `docs/agents/permissions.md` | Your task needs git, an install, or a config change | Most tasks are pure source or docs edits with no git or permission action. |
| `docs/agents/delegation.md` | You delegate work to a subagent | Delegation is optional. Many runs never delegate. |
| `docs/AGENTS.md` | You write, edit, or review a doc | Only a docs run needs Diátaxis conventions and the index-regen rules. |
| `CODING_STANDARDS.md` | You write or review production code | A docs-only or triage run never touches production code. |
| `ENVIRONMENT.md` | A build is slow, waits for the slot, or hits the memory limit | The material is diagnostic. A normal successful build never reads it. |
| `docs/agents/issue-tracker.md` | You file, fetch, or resolve an issue | Issue tracking fires only in tracked-work runs. |
| `docs/agents/triage-labels.md` | You apply a triage label | Only a triage run needs the five strings. |
| `docs/agents/domain.md` | You explore the codebase with an engineering skill | Only a skill-driven exploration needs the consumption guidance. |
| `CONTEXT.md` | You name a domain concept in a title, proposal, or test | Naming domain terms matters only when producing domain language. |

`## Communication` / `### Decision Making` / `### Progress updates` stay inline: every branch
communicates and every branch decides, so by the skill's branching test they are not disclosable.

## 3. What moved where

| Old AGENTS.md material | New home |
|---|---|
| `## Your Responsibility` health rule, untracked-file rule, code-review rule | AGENTS.md `## Guardrails` (compressed) |
| `## Your Responsibility` code-review rule | also `CODING_STANDARDS.md` `## Code Reviews` (requirement E) |
| `## Communication`, `### Decision Making`, `### Progress updates` | stay inline in AGENTS.md, tightened |
| `## The Test-First Philosophy` (incl. analysis-paralysis paragraph) | `tests/AGENTS.md` `## Test-First Philosophy` (new, co-located above `## Failure handling`) |
| `## Documentation Index` ¶1: generated indexes + pre-commit hook | `docs/AGENTS.md` `## Generated indexes` (new; the single home for these rules) |
| `## Documentation Index` ¶2: CODING_STANDARDS pointer | AGENTS.md `## Pointers` row for `CODING_STANDARDS.md` |
| `## Development Loop` tmp-folder, re-read-before-edit, no-glob rules | `docs/agents/development-loop.md` `## Working rules` |
| `## Development Loop` build-log/tail/Pi-timeout facts | `docs/agents/development-loop.md` `## Build logs` |
| `## Development Loop` "~2 min warm / cold far longer" | `ENVIRONMENT.md` `## Typical gate cost` |
| `### Commands`, `#### Iteration`, `#### Final Validation` | `docs/agents/development-loop.md` |
| `## Concurrent Builds` | `docs/agents/development-loop.md` `## Concurrent builds` (workflow rules); sccache/seeding/slot reasons already live in `ENVIRONMENT.md` |
| `## Agent Skills` (`### Issue tracker`, `### Triage labels`, `### Domain docs`) | AGENTS.md `## Pointers` rows; the three docs are unchanged |
| `## Permissions System` | `docs/agents/permissions.md` (new); commit + permission guardrails also inline |
| `## Subagents and delegation extra rules` | `docs/agents/delegation.md` (new) |

### Requirement A split (development-loop.md vs ENVIRONMENT.md)

- **To `docs/agents/development-loop.md`:** the command surface, final validation, the working rules,
  the concurrent-build workflow rules, the log file location, the tail command, and the 1200-second
  Pi timeout. Reason: these are steps the agent takes, not machine facts.
- **To `ENVIRONMENT.md`:** the build-cost fact ("A warm `build.py` gate finishes in about 2 minutes. A
  cold target dir takes far longer — seed it from a warm sibling, or accept the cost in the table
  below"). Reason: `ENVIRONMENT.md` already owns measured build costs (its "Typical gate cost" table),
  build limits, and slow-build diagnosis. The measured numbers live there; `development-loop.md`
  points at it instead of repeating them. I also retargeted `ENVIRONMENT.md`'s own first line from
  "`AGENTS.md` says how to run builds" to "`docs/agents/development-loop.md` says how to run builds".

## 4. No-ops deleted

1. `**Don't assume. Don't hide confusion. Surface tradeoffs.**` — a heading that restated the five
   bullets directly beneath it.
2. "Do not silently work around the mismatch or proceed as if the instruction were accurate." —
   restated the preceding sentence.
3. "Do not bury it." — restated "name the trade-off in your response".
4. "Integration tests go in `tests/`; the failure-handling protocol lives in `tests/AGENTS.md`." —
   a self-reference once the paragraph moved into `tests/AGENTS.md`.
5. "Read it before writing or reviewing code." — the Pointers trigger now carries this.
6. The long untracked-file rationale clause ("especially untracked file … could interfere with other
   work going on … at the same time") — replaced by the positive rule "Leave unknown and untracked
   files in place."
7. "### Issue tracker" / "### Triage labels" / "### Domain docs" section scaffolding — one table row
   each replaced three headings plus three pointer sentences.

No unique meaning was dropped; every remaining sentence has a home.

## 5. Exact validation output

### `git diff --stat` (untracked new docs are not shown by diff; `git status` follows)

```
 AGENTS.md           | 141 +++++++++++++---------------------------------------
 CODING_STANDARDS.md |   2 +
 ENVIRONMENT.md      |   8 ++-
 docs/AGENTS.md      |  11 ++++
 tests/AGENTS.md     |   6 +++
 5 files changed, 60 insertions(+), 108 deletions(-)
```

```
$ git status --porcelain
 M AGENTS.md
 M CODING_STANDARDS.md
 M ENVIRONMENT.md
 M docs/AGENTS.md
 M tests/AGENTS.md
?? docs/agents/delegation.md
?? docs/agents/development-loop.md
?? docs/agents/permissions.md
```

No path under `src/` or matching `tests/*.rs` appears.

### `python scripts/generate_structure_index.py` (succeeded; left prose unchanged)

```
  Updated STRUCTURE section in AGENTS.md
  Structure index regenerated successfully
exit=0
```

Integrity after regeneration:

```
structure block (lines 6-308) md5 working: d15d0aa434d5ef970d4f412fb84c8208
structure block (lines 6-308) md5 HEAD   : d15d0aa434d5ef970d4f412fb84c8208   (byte-identical)
prose (lines 310+) md5 before regen: b4f3c44530dbb5ee3d4d4a5976f45897
prose (lines 310+) md5 after  regen: b4f3c44530dbb5ee3d4d4a5976f45897   (unchanged)
prose line count after marker       : 52
```

### `python scripts/validate_docs.py` (tail)

```
============================================================
Scanned 25 file(s)
Summary: 0 error(s), 0 warning(s) across 0 file(s)

============================================================
Scanned 420 file(s)
Summary: 0 error(s), 0 warning(s) across 0 file(s)

PASS
exit=0
```

### Relative-link check (how)

I extracted every `[x](y)` link in the changed/new markdown and listed the backticked repo paths used
by the Pointers table, then tested each with `[ -f "$p" ] || [ -d "$p" ]` from the worktree root.
Result: every path resolves.

```
OK  CODING_STANDARDS.md        OK  docs/agents/development-loop.md   OK  .pi/extensions/pi-permission-system/config.json
OK  tests/AGENTS.md            OK  docs/agents/permissions.md        OK  scripts/generate_structure_index.py
OK  tests/STRATEGY.md          OK  docs/agents/delegation.md         OK  scripts/generate_tests_structure_index.py
OK  docs/AGENTS.md             OK  docs/agents/issue-tracker.md      OK  scripts/generate_docs_index.py
OK  ENVIRONMENT.md             OK  docs/agents/triage-labels.md      OK  scripts/generate_guardrails_doc.py
OK  CONTEXT.md                 OK  docs/agents/domain.md             OK  scripts/precommit_regenerate.py
                              OK  scripts/validate_docs.py        OK  build.py
```

The only inline markdown link in the changed files is `tests/AGENTS.md`'s existing
`[STRATEGY.md](STRATEGY.md)`, which resolves relative to `tests/`.

`validate_docs.py` cannot check these links: it only scans `docs/diataxis/` and stops on
`role != "STANDARD"`. The check above is therefore manual.

## 6. What this variant sacrifices

**A guardrail that lost force — repository health.** The old text carried the reason: deleting or
reverting an unknown file "could interfere with other work going on in the repository at the same
time." That reason is gone from always-loaded context; only the rule remains. An agent whose task is
blocked by a stray file no longer sees why the rule exists, so it is more likely to decide the rule is
over-cautious. The same applies to the untracked-file rule, whose "arbitrarily" qualifier is gone:
the rule is now absolute and forbids removing a file you did not create even when that file plainly
breaks the build. That is the intended safety bias, but it is stronger than before.

**The weakest pointer — `docs/agents/domain.md`.** "You explore the codebase with an engineering
skill" is the vaguest branch in the table. An agent may not recognise its run as skill-driven, and the
row overlaps `CONTEXT.md` (both are about domain vocabulary). A secondary weakness: the
`docs/AGENTS.md` row bundles two triggers (doc writing and index regeneration) into one row, because
it points at one doc.

**AGENTS.md is not a pure router.** `## Communication` / `## Decision Making` / `## Progress updates`
(20 lines) remain inline. Every branch communicates and decides, so disclosing them would produce a
pointer that fires on every run — a variance bug by the skill's own test. The cost is that AGENTS.md
still holds 34 non-blank lines of always-loaded behaviour.

**Deliberate duplication, forced by the constraints.** Two of the five guardrails now have a second
copy: the code-review rule in AGENTS.md and `CODING_STANDARDS.md` `## Code Reviews` (constraint 2
pins it inline; requirement E moves code-review rules into CODING_STANDARDS.md), and the
commit/permission rules in AGENTS.md and `docs/agents/permissions.md` (constraint 2 pins them inline;
requirement C creates the doc). Changing either rule is now a two-place edit.

**Interpretation on requirement E.** No implementation, code-comment, or testing rule remained in
AGENTS.md after A–D. The only code-review rule was guardrail (c). I read E as: CODING_STANDARDS.md
stays the single home for those rule categories, and AGENTS.md keeps exactly one pointer (the table
row). I added guardrail (c) to CODING_STANDARDS.md to satisfy E literally, while keeping the inline
copy that constraint 2 demands. If E meant "no new copy at all", the CODING_STANDARDS.md line is the
one to drop.

**Stale downstream references, not updated (out of scope).** My change moved material that other
files still point at through AGENTS.md:

- `.agents/skills/test-police/SKILL.md:108` — "`AGENTS.md` — test-first philosophy, … commands,
  development loop, concurrent-build flags". The test-first material is now in `tests/AGENTS.md`;
  commands/development-loop/concurrent-builds are in `docs/agents/development-loop.md`.
- `.agents/skills/test-police/SKILL.md:114` — "Full command reference: `AGENTS.md`".
- `.agents/skills/chronicler-after-plan-workflow-plus-review/SKILL.md:24` — "(see Concurrent Builds in
  `AGENTS.md`)".
- `.cargo/config.toml:4` — "(AGENTS.md, \"Concurrent Builds\")".

I left these alone because the assigned work (A–G) does not include editing skills or cargo config,
and narrowing scope is safer than a speculative sweep. They are the main remaining work of this
variant.

## Remaining work

1. Retarget the four stale references listed above.
2. Decide whether to keep the deliberate code-review duplication in `CODING_STANDARDS.md`.
3. Optional: split the `docs/AGENTS.md` Pointers row if the index-generation trigger should stand
   alone.
