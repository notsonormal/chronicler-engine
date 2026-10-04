# Map: agent workflow friction

Labels: wayfinder:map

## Destination

Every finding from the [2026-10-04 reflect sweep](assets/findings.md) (F01–F35) is dispositioned: built as a mechanism, adopted as your own habit, placed as a fact where it is read, or dropped with a written reason.

## Notes

- **Domain.** How agents work on this repo: skills, scripts, the `.scratch` tracker, worktrees, commits. This is not Chronicler Engine code.
- **Destination is disposition, not observed recurrence (Q1=A).** The map closes when the last ticket's decisions are carried out. Nobody waits for a later window to show the frictions stopped. The next `/reflect` run is where a repeat would show up.
- **This map carries execution (Q2=B).** Wayfinder's plan-only default is overridden. A ticket's grilling graduates its adopted changes into implementation tickets on this map. A decision that is one edit in one file may be applied in the grilling session itself.
- **Tickets are cut by lever, not by topic (Q7, replaces Q3/Q5's five topic clusters).** A grilling only pays when your answer changes something real. Skill-body advice is what the sweep shows failing to bind: F02's no-build rule was already written twice, and F05's proof step already exists but never fires. So each finding is grouped by what actually changes when it is decided:
  - 01: your own process, what you ask for, approve, or invoke
  - 02: mechanisms, which run whether or not a model reads anything
  - 03: single facts placed in text loaded at the moment of action, answered yes/no
  - Judgement advice in skill bodies has no ticket. It defaults to **drop** unless 01 adopts it as a habit or 02 builds it as a mechanism.
- **No execution ticket until a grilling decides what to implement (Q3).**
- **Sizing.** There is no ticket-sizing rule (F16 dropped in ticket 01). Each grilling cuts its graduated work case by case.
- **Evidence standard.** The sweep's proposals are hypotheses. The ✔ marks in the catalogue were checked on 2026-10-04, and the tree moves. Before deciding, re-check any fact the decision rests on.
- **Facts are dispatched, not asked.** When a grilling needs a repo fact, it sends a `scout`. It asks the user only for preference and judgement.
- **Concurrency.** Other efforts edit this checkout at the same time (e.g. `dashboard-ui-review` has uncommitted edits). Stage explicit paths, and never stage another effort's files.
- **Skills to consult.** `/grilling` and `/domain-modeling` for every ticket. `/writing-for-agents` for any skill or `AGENTS.md` edit. `/code-review` for graduated implementation tickets.

## Decisions so far

<!-- one line per resolved ticket: gist + link -->

- [Decide what you will do differently](issues/01-decide-your-process-changes.md) — no new habits; F16, F17, F18 and the six `[01]` default-drops are dropped; F25 becomes an automated-test gap ([Close the browser-test gap for stuck DOM states](issues/04-browser-test-gap-stuck-dom-states.md)).
- [Decide which mechanisms to build](issues/02-decide-mechanisms-to-build.md) — build four: [Prune worktrees safely](issues/05-safe-worktree-prune.md), [Give the pre-commit hook a non-destructive path](issues/06-pre-commit-hook-non-destructive-path.md), [Isolate the UI investigator's probe server](issues/07-isolated-probe-server.md), [Record contention and effect size in benchmark reports](issues/08-benchmark-contention-effect-size.md); F15 applied as `/core.[0-9]*`; F07, F08, F09, F10, F11, F20, F21, F23, F24, F30 dropped; F28 absorbed into the benchmark fields.

## Not yet specified

- **Fact placements from ticket 03.** Graduates from ticket 03. Sized case by case.

## Out of scope

- **A re-sweep to confirm the frictions stopped.** Ruled out by Q1=A. A later `/reflect` run is a fresh effort.
- **The synthesizer's 9 rejected findings.** Listed in [findings.md § Rejected](assets/findings.md#rejected-by-the-synthesizer). They come back only through a ticket that brings new evidence.
- **Rewording skill-body advice in the hope it binds.** Ruled out by Q7. Such findings are dropped unless they become a habit or a mechanism.
- **New skills for agent config or worktree lifecycle.** Rejected as `existing-skill-first`. Their mechanism parts were routed to F09, F10 and F13; F09 and F10 were later dropped in [Decide which mechanisms to build](issues/02-decide-mechanisms-to-build.md), and F13 became [Prune worktrees safely](issues/05-safe-worktree-prune.md).
- **Chronicler Engine code changes**, except where a finding's mechanism needs one, such as the pre-commit hook or `build.py` docstrings.

## Assets

- [`assets/findings.md`](assets/findings.md): the catalogue. Stable IDs F01–F35, owning ticket, problem, sweep proposal, source row, and repo facts checked on 2026-10-04. Includes the rejected list.
- [`assets/sweep-method.md`](assets/sweep-method.md): corpus, digests, lenses, and known limits. Read it to judge how far to trust a finding.
- [`assets/reviewer-judgment.md`](assets/reviewer-judgment.md), [`assets/reviewer-tooling.md`](assets/reviewer-tooling.md), [`assets/reviewer-divergent.md`](assets/reviewer-divergent.md): verbatim output of the three lenses.
- [`assets/synthesis.md`](assets/synthesis.md): verbatim synthesizer output.
- [`assets/mechanism-notes.md`](assets/mechanism-notes.md): the 8 misfiled tickets, kept as evidence only.
- `tmp/reflect/` (throwaway): `extract.py`, `index.md`, `corrections.md`, `outcomes.md`, `sessions.json`. These are the digests the lenses read.
