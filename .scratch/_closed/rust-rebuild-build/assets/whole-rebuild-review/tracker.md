# Review of the whole Rust rebuild

The tracker for the `/chronicler-after-plan-workflow-plus-review` run on the whole rebuild (P0 to C7, plus tickets 09 to 12).

## Scope

| Item | Value |
|---|---|
| Fixed point | `bc7d6cd4` (last `main` commit before P0 `977ba3f7`; it changes only the plan doc) |
| HEAD at start | `8036bbc3` on `wf/rebuild-core` |
| `main` | `f64f5d41` (P5 merge), so the gate's own duplicate step covers only the core step |
| Plan | `docs/plans/rust-rebuild-build-plan.md` |
| Map | `.scratch/_closed/rust-rebuild-build/map.md` (wayfinder, so the map is closed out; the plan is not archived by hand) |
| Review bundle | `tmp/review/8036bbc3-ref` (`prepare_review_bundle.py --ref bc7d6cd4`; 340 files, 2.5 MB) |
| Unrelated commits in range | `f3af8992` (pi delegate extension), `c05e2395` (subagent roles), `16f5870d` (writing-for-agents skill), `04c9180d` (doc path check) |

## Layout

| Path | Holds |
|---|---|
| `README.md` | this tracker: step status, decisions, open questions |
| `findings/` | one file per source: plan checks, duplicates, each reviewer, my own checks |
| `gates/` | the stdout of each full gate run, with the log name |
| `FINDINGS.md` | the de-duplicated, verified list (step 13); written last |

## Steps

| # | Step | Status | Output |
|---|---|---|---|
| 1 | Plan check: P0 to P5 | done: no unexplained gap; 4 notes (P-1 to P-4) | `findings/plan-p0-p5.md` |
| 1 | Plan check: core C1 to C7 | done: no unexplained gap; 4 notes (C-1 to C-4) | `findings/plan-core.md` |
| 1 | Plan check: tickets 09 to 12 | done: all four match; no blocking finding | `findings/plan-tickets-09-12.md` |
| 2/3 | Wayfinder close-out of the map | done: all 12 tickets resolved; map `Not yet specified` now says the Destination waits for the `main` merge (owner). Plan stays in `docs/plans/` (the maps link it), no archive | `.scratch/_closed/rust-rebuild-build/map.md` |
| 4 | Docs that the changes make false | done: 8 of 9 fixed by lead; DOC-8 open (low) | `findings/docs.md` |
| 5 | Tests for the changes | done: `fix-tests` added scenarios 36.5, 8.9, 8.10, 38.11, 38.12 with tier-1 tests; T-1, T-2, T-5 fixed; helpers D-1..D-6 (`story_of`, `assert_still_thinking`, `app_with_a_triggered_npc`, `long_story_app`, `post_form`, `two_game_app`); spec-coverage 297/297; `--no-browser` green `logs/build_20261019_222224.log` (integration 1337) | `findings/tests-and-consistency.md` |
| 6 | AI slop and hacks | done: fix pass `fix-code` applied all rows (35 files, +145/−279; `tokio-util` dependency dropped); `--no-browser` green `logs/build_20261019_215908.log` (integration 1332, −3 deleted unit tests) | `findings/slop-and-hacks.md` |
| 7 | Duplicates, bad tests, missing features | done: D-1..D-6 applied; D-7, D-8 left (low) | `findings/duplicates-ref-bc7d6cd4.md` (438 clone pairs, mostly `tests/http/`) |
| 8 | Consistency with patterns | findings in: K-1..K-4 | `findings/tests-and-consistency.md` |
| 9 | `/code-simplification` | done: 5 small wins (clones, `plumbing.rs` closure x19, `to_json` reuse); 7 skipped candidates listed in session a4c24ef9 | `logs/build_20261019_222930.log` |
| 10 | `/chronicler-comment-fixer` | done: 2 deleted; flagged `job_tasks.rs` (test affordance shipped in production because `testing` is a default feature) and the two test-only reads | see Notes |
| 11 | Full gate | done: green after fixes `gates/gate-2-after-fixes.out` → `logs/build_20261019_223126.log` (guardrails 189, integration 1337 + 2 skipped, browser 29; 161 s). Baseline green | `gates/gate-1-baseline.out` → `logs/build_20261019_213433.log` (guardrails 189, integration 1335 + 2 skipped, browser 29; 259.6 s) |
| 12 | Four reviewers | done (`findings/review-*.md`); bundles `tmp/review/8036bbc3-ref` (committed range) + `tmp/review/8036bbc3-uncommitted-fixes` (this review's fixes) | `findings/review-*.md` |
| 13 | Report | done | `FINDINGS.md` |

## Decisions

- Wayfinder close-out: the map is not marked as reached, because its Destination says "merged to `main`" and `main` is still `f64f5d41`.
- File lists of the range: `changed-src.txt`, `changed-tests.txt`, `changed-docs.txt` (added or modified only).

- The fixed point is `bc7d6cd4`, not `main`: `main` already holds P0 to P5.

## Notes for the final report

- Correction (lead error): the gate log DOES hold the duplicate summary (`grep -c "Duplicate Summary" logs/build_20261019_213433.log` = 1). The earlier note came from a grep cut off by `head`. The step has no row in the Step Timing Summary, which made it look as if it did not run.

- Comment-fixer flag: `src/application/generation/job_tasks.rs` is test-only machinery built into production because the `testing` Cargo feature is on by default (lines 6-7). A workaround explained by a comment. Carry to report.

## Open questions for the owner

(none yet)
