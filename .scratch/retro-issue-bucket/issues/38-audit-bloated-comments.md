# Audit the bloated comments in the dense files

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Some files carry long "why" comment blocks that read as design notes. The comment-fixer keeps them, because each block explains a reason. Should an audit cut the existing bloated comments in the densest files, and what is the rule for a "why" comment that is too long (move it to a doc, to the ticket Answer, or shorten it)?

## Context

- Source: `/retro` 2026-10-11 on the dashboard UI review phase 2 run (Manager session `01a12b7d`).
- The user's hypothesis: files that already have too many comments teach agents to write more, and the comment-fixer matches the density it sees. Measured in the phase-2 run (comment lines / all lines, `c739932f` → `phase2-after-plan`):

  | File | Before | After | New lines that are comments |
  |---|---|---|---|
  | `assets/dashboard-panels.js` | 11% | 19% | 26 of 47 |
  | `src/application/pipeline/action_pipeline/retry.rs` | 7% | 13% | 35 of 123 |
  | `assets/dashboard-errors.js` | 7% | 7% | — |
  | `tests/test_utils/stub_server.rs` | 5% | 5% | — |

  So the two files with the most bloat were not dense before the run; the run's implementers wrote the blocks (inferred: review rounds asked them to justify timing decisions, and the reason went into a comment, not the Answer). The audit should test the hypothesis on the files that are dense today, not assume it.
- The comment-fixer pass on the same diff classified 606 comment lines and kept 473 (session `01a12c59-8ad9`, report in its last message).
- `CODING_STANDARDS.md` "Code comments": "Comments explain the WHY only when non-obvious". The rule has no length limit and no place named for a long reason.
- Candidates: a density ranking (`rg -c` comment lines per file under `src/`, `tests/`, `assets/`) to choose the audit set; a length rule for "why" comments; where long reasons go (reference doc, ticket Answer, commit message).
- Related: [Cover JavaScript in the comment finder](37-comment-finder-javascript-coverage.md) (the JS files must be in the finder's scope first), [Scope the comment pass to the lines the diff adds](12-comment-pass-scope-to-the-diff.md).

## Comments

- 2026-10-21: ticket [37](37-comment-finder-javascript-coverage.md) is done; `comment_finder.py` now lists JS (all forms) and Python under `--all`. The deferred re-run of the comment pass on `assets/dashboard-panels.js` and `assets/dashboard-errors.js` belongs here, after the length rule.
