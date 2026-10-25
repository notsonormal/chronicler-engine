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
- **39 — Development Loop is information only.** A table of commands with coverage and cost; no "done" definition, because the work assignment sets it. Delete `unit`; `guardrails` takes in `architecture`; `fmt` and `check` leave `AGENTS.md`. [39](./issues/39-rethink-the-development-loop-section.md)
- **48 — implementation of 39.** `unit` and `architecture` subcommands gone (`guardrails` runs both binaries); `AGENTS.md` Development Loop is one command table; DB/data paragraph in `ENVIRONMENT.md`. Two stale pointers left: `.cargo/config.toml` (seed signature) and the permissions `reason` (user edit). [48](./issues/48-trim-build-commands-and-development-loop.md)
- **50 — dead-path guardrails.** Option C with A, built in core step C4 and merged to `main`. Residual: exemption paths are not checked. [50](./issues/50-dead-path-guardrails.md)
- **37 and 29 — comment finder covers JavaScript and Python.** Every mode reads `.js` (all comment forms, `*.min.js` skipped); `--all` adds root and `scripts/` Python; the JS re-run waits for 38. [37](./issues/37-comment-finder-javascript-coverage.md), [29](./issues/29-comment-finder-python-coverage.md)
- **80 and 72 — slice mode for `/simple-after-plan`.** Five-step section: implementer cleans comments and runs `--no-browser`; one combined reviewer; fixes to the resumed implementer; full gate once, then commit; retro once per effort. `/code-review` unchanged. [80](./issues/80-simple-after-plan-slice-mode.md), [72](./issues/72-code-review-single-session-mode.md)
- **Triage pass 2 drops (2026-10-21).** Closed with a written reason in each ticket; see [`triage.md`](./triage.md) pass 2.
  - Stale: [40](./issues/40-lead-file-tools.md) (delegation is now `.pi/extensions/delegate/`), [41](./issues/41-delegated-report-truncation.md) (the cut names `result.json`), [43](./issues/43-reviewer-shell-premise.md) (every role grants `bash`).
  - Weak: [31](./issues/31-skip-browser-tier-on-unchanged-inputs.md) (`--no-browser` covers it; the skip would have saved no run).
  - Folded: [33](./issues/33-check-ui-names-in-ticket-copy.md) and [56](./issues/56-ticket-edit-lists-from-the-tree.md) into 75; [36](./issues/36-next-free-scenario-number.md) into 64; [53](./issues/53-focused-checks-miss-a-binary.md) into 80; [55](./issues/55-tier1-oracle-production-writes.md) into 23.

## Not yet specified

- **Findings not yet discovered.** Any improvement a later `/retro` run surfaces. It graduates into a ticket the moment its question is sharp.

## Out of scope

- **Chronicler Engine product defects.** They belong to the effort that owns the code, not to this bucket.
- **Re-running a retro to confirm a fix held.** A later `/retro` is a fresh run, not a ticket here.

## Open tickets

All are grilling (HITL) and unblocked. Each ticket names its source session in its Context. Source evidence for 10–29: [`assets/reflect-2026-10-07/`](./assets/reflect-2026-10-07/README.md). Verdicts and clusters for every open ticket: [`triage.md`](./triage.md).

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
- [32 — Let the review bundle take a path list](./issues/32-review-bundle-path-scope.md)
- [34 — Sweep every caller when a change alters what a port returns](./issues/34-sweep-callers-of-a-changed-seam.md)
- [35 — Lint a spec scenario's shape and vocabulary](./issues/35-lint-spec-scenario-shape.md)
- [38 — Audit the bloated comments in the dense files](./issues/38-audit-bloated-comments.md)
- [42 — Check the tickets a doc review is about to make false](./issues/42-doc-review-ticket-claims.md)
- [44 — Decide whether the comment pass renames or gates instead of delete-and-flag](./issues/44-comment-pass-rename-or-gate.md)
- [45 — Reconcile a ticket's Done-when with its Answer at resolution](./issues/45-done-when-versus-answer.md)
- [46 — Never issue a commit in the same step as the edit it commits](./issues/46-commit-in-its-own-step.md)
- [47 — Read the next free bucket number across every checkout](./issues/47-next-free-bucket-number.md)
- [49 — Stop the comment pass from deleting code inside string literals](./issues/49-comment-pass-code-inside-string-literals.md)
- [51 — Give each branch of a design rule a test before the review passes](./issues/51-design-rule-branch-without-a-test.md)
- [52 — Make the comment pass list every deleted reason comment](./issues/52-comment-pass-deletes-real-reasons.md)
- [54 — State that a scenario's Given must be a state a client can reach](./issues/54-spec-given-a-client-can-reach.md)
- [57 — Check the docs again after the code-changing steps of the after-plan workflow](./issues/57-after-plan-docs-after-code-fixes.md)
- [58 — Say how to find the fixed point of a review for a multi-branch wayfinder effort](./issues/58-after-plan-fixed-point-of-a-map.md)
- [59 — Let the review bundle diff a ref against the working tree](./issues/59-review-bundle-ref-plus-uncommitted.md)
- [60 — Give the duplicate step a row in the Step Timing Summary](./issues/60-timing-summary-lists-every-step.md)
- [61 — Say how an after-plan workflow closes out a wayfinder execution map](./issues/61-wayfinder-close-out-of-an-execution-map.md)
- [62 — Reword the build timeout instruction so it does not read as the `timeout` command](./issues/62-agents-md-timeout-wording.md)
- [63 — Deny `rg -r` and `rg -rn` with a reason in the permission config](./issues/63-deny-rg-replace-flag.md)
- [64 — Give agents one way to print a spec scenario by number](./issues/64-show-scenario-helper.md)
- [65 — Check that every backticked identifier in a Diátaxis doc exists in the tree](./issues/65-docs-identifier-check.md)
- [66 — Make the docs-hygiene skill run its validator under a no-build brief](./issues/66-docs-hygiene-runs-the-validator.md)
- [67 — Make a red gate name the failing test and its message](./issues/67-gate-failure-names-the-test.md)
- [68 — Put formatting into the narrow dev loop](./issues/68-narrow-loop-runs-fmt.md)
- [69 — Give implementers a safe way to rewrite many call sites](./issues/69-codemod-for-multi-site-rewrites.md)
- [70 — Give agents one call that waits for a running build](./issues/70-build-wait-helper.md)
- [71 — Scope the coverage step and the target dir of `/test-police`](./issues/71-test-police-coverage-and-target-dir.md)
- [73 — Show how to slice one file out of a bundle patch](./issues/73-bundle-readme-patch-slice.md)
- [74 — Make `build_slot.py --status` give one verdict](./issues/74-build-slot-status-stale-holder.md)
- [75 — Check a delegate brief for unresolved references and stale premises](./issues/75-lead-delegate-brief-quality.md)
- [76 — Name the tier-1 wait helpers in `tests/AGENTS.md`](./issues/76-tests-agents-names-wait-helpers.md)
- [77 — Plan text asks for free functions that the free-fn guardrail refuses](./issues/77-plan-text-versus-free-fn-guardrail.md)
- [78 — A stale `guardrails.md` fails `build.py` before the hook can regenerate it](./issues/78-stale-guardrails-doc-fails-the-gate.md)
- [79 — `ScriptedModel` does not fail on a scripted reply that no call uses](./issues/79-scripted-model-unused-replies.md)
- [81 — The plan review checks each test row and Done-when against its Content item](./issues/81-plan-review-checks-rows-against-content.md)
- [82 — Lint the mechanical part of the recurring `docs/diataxis/` review findings](./issues/82-lint-recurring-diataxis-doc-findings.md)
- [83 — Give the mutation check a script that restores by hash and runs on the final tree](./issues/83-mutation-check-script.md)
- [84 — The strong-implementer re-ran narrow builds and the gate many times in one task](./issues/84-implementer-repeats-builds.md)
- [85 — The comment finder flags comments that narrate a change](./issues/85-comment-finder-flags-change-narration.md)
