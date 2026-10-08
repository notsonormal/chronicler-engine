# Map: retro issue bucket

Labels: wayfinder:map

## Destination

Every retro finding about how agents work on this repo is dispositioned: decided by a grilling ticket, carried out as a fix, or dropped with a written reason. Findings arrive from `/retro` and `/reflect` sessions and become tickets; the bucket is kept empty of undispositioned findings.

## Notes

- **Domain.** How agents work on this repo: `AGENTS.md`, `CODING_STANDARDS.md`, `.agents/skills/`, `tests/infrastructure/guardrails/`, `scripts/`, `build.py`, the `.scratch` tracker, worktrees, and commits. Chronicler Engine product code is out of scope, except where a finding's fix is a guardrail or a script.
- **Every ticket is a grilling (HITL) ticket.** It decides a fix with the user. The agent never answers the user's side of the exchange.
- **Source.** A `/retro` or `/reflect` session. One finding, one ticket, with the source session and the `file:line` evidence in its Context.
- **This map carries execution.** Wayfinder's plan-only default is overridden. A grilling that adopts a fix applies a one-file edit in its own session; anything larger graduates into an implementation ticket on this map.
- **Evidence is re-checked, not trusted.** A finding is a hypothesis, and the tree moves. Re-check the cited `file:line` before deciding.
- **Concurrency.** Other efforts edit this checkout at the same time. Stage explicit paths; never stage another effort's files.
- **Skills.** `/grilling` and `/domain-modeling` for every ticket. `/writing-for-agents` for any skill or `AGENTS.md` edit. `/code-review` for an implementation ticket.
- **Numbering.** A new ticket takes the next free number. Numbers are never reused or reordered.

## Decisions so far

<!-- one line per resolved ticket: gist + link -->

- **03 — full gate once.** `python build.py --no-browser` is the iteration tier; the bare full gate runs once, at the end; `AGENTS.md` defines "green" as the full gate on the reported tree. [03](./issues/03-run-the-full-gate-once.md)
- **22 — gate provenance.** A machine stamp: the journal records the tree and per-tier test counts, and the epilogue prints each tier's count change. The commit-report citation stays open. [22](./issues/22-record-gate-provenance.md)
- **30 — implementation of 03 and 22.** `build.py`, its tests, `AGENTS.md`, `tests/AGENTS.md`, `ENVIRONMENT.md`. [30](./issues/30-gate-no-browser-and-verdict-record.md)

## Not yet specified

- **Findings not yet discovered.** Any improvement a later `/retro` run surfaces. It graduates into a ticket the moment its question is sharp.

## Out of scope

- **Chronicler Engine product defects.** They belong to the effort that owns the code, not to this bucket.
- **Re-running a retro to confirm a fix held.** A later `/retro` is a fresh run, not a ticket here.

## Open tickets

All are grilling (HITL) and unblocked. Ticket 01 predates the 2026-10-07 reflect run; 10–29 came from it. Source evidence for 10–29: [`assets/reflect-2026-10-07/`](./assets/reflect-2026-10-07/README.md).

- [01 — Give ticket resolution an owner after the commit](./issues/01-ticket-resolution-owner-after-commit.md)
- [02 — Take the quarantine count out of parallel-ticket conflicts](./issues/02-quarantine-pin-parallel-conflicts.md)
- [04 — Stop the hand-copied stub fixtures from drifting](./issues/04-stub-fixture-drift.md)
- [05 — Decide how a test reads a response body](./issues/05-response-body-reads-in-tests.md)
- [06 — Decide how the test suite asserts HTML escaping](./issues/06-html-escaping-assertions.md)
- [07 — Decide how a settings test slices a rendered panel](./issues/07-settings-test-panel-slicing.md)
- [08 — Decide whether `read` with `limit=0` needs a note](./issues/08-read-limit-zero.md)
- [09 — Choose the AGENTS.md restructure to adopt](./issues/09-choose-agents-md-restructure.md)
- [10 — Make every grilling question name its lever and restate its fact](./issues/10-grilling-question-lever-and-fact.md)
- [11 — Check a delegated brief against the role's tool grants before dispatch](./issues/11-delegated-brief-tool-grants.md)
- [12 — Scope the comment pass to the lines the diff adds](./issues/12-comment-pass-scope-to-the-diff.md)
- [13 — Stop `commit-and-push` from staging the whole shared tree](./issues/13-commit-staging-shared-tree.md)
- [14 — Catch hand-typed derived numbers in prose](./issues/14-derived-numbers-in-prose.md)
- [15 — Widen the stale-reference sweep to `.agents/skills/`](./issues/15-stale-references-in-skills.md)
- [16 — Validate a skill edit from a clean context](./issues/16-validate-skill-edit-from-clean-context.md)
- [17 — Put coordinator conventions in the map Notes before fan-out](./issues/17-map-notes-coordinator-conventions.md)
- [18 — Give the Standards review the clone report](./issues/18-standards-review-clone-report.md)
- [19 — Add a GitHub-side push-failure path](./issues/19-github-side-push-failure.md)
- [20 — Read the implementing session before writing a ticket's answer](./issues/20-read-implementing-session-for-answer.md)
- [21 — Never tell a review axis to skip `tooling.patch`](./issues/21-review-bundle-tooling-patch.md)
- [23 — Make the mutation check unconditional for state-repair tests and fixtures](./issues/23-unconditional-mutation-check.md)
- [24 — Record adjacent gaps in a research ticket's answer](./issues/24-research-answer-adjacent-gaps.md)
- [25 — Make the `tdd` skill fire on test-strengthening work](./issues/25-tdd-skill-triggers.md)
- [26 — Allocate the probe server a free port under concurrency](./issues/26-probe-server-port-allocation.md)
- [27 — Give every review finding an evidence class](./issues/27-review-finding-evidence-class.md)
- [28 — Add a usage-evidence phase to `chronicler-docs-hygiene`](./issues/28-docs-hygiene-usage-evidence.md)
- [29 — Cover Python in the comment finder's full sweep](./issues/29-comment-finder-python-coverage.md)
