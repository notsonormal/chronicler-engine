# Choose the AGENTS.md restructure to adopt

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`AGENTS.md` is 430 lines. About 300 of them are the generator-owned Structure block; 123 are hand-written prose. Three subagent sessions restructured it at escalating radicalness, each in its own worktree, each validated. Which variant — or which combination of parts — do we adopt?

| Variant | Always-loaded lines | Prose lines | New files | Scripts touched |
| --- | --- | --- | --- | --- |
| `v1-prune` — prune and co-locate | 395 | 88 | 0 | 0 |
| `v2-router` — AGENTS.md as a router | 360 | 52 | 3 | 0 |
| `v3-radical` — router plus the Structure-block move | 26 | 26 | 9 + `STRUCTURE.md` | 2 |

- **Adopt `v1-prune`.** Delete the no-op sentences, co-locate the split facts, move the editing rules into `CODING_STANDARDS.md`. No new files; the Structure block stays inline, so the always-loaded cost barely moves.
- **Adopt `v2-router`.** Keep the five every-run guardrails inline and disclose the rest behind an 11-row pointer table. Every pointer resolves; the Structure block stays inline.
- **Adopt `v3-radical`.** Shrink `AGENTS.md` to 26 lines and move the Structure block to `STRUCTURE.md`, retargeting `generate_structure_index.py` and `precommit_regenerate.py`.
- **Adopt a hybrid.** Take `v2-router` as the base, apply `v1-prune`'s pruning discipline to the disclosed docs, and land the `STRUCTURE.md` relocation from `v3-radical` as a separate follow-up change.

The `STRUCTURE.md` relocation is the only change that materially cuts always-loaded context. `v1` and `v2` fix prose quality, not sprawl: 300 of the 430 lines are the generated block.

## Context

- **Source.** Session `01a1133f-1bca-7318-a1a3-5b28d267ac44`, request "propose a series of restructurings" of `AGENTS.md`. Three generalist subagents produced the variants.
- **Assets.** `assets/agents-md-restructure/` holds each variant's full file set, its `git diff HEAD` patch, and the `REPORT.md` the session wrote. `assets/agents-md-restructure/comparison.md` holds the side-by-side table and the acceptance findings.
- **Worktrees.** `tmp/agents-restructure/{v1,v2,v3}/wt`, detached at `5fa46615`, uncommitted. This branch carries no edit to `AGENTS.md` or `CODING_STANDARDS.md`.
- **Evidence.** HEAD `AGENTS.md` is 430 lines; the generated block is lines 6–308. Per-variant counts come from `wc -l` in the worktrees.
- **Acceptance findings.** `v3-radical` drops the only pointers to `docs/agents/issue-tracker.md`, `docs/agents/triage-labels.md` and `docs/agents/domain.md`; `v2-router` keeps all three. All five every-run guardrails stay inline in all three variants. `scripts/validate_docs.py` passes for all three. The Structure block is byte-identical in `v1` and `v2`, and moves intact in `v3`.
- **`v3-radical` follow-up if adopted.** Six consumers need edits: the `commit-and-push` skill's four-file list, `chronicler-comment-fixer`, `test-police`, `ENVIRONMENT.md:3`, `.cargo/config.toml:4`, `docs/diataxis/explanation/diataxis.md:67`. The first commit must `git add STRUCTURE.md`, or the pre-commit hook aborts.
- **`v3-radical` validation gap.** `pytest` is absent from this environment, so the generator tests ran under `python -m unittest discover scripts/tests`: 285 tests, OK. Not independently re-run.
- **Overlap.** [Run the full build gate once per change set](03-run-the-full-gate-once.md) option A rewrites the `AGENTS.md` build-loop guidance, and [Decide whether `read` with `limit=0` needs a note](08-read-limit-zero.md) option A adds a note to `AGENTS.md`. Both edit material this ticket re-homes. The order is undecided: restructure first, and those fixes land in the new homes; decide those first, and the restructure carries their wording.
- **Not decided by this ticket.** Whether the `STRUCTURE.md` relocation earns its tooling cost on its own.
