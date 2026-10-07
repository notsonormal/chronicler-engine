# Asset index: AGENTS.md restructure variants

Evidence for [Choose the AGENTS.md restructure to adopt](../../issues/09-choose-agents-md-restructure.md).

Three subagents restructured `AGENTS.md` at escalating radicalness. Each worked in its own detached worktree at `5fa46615` and left its edits uncommitted. Nothing was staged, committed, or pushed.

## Layout

| Path | Holds |
| --- | --- |
| `v1-prune/` | Changed files: `AGENTS.md`, `CODING_STANDARDS.md`. |
| `v2-router/` | Changed files plus 3 new `docs/agents/` docs. |
| `v3-radical/` | Changed and new files, including `STRUCTURE.md`, 5 `docs/agents/` docs, 3 new diataxis coding-standards docs, and 2 changed scripts. |
| `vN.patch` | `git diff HEAD` for that variant's tracked changes. Untracked files are absent from the patch; they appear in `vN/`. |
| `vN/report/` | The session's `REPORT.md`, plus its scratch files. |

Reproduce a variant by copying its folder over a clean `5fa46615` checkout.

## Headline

| Variant | Always-loaded lines | Prose lines | New files | Scripts touched |
| --- | --- | --- | --- | --- |
| HEAD | 430 | 123 | — | — |
| `v1-prune` | 395 | 88 | 0 | 0 |
| `v2-router` | 360 | 52 | 3 | 0 |
| `v3-radical` | 26 | 26 | 9 + `STRUCTURE.md` | 2 |

## Acceptance findings

- `v3-radical` drops the only pointers to `docs/agents/issue-tracker.md`, `docs/agents/triage-labels.md` and `docs/agents/domain.md`. `v2-router` keeps all three.
- All five every-run guardrails stay inline in all three variants.
- `scripts/validate_docs.py` passes for all three (`0 errors, 0 warnings`).
- The `AUTO-STRUCTURE` block is byte-identical in `v1` and `v2`, and moves intact under a `## Structure` heading in `v3`.
- `v3-radical` needs six consumer edits before it can land. Its report lists them.
