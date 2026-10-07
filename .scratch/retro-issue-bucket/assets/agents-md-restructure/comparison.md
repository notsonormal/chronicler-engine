# Comparison: the three AGENTS.md restructures

Acceptance review by the lead session `01a1133f-1bca-7318-a1a3-5b28d267ac44` for [Choose the AGENTS.md restructure to adopt](../../issues/09-choose-agents-md-restructure.md).

Read the resulting files in `v1-prune/`, `v2-router/` and `v3-radical/`. This file records the judgement, not the content.

## The escalation

| | `v1-prune` | `v2-router` | `v3-radical` |
| --- | --- | --- | --- |
| Shape | Same headings, pruned prose | Core plus pointer table | Pointer table only |
| Always-loaded lines | 430 → 395 | 430 → 360 | 430 → 26 |
| Prose lines | 123 → 88 (−28.5%) | 123 → 52 | 123 → 26 |
| `AUTO-STRUCTURE` block | Inline, byte-identical | Inline, byte-identical | Moved to `STRUCTURE.md` |
| New files | 0 | 3 | 9 + `STRUCTURE.md` |
| Scripts touched | 0 | 0 | 2 |
| Pointers to `issue-tracker`/`triage-labels`/`domain` | Kept | Kept | Lost |
| `validate_docs.py` | PASS 0/0 | PASS 0/0 | PASS 0/0 |

`v1` and `v2` fix prose quality but barely move context load: 300 of the 430 lines are the generated block, which both keep inline. Only `v3` attacks that block.

## `v1-prune` — prune and co-locate

Two moves. The per-edit discipline and the tmp-file rule left `AGENTS.md` for a new `## Editing` section of `CODING_STANDARDS.md`. The 11-command code block collapsed into a `python build.py --help` pointer plus the one fact `--help` lacks: which test tier to run.

Deleted as no-ops, each with a recorded reason: the `"if you don't know something, say so"` line, `"These guidelines bias toward caution over speed"`, the analysis-paralysis sentence, `"A comprehensive suite … source of truth"`, the bolded `"Don't assume. Don't hide confusion. Surface tradeoffs."`, `"Do not bury it."`, `"This is a preference"`, `"These restrictions exist to prevent the agent from touching git"`, `"Almost every full-gate step is also a subcommand"`, and the 11 `#` comments that restated `--help`.

Kept against the no-op test, with reasons: agree/disagree before a change, epistemic labels, the five guardrails, the `scout` model rule.

## `v2-router` — AGENTS.md as a router

Five guardrails stay inline. An 11-row pointer table routes the rest. Three new docs carry the disclosed material; the Test-First Philosophy joins `tests/AGENTS.md`, and the generated-index rules join `docs/AGENTS.md`.

Every pointer path resolves on disk. `docs/agents/` is `EXCLUDED` by `validate_docs.py`'s `classify_file`, so the new docs are not scanned — the session's claim is correct.

Named sacrifices from the session's report: the repository-health guardrail loses its rationale from always-loaded context; two guardrails gain a permitted second copy; five downstream references still name `AGENTS.md`.

## `v3-radical` — router plus the Structure-block move

`AGENTS.md` becomes 26 lines. `STRUCTURE.md` takes the moved block under its `## Structure` heading, with both markers intact for the pre-commit contract. Two script edits retarget the generator and the pre-commit file list; the four-file count and the marker contract survive. `CODING_STANDARDS.md` becomes a 14-line branch router over three new diataxis coding-standards docs.

`build.py` does not regenerate the structure index, so no build step is affected. The generator runs and reports `Updated STRUCTURE section in STRUCTURE.md`; rerunning is idempotent.

Validation gap: `pytest` is absent and `pip` is unavailable, so the generator tests ran under `python -m unittest discover scripts/tests`: 285 tests, OK. The lead session did not re-run them.

Six consumers need edits before this can land: the `commit-and-push` skill's four-file list, `chronicler-comment-fixer`, `test-police`, `ENVIRONMENT.md:3`, `.cargo/config.toml:4`, `docs/diataxis/explanation/diataxis.md:67`.

## Lead judgement

`v2-router` is the best standalone result. It delivers real progressive disclosure, keeps all eleven pointers, passes the validators, and needs no tooling change. `v1-prune` is the safe fallback. `v3-radical` has the cleanest shape but loses three pointers and couples the top-level file layout to two scripts.

Recommended order of landing:

1. Adopt `v2-router`.
2. Apply `v1-prune`'s pruning discipline to the newly disclosed docs, since `v2` disclosed before it pruned.
3. Land the `STRUCTURE.md` relocation from `v3-radical` as its own change, with the six consumers fixed and the three lost pointers restored.

Step 3 is the only one that pays for itself in always-loaded context. It should land alone, deliberately, not as a side effect.
