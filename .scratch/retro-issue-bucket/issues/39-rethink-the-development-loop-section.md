# Rethink the Development Loop section of AGENTS.md

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

The `## Development Loop` section of `AGENTS.md` (lines 348–394) gives detailed build instructions, but agents still run the full gate many more times than needed. Does the detail help current models, or does it push them toward repeated full gates? What should the section say, and how short can it be?

- **Shorten to a few rules.** For example: iterate with `test-pattern`; run the full gate once, after the last change to code, tests, assets or docs; a commit, a `.scratch/` edit or a regenerated index does not need a new run. Move the rest to `ENVIRONMENT.md` or `python build.py --help`.
- **Keep the detail, fix the wording.** Remove the contradictions and name the trigger for a new gate run.
- **Leave the prose and enforce in `build.py`.** Rely on [Skip the browser tier when its inputs have not changed since a green run](31-skip-browser-tier-on-unchanged-inputs.md), widened to every gate tier, so a repeated gate on the same inputs costs nothing.

## Context

- Source: `/retro` 2026-10-11 on the dashboard UI review phase 2 run (Manager session `01a12b7d`; Lead and implementer sessions under `~/.pi/agent/sessions/--home-node-.herdr-worktrees-chronicler-engine-phase2-*`).
- **Evidence: full gates.** 5 tickets ran 15 bare `python build.py` runs (45 min). Ticket 12's implementer ran 6, ticket 09's ran 4, ticket 10's ran 3 within 11 minutes. Its own notes show why: ticket 12 at 16:43 "run gate L1 on this content (staged everything…)", then at 16:47 "Now the final gate (L2) on this clean tree" with no code change between them; ticket 10 ran again after its commit.
- **Evidence: iteration worked.** The same sessions ran `test-pattern` 91 times (41 min) and `--no-browser` once. The `browser` runs (9, 17 min) were mostly in tickets 12 and 13, which wrote browser tests. So the iteration advice of [Run the full build gate once per change set](03-run-the-full-gate-once.md) is followed; the waste is in the full gate.
- **What the section says today:**
  - `AGENTS.md:352` re-read before every edit; `:354` dev server DB and data dir; `:356` "A standard build takes about 2 minutes"; `:358` "timeout of 1200 seconds" and how to tail the log; `:367` `mrn-context` log attribution.
  - `:369–381` 11 iteration commands, including `unit`, which ticket 03 found adds nothing over `integration`.
  - `:385` "Final Validation (once, at the end)" lists both the full gate and `--no-browser`; `:392–393` "Green means the full gate ran on the tree you report" then names `--no-browser` as an iteration step. The rule names no trigger for a new run, so a commit or a regenerated index reads as a new tree (inferred).
  - The full gate appears 4 times in the section.
- **Briefs amplified it.** The Manager's `tmp/phase2-leads-brief.md:18` and `tmp/phase2-after-plan-brief.md:16` said "must be green on your final tree" / "on the final tree".
- **Measurement.** `logs/build_history.txt` records every gate run; ticket 03 used it to measure the old wording. Use it before and after the change.
- **Decided before.** Ticket 03 rejected making bare `python build.py` the fast tier: "it drops browser coverage silently". The user confirmed that concern on 2026-10-11.
- **Related.** [Choose the AGENTS.md restructure to adopt](09-choose-agents-md-restructure.md) rewrites `AGENTS.md`; each of its variants rewrites or moves this section. Decide this ticket first, so the chosen variant carries the agreed Development Loop text.

## Answer

Decided 2026-10-11 with the user. Implemented in [48 — Trim the build commands and make the Development Loop section information only](48-trim-build-commands-and-development-loop.md).

- **Information, not orders.** The section gives the facts that a model needs to choose a command: what each command covers and its warm cost. It has no "Iteration" or "Final Validation (once, at the end)" headings, and no "use these", "run once" or "at the end" phrases.
- **No definition of "done".** Rejected: a fact line "Done evidence is a green full gate on the content you report." The user's reason: "done" depends on the brief, the plan or the ticket. Agents rerun the gate because each one has its own idea of "done" from its context, and some are done without any `build.py` run. A repo-wide definition would be one more competing opinion. The work assignment sets the "done" bar.
- **Commands.**
  - `fmt`: not in `AGENTS.md`. The full gate runs it first. The subcommand and `--no-fmt` stay, and `--help` shows them.
  - `check`: not in `AGENTS.md`. `clippy` covers it (`cargo clippy --all-targets --all-features -- -D warnings` against `cargo check --all-targets --all-features`). The subcommand stays: it is about 5 s faster (median 7.0 s against 12.3 s), and it passes while warnings are still present.
  - `unit`: delete the subcommand. `integration` already runs the lib unit tests, and `unit` is not faster (median 22.1 s against 21.4 s).
  - `architecture` + `guardrails`: one `guardrails` subcommand runs both binaries. It is one gate step and one journal tier, with the key `guardrails`. Delete `architecture`. The architecture binary holds one test (the last full gate: `architecture=1 guardrails=162`). The first run after the change shows "+1" against the old `guardrails` baseline. That is a true count change.
- **Other lines.**
  - Re-read rule (`AGENTS.md:352`): remove it.
  - `tmp` line (`:350`): keep it, as a fact: "`tmp/` is the git-ignored scratch folder."
  - Dev-server DB and data paragraph (`:354`): move it to `ENVIRONMENT.md`. The `run` row already points there.
  - Build mechanics: keep the 1200 s bash timeout and the `tail` of the newest `logs/build_*.log`. A piped `tail` of stdout loses the result when the run times out. Remove the "about 2 minutes" timing (`ENVIRONMENT.md` has a cost table), the `mrn-context` line, and the "Almost every full-gate step is also a subcommand" paragraph (`--help` says it).
  - `## Concurrent Builds`: move the `ENVIRONMENT.md` pointer into the build section. Remove the sccache paragraph (`ENVIRONMENT.md:29–31` covers it) and the heading.
- **Not adopted.** Ticket 31 (skip a tier whose inputs did not change) stays a separate open ticket. It does not replace this answer.
- **Measurement.** No measurement step. The journal cannot count gates per ticket: it has no session or ticket column, `_tree_identity` changes on a commit because it includes `HEAD`, and worktree journals disappear with the worktree. The next normal `/retro` on a multi-ticket run counts bare `python build.py` calls per session from the transcripts, as this retro did. Baseline: 15 bare full gates in 5 tickets (3 per ticket), from the phase 2 run.
- **Ticket 09.** [Choose the AGENTS.md restructure to adopt](09-choose-agents-md-restructure.md) already carries a note to apply this answer to the chosen variant.
