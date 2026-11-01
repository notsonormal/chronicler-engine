# Research 12: where evidence of a working feature lands today

Status: reference

Ticket: [12 Find where evidence of a working feature lands today](../issues/12-research-where-evidence-lands.md)

Sources: this repo at `1a0c9162` plus the working tree of 2026-11-01. Hex at `7609574ca92c` (clone in `tmp/hex-compare/anomalyco-hex/`), read through [research-04](./research-04-feature-map.md). Covers C-12, C-13 and C-14 of [candidates.md](./candidates.md).

## Summary

- **Known.** Evidence that someone saw a Chronicler feature work lands in three places. None is a doc of the behaviour. The places are: ticket answers in `.scratch/` (60 ticket files mention a shot), untracked files in `tmp/` (754 PNG files, a lower bound), and 6 trial reading packs in `tmp/`. `docs/` has 0 files with "observed", "manually verified", "verified by hand" or "saw it".
- **Known.** The link from a ticket to its evidence rots. Tracked notes in `.scratch/` cite 207 concrete `tmp/` paths. 73 of them (35%) do not exist now.
- **Known.** The known gaps of Autoplay sit in 4 tickets and in a deleted plan. A reader of `docs/specs/autoplay.md` finds none of them.
- **Known.** Both draft forms pass the gate checks as they stand. The spec check, the doc check and Vale all pass. No check keeps a draft true.
- **Inferred.** Inside the spec is the better place. The `chronicler-ui-investigator` skill tells its reader to read the spec before driving a feature. A separate file would not be read unless something points to it.
- **Recommendation.** C-14 keep. C-13 keep, narrow. C-12 keep, narrow, for what the gate cannot show.

## Who observes and who reads

| Role | Who | How | Source |
|---|---|---|---|
| Observer | The owner | Plays the game on the live dashboard. He is the only user. | map Notes; `AGENTS.md` (owner approves each commit) |
| Observer | An agent, in a browser | `chronicler-ui-investigator`: a probe server, DOM and endpoint reads, and shots. The skill says the work is done only when the agent has looked at a shot. | `.agents/skills/chronicler-ui-investigator/SKILL.md` ("Done only when you have looked at a shot") |
| Observer | An agent, with a real provider | `scripts/trial.py run` plays Autoplay on a probe server and writes a reading pack. A real provider answers. A turn costs tokens. | `scripts/trial.py:1-17`; the SKILL.md says a turn costs less than a cent |
| Not an observer | The gate | `ScriptedModel` answers the tier-1 tests. `tests/llm/` holds the real-provider tests, and they are `#[ignore]`d. | `tests/STRATEGY.md` tier table, lines 16 and 23 |
| Reader | An agent that changes or drives a feature | The skill says: "Before you drive a feature, read its spec in `docs/specs/<feature>.md`." | SKILL.md, section "Driving gameplay" |
| Reader | The owner | Reads packs and ticket answers when he judges output. | `.scratch/prompt-flexibility/issues/18-make-the-trial-script.md` (the pack is made for reading) |

- **Known.** An agent can observe a feature. So the record does not need the owner as observer.
- **Inferred.** The reader that matters most is a later agent session. The owner is the second reader. Nobody reads `tmp/` by accident, so evidence that lives only there has no reader.

## Where evidence lands today

Counts ran on 2026-11-01. A count in `tmp/` is a lower bound, because `tmp/` is git-ignored and can be cleared. The oldest PNG there is dated 2026-10-03.

| Place | Count (known) | Example | Tracked in git | Read path |
|---|---|---|---|---|
| Ticket answers in `.scratch/` with a method line and a table of findings | 60 ticket files in 9 efforts mention a shot or screenshot. 30 of them are in `_closed/dashboard-ui-review`. 15 files carry `[known]` and 17 carry `[inferred]`. | `.scratch/_closed/dashboard-ui-review/issues/04-review-text-check-enabled.md`: a method line (port, DB copy, viewport), a findings table, a "What works" list | yes | only a reader of that ticket |
| Shots committed in `.scratch/` | 17 images in 4 efforts | `.scratch/dashboard-design/assets/baseline-shots/` (5), `game-tab-critique/` (6) | yes | linked from the effort |
| Shots in `tmp/` | 754 PNG files (lower bound). 284 belong to this map. About 160 are shots of other apps (`marinara`, `sillytavern-13`). Product shots: `ui-review` 90, `screenshots` 55, `ui-gates` 53, `t24` 34, `p2-07` 20, `autoplay-ui` 13. 333 PNG files are dated 2026-11-01. | `tmp/ui-review/` | no | none |
| Test failure shots | 55 files in `tmp/screenshots/`. The Playwright harness writes one on a failed wait. They show a failure, not a working feature. | `tests/test_utils/browser.rs` (K-28 of candidates) | no | none |
| Trial reading packs | 6 `pack.md` files in `tmp/` and 4 `compare.md`. A pack holds `run.json`, `summary.json`, `story.json`, `setup.json` and `calls/`. | `tmp/pf20/redmist_julian/trial-1` | no | the ticket that cites it |
| Plan Status lines | 4 plans in `docs/plans/`. Each Status line says Draft or Open. None records an observation. | `docs/plans/narration-line-break-tags.md` | yes | n/a |
| Commit messages | 905 commits. 7 match "screenshot" or "shot of". 1 matches "observed". 3 match "manually" or "by hand". | `git log --grep` | yes | rarely read |
| `docs/` | 0 files match "observed", "manually verified", "verified by hand" or "saw it". | `grep -rliE` over `docs/` | yes | n/a |

### The link rots

- **Known.** A script read every tracked `.scratch/**/*.md` outside this map. It found 207 distinct concrete `tmp/...` paths. A path that ended in `*`, `<`, or `/` was skipped. 134 paths exist now. 73 do not. 37 of the 73 are cited from `_closed/`.
- **Known.** A plan link rots as well. Ticket `autoplay-follow-ups/issues/03` cites `old-docs/archived-plans/autoplay-plan.md`. Commit `3e42a26f` ("Removing the old plans and reviews") deleted that file, and the file is not in the tree.
- **Inferred.** A ticket is a poor home for evidence. It records that someone looked, but its reader must know to open the ticket. A closed effort moves to `_closed/`, and the shots stay in `tmp/` or vanish.

### What a trial pack has and lacks

- **Known.** `run.json` names the start time, the Budget, the cycles played, the stop reason and the restarts. `summary.json` names cycles, entries, seconds, failed calls and tokens.
- **Known.** No field names the engine commit. `scripts/trial.py` has no `git` or `commit` call (grep of the file).
- **Inferred.** Without a commit, a pack cannot say which build it observed. Hex's recipe asks for the revision (`docs/features/README.md:1849-1853`).

### Hex form, for comparison

- **Known.** A Hex record is a dated paragraph. Example: "Observed September 4, 2026 (2.1.16), existing model picker: all 455 Rust tests passed ... Native menu interaction, installed-app switching, and Linux runtime behavior were not exercised." (`docs/features/README.md:406-411`).
- **Known.** A Hex "Proof limit" is one or two lines after the check: "Proof limit: controlled ordered output, paste commitment, and retained text; not target-app clipboard consumption." (`docs/features/recovery.md:60-61`). There are 17 in 3 of the 10 feature files (K-17 of candidates).
- **Known.** A Hex gap has a diagram, a "Source/check" line, an issue link, and a "Missing evidence:" line (`docs/features/recovery.md:606-625`).
- **Known.** Hex's own recipe for a record: behavior id, entry point, revision or binary, platform, prerequisites, outcome, artifact location, and the paths that were skipped. It forbids tokens and personal transcripts as proof (`docs/features/README.md:1849-1853`).

## Autoplay: the known gaps

| Gap | Where it is recorded | Status | Would a reader of `docs/specs/autoplay.md` find it? |
|---|---|---|---|
| Two Games in two browser tabs. The client polls carry no Game id. | `.scratch/autoplay-follow-ups/issues/01-per-tab-games.md` | needs-triage | No |
| A stop does not cancel the model call. The late answer still counts for the role health of the Narrator. | `.scratch/autoplay-follow-ups/issues/02-narrator-role-health-after-a-stop.md` | needs-triage | No. Scenario 41.6 states only that the story log holds no Message of the run. |
| No history of runs. A run lives in memory, and only tracing lines remain. | `.scratch/autoplay-follow-ups/issues/03-autoplay-runs-history-table.md` | needs-triage | No |
| The Games panel poll closed an open rename form. | `.scratch/autoplay-follow-ups/issues/04-rename-form-survives-the-games-panel-poll.md` | closed (stage 3a of the change-driven refresh plan) | Not needed. The gap is closed. |
| Two more items: no machine-readable busy signal, and no posture re-render on a refused config write. | section 6b of `autoplay-review-fixes-plan.md`, which only git history holds now (`git show 3e42a26f^:old-docs/archived-plans/autoplay-review-fixes-plan.md`) | not re-checked | No |
| The checks use `ScriptedModel`. No check shows a real provider's answers (a reasoning-only reply, a `content_filter` answer, a 401). | nowhere as a gap. `tests/STRATEGY.md` names the stand-in per tier. `scripts/trial.py` restarts a run that a filtered answer ended. | n/a | No |

- **Known.** A search of `docs/specs/autoplay.md` and `browser_autoplay.md` for "role health", "history", "cancel", "in-flight" and "confirm" finds no gap. The only matches are the `/history/:id` route and the Games tab.
- **Known.** Autoplay has 0 mentions in `.scratch/_closed/`. Its gaps were never in closed tickets. They are in open tickets and in git history. The ticket text of C-14 ("known gaps end in closed tickets") is true for other features (for example `_closed/narrator-modes-and-options/issues/10-implement-options-agent-pipeline.md` has a section "Known gaps (recorded, deliberately deferred)"), and not for Autoplay.
- **Known.** The 25 scenarios of `docs/specs/autoplay.md` (41.1 to 41.25) carry 32 tags in `tests/http/autoplay/run.rs` (25) and `fragments.rs` (7). 26 lines of `run.rs` and 7 of `fragments.rs` name `ScriptedModel`. So the whole spec rests on a scripted model (inferred from the use counts).
- **Known.** Real runs exist. 5 trial packs of prompt-flexibility record Autoplay on a real provider: three trials of 5, 20 and 100 cycles with 0 failed calls, and two of 34 and 88 cycles with 1 and 5 failed calls (`summary.json` of `pf18`, `pf19`, `pf20`, `pf08/trial-a`, `pf08/trial-b`). None of them is cited from the spec.

## Draft: the proof record, the proof limit and the gaps for `docs/specs/autoplay.md`

The text below is a draft in this asset only. `docs/specs/` is not edited. Facts in the draft come from the sections above. The text of both forms is the same, except for the first lines of Form B.

### Form A: inside the spec

Append after Scenario 41.25. The draft is 21 lines (1,886 bytes). `autoplay.md` has 383 lines, so this adds 5.5%.

```markdown
## Proof

Each scenario above has a tagged check that the gate runs. This section says what the checks do not show, what someone saw work, and what is still missing.

### Proof limit

- Scenarios 41.1 to 41.25 run on `ScriptedModel`, a model with fixed answers. They do not show how a real provider answers. A reasoning-only reply, a `content_filter` answer and a 401 are not exercised.
- Scenario 41.6 stops the run at a held model call. Scenario 41.7 stops it at the test pause point between two Jobs. A stop at any other moment is not exercised.
- Scenarios 42.1 to 42.7 check the controls in a browser. They do not show how a player reads the status display over a long run.

### Observed

**Observed 2026-11-01 on the probe server, trial `pf20` of `scripts/trial.py`, commit not recorded.** The setup was World `redmist_estate`, Persona `julian`, Narrator Mode `novel`, and one OpenCode Go Connection for the Narrator and the Quantifier. The run had a budget of 100. It played 100 cycles and wrote 201 story entries with 0 failed calls. It ran 1241 seconds and ended by its budget.

Not exercised: a stop, a failed call, a `content_filter` answer and a second browser tab. The reading pack is `tmp/pf20/redmist_julian/trial-1`. That folder is not in git.

## Known gaps

- **Two browser tabs.** The run is keyed by Game, but the client polls carry no Game id. Ticket: `.scratch/autoplay-follow-ups/issues/01-per-tab-games.md`.
- **Role health after a stop.** A stop does not cancel the model call. The late answer is refused for the story, but it still counts for the role health of the Narrator. Ticket: `.scratch/autoplay-follow-ups/issues/02-narrator-role-health-after-a-stop.md`.
- **History of runs.** A run lives in memory. No record of it remains after the run ends, except the tracing lines. Ticket: `.scratch/autoplay-follow-ups/issues/03-autoplay-runs-history-table.md`.
```

- **Known.** Each fact in the "Observed" paragraph is read from `tmp/pf20/redmist_julian/trial-1` (`setup.json`, `run.json`, `summary.json`). The start time is 2026-11-01 00:23 UTC. The commit is not in the pack. The record says so.
- **Inferred.** The trial ran with the builds of 2026-10-31 and 2026-11-01. A real record would name the commit that was running.

### Form B: a separate file next to the spec

File `docs/specs/autoplay_evidence.md`. Same body as Form A, with this head:

```markdown
# Evidence: Autoplay

Spec: [autoplay.md](./autoplay.md). Browser spec: [browser_autoplay.md](./browser_autoplay.md).

## Proof

Each scenario of the spec has a tagged check that the gate runs. ...
```

The draft is 25 lines.

### Checks of each form

Run on 2026-11-01 in a scratch tree (`tmp/ev12/tree/`: a copy of `docs/specs/`, a link to the real `tests/`). The script patched only the paths that `validate_feature_spec.py` reads. `validate_docs.py` and `vale_lint.py` ran on the draft files with their real rules. Each check ran a negative case as well, to show that it is live.

| Check | Form A (inside) | Form B (file in `docs/specs/`) | Form B2 (file in `docs/specs/evidence/`) |
|---|---|---|---|
| `validate_feature_spec.py` | Pass. 561 declared, 561 covered, 0 gap. Same as the baseline. | Pass. Same numbers. The script globs `docs/specs/*.md`, so it reads the file and finds 0 scenarios. | Pass. The glob is not recursive, so it does not read the file. |
| `validate_docs.py`, role | The spec stays EXCLUDED. `classify_file` returns EXCLUDED, because only `reference`, `explanation`, `how-to` and `tutorials` are STANDARD. | EXCLUDED | EXCLUDED |
| `validate_docs.py`, `UNKNOWN_ROUTE` | Pass. The rule runs on every `docs/**/*.md`, so it protects a route that the record names. A negative case with `/autoplay/history` failed, as it should. | Pass, same rule | Pass, same rule |
| `validate_docs.py`, `LINE_CITATION` and `BROKEN_SOURCE_PATH` | Not run on specs. A `.scratch/` or `tmp/` path in the text is not checked. | Not run | Not run |
| `vale_lint.py` | Pass. A negative case with a 30-word sentence and a semicolon failed with 2 errors. Specs are in Vale scope, so the rules of STE bind the record. | Pass | Pass |
| Trap | A heading of the form `#### Scenario 41.99` in the evidence text counts as a declared scenario with no test. In Form B the run failed with 1 gap. | same | no trap (file not read) |

- **Known.** No check reads the proof record. A scenario number that the record names (41.6) is plain text. A renumber leaves it stale, and nothing fails.
- **Known.** No check reads a path in the "Known gaps" list. A ticket that moves to `_closed/` leaves a dead path. Ticket 03 of `autoplay-follow-ups` already cites a deleted plan.
- **Inferred.** One check could verify that each "Scenarios N.M" in the record resolves to a declared scenario. It would be a few lines in `validate_feature_spec.py`. This asset does not build it. It would answer the drift that Hex shows: 11 of 611 named checks no longer resolve.

### Comparison

| Point | Form A (inside) | Form B (next to) |
|---|---|---|
| Reader path | The skill and the agents read the spec first. The record sits in that read. | A reader must know the file exists. The skill names `docs/specs/<feature>.md` only (inferred). A pointer line in the spec or the skill would be needed. |
| Size of the spec | +21 lines (5.5% of 383). Every reader of the spec pays it. | The spec stays unchanged. |
| `docs/AGENTS.md` "Where a fact lives" | No layer names a dated observation. The table lists purpose, why, event rules, and fine behaviour. One added row is needed. | Same. One added row is needed. |
| Change in the same commit | One file, so a scenario edit and a limit edit meet in one diff. | Two files. The direction rule "a behaviour change edits its spec and its test" does not name the evidence file. |
| Gate risk | None found. | A heading with "Scenario N.M" in the file breaks the gate. A file in a subfolder avoids it. |
| Fit with a later feature map | The sections move with the page. | The file becomes a section of the page, or stays beside it (inferred). |
| Stale record risk | Same for both. A dated "Observed" paragraph becomes false after the next change, and no check shows it. | Same. |

- **Inferred.** Form A wins on the reader path and on the single-diff rule. Form B wins only on the size of the spec.
- **Known.** The layout of docs waits for [ticket 10](../issues/10-research-agent-use-of-docs-and-plans.md). Both forms fit the 31 specs of today. Neither form depends on how the docs are organized.

## The three practices

### C-12: three proof states and a dated observed record

- **Hex (known).** `AGENTS.md:66-70` keeps three states apart. Dated records are at `docs/features/README.md:246` and `:406-411`.
- **Chronicler (known).** Gap. Observation happens (60 ticket files mention shots, 6 trial packs), and it lands where nobody reads it. 35% of the `tmp/` paths that tickets cite are dead.
- **What the gate cannot show (known).** A real provider's answer, and a look at a rendered page. The tier table puts a real provider only in `#[ignore]`d tests. The browser tier checks DOM and selectors, and takes shots only on failure.
- **Case against (inferred).** The owner is the only user. An AI writes and reads the record. No check keeps it true. Hex's map drifts the same way (11 of 611 named checks do not resolve; [research-04](./research-04-feature-map.md)). A false "Observed" line is worse than none, because a reader trusts a date.
- **Recommendation: keep, narrow.** Record an observation only where the gate cannot make it: a trial on a real provider, or a look at a shot. One paragraph in each spec. Replace it at the next observation. Name the commit. Name what was not exercised. Do not add a per-scenario ladder. A small side task: make `scripts/trial.py` write the commit into `run.json` (inferred: about one session or less).

### C-13: a proof limit beside each check

- **Hex (known).** 17 sentences in 3 of 10 feature files, 15 of them in `recovery.md` (K-17 of candidates).
- **Chronicler (known).** Partly covered at tier level: `tests/STRATEGY.md` names what each tier uses as a stand-in. No spec says what the stand-in hides.
- **Draft (known).** 3 bullets cover the whole of `autoplay.md`. The draft takes one bullet for each stand-in (`ScriptedModel`, the stop moments, the browser tier), not one for each scenario.
- **Recommendation: keep, narrow.** One "Proof limit" list per spec, only for a stand-in that hides something. A line per scenario would cost 25 lines in this spec and add no fact (inferred).

### C-14: known gaps beside the behavior

- **Hex (known).** "Gaps And Constraints" sections with "Missing evidence:" lines and issue links (`recovery.md:606-672`).
- **Chronicler (known).** Gap. For Autoplay, 3 open gaps are in tickets, and 2 more are in a deleted plan. The spec shows none.
- **Draft (known).** 3 lines, each with the gap, the effect, and a ticket path.
- **Recommendation: keep.** It is the cheapest of the three, and each line points to a ticket that is the home of the fact. The line holds one sentence and a link, so `docs/AGENTS.md` "one home for a fact" still holds (inferred). The cost: a closed ticket leaves a dead link, so a gap line should be removed when its ticket closes.

## Questions for ticket 16

1. **Where does the record sit: inside the spec (Form A) or in a file next to it (Form B)?** My recommendation: inside, because the investigator skill and later agents read the spec first, and a separate file has no reader. It needs one added row in `docs/AGENTS.md` "Where a fact lives".
2. **Does the owner accept an "Observed" paragraph that an agent writes from a trial or a shot, with no gate check that it stays true?** If no, drop C-12 and keep C-13 and C-14. If yes, the paragraph must carry a date and a commit, and a later observation replaces it.

## What I did not check

- **Known.** I did not run a probe server or a new trial. The draft uses the reading pack that exists.
- **Known.** The `tmp/` counts are a lower bound, and other agents add files to `tmp/` while this ticket runs (333 PNG files are dated 2026-11-01).
- **Not checked.** Section 6b of the deleted plan is read from git (`3e42a26f^`). Whether its two minor items still hold in `src/` was not checked.
- **Not checked.** The wording "OpenCode Go Connection for the Narrator and the Quantifier" comes from `setup.json` of `pf20` (both slots name the Connection `Go check`, provider `OpenCodeGo`).
- **Known.** The number of declared scenarios is 561 today. K-26 of candidates says 555. The spec files and tests grew since.
- **Not checked.** The 60 ticket files that mention a shot were counted by word. A word match can name a shot without a record of looking at it.
- **Known.** The draft files and the scripts of this research are in `tmp/ev12/` (git-ignored): `block.md`, `autoplay_evidence.md`, `run_fs.py`, `tree/`, `trap.md`.
