# Weekly retro 2026-11-04: shared context for every audit agent

## What this is

The owner made many changes in the 7 days 2026-10-28 .. 2026-11-04. Each branch or ticket already had a per-change review (`.agents/skills/code-review/SKILL.md`: a Standards axis and a Spec axis on one diff, each report under 400 words). That review sees one ticket at a time. This weekly audit looks at what landed **as a whole**. Report what per-change review structurally cannot see:

- cross-ticket drift: one concept with two names, two mechanisms for one job, a rule applied in one ticket and not in the next
- emergent duplication across tickets (one copy per ticket, so no single diff showed it)
- leftovers: dead code, shims, flags, comments or docs that describe a state that a later ticket changed
- complexity that grew in steps: a file or function that each ticket made a little bigger
- tests that grew without adding confidence (copies, tautologies, tests that pin implementation details)
- work that does not serve the product goal (`PRODUCT.md`, `CONTEXT.md`)

## The window

- Fixed point: `3525193c` (last commit before 2026-10-28). `HEAD` is branch `long-term-memory` (= `main` plus 1 commit).
- `git log --oneline 3525193c..HEAD` lists the 79 commits. `git diff 3525193c HEAD -- <paths>` gives the net change of an area.
- The working tree has uncommitted changes of another session (`.pi/extensions/delegate/`, `work/wayfinder-maps/long-term-memory-and-state/`). They are out of scope. Do not touch them.
- Efforts this week (ticket maps under `.scratch/<effort>/map.md`, plans under `docs/plans/` and `old-docs/archived-plans/`): dashboard action-answers stages 2-5 and fix passes; removal of unused part routes; build.py refactor (structured nextest/cargo output, fail-fast); one chat-completions backend with OpenCode Go provider; STE structural prose rules in the docs gate; prompt-flexibility tickets 19-24 and its review-fixes plan (phases A-H); layout-trials tickets 09, 11, 12, 15; long-term-memory ticket 09; `scripts/trial.py`; pi extensions (delegate, rate-limit-wait).

## Data already gathered

- `tmp/retro-2026-11-04/data/duplicates.txt`: jscpd clone pairs in the files changed since `3525193c` (top 25 file pairs).
- `git log 3525193c..HEAD --name-only` for churn. Hot files (commits touching): `tests/test_utils/stub_server.rs` 12, `src/application/narration/narrator_tests.rs` 12, `tests/http/screen.rs` 11, `src/adapters/driving/http/settings/handlers/settings.rs` 9, `scripts/trial.py` 9, `src/domain/model/settings.rs` 8, `src/application/prompting/assembler.rs` 8, `src/application/pipeline/job.rs` 8, `src/adapters/driving/http/utils/answer_words.rs` 8.
- Size now: `src/` non-test 23.6k lines; `src/**/*_tests.rs` 22.4k; `tests/` 53.9k; `build.py` 1783; `scripts/trial.py` 1472 (+1274 test lines).

## Rules for you

- Read `CODING_STANDARDS.md`, `CONTEXT.md` (the glossary) and `tests/STRATEGY.md` first. Use the project terms.
- Review only. Do not edit, create or delete any file outside `tmp/retro-2026-11-04/findings/`. Do not run `build.py` or cargo (the build slot is shared with other sessions). `git`, `rg`, `wc`, and reading files are fine.
- Every finding carries evidence: `file:line` at `HEAD` (or a commit hash), and a one-line quote. A finding without evidence is a guess; label it so.
- Label epistemic status: known (you read it), inferred, or guessed.
- For each finding, say why per-change review missed it (for example: it spans tickets X and Y; it is a leftover of commit Z).
- For each finding, name the fix class: **structure** (make it unrepresentable: a type, a module boundary), **check** (a deterministic test, guardrail or lint), **rule** (a judgement call for `CODING_STANDARDS.md`), or **one-off fix**.
- Severity: high (wrong behaviour, a trap the next change will fall into, or a large ongoing cost), medium (real cost, contained), low (cleanup).
- Be selective. At most 12 findings, ranked by severity. Leave out style nits that a linter or the per-change review already catches.

## The bitter-lesson lens

Rich Sutton, *The Bitter Lesson* (2019): general methods that scale with computation (search, learning) beat methods that build in human knowledge, as computation grows. Hand-built knowledge gives a short-term gain, then plateaus and gets in the way.

Apply it as a concrete test, not as philosophy. For the code in your area, answer:

1. Which code encodes a workaround for a weakness of today's models (a parser for one answer shape, a text fix-up, a prompt-layout trick, a role merge)? Is each workaround isolated behind one seam, with a measurement (a trial, a test) that says when it can be deleted? Or is it spread so that a better model leaves it as dead weight that nobody can find?
2. Which code builds a general method that gets better with more compute or a better model (an evaluation harness, a trial runner, a search over options, a check that scales)? Is it given the investment it deserves?
3. Where does the code or the process spend human effort on hand-tuning that a measurement loop could do instead?

Put this in its own section, at most 5 points, each with evidence.

## Output

Write one Markdown file: `tmp/retro-2026-11-04/findings/<your-area>.md`. Structure:

1. `## Summary` — 3-5 sentences: the health of the area this week.
2. `## Findings` — ranked; each with severity, title, evidence, why review missed it, fix class, suggested fix.
3. `## Bitter lesson` — at most 5 points.
4. `## What went well` — at most 3 points, with evidence. Something to keep.

Under 1800 words. Your final message: the file path and a 5-line summary.
