# Research 02: The Hex issue pipeline, from issue to merged change

Status: reference

Source: [anomalyco/hex](https://github.com/anomalyco/hex), commit `7609574ca92c` (2026-10-31), clone `tmp/hex-compare/anomalyco-hex/`. GitHub REST API answers kept in `tmp/hex-compare/github/`. Chronicler Engine: this repo, 2026-11-01.

## Summary

The pipeline is a loop that runs on the issue, not on a branch. One issue becomes one run with an id, one pull request, and one squash commit on `dev`. The same run also writes to the issue: it acknowledges, it labels, and it closes with a shipping note.

The merge is fast and small. 60 of 64 agent PRs merged. The median PR merged 3 seconds after it opened. The median pipeline commit changes 80 lines in 3 files, and the largest changes 475 lines.

A platform check does not hold the merge. The land queue verifies the library tests and an app build on one host. Breakage on the other platforms lands and a later run repairs it. 8 of the 59 pipeline commits in the clone are such repairs.

Two deterministic steps, and one model review, decide the risk. The policy sorts a diff into `clean`, `flagged` or `dax` and runs from the base branch. A `flagged` diff needs a label that a second model's approval sets. No gate is a person.

Four agent PRs were closed unmerged, and none of the 59 pipeline commits was reverted. One of those four says in plain words, "Closing: maintainer test of the issue pipeline".

Chronicler has none of this loop. It has no pull requests, no CI file, no risk sort of a diff and no automatic land. It does have a ticket lifecycle, five triage roles and a review skill that runs two axes. All of those act inside a session and leave no verdict outside it.

## Practices

12 practices. 6 are marked new.

| Practice | Hex evidence | Chronicler state | Candidate |
|---|---|---|---|
| 1. A required issue form is the only text the fixer and the reviewer see | `.github/ISSUE_TEMPLATE/bug_report.yml:13-52` (version, platform, environment, behavior, reproduction all `required: true`); `feature_request.yml:8-20` (problem, outcome). PR #714 body states it in one line. **Known.** | **Gap.** `.scratch/<effort>/issues/NN-<slug>.md` is free prose under fixed headings. `docs/agents/issue-tracker.md` gives no field schema. **Known.** | new |
| 2. One issue makes one run with an id, and the id appears in the PR body, the commit body and the issue | PR #714 body `Run: 712-2026-10-31t23-48-23-441z`; commit `cb8c4c30` body "run 712-2026-10-31t23-48-23-441z". The head branch carries the same id: `agent/issue-N` for 56 PRs and a `agent/ci-` name for the 8 repairs. Each of the 64 agent PRs carries a run id, and all 64 differ; a retry takes a new id (#405 `404-...20-38-09`, #407 `404-...20-42-41`). **Known.** | **Partly covered.** The `/wayfinder` lifecycle in `docs/agents/issue-tracker.md:25-30` gives each ticket one claim (`Status: claimed`) and one `## Answer`. No run record exists. **Known.** | 4 |
| 3. The PR title names the issue and the change class | 59 pipeline commits in the clone: 40 start `Fix #N:`, 11 start `Labs #N:`, 8 start `fix(ci):`. Every title ends `(#PR)`. **Known** (counted). | **Partly covered.** Commit subjects carry a scope and the ticket, e.g. `feat(prompt): fix the prompt text (prompt-flexibility ticket 19)` and `wayfinder(prompt-flexibility): claim tickets 20 and 21`. No issue number, because there is no issue number. **Known.** | 4 (a smaller habit) |
| 4. A deterministic three-tier risk sort of the diff, read from the base branch | `.github/agent-policy/check.ts:10-17` `dax` paths, `:19-32` `flagged` paths, `:34-43` line rules, `:45-46` 600 lines / 25 files, `:104-108` size and verdict. `.github/workflows/agent-policy.yml:26` checks out `base.sha`. 17 of 64 agent PRs were `flagged`, 46 `clean`, 1 `dax` (corrected 2026-11-01, [candidates](./candidates.md) K-05). **Known.** | **Gap.** `tests/infrastructure/guardrails/` checks the shape of the source tree, not a diff. `.pi/extensions/pi-permission-system/config.json` limits tool calls. Nothing sorts a diff. **Known.** | 1 |
| 5. A second model is the strict reviewer, and its approval is a label | Bodies name the model: `console-anthropic-cyber/claude-opus-5-5` early, `console-anthropic-cyber/claude-mythos-5-1` later. PR #526 ran two models (`mythos-5-1` and `console-google/gemini-4-carbon-eap`) on a `dax` diff. `.github/workflows/agent-policy.yml:35,44` pass only on the label `agent-strict-review`. **Known.** | **Partly covered.** `.agents/skills/code-review/SKILL.md:8-11` runs Standards and Spec as two parallel `reviewer` sub-agents. `chronicler-after-plan-workflow-plus-review` step 11 adds two more. The verdict is a chat report; nothing stores it. **Known.** | 1 and 4 |
| 6. The PR body is a run report that states what was not run | 64 bodies share one template: `Fixes #N`, the form read back, `**Change**`, `**Gates**`, `Run:`, "— from 𝕺𝖕𝖊𝖓𝕮𝖔𝖉𝖊". PR #532: "Nothing was compiled or run yet, and the rune_ui goldens still need regenerating on Linux". PR #639: "part of it was checked, and the compile has not run yet". PR #419: "This is a docs-only change, so no test was needed". **Known.** | **Gap.** `chronicler-after-plan-workflow-plus-review` step 1 reports deferred and missing work to the owner in the session. No durable artifact records it. **Known.** | new |
| 7. The land queue verifies a subset, then installs the app | `.hivemind.toml:3` `batch_max = 4`; `:4-8` `verify` runs `cargo test --locked --workspace --lib`, an SDK install and `scripts/build-app.sh`; `[land.app]` copies the build to `/Applications/Hex.app` and open the app. Median PR open→merge is 3 s (59 of 60 under 60 s). **Known.** | **Gap.** `build.py` is the whole gate and it runs before the commit, on one host. Nothing lands without the owner.  `AGENTS.md:414` "Don't commit without explicit approval". **Known.** | 4 |
| 8. Platform breakage on `dev` is repaired by a later run, not held before the merge | 8 of the 59 pipeline commits are `fix(ci)`, each starting "Check macOS/Windows failed on dev since [first broken commit]. Automated CI fix from the Hex pipeline (run ci-[actions run id]-[time])". 3 of the 8 name a pipeline commit as the broken sha (`eef51da7` twice, `83d4bae6`). `.github/workflows/check-windows.yml:3-7` also runs on `push: branches: [dev]`. **Known** (commit bodies); the "since" attribution is the pipeline's own. | **Gap, and the failure cannot take the same shape.** `build.py` runs the full gate, browser tier included, before the owner commits. A red gate blocks the commit. **Known.** | new |
| 9. The issue gets the same run as a conversation | Issue #627 timeline: `18:06:30` "Thanks for the suggestion! …", `18:06:31` label `needs-design`, `18:24:53` "Thanks for the follow-up! We couldn't resolve this automatically yet. **What we understood** - Area: …", `18:41:37` "It's merged into `dev` … as a Labs feature, off by default: turn on **Second dictation shortcut** under **Settings › Labs**". Issue #712 gets the merged note 3 s after the PR. **Known.** | **Partly covered, with no reporter.** `.scratch` tickets end with `## Answer` and `Status: resolved` (`docs/agents/issue-tracker.md:32`); the map keeps the gist (`map.md`, "Decisions so far"). No outside person reads it. **Known.** | new |
| 10. A triage label diverts an issue away from the automated path | Labels `needs-design` (40 of 96 issues in the window) and `needs-repro` (9). 34 of the 46 issues that got no PR carry one. The label is advisory: 11 issues still merged with `needs-design` (issue #627 is one). **Known** (counts); advisory status is **inferred** from the 11. | **Covered in shape, not as a gate.** `docs/agents/triage-labels.md` gives 5 roles; `.scratch` holds 11 `needs-triage` tickets on 2026-11-01. No automated run exists for a label to divert. **Known.** | new |
| 11. The unit is small by construction | 59 pipeline commits: median 80 changed lines, median 3 files, p90 253, max 475. All 300 recent commits: median 152, p90 1543. No pipeline diff reached the 600-line rule. **Known** (measured). | **Partly covered.** Chronicler's median commit changes 382 lines, p90 3049 (`findings.md`, "Numbers"). Nothing caps a diff. **Known.** | 4 |
| 12. The verdict is a public artifact, not a chat log | A person can reconstruct a run from GitHub alone: the PR body (`Gates`, `Run:`), the label `agent-strict-review`, the head branch name, and the issue timeline. The 7 PRs sampled for reviews returned 0 review objects and 3 PRs sampled for comments returned 0, so the body and the label carry all of it. **Known.** | **Partly covered.** `.scratch/` tickets, `map.md` and `logs/build_history.txt` are durable. A review verdict stays in the session transcript. **Known.** | new |

## Detail

### How an issue reaches `dev`

**Known** from the artifact trail of issue #712 (all times UTC, 2026-10-31):

| Time | Event | Source |
|---|---|---|
| 23:41:26 | Issue #712 opened, bug form filled | `issues_p1.json` |
| 23:48:23 | Run `712-2026-10-31t23-48-23-441z` starts (run id in the PR body) | PR #714 body |
| 23:52:32 | PR #714 opens on branch `agent/issue-712` | `pulls_p1.json` |
| 23:52:35 | PR #714 merges, 3 s later | `pulls_p1.json` |
| 23:52:38 | Issue #712 gets the shipping comment and closes | `timeline_712.json` |

Total: 11.2 minutes from issue to merged commit.

**Inferred:** the run starts when the issue is created, because the gap repeats on every sampled issue (median 6.7 minutes over 48 issues, minimum 2.4). The pipeline messages come from the `thdxr` account, which is the maintainer's token, not a bot account.

### The gates, in order

**Known** from `.github/workflows/agent-policy.yml` and the PR bodies:

1. **The sandbox build.** "build passed in the apollo sandbox" in 56 of the 64 bodies; the other 8 say "built in the apollo sandbox (Linux)". This is a build. 16 of the 64 bodies say in the `**Change**` text that the change was not compiled, not run, or not re-run (for example #713, #706, #639, #532). 3 more say no test was needed (#407, #419, #670).
2. **The policy sort.** `.github/agent-policy/check.ts` returns `clean`, `flagged` or `dax`, and the workflow reads the script from `base.sha` so a patch cannot weaken its own policy.
3. **The strict reviewer.** A model approves a `flagged` diff, and the approval becomes the label. The workflow then passes.
4. **No person.** `check.ts:6` says "`dax` patches need Dax and are never auto-merged", yet one `dax` diff merged: PR #526, with two model approvals in place of a person. **Known** from the body; why it merged is **inferred**.

**Known** flag reasons across the 17 `flagged` PRs, as the `**Gates**` line words them: new URL 9, `unsafe` code 5, new host string 3, process spawning 1. That is 18 findings on 17 PRs, because one diff broke two rules. The URL and host-string rules carry 12 of the 18.

The size rule never fired. The largest pipeline commit changed 475 lines, which is under the 600-line limit. **Known.**

### What the reviewer flags

**I could not measure this.** `GET /repos/anomalyco/hex/pulls/{410,463,569,634,706,526,532}/reviews` returned `[]` for all seven, and `GET /issues/{532,405,714}/comments` returned `[]` for all three. The strict reviewer is not a GitHub review object. Its only public traces are the label and one sentence in the body. **Known.**

**Inferred:** the reviewer is a step inside the private pipeline, and it feeds the label, not GitHub's review API.

### What a failed run looks like

**Known.** 4 of 64 agent PRs closed unmerged:

| PR | Issue | Verdict in the body | What closed it |
|---|---|---|---|
| #405 | #404 | policy `clean`, model approved | No comment. A second run (#407) retried the same issue. |
| #407 | #404 | policy `clean`, model approved | No comment |
| #419 | #418 | policy `clean`, model approved | Issue #418 has state `not_planned`, 6 s after the PR closed. No comment. |
| #532 | #531 | policy `clean`, model approved | Issue #531 comment: "Closing: maintainer test of the issue pipeline." |

So the public record shows no gate that rejected a change. One close says the owner was testing the pipeline, and **inferred:** the other three follow the same pattern. The #405 and #407 pair is **inferred** as a retry, from the two run ids and the two PRs on one issue.

46 issues got no PR at all. 34 of those carry `needs-design` or `needs-repro`, and 12 carry no label (issue #650 has an empty timeline: no label, no comment, no PR). **Known.** Why the unlabeled 12 got nothing is **unknown**.

### Breaking `dev` and repairing it

**Known** from the 8 `fix(ci)` commit bodies. The pipeline names the check that failed, the first commit where it failed, and the actions run that will fix it:

```
fix(ci): enlarge the Labs test windows so the macOS-only rows and Run setup again stay on screen (#644)
Check macOS failed on dev since 83d4bae6. Automated CI fix from the Hex pipeline (run ci-41230647226-2026-10-31t18-52-01-798z).
```

3 of the 8 name a pipeline commit as the broken sha, and 2 distinct commits do that (`eef51da7` twice, `83d4bae6` once). So **inferred:** 2 of the 51 pipeline feature commits, about 4%, broke `dev` on a platform the land queue does not check. The other 5 repairs name earlier, non-pipeline commits.

The whole repo has 11 `fix(ci)` commits. 8 come from the pipeline, 3 from other agent work. **Known.**

### Reverts

**Known.** 4 commits start `Revert` (corrected 2026-11-01, [candidates](./candidates.md) K-07). This sentence first said 5, and the list of 5 reverted commits after it was not re-checked one by one. They revert `7a2a03c9`, `2ec61b23`, `5e9c06bd`, `c65421be` and `34b74d3e`. None of those 5 carries the pipeline trailer, so **0 of 59 pipeline commits were reverted** in the window.

### Volume and latency

**Known** (measured on the JSON kept in `tmp/hex-compare/github/`):

| Measure | Value |
|---|---|
| Agent PRs, all time at the snapshot | 64 (60 merged, 4 closed unmerged, 0 open) |
| Agent PRs, first to last | #374 (2026-10-29) to #717 (2026-11-01): about 21 a day |
| Median PR open→merge | 3 s (p75 4 s, p90 5 s, max 1055 s on the rebased #639) |
| Median issue→merge, 48 issues | 6.7 min (min 2.4, max 256.6) |
| Pipeline commits in the clone | 59 (the clone stops about 32 minutes before #717 merged) |
| Head branches | 56 name the issue (`agent/issue-N`); 8 name the failed check, the broken commit and an attempt number (`agent/ci-check-macos-eef51da7-1`, `...-eef51da7-2`) |
| Issues created 2026-10-29…11-01 | 96 (48 merged, 2 with a rejected PR, 46 with no PR) |

The policy and the land queue landed on 2026-10-29 (`3f158298` "Add hex-agent-policy check for issue-pipeline pull requests (#371)" and `9aa79bc9` "feat(land): add declarative .hivemind.toml land queue", both 2026-10-29). The whole path is 3 days old at the snapshot. **Known.**

### How this differs from Chronicler

**Known:**

- Chronicler has no `.github/` directory at all. There is no workflow file, no issue template, no policy script and no PR. `git remote` is `git@github.com:notsonormal/chronicler-engine.git`.
- Chronicler's issue tracker is local markdown in `.scratch/` (`docs/agents/issue-tracker.md`). It holds 339 resolved, 34 open and 11 `needs-triage` tickets on 2026-11-01.
- The gate is `build.py`, run in the foreground by the agent, and the owner approves each commit (`AGENTS.md:414`).
- The closest analogue to the policy sort is the guardrail suite in `tests/infrastructure/guardrails/`. It reads `src/` and `tests/` for structure, naming, layering and test hygiene. It judges the tree, not a change.
- The closest analogue to the strict reviewer is `.agents/skills/code-review/SKILL.md`. It judges a change against Standards and Spec, in two sub-agents, and reports to the owner.

**Inferred:** the two systems solve the same problem from opposite ends. Hex moves the trust to a public, replayable trail and accepts later repair. Chronicler moves the trust to a heavy local gate and a person.

## What I could not find or did not read

- **The pipeline itself.** No file in the repository defines the run: no prompt, no config, no workflow that starts it. `AGENTS.md:1244` is the only mention of "the issue pipeline" in the repo. The two behaviours (`hivemind`, `.github/agent-policy/`) are public, and the loop between them is not. **Known** (searched with `rg` for `issue pipeline`, `hivemind`, `agent-strict-review`, `opencode-agent`, `apollo sandbox`).
- **The strict reviewer's criteria.** Zero GitHub reviews and zero comments on the sampled PRs, so no public text says what it flags. The label says only that it approved.
- **`needs-design` and `needs-repro`.** Neither label appears anywhere in the repo. Their meaning is **inferred** from how they are used.
- **Who closes a failed PR.** The closes come from `thdxr`, which is also the identity on every pipeline comment and every merge. Pipeline comments end with "— from 𝕺𝖕𝖊𝖓𝕮𝖔𝖉𝖊" and the maintainer's own comments do not, but a close carries no such mark. I cannot separate the two in the API. **Known** limitation.
- **PR sizes from the API.** The list answer has no `additions`/`deletions` fields, so I measured the squash commits in the clone instead.
- **Issues older than 2026-10-29.** The 3 issue pages came back sorted by creation and stopped at 300 items, so 96 pooled. The repo had 219 issues on 2026-11-01, so about 123 older issues were not read. They predate the pipeline, so they do not change the funnel.
- **Any post-`7609574c` history.** The clone stops about 32 minutes before PR #717 merged.

### Sampling

- `pulls?state=all&per_page=100`: all 5 pages, 498 PRs. This is the whole population, not a sample.
- `issues?state=all&per_page=100`: 3 pages, 300 items, 96 of them issues, covering 2026-10-29…11-01. This is the whole pipeline window.
- `pulls/{410,463,569,634,706,526,532}/reviews`: 7 PRs, chosen to cover the first and last `flagged` PR, the `dax` PR and one unmerged PR. All empty.
- `issues/{532,405,714}/comments`: 3 PRs, one from each outcome class. All empty.
- `issues/{627,531,650,712}/timeline`: 4 issues, chosen to cover a `needs-design` success, a maintainer-test close, an unexplained no-PR issue and a fast success.
- 26 of the 60 unauthenticated requests available in the hour were used, including one `rate_limit` read at the start.

**Known:** the counts in this asset come from the JSON in `tmp/hex-compare/github/` and from `git log` on the clone at `7609574ca92c`. I did not re-measure the `.scratch` or commit numbers of the findings asset.
