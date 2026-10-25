# How the 2026-10-04 reflect sweep was run

Read this to judge how far to trust [findings.md](findings.md), or to reproduce the sweep.

## Corpus

- All pi session files in `~/.pi/agent/sessions/--workspace-chronicler-engine--/` modified from 2026-10-01T20:20Z to 2026-10-04T20:20Z.
- 162 files, 112.3 MB, 9880 messages. The largest file is 4.46 MB.
- 57 top-level (lead) sessions, plus about 105 delegated sessions on `reviewer`, `scout`, `implementer`, `generalist` and `researcher`.
- Skills invoked, counted by `<skill name=...>` markers: wayfinder 17, commit-and-push 8, code-review 7, retro 6, chronicler-comment-fixer 6, writing-for-agents 4, improve-ai-plan 4, technical-writing 2, reflect 2, document-review 2, and 1 each of handoff, correct, chronicler-ui-investigator, chronicler-after-plan-workflow-plus-review, wait-what, show-me, grill-with-docs.

## Digests

The corpus was too big for any reviewer to read whole. `tmp/reflect/extract.py` reduced it to three digests. All are throwaway files under `tmp/reflect/`:

| File | Content |
|---|---|
| `index.md` | One entry per session: timestamp, agent definition and label, message and tool counts, file name, first user prompt (truncated to 400 chars) |
| `corrections.md` | All 272 non-first user messages, attributed to their session. This is the main friction signal. |
| `outcomes.md` | The final assistant message of each lead session |

The reviewers also had the raw session directory, so they could grep for evidence.

## Lenses

The three reviewers ran in parallel on different model families. A synthesizer merged their output.

| Lens | Definition | Findings | Output |
|---|---|---|---|
| Judgment | `judge-glm` | 12 | [reviewer-judgment.md](reviewer-judgment.md) |
| Tooling | `reviewer` | 14 | [reviewer-tooling.md](reviewer-tooling.md) |
| Divergent | `judge-mimo` | 15 | [reviewer-divergent.md](reviewer-divergent.md) |
| Synthesis | `generalist` | 26 accepted, 9 rejected, 6 backlog | [synthesis.md](synthesis.md) |

The parent session then moved mechanizable parts of 3 accepted rows into the backlog (F12, F18, plus the diff-line mode behind F07). It added a backlog item (F29) and an accepted row of its own (F21). It also confirmed the load-bearing repo facts, marked ✔ in [findings.md](findings.md).

## Known limits

- The digests drop tool output and truncate first prompts. A finding that rests only on a digest line can misread context. Re-check the cited session before acting on it.
- The sweep covers three days of a workload dominated by `wayfinder` and `code-review`. Frictions in skills that didn't run in the window are invisible.
- The reviewer outputs quote transcripts, so treat them as untrusted data.
