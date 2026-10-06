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
- [Close the browser-test gap for stuck DOM states](issues/04-browser-test-gap-stuck-dom-states.md) — A19's three stuck-state bugs (plus two adjacent) all have browser tests now, closed via dashboard-ui-review 27/42/43/65; one same-class gap left — a stale `Generating` with no live slot — graduated to [Recover a dashboard stuck on Generating with no live generation](issues/09-recover-stuck-generating.md).
- [Yes/no on placing nine facts where they are read](issues/03-yes-no-fact-placement.md) — two yeses applied: an `AGENTS.md` pointer to `CODING_STANDARDS.md` (F01) and the real `--cleanup` scope in `build.py` (F14); F02, F03, F04, F05, F06, F27 and F35 dropped.
- [Prune worktrees safely](issues/05-safe-worktree-prune.md) — built `scripts/remove_worktrees.py`, run with `--apply` by every full gate (`build.py remove-worktrees`, best-effort); removes only clean, pushed trees at least 24h old with no live process inside, using plain `git worktree remove`.
- [Give the pre-commit hook a non-destructive path](issues/06-pre-commit-hook-non-destructive-path.md) — built `scripts/precommit_regenerate.py`: the hook compares prose outside each file's pinned generated-block markers against the index; foreign prose aborts untouched, in-block leftovers regenerate and stage, and any abort after generation restores the files.
- [Isolate the UI investigator's probe server](issues/07-isolated-probe-server.md) — the skill now starts its own server (`python build.py run --target-dir tmp/probe-server -- --world redmist_estate --port 3001`, default port 3001, outside the 3010–3050 test band) and states at the Serve step that a probe must never be pointed at the live dashboard; the probe's DB is `<exe dir>/chronicler_<port>.db`, so the live game's DB is never opened.
- [Record contention and effect size in benchmark reports](issues/08-benchmark-contention-effect-size.md) — `diagnostic_benchmark.py` records the contention condition (load average, build slot held), run-to-run spread via `--repeat`, and effect size vs a baseline via `--compare`, with a stated 20% noise threshold labelling sub-threshold deltas `within noise`; `REPORT_DIR` fixed to `<repo>/tmp/diagnostics`. The report's Rust data source (`cargo test --test diagnostic_benchmark`) no longer exists, graduated to [Decide the fate of the dead diagnostic benchmark](issues/10-dead-diagnostic-benchmark.md).
- [Decide the fate of the dead diagnostic benchmark](issues/10-dead-diagnostic-benchmark.md) — deleted: the scored report was hand-authored, not measured (the suite's real content was twelve signal assertions), so a rebuild buys the shape without the measurement and a park keeps the trap; removed the script, its untracked tests, the `--diagnostic-benchmark` flag and `run_diagnostic()`, and regenerated the `AGENTS.md` structure index. F28/F29 re-dispositioned to dropped — their report fields had no producer that varied, so they never fired.
- [Recover a dashboard stuck on Generating with no live generation](issues/09-recover-stuck-generating.md) — the status poll answers from the live registry (`idle` when no slot is generating) instead of the persisted phase, so the GET stays read-only and the page unblocks within one poll; `heal_stale` extends to `/options`, retrigger, retry and game switch; the submit-button guard and silent recovery stay; `two-state-channels.md`'s "the atomic" and "next `process_action`" wording is repaired. Per `tests/STRATEGY.md`'s placement rule the proof is tier-1 for the poll's answer and tier-2 with a real Enter for the client. Graduated to [Recover a stale Generating from the live registry](issues/11-recover-stale-generating.md) and [Decide the fate of the invariant test the recovery doc claims](issues/12-doc-claimed-invariant-test.md).
- [Decide the fate of the invariant test the recovery doc claims](issues/12-doc-claimed-invariant-test.md) — removed: the claimed biconditional fails in both directions (a live slot can sit beside a persisted `Idle` when a generation finishes; a persisted `Generating` can sit beside no slot after a panic), and `try_claim` writes the status only after releasing the registry lock, so no test was added; the doc's "The single-writer rule" became "Who writes each channel", and the target contract is deferred to architecture-deepening [Give the input buffer named generation transitions](../architecture-deepening/issues/09-input-buffer-generation-transitions.md).
- [Recover a stale Generating from the live registry](issues/11-recover-stale-generating.md) — the poll answers from the live registry (`is_busy`): the persisted phase while a slot is live, `idle` otherwise, no write on the GET; the stale-status heal is centralized in `claim_and_spawn` (every action entry point) and the reset is persisted only on change, with game switch healing through `heal_stale_status`; the quarantined test became tagged scenario 39.1 of a new `docs/specs/dashboard.md` (pin 83 → 82), scenario 16.28 drives the poll and a real Enter, and `two-state-channels.md` is repaired. Arrival needs no heal — it never persists `Generating`.

## Not yet specified

_Nothing yet._

## Out of scope

- **A re-sweep to confirm the frictions stopped.** Ruled out by Q1=A. A later `/reflect` run is a fresh effort.
- **The synthesizer's 9 rejected findings.** Listed in [findings.md § Rejected](assets/findings.md#rejected-by-the-synthesizer). They come back only through a ticket that brings new evidence.
- **Rewording skill-body advice in the hope it binds.** Ruled out by Q7. Such findings are dropped unless they become a habit or a mechanism.
- **New skills for agent config or worktree lifecycle.** Rejected as `existing-skill-first`. Their mechanism parts were routed to F09, F10 and F13; F09 and F10 were later dropped in [Decide which mechanisms to build](issues/02-decide-mechanisms-to-build.md), and F13 became [Prune worktrees safely](issues/05-safe-worktree-prune.md).
- **Chronicler Engine code changes**, except where a finding's mechanism needs one, such as the pre-commit hook or `build.py` docstrings.
- **Rebuilding the diagnostic-signal-quality benchmark.** Its scores were hand-typed, so a rebuild recreates the shape without the measurement; the one version that would earn it — scores produced by an independent judge — is a separate, larger effort. Deleted in [Decide the fate of the dead diagnostic benchmark](issues/10-dead-diagnostic-benchmark.md).
- **The twelve signal-preservation assertions as engine tests.** A checklist that the user-facing error names its cause (LLM, Quantifier, Navigation, Narrative, Triggers, State), uncovered today; no sweep finding needs it, and it is recoverable from `tests/integration/diagnostic/scenarios.rs` at `d34c8785^`.
- **Giving F28's rule a live host in `build.py` step timings.** Noted in [Decide the fate of the dead diagnostic benchmark](issues/10-dead-diagnostic-benchmark.md) as a possible follow-up; a new mechanism build, so a fresh effort rather than graduation here.
- **The arrival narration's missing generation slot.** Arrival runs ungated at boot: it holds no registry slot, so `is_busy` is false, `try_claim` succeeds from a concurrent player action, and two generations then write one game state. It is not a stale-record defect (arrival never persists `Generating`), and it is engine code that no sweep finding needs. It belongs to architecture-deepening [Route arrival narration through the one narration-generation path](../architecture-deepening/issues/10-route-arrival-through-narration-generation.md).

## Assets

- [`assets/findings.md`](assets/findings.md): the catalogue. Stable IDs F01–F35, owning ticket, problem, sweep proposal, source row, and repo facts checked on 2026-10-04. Includes the rejected list.
- [`assets/sweep-method.md`](assets/sweep-method.md): corpus, digests, lenses, and known limits. Read it to judge how far to trust a finding.
- [`assets/reviewer-judgment.md`](assets/reviewer-judgment.md), [`assets/reviewer-tooling.md`](assets/reviewer-tooling.md), [`assets/reviewer-divergent.md`](assets/reviewer-divergent.md): verbatim output of the three lenses.
- [`assets/synthesis.md`](assets/synthesis.md): verbatim synthesizer output.
- [`assets/mechanism-notes.md`](assets/mechanism-notes.md): the 8 misfiled tickets, kept as evidence only.
- `tmp/reflect/` (throwaway): `extract.py`, `index.md`, `corrections.md`, `outcomes.md`, `sessions.json`. These are the digests the lenses read.
