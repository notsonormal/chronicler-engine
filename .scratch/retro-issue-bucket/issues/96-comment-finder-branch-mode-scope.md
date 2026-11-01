# The comment finder's branch mode lists every comment of a changed file

Type: grilling
Status: open
Blocked by: —

## Question

`comment_finder.py --branch <base>` returns every comment in each file that the branch changed, not only the comments on changed lines. The `/chronicler-comment-fixer` pass then judges and deletes comments that the change never touched. Must the finder limit its branch and uncommitted modes to the changed lines, and must `/simple-after-plan` use that mode?

## Context

Source: the coordinator session of 2026-11-01 that ran `/simple-after-plan` on prompt-flexibility tickets 20 and 21. Comment-fixer session `663d0ccc` (implementer, deepseek-v4.1-flash).

- `get_branch_files` (`.agents/skills/chronicler-comment-fixer/scripts/comment_finder.py`) uses `git diff --name-only`, so the scope is whole files.
- The finder listed 591 comment lines. The implementer changed 235 lines in 21 files. A `git blame` check showed that about 190 of them were on lines that the two tickets did not write.
- Some deletions removed real rules: the blank-field rules of the settings form, the boot rule of `seed_settings` (a malformed file fails the boot), and the invariant of `Job::generate` (only ids, texts and step results move between steps).
- The coordinator kept only the hunks on lines that the tickets added (45 lines in 9 files). The full patch is `tmp/pf2021/comment-pass-full.patch`. The split took a blame script and one manual review.
- One rewrite repeated a false claim ("it holds still between two calls of one kind") that the doc review had just removed from the docs.

## Options to decide

- **A.** A `--changed-lines` filter: the finder reads `git diff -U0` and keeps only comments inside the added hunks. `/simple-after-plan` step 1 names this mode.
- **B.** Keep the whole-file scope, and add a rule to the skill: a comment on a line outside the diff is reported, not edited.
- **C.** No change. The coordinator filters the result by hand.
