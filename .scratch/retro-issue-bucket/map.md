# Map: retro issue bucket

Labels: wayfinder:map

## Destination

Every retro finding about how agents work on this repo is dispositioned: decided by a grilling ticket, carried out as a fix, or dropped with a written reason. Findings arrive from `/retro` and `/reflect` sessions and become tickets; the bucket is kept empty of undispositioned findings.

## Notes

- **Domain.** How agents work on this repo: `AGENTS.md`, `CODING_STANDARDS.md`, `.agents/skills/`, `tests/infrastructure/guardrails/`, `scripts/`, `build.py`, the `.scratch` tracker, worktrees, and commits. Chronicler Engine product code is out of scope, except where a finding's fix is a guardrail or a script.
- **Every ticket is a grilling (HITL) ticket.** It decides a fix with the user. The agent never answers the user's side of the exchange.
- **Source.** A `/retro` or `/reflect` session. One finding, one ticket, with the source session and the `file:line` evidence in its Context.
- **Check Won't do before filing.** A new finding that matches a line in [Won't do](#wont-do) is not filed. It is filed only with new evidence that defeats the recorded reason, and the ticket names that line.
- **A closed ticket is deleted.** Its gist goes to [Decisions so far](#decisions-so-far) (a fix landed) or to [Won't do](#wont-do) (with the reason). The full text stays in git history.
- **This map carries execution.** Wayfinder's plan-only default is overridden. A grilling that adopts a fix applies a one-file edit in its own session; anything larger graduates into an implementation ticket on this map.
- **Evidence is re-checked, not trusted.** A finding is a hypothesis, and the tree moves. Re-check the cited `file:line` before deciding.
- **Concurrency.** Other efforts edit this checkout at the same time. The user decides the scope of each commit.
- **Skills.** `/grilling` and `/domain-modeling` for every ticket. `/writing-for-agents` for any skill or `AGENTS.md` edit. `/code-review` for an implementation ticket.
- **Numbering.** A new ticket takes the next number after the highest number used. **Highest number used: 90.** Closed tickets are deleted, so read the number here, and update this line when you file a ticket. Numbers are never reused or reordered.
- **Audit.** [`audit.md`](./audit.md) (2026-10-25) sorts the open tickets by whether the fix changes what the next agent does.

## Decisions so far

<!-- one line per ticket whose fix landed: number and gist; the ticket file is deleted -->

- **03 — full gate once.** `python build.py --no-browser` is the iteration tier; the bare full gate runs once, at the end.
- **22 — gate provenance.** A machine stamp: the journal records the tree and per-tier test counts, and the epilogue prints each tier's count change.
- **30 — implementation of 03 and 22.** `build.py`, its tests, `AGENTS.md`, `tests/AGENTS.md`, `ENVIRONMENT.md`.
- **39 — Development Loop is information only.** A table of commands with coverage and cost; no "done" definition, because the work assignment sets it. Delete `unit`; `guardrails` takes in `architecture`; `fmt` and `check` leave `AGENTS.md`.
- **48 — implementation of 39.** `unit` and `architecture` subcommands gone (`guardrails` runs both binaries); `AGENTS.md` Development Loop is one command table; DB/data paragraph in `ENVIRONMENT.md`. Two stale pointers left: `.cargo/config.toml` (seed signature) and the permissions `reason` (user edit).
- **50 — dead-path guardrails.** Option C with A, built in core step C4 and merged to `main`. Residual: exemption paths are not checked.
- **37 and 29 — comment finder covers JavaScript and Python.** Every mode reads `.js` (all comment forms, `*.min.js` skipped); `--all` adds root and `scripts/` Python.
- **80 and 72 — slice mode for `/simple-after-plan`.** Five-step section: implementer cleans comments and runs `--no-browser`; one combined reviewer; fixes to the resumed implementer; full gate once, then commit; retro once per effort. `/code-review` unchanged.
- **12 — comment pass scope.** The user ruled (2026-10-11): the scope is every comment in each touched file, not only the added lines. `code-review` says that cleanup in touched files is expected, not scope creep. The string-literal part is open as 49.
- **62 and 70 — build timeout wording.** `AGENTS.md` Development Loop: run `build.py` in the foreground, with the `timeout` parameter of the bash tool set to 1200.
- **44 — comment pass applies an in-file rename or gate.** `chronicler-comment-fixer` KEEP test: a rename or a visibility or `cfg` gate in the judged file is made and reported as a rewrite; only a cross-file change or a design call raises a reshape flag.
- **52 — comment pass lists deleted reasons.** The `chronicler-comment-fixer` output format has a `DELETED REASONS` section: each deleted comment that states a why, in full, with the KEEP part it fails.
- **57 — docs step after the code steps.** In both after-plan skills, the docs step runs after the steps that change code and before the comment pass and the full build. A fix pass after the reviews is not covered.
- **83 — moved out (2026-10-25).** The mutation-check script question moved to the [`unclebob-quality-tools`](../unclebob-quality-tools/map.md) map; its evidence is in that map's ticket 03. The file is deleted.
- **25 — `tdd` trigger.** The description also fires when an agent adds, strengthens or de-tautologizes tests.
- **74 — `build_slot.py --status` gives one verdict.** A free slot prints only `Build slot: free.`; the stale-holder note is gone from the free branch. The held branch is unchanged.
- **76 — the wait helpers are in the test index.** The `http_requests.rs` module summary names `wait_idle`, `wait_settled` and `wait_for_jobs`, so `tests/AGENTS.md` lists them.
- **85 — comment finder flags change narration.** `comment_finder.py` ends a comment line that has a change word (`rename`, `renamed`, `became`, `stays`, `no longer`, `used to`, `was changed`) with `[change-word: …]`. Report-only; the fixer judges.
- **59 — review bundle of a ref plus the working tree.** `prepare_review_bundle.py --ref REF --worktree` diffs `REF` against the working tree, with untracked files.
- **18 and 73 — bundle README.** A row names `python scripts/healthcheck.py duplicates --ref <fixed point>` for the clone pairs in the changed files; "How to read" has an `awk` command that slices one file out of a patch; the "largest areas first" contradiction is gone.
- **65 (and 15) — route check.** `validate_docs.py` fails with `UNKNOWN_ROUTE` when `docs/**/*.md` (not `docs/plans/`) or `.agents/skills/*/SKILL.md` names a route that the generated `http_routes.md` does not list. Residual: a route whose first segment is in no listed route (such as `/check-text`) is not checked. Other backticked identifiers are not checked.
- **67 and 78 — a red gate names its cause.** The `build.py` epilogue prints a `--- Failures ---` block: each nextest `FAIL` test id with its first stderr or panic line (at most 20), and the last 1–3 output lines of a failed step that is not a nextest run.
- **60 — report-only steps in the timing summary.** The hook install, the worktree removal and `duplicates` have `(report-only)` rows with `[OK]` or `[WARN]`. Residual: the asset-copy and coverage-report steps still have no row.
- **68 — `clippy` checks formatting.** `python build.py clippy` runs `cargo fmt --check` first; on failure the Failures block lists the unformatted files and `fix: python build.py fmt`. The full gate and `--no-fmt` are unchanged.
- **49 — the probe marker is not a comment.** `tests/browser/stub/failure_display.rs` uses `__MEASURE_CLAMPED_TEXT__`, so the comment finder cannot list it.
- **`validate_docs.py --path` (found in the audit, 2026-10-25).** It compared a tuple with a list, so it rejected every file. Fixed, with a test that fails on the old code.
- **79 — `ScriptedModel` fails on an unused reply.** `Drop` also fails when a scripted reply was never used, and names each one. `.optional()` marks one reply as allowed to stay unused; only the stop-row script in `tests/http/races.rs` uses it. 23 tests lost a dead script line; no assertion changed.

## Won't do

<!-- one line per dropped finding: the symptom, so a new finding can be matched, and the reason -->

**User decisions (2026-10-25, see `audit.md`).**

- **09 — Restructure `AGENTS.md` or move the generated Structure block out of it.** The always-loaded file, with the block, has value now.
- **Shared-checkout collisions (D2).** `git add -A` or "commit all" sweeps in another effort's work (13); a review bundle holds other efforts' files (32); a commit in the same message as its edit sees the old tree (46); the next free ticket number differs between checkouts (47); parallel tickets conflict on a shared counter such as `REQUIRES_MIGRATION_TEST_COUNT` (02); parallel briefs each write their own conventions (17); the resolve step falls between the coordinator and the commit session (01). Accepted: many efforts share one checkout, and the user often tells an agent on purpose to commit every change.
- **Ticket text against docs and code (D3).** A doc review deletes a line that a resolved ticket's Answer says the doc carries (42); a Done-when and its Answer state different rules (45); a plan's test rows or Done-when lines contradict its Content items or `CONTEXT.md` (81). Authority order: a ticket is the acceptance target only for the change that implements it; after that it is a record, and the specs, the docs and the code are the truth. `document-review` already skips `.scratch/`, and the Spec axis already reads only the change's own tickets.
- **82 — Lint the recurring `docs/diataxis/` review findings** (negative explaining, code-indexing, edge cases, term reuse). Measured on 25 findings in 10 slices: a lint would have saved 0 of 10 fix rounds. The review stays the check.
- **16 — Validate a skill edit with a clean-context run.** Not required; the first real run of the skill is its test.

**"Remember to check X" rules.** A rule in a skill or a brief does not hold, and the review already caught each case.

- **24 — A research answer leaves out the adjacent gaps the recon found.**
- **34 — A change to a port's contract does not list every caller.** The spec reviewer listed them without being told.
- **51 — A branch of a design rule has no test.** The doc review caught it.
- **75 — A lead's delegate brief has an unresolved reference, a stale premise, a UI name that does not exist (33), or an edit list not built from the tree (56).** The delegate's report caught each case.
- **84 — An implementer re-runs `clippy`, `test-pattern` and the gate many times in one task.** The rule from 03 did not hold; refusing a repeat on an unchanged tree saves little.

**Model slips and one-offs.** A loud error or the next run corrects them; a rule costs more than the slip.

- **06 — An escaping assertion uses the wrong entity form (`&lt;` against Askama's `&#60;`).** The test fails loudly.
- **08 — `read` with `limit=0` returns no content.** Harness behaviour.
- **19 — A GitHub-side push failure is retried many times.** An external outage; the `commit-and-push` skill is deleted.
- **20 — The agent does not know to recover a ticket's answer with `session_search` / `session_read`.** Naming the tool once was enough.
- **21 — A brief tells a reviewer to skip `tooling.patch`.** One hand-written brief.
- **58 — The fixed point of a review for a multi-branch effort is unclear.** The user named it once.
- **63 — `rg -r` / `rg -rn` read as recursive** (in ripgrep `-r` is `--replace`). The owner may still add a deny rule in the permission config.
- **66 — A scout reads "do not run `build.py`" as a ban on `validate_docs.py`.**
- **69 — Regex heredocs for a rewrite of many call sites fail.** Model capability; no codemod dependency.
- **77 — A plan asks for a free fn that the free-fn guardrail refuses.** The guardrail message teaches the fix.

**Other.**

- **28 — A usage-evidence phase in `chronicler-docs-hygiene`.** The user asks for usage archaeology when needed; no skill is needed for that.
- **61 — A close-out operation for a wayfinder execution map.** No agent failed because of it.
- **31 — Skip the browser tier when its inputs are unchanged.** `--no-browser` covers it; the skip would have saved no run.

**Audit closes (2026-10-25, user).**

- **71 — `/test-police` builds coverage in its own cold target dir (2 × 7.4 GiB, 877 s).** Deliberate: coverage overrides `RUSTC_WRAPPER` and needs a dedicated `--target-dir` (`ENVIRONMENT.md`, Coverage). The wrong `cd` line is gone.
- **27 — Review findings carry no evidence class, so the lead applies all.** Step 12 of the after-plan skill already says to verify each finding.
- **54 — A spec scenario's Given is a state no client request can reach.** The review found all six cases.
- **10 — Grilling questions cite opaque ids or ask about things the user cannot control.** `grilling/SKILL.md` already says to restate the context and assume the user has not read the files.
- **38 — Long "why" comment blocks in dense files.** `check_long_comment_runs` (`tests/infrastructure/guardrails/style.rs`) fails on 5 or more comment lines in a row.
- **05 — A test reads a body with `to_bytes(…, <literal>)` and truncates.** The panic names the limit; the fix is local.
- **04 — Four stub fixtures (`games.html`, `prompt_presets.html`, `visual_sidebar.html`, `worlds.html`) are hand copies of template output and drift.** The drift happens only when one of those templates changes; it costs one hand edit, and the stub tests fail loudly.
- **26 — The probe server is pinned to port 3001 and one target dir.** It matters only when two probes run at once.

**Skipped (2026-10-25, user).**

- **64 — A script that prints a spec scenario by number (`--show`, `--next`; 36 was folded in).** Skipped by the user.

**Stale.** The premise no longer holds.

- **07** — a settings test slice matched a class prefix (the classes were renamed). **11** — `scout` has no shell (every role grants `bash`). **14** — a hand-typed route count in a doc (removed). **40** — the lead lost its file tools (delegation is now `.pi/extensions/delegate/`). **41** — a long delegated report was cut (the cut names `result.json`). **43** — `reviewer` has no shell.

**Folded into another ticket.** 15 into 65 (a removed route left in a skill); 70 into 62 (run builds in the foreground); 53 into 80 (done); 55 into 23 (a test that turns green in the step that changes its subject).

## Not yet specified

- **Findings not yet discovered.** Any improvement a later `/retro` run surfaces. It graduates into a ticket the moment its question is sharp.

## Out of scope

- **Chronicler Engine product defects.** They belong to the effort that owns the code, not to this bucket.
- **Re-running a retro to confirm a fix held.** A later `/retro` is a fresh run, not a ticket here.

## Open tickets

All are grilling (HITL) and unblocked. Each ticket names its source session in its Context. Source evidence for 10–29: [`assets/reflect-2026-10-07/`](./assets/reflect-2026-10-07/README.md). Verdicts for every open ticket: [`audit.md`](./audit.md).

- [23 — Make the mutation check unconditional for state-repair tests and fixtures](./issues/23-unconditional-mutation-check.md)
- [35 — Lint a spec scenario's shape and vocabulary](./issues/35-lint-spec-scenario-shape.md)
- [86 — Per-stage implement, review and fix loops move the code away from the approved plan](./issues/86-staged-fixes-drift-from-the-plan.md)
- [87 — Repeat one test until it fails, without a hand-written loop](./issues/87-repeat-a-test-until-it-fails.md)
- [88 — Grilling questions use review IDs and rest on facts nobody checked](./issues/88-grilling-questions-unreadable-and-unchecked.md)
- [89 — The full gate fails on a stale generated doc that it could regenerate](./issues/89-full-gate-fails-on-a-stale-generated-doc.md)
- [90 — `pkill -f` kills the shell that runs it](./issues/90-pkill-f-kills-its-own-shell.md)
