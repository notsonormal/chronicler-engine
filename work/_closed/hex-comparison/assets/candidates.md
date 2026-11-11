# Candidates: the unique Hex practices, checked, with a proposal for each

Status: reference. Asset of the ticket [Prepare the candidate list for the grilling](../issues/09-task-prepare-the-candidate-list.md) in the [map](../map.md). Made on 2026-11-01.

The proposals are proposals only. The owner decides in the grilling [Agree the practices to decide](../issues/08-grilling-agree-the-practices-to-decide.md).

## Summary

- The findings asset and the seven research assets hold 103 practice rows. They merge into **52 unique practices** in 7 themes.
- Proposals: **keep 19** for a decision, **already covered 13**, **drop 20**. Each drop has a reason.
- The 19 kept practices fit in **6 decision tickets** (part 3). One of the six can split into two, so the count is 6 or 7.
- Part 4 has 47 checks. 41 confirm a claim, 6 of them with a small correction. 5 find a claim wrong. 1 is a new check (K-31). One more claim is not checkable. No wrong claim changes a proposal from keep to drop, or from drop to keep. Two results change the evidence of a kept practice:
  - Candidate 6: three tests read the clock apart from the code under test, not one (K-32).
  - Candidate 7: Chronicler does not fully follow the seam rule. The two agent hook traits each have one impl and no test impl (K-31).
- One new fact matters for candidate 5. The closed `ui-verification-redesign` map did not weigh image comparison. But Chronicler already depends on `playwright-rs` 0.9, and that map's research notes a `to_have_screenshot` builder in it (K-29).

## How to read this asset

- **Source codes.** `F-n` is practice n of the [findings asset](./findings.md). `Fo` is its "Other facts" list. `Rnn-m` is row m of the practice table in `research-nn-*.md`. `NYS` is the map entry **Docs organized as feature maps**.
- **Check codes.** `K-nn` is a row in part 4. A Hex evidence cell names the check that confirmed it. "Not re-checked" means the cell rests on the research asset only.
- **Labels.** Each claim is **known** (a file, a command or a commit shows it) or **inferred** (a reading of that evidence). In the tables, a claim is known unless the cell says inferred.
- **Proposal words.** *Keep* means: give it a decision ticket. *Covered* means: Chronicler already has it, or has a deliberate equal. *Drop* means: no decision ticket, for the reason given.
- **Linked.** The candidate numbers 1–8 of the map, "habit" for a smaller habit of the map, "NYS" for docs as feature maps, or "new".
- Part 6 maps each of the 103 source rows to its practice id, so a reader can trace every row.

## 1. Themes

| Theme | Practices | Keep | Covered | Drop |
|---|---|---|---|---|
| A. Agent instructions and spec | C-01 to C-08 | 2 | 3 | 3 |
| B. Docs organization | C-09 to C-11 | 2 | 1 | 0 |
| C. Proof and verification | C-12 to C-16 | 4 | 0 | 1 |
| D. Landing and gates | C-17 to C-29 | 5 | 1 | 7 |
| E. Code and test style | C-30 to C-41 | 3 | 4 | 5 |
| F. Planning and tracking | C-42 to C-49 | 3 | 4 | 1 |
| G. Product and working habits | C-50 to C-52 | 0 | 0 | 3 |
| **Total** | **52** | **19** | **13** | **20** |

C-35 counts as covered: the capability folder is covered, and its parent-file form is dropped.

The first proposal of themes held, with two changes. "Other" became "Product and working habits", because all three of its rows are habits of the authors or of the product. Labs (candidate 8) went there, because it is a product release practice and not a gate.

## 2. The practices, by theme

### A. Agent instructions and spec

| ID and name | Description | Sources | Hex evidence (checked) | Chronicler state | Conflict | Linked | Proposal | Type |
|---|---|---|---|---|---|---|---|---|
| **C-01** One always-loaded spec | One file loads into every session. It holds the purpose, a hand-written module map and the invariants. | F-2, R01-1, R04-11 | `AGENTS.md`: 1486 lines, 100,349 B; `## Purpose` :23, `## Architecture` :72, `## Invariants` :527 (K-11) | **Partly covered.** The facts are in `AGENTS.md` (29,524 B), `CODING_STANDARDS.md`, `CONTEXT.md`, `tests/AGENTS.md`, `docs/diataxis/` and `docs/specs/`. The module map is generated from `//!` lines (`AGENTS.md` Structure block). | `docs/AGENTS.md` "Where a fact lives" puts each fact in one layer, and the layers are separate files. Cost: the Hex file is about 3.4 times the size of Chronicler's `AGENTS.md` (inferred from the byte counts). | 2 | **Keep.** It is the base of candidate 2. The owner joined candidates 2 and 3. | in G2 |
| **C-02** Invariants with exact values | Each invariant states the behavior and its number, for example "Captures shorter than 300 ms discard". | F-2, R01-2, R04-11, R07-17 | 73 invariant bullets; 23 carry a number with a unit, 33 carry any digit (K-11). `AGENTS.md:606` "300 ms discard" (K-11) | **Partly covered.** Diátaxis Reference states invariants per subject, as cardinality ("at most one Job"). Values stay in the code. | `docs/AGENTS.md:149` "Values stay in the code" bars constants and caps from docs (K-22). `docs/AGENTS.md:155-158` "No negative explaining" opposes Hex's many prohibitions (K-22). | 2 | **Keep.** Two written Chronicler rules oppose it, so only the owner can decide it. Dax Raad's question "where is the source of truth for how your software is supposed to work?" (R07-17) is the reason Hex gives for it. | in G2 |
| **C-03** Docs change with the behavior, in both directions | The change that adds a behavior adds its docs. The change that removes a feature removes its rules. | R01-3, R01-4, R04-10 | 8 of 228 `AGENTS.md` commits touch only that file (K-12). `AGENTS.md:57` "Update the affected map in the same change" (K-15). `a8a4645d` removes a feature and its rules (not re-checked). | **Covered for `docs/`.** `docs/AGENTS.md:38` says a behaviour change edits its spec and its test, and a Diátaxis doc changes when a part, an invariant or a name is added, removed or renamed (K-23). The `spec-coverage` gate step enforces the spec-to-test link. The small gap: no rule covers hand-written `AGENTS.md` prose or skills. | none | 2, 3 | **Covered.** If G2 adopts one spec file, add one sentence for that file there. | — |
| **C-04** Workflow rules in a private global file | The public `AGENTS.md` keeps workflow short and points to `~/code/control/AGENTS.md`. | R01-8, R07-1 | `AGENTS.md:3-11` points to `~/code/control/AGENTS.md` (K-09). That file returns 404 (R07, not re-checked). | Gap. Chronicler keeps its rules in the repo. | none | new | **Drop.** Hex shares one guide across many repos (inferred). Chronicler has one repo. A private file would hide rules from the reviewer, the permission system and this map's outside check. This research could not read Hex's file at all. | — |
| **C-05** The app ships agent instructions to its users | The product writes an `AGENTS.md` and a public skill for the agents of its users. | R01-9 | `src/personal_commands/AGENTS.md`, `skills/hex-personal-commands/SKILL.md` (not re-checked) | Gap. | none | new | **Drop.** Chronicler has no surface where a user's agent edits product configuration (inferred from `README.md`: a web dashboard). | — |
| **C-06** Periodic audit sweep of every guide | One commit aligns all guides with current behavior. | R01-10 | `9e9da53a` "docs: audit guides against current behavior" (not re-checked) | **Covered.** `build.py validate-docs` and `build.py vale` run in every gate. The skills `chronicler-docs-hygiene` and `diataxis-doc-review` audit on demand. | none | new | **Covered.** A schedule is a choice of when to run a skill, not a practice to decide (inferred). | — |
| **C-07** Naming rules for commits and branches | Conventional commit subjects; short branch names. | R07-3, R02-3 | `anomalyco/opencode/AGENTS.md` (R07, not re-checked). 40 of 59 pipeline subjects start `Fix #N:` (R02, not re-checked). | **Covered for commits.** 217 of the last 300 subjects use `type(scope): …` (K-40). Branches are not part of the Chronicler flow. | none | 4 (habit) | **Covered.** | — |
| **C-08** Published skills | The author publishes skills as public artifacts. | R07-18 | skills.sh page (R07, not re-checked) | Chronicler has 38 repo skills in `.agents/skills/`. | none | new | **Drop.** Publishing serves other people. It does not change how this repo works. | — |

### B. Docs organization

| ID and name | Description | Sources | Hex evidence (checked) | Chronicler state | Conflict | Linked | Proposal | Type |
|---|---|---|---|---|---|---|---|---|
| **C-09** Feature map by user capability | The docs follow what the user can do: entry points, prerequisites, behaviors and checks per capability. The map is not a roadmap. | F-3, R04-1, R04-8, R04-13, R04-14, NYS, poteto excerpt | `docs/features/`: 10 files, 61,371 words (K-15). `AGENTS.md:60` "Organize by user capability, not Rust module or UI pane" (K-15). | **Partly covered.** `docs/specs/` goes by feature and route: 31 files, 555 scenarios (K-26). Reference follows the code: 222 DOC anchors in `src/`, all to `docs/diataxis/reference/` (K-27). | `CODING_STANDARDS.md:11-15` gives each source file a DOC anchor. `docs/AGENTS.md` "Where a fact lives" places facts by layer. | 3, NYS | **Keep.** The owner finds it hard to decide how to organize the docs (map, NYS). This is the main question of G2. | in G2 |
| **C-10** Stable behavior id per behavior | Each behavior has a dotted id, for example `setup.model-card`. An id dies with its feature and is not renamed for a refactor. | R04-2 | 142 distinct ids in `docs/features/` (K-15). No code or test refers to an id (R04, not re-checked). | **Partly covered.** A scenario number (`41.1`) is stable and the validator checks it. It names a test case, not a behavior. `statechart-models` proposes rule ids paired with tests (K-44). | none | 3, 2 | **Keep.** It decides how a doc, a test and a model refer to one behavior. That overlaps `statechart-models`. | in G2 |
| **C-11** A mechanical check on the map | Hex has no check on its map, and the map drifts. The lesson: a doc that links to tests needs a check. | R04-6, R04-15 | `docs/features/README.md:31` still says Labs is "one switch, on by default" (K-16). `developer_panes_show_only_in_developer_builds` appears only in the map (K-16). `scripts/check` runs fmt, clippy and tests only (K-16). | **Covered, and stronger.** `scripts/validate_feature_spec.py` (gate step `spec-coverage`) fails a scenario with no test and a tag with no scenario. | none | 3, 1 | **Covered.** Carry it into G2 as a constraint: any adopted map gets a mechanical check. | — |

### C. Proof and verification

| ID and name | Description | Sources | Hex evidence (checked) | Chronicler state | Conflict | Linked | Proposal | Type |
|---|---|---|---|---|---|---|---|---|
| **C-12** Three proof states and a dated observed record | Keep apart: the behavior is built, a check exists, someone saw it work. The third state is a dated paragraph that names the machine, the build and what was not exercised. A checklist names what a person must observe. | F-3, R03-3, R04-3, R04-4, R04-12, R07-9 | `AGENTS.md:67` "A test's existence is not a passing run" (K-15). Dated records at `README.md:246`, `:406-411` (R04, not re-checked). | **Gap.** No doc in `docs/` contains "observed", "manually verified" or "verified by hand" (K-26). A green `python build.py` is the only proof. One plan records that a person ran checks (R03-3). | none | 3 | **Keep.** No Chronicler place records what someone saw work. This part does not depend on how the docs are organized (inferred). | in G2 (or G3) |
| **C-13** A proof limit beside each check | One sentence says what a check cannot show. | R04-5 | 17 "Proof limit" sentences: 15 in `recovery.md`, 1 in `dictation.md`, 1 in `README.md` (K-17) | **Gap.** `tests/STRATEGY.md` gives each test a tier. No doc says what a test cannot show. | none | 3 (habit) | **Keep.** It costs one sentence per check (inferred) and it stops a test name from reading as proof. | in G2 (or G3) |
| **C-14** Known gaps beside the behavior | A gaps section per feature, with "Missing evidence:" lines and issue links. | R04-7 | `recovery.md:606-672`, `dictation.md:1327-1343` (R04, not re-checked) | **Gap.** Known gaps of a feature end in closed tickets in `.scratch/_closed/` (R04, not re-checked). | none | 3 (habit) | **Keep.** A reader of a behavior does not look in closed tickets (inferred). | in G2 (or G3) |
| **C-15** Golden-image UI tests | Render each UI state and compare it to a checked-in PNG, with a tolerance and an update switch. A person looks at each changed image. The list of named scenes is the list of cases. | F-5, R05-9, R07-10, R04-9, R03-13, R07-6 | 122 PNGs in `tests/golden/`; `HEX_UPDATE_GOLDENS` at `src/rune_ui/tests.rs:1985`; tolerance 12 per channel and `width*height/500` pixels (K-18). Rule at `AGENTS.md:12-18` (K-09). | **Gap.** Browser screenshots are failure diagnostics only (`tests/test_utils/browser.rs:477-484`, K-28). The closed `ui-verification-redesign` map did not weigh image comparison (K-29). `Cargo.toml:37` has `playwright-rs = "0.9"`. That map's research notes a `to_have_screenshot` builder in it (K-29). | No rule conflicts. Risk (inferred): flake. The closed map's goal was a browser tier that is free of flakes by construction. | 5 | **Keep.** It is a real gap, and the tool may already be a dependency. The open question is cost and stability, so a cheap artifact answers it better than a talk. | prototype (G4) |
| **C-16** A design notation file whose scenes become test cases | `design/capture_capsule.sigil` declares the state model and 19 named scenes. | R03-13 | 12 of 19 scene names match golden names (R03, not re-checked). No script reads the file (R03, not re-checked). | Gap. | none | new | **Drop.** Sigil is a private tool, and nothing reads the file. Its useful idea, a named scene list as the case list, is part of C-15. | — |

### D. Landing and gates

| ID and name | Description | Sources | Hex evidence (checked) | Chronicler state | Conflict | Linked | Proposal | Type |
|---|---|---|---|---|---|---|---|---|
| **C-17** A deterministic risk sort of a diff | A script with no AI sorts a diff into `clean`, `flagged` or `dax` by path, line and size rules. The workflow runs the script from the base branch, so a patch cannot change its own policy. | F-1, R01-7, R02-4, R06-2, R07-14 | `.github/agent-policy/check.ts`, 118 lines; `.github/` is `dax`; `AGENTS.md` is `flagged`; 600 lines or 25 files (K-02). Checkout of `base.sha` (K-03). 46 `clean`, 17 `flagged`, 1 `dax` of 64 agent PRs (K-05). | **Gap.** Guardrails judge the tree, not a change. The permission system limits tool calls. | none directly. Its value depends on whether anything lands without the owner (C-19). | 1 | **Keep.** | in G1 |
| **C-18** A second model as the strict reviewer | A `flagged` diff passes only when a second model approves it. The approval is a label. | R02-5, R07-14 | `agent-policy.yml:35,44`: the job passes a `flagged` diff only with the label `agent-strict-review` (K-03). 17 PRs carry the label (K-05). | **Partly covered.** The `code-review` skill runs two `reviewer` sub-agents. Its verdict stays in the chat. | none | 1, 4 | **Keep.** | in G1 |
| **C-19** The agent lands small units itself | One PR per change, one squash commit, and the agent merges when the checks pass. A land queue verifies, builds and installs. | F-4, R02-7, R02-11, R06-1, R06-6, R07-2, R07-7 | `AGENTS.md:5-9` "merge it yourself once its checks pass" (K-09). `.hivemind.toml`: `batch_max = 4`, three `verify` commands, install and stamp (K-10). 60 of 64 agent PRs merged, median 3 s from open to merge (K-04). | **Gap, and the opposite rule.** `AGENTS.md:415` "Don't commit without explicit approval" (K-21). Median commit 377.5 changed lines, p90 3049 (K-41). | `AGENTS.md:415` and the permission system (`.pi/extensions/pi-permission-system/config.json`). | 4 | **Keep.** Only the owner can change the approval rule. A yes is larger than one session, so it leaves the map as a brief or a question for a new map (inferred). | in G1 |
| **C-20** A durable run report | Each run leaves a record outside the session: the run id, what ran, what did not run, and the verdict. | R02-2, R02-6, R02-12 | Every agent PR body has a `Run:` id and a `**Gates**` line (K-04). 13 to 18 bodies say that a part was not compiled or not run; the count depends on the pattern (K-06). | **Partly covered.** `logs/build_history.txt` records each gate run. A ticket `## Answer` records results. A review verdict stays in the session transcript. | none | 1, 4 | **Keep.** It does not need automatic landing (inferred). It could be decided first, and alone. | in G1 |
| **C-21** Change-scoped checks | A change runs only the checks for the paths it touches. A docs-only change cuts no build. | R06-3 | Path filters in `check-linux.yml`, `check-macos.yml`; `paths-ignore` in `release.yml:55-58` (R06, not re-checked) | **Gap.** The gate runs the fixed tuple `_GATE_CHECKS` of 11 checks (`build.py:1191-1203`, K-39). `build.py` has no changed-file logic (K-39). Narrow loops exist: `test-pattern`, `--test-target`. | none | new | **Keep.** The full gate costs about 145 s warm (`AGENTS.md`), and it runs the same for a docs-only change. A yes is probably one session of work in `build.py` (inferred). | grilling (G6) |
| **C-22** One exit code per stage of the check loop | `scripts/check` exits 10 for fmt, 11 for clippy and 12 for tests. | R05-15, R06-5 | `scripts/check` (K-16) | **Covered.** `build.py` has a filter (`test-pattern`, `--test-target`), so R05-15 is wrong (K-38). It returns 1 for any failure (`build.py:1581`), and the log names the failed step. | none | new | **Covered.** Agents read the step from the log, as `AGENTS.md` tells them. No caller branches on the exit code (inferred). | — |
| **C-23** A cross-platform CI matrix | Every PR builds and tests on Linux with Nix, macOS and Windows. | R06-4 | Four workflows (R06, not re-checked) | Gap. No `.github/` (K-24). | none | new | **Drop.** Chronicler is a web server with one target host (inferred from `README.md` and `ENVIRONMENT.md`). The question of a remote gate at all belongs to G1. | — |
| **C-24** Repair after the merge, not a hold before it | A platform break lands on `dev`, and a later run repairs it. | R02-8 | 11 `fix(ci)` commits, 10 dated 2026-10-31 (K-08). 8 of the 59 pipeline commits are `fix(ci)` (K-07). | Chronicler runs the full gate before each commit. | none | new | **Drop.** Hex repairs cross-OS breaks that its one-host land queue cannot see. Chronicler has one platform and runs the full gate first (inferred). G1 can use these numbers as the cost of a narrow pre-merge check. | — |
| **C-25** Channels after the merge, and an install stamp | A push to `dev` publishes a dev channel and test builds. The installed app stamps its commit. | R06-6, R06-7 | `release.yml`, `test-builds.yml` (R06, not re-checked). `stamp` in `.hivemind.toml` (K-10). | Gap. `build.py` copies `data/` and `assets/` into `target/` (R06, not re-checked). | none | habit, new | **Drop.** Chronicler ships no build to other machines. | — |
| **C-26** A release boundary for secrets | Build jobs hold no secret; sign and publish jobs check out only `.github/release/`. | R06-8 | `docs/releasing.md:11-28` (R06, not re-checked) | Gap. No release process. | none | new | **Drop.** No release path exists. R06 says the same. | — |
| **C-27** No generator in the commit path | A commit never rewrites a tracked file. | R06-9 | No hook in the repo (R06, not re-checked) | **Opposite, by choice.** The pre-commit hook regenerates four indexes and stages them (`AGENTS.md`, Documentation Index). | The hook is a written repo rule. | new | **Drop.** Hex has no generated indexes, so the comparison does not transfer (inferred). | — |
| **C-28** Intake for outside reporters | A required issue form is the only text the agents see. The run talks on the issue. A triage label diverts an issue. No outside PRs; a reporter gets credit. | R02-1, R02-9, R02-10, R07-12, R07-15 | Issue templates, issue timelines and `CONTRIBUTING.md:2-10` (R02, R07, not re-checked) | Triage roles are covered (`docs/agents/triage-labels.md`). The rest is a gap. | none | 4, new | **Drop.** These parts serve outside reporters, and Chronicler has none (inferred). The triage roles are covered. | — |
| **C-29** Build jobs on named remote workers | HiveMind runs cargo jobs on named hosts. | R07-8 | `hivemind run --worker` in rune `scripts/rune-dev` (R07, not re-checked). The tool returns 404. | `scripts/build_slot.py` holds one cargo step at a time on one machine. | none | new | **Drop.** The tool is private. The build slot answers Chronicler's local contention. | — |

### E. Code and test style

| ID and name | Description | Sources | Hex evidence (checked) | Chronicler state | Conflict | Linked | Proposal | Type |
|---|---|---|---|---|---|---|---|---|
| **C-30** No seam until two real adapters or a test needs one | A trait appears only for a second adapter or a test substitute, and its doc names that adapter. Otherwise a parameter (`_with`, `_in`) is the seam. | F-7, R01-6, R05-11, R05-12, R07-11 | `AGENTS.md:1468-1469`, `CONTRIBUTING.md:63` (K-13) | **Covered in substance, not in full.** `codebase-design/SKILL.md:65` and `architecture.md:14` state the rule (K-30). `LlmProvider` and `TextChecker` pass it. `AfterNarrationAgent` and `EndOfJobAgent` each have one impl and no test impl, held as `Option<Box<dyn …>>` in `runner.rs:16-17` (K-31). | none | 7 | **Keep.** Two traits break the rule (inferred from K-31). The clock seam of C-31 is the rule's live test case. | in G5 |
| **C-31** Env and clock reads behind one door, with a lint ledger | `clippy.toml` bans `std::env::var` and `SystemTime::now`. A lint that is not yet clean is turned off with its remaining error count. | F-6, R05-7, R05-8, R07-5 | `Cargo.toml` lists 7 lints turned off, each with "N errors left (t1003)" (K-19). The ban is off in Hex (map, corrected). | **Gap.** `clippy.toml` sets thresholds only. `src/lib.rs` denies lints outright, so no ledger exists. 3 `env::var` sites in `src/`, 6 in tests; 17 clock-read lines in 15 `src/` files; 3 tests read the clock apart from the code under test (K-32). | none | 6 | **Keep.** The three clock-dependent tests fail if the date changes between two reads (K-32). The map already lists three options. | in G5 |
| **C-32** Test names carry the invariant's number; a boundary triad | A test name is a sentence of behavior with its number. A test runs `[n-1, n, n+1]` around the value. | R05-1, R05-2 | `src/suppression.rs:2841` loops `[299, 300, 301]` (R05, not re-checked) | **Partly covered.** `tests/STRATEGY.md:245-247`: a unit test name describes the behaviour (K-37). 1451 test functions start `test_` (K-37). | none | 2 | **Keep, if G2 keeps C-02.** The number in the name and the triad are the test side of an invariant with a value. Without C-02, drop it. | in G2 |
| **C-33** Long why-comments and section banners | Comments of six lines or more name a measured fact; `// ---` banners split a file. | R05-4 | `src/rune_hud/hud_placement.rs:11-25` (R05, not re-checked) | **Opposite, by choice.** Five consecutive comment lines fail the gate, and `// ===` banners fail it (K-34). `// ---` banners would pass (K-34). | `tests/infrastructure/guardrails/style.rs` (`check_long_comment_runs`) | new | **Drop.** Chronicler moves the reason to Explanation docs through the DOC anchor. No evidence shows that Chronicler loses facts this way (inferred). | — |
| **C-34** A module doc says why the module exists | The `//!` block names the reason and the module's partners. | R05-3 | 200 of 256 files carry `//!` (R05, not re-checked) | **Covered differently.** `CODING_STANDARDS.md:11-15`: a DOC anchor and a one-line summary. | none | new | **Covered.** | — |
| **C-35** Capability folders with code in the parent file | `src/foo.rs` holds code and declares `src/foo/`. Only 4 `mod.rs` files. | R05-5, Fo | Commit `38de838a` "Fold dictation_* into dictation/" (R05, not re-checked) | **Covered for the folder.** The parent-file form is barred: `check_mod_purity` allows no code in `mod.rs` (K-35). | `check_mod_purity` | habit | **Covered** (folders). Drop the parent-file form: it conflicts with a guardrail and gives no gain that was asked for (inferred). | — |
| **C-36** One string error type for the crate | `color_eyre` with string context. Few error enums. | R05-6 | 80 files import `color_eyre` (R05, not re-checked) | **Opposite, by choice.** `thiserror` enums per layer, mapped to HTTP status. | none | new | **Drop.** The HTTP adapter maps error arms to status codes. Strings would lose that map. | — |
| **C-37** Inline test modules | Unit tests sit in `#[cfg(test)] mod tests` in the same file. | R05-16 | 241 files (R05, not re-checked) | **Opposite, by choice.** `scripts/check_test_structure.py` fails an inline test module (K-36). | `check_test_structure.py` | new | **Drop.** Deliberate Chronicler rule. | — |
| **C-38** Test-only code in production types | `#[cfg(test)]` methods, `for_test()` constructors, and `cfg!(test)` function pointers. | R05-13, R05-14 | `src/suppression.rs:1545` (R05, not re-checked) | **Opposite, by choice.** `antipattern-checker/SKILL.md:16` names "a test-only type threaded through production code" a symptom refactor (K-47). | `antipattern-checker` | new | **Drop.** Chronicler substitutes at the port or by a `_with` parameter. | — |
| **C-39** Large files with no length cap | `src/suppression.rs` has 6120 lines. | Fo | R05 (not re-checked) | **Opposite, by choice.** 2000 non-blank lines fail (`check_file_length`, K-35). | `check_file_length` | new | **Drop.** Deliberate Chronicler rule. | — |
| **C-40** nextest kills and names a hung test | A slow timeout ends a hung test and names it. | Fo | `.config/nextest.toml` (findings, not re-checked) | **Covered.** `.config/nextest.toml` sets `slow-timeout` with `terminate-after = 1` (checked in this session). | none | — | **Covered.** | — |
| **C-41** An adapter test uses a real fixture at the boundary | A loopback server or a temp directory, not a mock trait. | R05-10 | `src/desktop/opencode_service.rs:396` (R05, not re-checked) | **Covered.** `src/test_support/chat_completions_mock.rs` is a one-request endpoint on a local port for provider tests (K-46). | none | 7 | **Covered.** | — |

### F. Planning and tracking

| ID and name | Description | Sources | Hex evidence (checked) | Chronicler state | Conflict | Linked | Proposal | Type |
|---|---|---|---|---|---|---|---|---|
| **C-42** One ranked work list and a block of live constraints | `ROADMAP.md` is the one work list. `## Current Work` in `AGENTS.md` keeps the constraints of the week visible. | R01-5, R03-10, R03-11, Fo | `AGENTS.md:1471-1486`: "`ROADMAP.md` is the authoritative work list" and a dated release freeze (K-13). 51 commits touch `ROADMAP.md` (K-20). | **Gap.** No `ROADMAP.md` and no live-constraints block (K-24). The work is in 11 effort folders with 34 open tickets, plus 5 plans (K-42, K-25). | none | habit | **Keep.** No single place ranks the 34 open tickets (inferred). | in G7 |
| **C-43** Plan conventions | The Status line names the built steps. A numbered landing order has a done mark on each step. A limits section says what the plan did not prove. | R03-1, R03-2, R03-4 | `docs/plans/capture-in-hex.md:3-9`, `:372-393`; `engine-service.md:193-205` (R03, not re-checked). 18 plans (K-20). | **Partly covered.** All 5 Chronicler plans have a Status line (K-25). Plans have phases. No plan has a limits section (R03). | none | new | **Keep.** The limits section is the planning form of C-13 (inferred). | in G7 |
| **C-44** A decision record folder | A document that holds only a decision: the choice, then the rejected alternatives. | R03-8, R07-17 | `docs/plans/rune-dependency.md:3-40` (R03, not re-checked) | **Partly covered.** `ADR-FORMAT.md` names `docs/adr/` and says to create it lazily. `docs/adr/` does not exist (K-24). Decisions live in ticket answers, map Notes and plan tables. | none | 2, new | **Keep.** Dax Raad's point (R07-17): agents undo a choice when no record says why the alternatives lost. This touches G2. | in G7 |
| **C-45** A prototype keeps its decisions beside the variants | `DECISIONS.md` and `SYNTHESIS.md` sit next to the prototype code. | R03-12 | `prototypes/app-shell/` (R03, not re-checked) | **Covered differently.** The `/wayfinder` resolution records the answer in the ticket, and the `/prototype` skill asks for a durable record. | none | new | **Covered.** | — |
| **C-46** A finished plan stays in place | The plan keeps its path; its Status is rewritten. | R03-9 | `docs/plans/rune-ui.md:3` (R03, not re-checked) | **Covered differently.** A finished plan moves to `old-docs/archived-plans/` with its build commits (K-25). | none | — | **Covered.** | — |
| **C-47** Recommendations, review records and outward links in plans | Each open question has a recommendation; a review record; links to files, tests and commits. | R03-5, R03-6, R03-7 | `docs/plans/transcription-quality.md:173-186` (R03, not re-checked) | **Covered.** `docs/plans/generic-chat-completions-connections.md` has a Decisions table, a findings table and Facts (R03). | none | — | **Covered.** | — |
| **C-48** A ticket id in the commit subject | `(tNNNN)` in a subject points at a tracker. | R03-14, R07-4 | 24 unique ids (map, corrected) | **Covered.** 56 of the last 300 subjects name a ticket or an issue, for example `(prompt-flexibility ticket 20)` (K-40). R03 said no commit names a ticket: wrong. | none | new | **Covered.** A rule to name the ticket always is a one-line edit, if the owner wants it. | — |
| **C-49** Short commit bodies | Median commit body: 3 lines. | Fo | findings (not re-checked) | Not measured. | none | habit | **Drop.** No source connects body length to an outcome. | — |

### G. Product and working habits

| ID and name | Description | Sources | Hex evidence (checked) | Chronicler state | Conflict | Linked | Proposal | Type |
|---|---|---|---|---|---|---|---|---|
| **C-50** Labs | New, unproven and user-requested features ship off by default behind a toggle. No master switch. | F-8, R07-13 | `AGENTS.md:33-34`, `:1244-1249` (K-14) | **Gap.** No match for "feature flag", "labs" or "experimental" in `src/`, `docs/diataxis/` or `CONTEXT.md` (K-43). | none | 8 | **Drop, unless G1 keeps automatic landing.** Hex uses Labs to protect users of a public release from features that land without a person. Chronicler has no public release (inferred). If G1 keeps C-19, Labs returns as a part of G1. | — |
| **C-51** One long session per project; all work goes to subagents | The orchestrator session does no work itself. | R07-16 | X post of Dax Raad via AI Coding Daily (R07, not checkable) | **Opposite rule.** `AGENTS.md`: "Delegate a task only when it is substantial, and do a short task yourself" (K-45). | `AGENTS.md`, Subagents and delegation | new | **Drop.** It is Dax Raad's personal habit, not a Hex repo rule. The Chronicler rule rests on the cost of loading context again. | — |
| **C-52** The product feeds the agent loop | The author's products serve coding agents. | R07-19 | `kitlangton.com` list (R07, not re-checked) | Not applicable. | none | new | **Drop.** Chronicler is a game engine, not an agent tool. | — |

## 3. Groups: the proposed decision tickets

Each group is one proposed decision ticket. The owner already joined candidates 1 and 4 (G1), and candidates 2 and 3 with docs as feature maps (G2).

| Group | Practices | Candidates | Proposed type | Why this type | Likely end |
|---|---|---|---|---|---|
| **G1. Landing and gates** | C-17, C-18, C-19, C-20 | 1, 4 | grilling | The first question is whether anything lands without the owner (`AGENTS.md:415`). Only the owner can answer it. C-17 and C-18 only matter after a yes (inferred). C-20 does not depend on the answer, so the grilling can take it first. | A question for a new map if C-19 is kept. A follow-on task if only C-20 is kept (inferred). Labs (C-50) joins if C-19 is kept. |
| **G2. Spec and docs as feature maps** | C-01, C-02, C-09, C-10, C-12, C-13, C-14, C-32 | 2, 3, NYS | prototype | The owner finds the doc organization hard to decide (map, NYS). A rough feature-map page for one Chronicler capability gives something concrete to react to. It shows ids, invariants with values, the proof states, proof limits and known gaps on one page. Constraint from C-11: an adopted map gets a mechanical check. | A question for `statechart-models` (rule ids) or `code-concepts`, or a new map (inferred). |
| **G3. Proof records** (optional split of G2) | C-12, C-13, C-14 | 3 | grilling | A record of what someone saw work does not depend on how the docs are organized (inferred). If the owner splits it off, it can be decided before the slower G2. | A follow-on task: a place and a format for the record (inferred). |
| **G4. Golden-image UI tests** | C-15 | 5 | prototype | The open question is cost and stability, not policy. One screenshot test of one dashboard part in a fixed state, run five times, answers both. If the `to_have_screenshot` builder is missing in `playwright-rs` 0.9, change the type to research (part 5). | A brief, or drop with the measured reason. |
| **G5. Seams, env and clock** | C-30, C-31 | 6, 7 | grilling (with `/codebase-design`) | The map lists three options for the env and clock ban. The three clock-dependent tests (K-32) are a "test needs one" case for the seam rule. The two hook traits (K-31) are the opposite case. | A follow-on task of one session (inferred). |
| **G6. Gate scope** | C-21 | new | grilling | A short decision: which steps a docs-only or scripts-only change may skip, and what still runs. | A follow-on task in `build.py` (inferred). |
| **G7. Planning and tracking** | C-42, C-43, C-44 | habits | grilling | Three conventions, each small. They share one question: where the current state of the work lives. C-44 touches G2 through the source-of-truth question. | Follow-on tasks, or edits to `docs/agents/` and the plan template (inferred). |

## 4. Claim checks

Checks ran on 2026-11-01 against the clones in `tmp/hex-compare/`, the cached GitHub JSON in `tmp/hex-compare/github/`, and this repo. No new GitHub API request was needed. The four claims that the ticket excludes were not checked again.

### Hex claims

| ID | Claim | Source | Check | Result |
|---|---|---|---|---|
| K-01 | The clones are at the commits of the findings asset. | findings, Sources | `git rev-parse --short=12 HEAD` in both clones | **Confirmed.** `7609574ca92c` and `3e300721b878`. |
| K-02 | `check.ts` sorts a diff into `clean`, `flagged` or `dax` with no AI. `.github/` is `dax`; `AGENTS.md` is `flagged`; size rule 600 lines or 25 files. | F-1, R01-7, R02-4 | `.github/agent-policy/check.ts:1-50`, `:95-110` | **Confirmed.** Correction: the file has 118 lines, not "about 130". |
| K-03 | The workflow runs `check.ts` from `base.sha`, only for `agent/*` branches; a `flagged` diff passes only with the label `agent-strict-review`. | R02-4, R02-5, R06-2, R07-14 | `.github/workflows/agent-policy.yml:3-45` | **Confirmed.** |
| K-04 | 64 agent PRs: 60 merged, 4 closed unmerged; median 3 s from open to merge. Each body has a `Run:` id and a `**Gates**` line. | R02 | Python over `pulls_p1.json` to `pulls_p5.json` (498 PRs) | **Confirmed.** |
| K-05 | Verdicts: 47 `clean`, 17 `flagged`, 1 `dax`. 17 PRs carry `agent-strict-review`. | R02-4 | Regex `policy (clean\|flagged\|dax)` on each agent PR body; label count | **Wrong.** 46 `clean`, 17 `flagged`, 1 `dax`. Each of the 64 bodies has exactly one verdict. The label count of 17 is confirmed. |
| K-06 | 16 of 64 bodies say that the change was not compiled, not run or not re-run. | R02, gates | Regex on the `**Change**` text of each body | **Confirmed in substance.** 13 to 18, by the pattern. The exact 16 is not reproduced. |
| K-07 | 59 pipeline commits; 8 of them `fix(ci)`; no pipeline commit reverted. | R02-8, R02 | `git log --grep='Co-authored-by: Hex issue pipeline'`; `git log` subjects that start `Revert` | **Confirmed.** Correction: 4 subjects start `Revert`, not 5 (R06 says 4). None of the 4 reverts a pipeline commit. |
| K-08 | 11 `fix(ci)` commits in the repo; 10 dated 2026-10-31. | R02, R06 §4 | `git log --date=short` | **Confirmed.** |
| K-09 | `AGENTS.md:3-11`: PRs only; the agent merges its own PR when the checks pass. `:12-18`: a UI change regenerates the goldens in the same PR. | R06-1, R07-2, R07-10 | `sed -n '3,18p' AGENTS.md` | **Confirmed.** |
| K-10 | `.hivemind.toml`: branch `dev`, `batch_max = 4`, three `verify` commands with `cargo test --locked --workspace --lib`, install to `/Applications/Hex.app`, a stamp file. | F-4, R02-7, R06-6, R07-7 | Read the whole file (17 lines) | **Confirmed.** |
| K-11 | `AGENTS.md` has 1486 lines; `## Invariants` from :527 has 73 bullets, 31 with a number; "300 ms discard" at :605-606. | R01-1, R01-2 | `wc -l`; `awk` on the section, bullet split, digit regexes; `grep -n '300 ms'` | **Confirmed in substance.** 1486 lines, 73 bullets, "300 ms discard" at :606. The numeric count depends on the pattern: 23 bullets have a number with a unit, 33 have any digit. |
| K-12 | Only 7 of 225 `AGENTS.md` commits touch nothing else. | R01-3 | `git log -- AGENTS.md`, file count per commit | **Confirmed in substance.** 8 of 228. The count depends on how `git log` follows the path. |
| K-13 | `## Current Work` at :1471-1486 names `ROADMAP.md` the authoritative list and holds a dated freeze. The seam rule is at :1468-1469 and `CONTRIBUTING.md:63`. | R01-5, R01-6, R03-11, R07-11 | `sed -n '1466,1486p' AGENTS.md`; `grep -n` in `CONTRIBUTING.md` | **Confirmed.** |
| K-14 | Labs: new, unproven and user-requested features land off by default; no master switch. | F-8, R07-13 | `grep -n` for "land in Labs" and "master switch" | **Confirmed** at `AGENTS.md:33-34` and `:1244-1248`. |
| K-15 | `docs/features/`: 10 files, 61,371 words, 142 ids. `AGENTS.md:67` "A test's existence is not a passing run". `:57` "Update the affected map in the same change". | F-3, R04 | `ls`, `wc -w`, id regex `// name.name`, `grep -n`, `sed -n '55,60p'` | **Confirmed.** |
| K-16 | The map drifts: `README.md:31` says Labs is one switch; `developer_panes_show_only_in_developer_builds` exists only in the map; `scripts/check` has no doc step. | R04 §8, R04-15 | `sed -n '31p'`; `grep -rn` in `docs` and `src`; read `scripts/check` | **Confirmed.** The other 10 stale names of R04 were not re-run. |
| K-17 | 17 "Proof limit" sentences, 15 in `recovery.md`. | R04-5 | `grep -c 'Proof limit' docs/features/*.md` | **Confirmed.** 15 + 1 + 1. |
| K-18 | 122 golden PNGs; `HEX_UPDATE_GOLDENS` rewrites them; tolerance 12 per channel and `width*height/500` pixels. | F-5, R05-9 | `ls tests/golden \| wc -l`; `src/rune_ui/tests.rs:1985`, `:2005-2015` | **Confirmed.** |
| K-19 | `Cargo.toml` keeps a ledger of lints turned off, each with its remaining error count. | R05-7 | `sed -n '160,190p' Cargo.toml` | **Confirmed.** 7 entries, each with "N errors left" and `t1003`. |
| K-20 | 18 plans in `docs/plans/`; 51 commits touch `ROADMAP.md`. | R03 | `ls docs/plans`; `git log -- ROADMAP.md \| wc -l` | **Confirmed.** 18 `.md` files and one folder. |

### Chronicler claims

| ID | Claim | Source | Check | Result |
|---|---|---|---|---|
| K-21 | `AGENTS.md:414` "Don't commit without explicit approval". | R02-7, R06, R07-2 | `grep -n` | **Confirmed** at line 415. The line moved by one. |
| K-22 | `docs/AGENTS.md:149` "Values stay in the code"; `:155-158` "No negative explaining". | R01-2 | `grep -n`, `sed -n '149,158p'` | **Confirmed.** Note: `:149` lets invariants into docs and exempts plans. It bars constants and caps. |
| K-23 | `docs/AGENTS.md:38` "A behaviour change edits its spec and its test". | R01-3 | `grep -n` | **Confirmed.** The same line says a Diátaxis doc changes when a part, an invariant or a name is added, removed or renamed. So it covers removal for `docs/`. |
| K-24 | No `.github/`, no `ROADMAP.md`, no `docs/adr/`; `ADR-FORMAT.md` names `docs/adr/`. | R02, R03-8, R03-10 | `ls -d`; `grep -n docs/adr` | **Confirmed.** `ADR-FORMAT.md:5` says to create the folder lazily. |
| K-25 | 5 plans in `docs/plans/`, each with a Status line; 4 files in `old-docs/archived-plans/`. | R03-1, R03-9 | `ls`; `grep -l Status` | **Confirmed.** |
| K-26 | `docs/specs/`: 31 files, 554 scenarios. No "observed", "manually verified" or "verified by hand" in `docs/`. | R04 §10 | `SCENARIO_RE` of `validate_feature_spec.py:42` over all specs; `grep -rliE` | **Confirmed.** 31 files and 555 numbered scenarios today. No match for the three phrases. |
| K-27 | Reference docs follow the code through DOC anchors. | NYS | `rg -o '\[DOC: docs/diataxis/([a-z]+)/' src` | **Confirmed.** 222 anchors, all to `docs/diataxis/reference/`. |
| K-28 | Browser screenshots are failure diagnostics only. | F-5, R05-9 | `grep -n screenshot tests/test_utils/browser.rs`; `grep -rn to_have_screenshot src tests` | **Confirmed.** `browser.rs:477-484` writes `tmp/screenshots/` on failure. No screenshot assertion exists. |
| K-29 | The closed `ui-verification-redesign` map never weighed image comparison. | R05-9 | `grep -rniE 'screenshot\|golden\|pixel\|visual regression\|image compar\|snapshot'` over its map, issues and assets | **Confirmed.** Its question was UI readiness (`map.md`, Destination and Notes). Its tickets name screenshots only as failure output. Addition: `assets/htmx-verification-research.md:127-129` notes a `to_have_screenshot` builder in `playwright-rs` 0.9.0, and `Cargo.toml:37` has `playwright-rs = "0.9"`. |
| K-30 | The seam rule is in `codebase-design/SKILL.md:65`; `architecture.md:14` rejects a phantom trait. | R01-6, R05-12 | `sed -n` on both lines | **Confirmed.** |
| K-31 | The port layer follows the seam rule. The findings asset says this was not checked. | F-7 | `rg 'pub trait'` in `src/` without tests; `impl <Trait> for` in `src/` and `tests/` | **New check: partly.** `LlmProvider` has `ChatCompletionsBackend`, `MockBackend`, `ScriptedModel` and 7 test impls. `TextChecker` has `HarperTextChecker` and 5 test impls. Both pass. `AfterNarrationAgent` and `EndOfJobAgent` (`src/application/agents/trait_def.rs`) each have one impl (`QuantifierAgent`, `OptionsGeneratorAgent`) and no test impl. `runner.rs:16-17` holds them as `Option<Box<dyn …>>`. By the Hex rule, these two traits are hypothetical seams (inferred). |
| K-32 | 3 `env::var` sites in `src/` and 6 in tests; about 20 clock reads; one test needs a substitute clock. | findings, Env and clock reads | `grep -rn 'env::var'`; `grep -rnE 'Utc::now\|Local::now\|SystemTime::now'` without `_tests.rs` and `test_support`; read `game_name.rs:6-7` and the tests | **Partly wrong.** Env: confirmed (`logging.rs:14`, `settings.rs:385`, `:399`; 6 in `tests/` and `test_support`). Clock: 17 lines in 15 files. Tests: **three**, not one. `game_tests.rs:15` and `:23` compute the date apart from `generate_game_name`. `catalogue_tests.rs:162` computes it apart from `create_game` (inferred: `create_game` names the Game through the clock). Each test fails if the date changes between the two reads. |
| K-33 | `Settings::resolve_api_key_with` takes the lookup as a parameter (`settings.rs:385`). | F-6, R05-11 | `grep -rn` | **Confirmed.** |
| K-34 | Five consecutive comment lines fail; `// ===` banners fail; both call `panic!`. | R05-4 | `style.rs` `check_long_comment_runs`, `check_separator_comments`; `mod.rs` `assert_violations` | **Confirmed.** The `warn` severity still fails, because `assert_violations` panics on any violation. Addition: the banner check matches `// ===` only, so Hex's `// ---` banners would pass. |
| K-35 | No code in `mod.rs`; 2000 non-blank lines per file at most. | R05-5, R05 | `structure.rs` `check_mod_purity`, `check_file_length` | **Confirmed.** |
| K-36 | `check_test_structure.py` fails an inline test module. | R05-16 | Read `scripts/check_test_structure.py:1-15` | **Confirmed.** |
| K-37 | `tests/STRATEGY.md:247`: a unit test name describes the behaviour; 1424 names start `test_`. | R05-1 | `sed -n '245,249p'`; `grep -rhoE 'fn test_' src tests` | **Confirmed.** 1451 names today. |
| K-38 | `build.py` has no filter argument and no per-stage exit code. | R05-15 | `grep -n` in `build.py`; `AGENTS.md` command table | **Wrong in part.** `python build.py test-pattern "<pattern>"` with `--test-target` is the filter (`build.py:449`). No per-stage code: confirmed, `build.py:1581` returns 1 for any failed step. R06-5 states this correctly. |
| K-39 | The gate runs 11 fixed checks with no path filter. | R06-3 | `build.py:1191-1203`; search for changed-file logic | **Confirmed.** |
| K-40 | R03-14: "A commit names no ticket". R02-3: commit subjects carry a scope and the ticket. | R03-14, R02-3 | `git log -300 --format=%s`, two regexes | **R03-14 is wrong.** 56 of the last 300 subjects name a ticket or an issue. 217 of 300 use `type(scope): …`. R02-3 is correct. |
| K-41 | Median commit 382 changed lines, p90 3049. | findings, Numbers | `git log -300 --no-merges --numstat` | **Confirmed.** 377.5 and 3049 today. |
| K-42 | The work list is 10 open maps with 97 open tickets. | R03 summary, R03-10 | `Status:` line of each ticket file outside `.scratch/_closed/` | **Wrong.** 11 effort folders hold 98 ticket files: 34 open, 3 claimed, 3 `needs-triage`, 1 closed, 57 resolved. R03 counted ticket files, not open tickets (inferred). R02's "34 open" is correct. |
| K-43 | Chronicler has no feature flags. | F-8, R07-13 | `grep -rniE 'feature.?flag\|\blabs\b\|experimental'` in `src/`, `docs/diataxis/`, `CONTEXT.md` | **Confirmed.** No match. |
| K-44 | `statechart-models` proposes rule ids paired with tests; `code-concepts` is a descriptive catalogue. | map, NYS; R04 §10 | Read both maps | **Confirmed.** `statechart-models/map.md:7`, `:30`; `code-concepts/map.md`, Destination. |
| K-45 | The Chronicler delegation rule: delegate only substantial work. | R07-16 | `AGENTS.md`, Subagents and delegation | **Confirmed.** |
| K-46 | `src/test_support/chat_completions_mock.rs` is a local-port endpoint for provider tests. | R05-10 | The summary line in the `AGENTS.md` Structure block | **Confirmed from the summary line.** The source was not read. |
| K-47 | `antipattern-checker/SKILL.md:16` names a test-only type in production code a symptom refactor. | R05-14 | `sed -n '16p'` | **Confirmed.** |

### Not checkable

| Claim | Source | Why |
|---|---|---|
| Dax Raad's statements on session shape and source of truth | R07-16, R07-17 | No X access. The quotes come through AI Coding Daily, which links and dates each post. C-51 is a drop, so no proposal depends on it. |

### Not checked

These claims were not checked again, because no proposal turns on them or because the cost was high:

- R04: the other 10 of 11 stale check names, and the 66% co-change rate of source and map commits.
- R02: the 6.7-minute median from issue to merge, and the issue timelines.
- R03: the 12 of 19 `.sigil` scenes that match goldens (C-16 is a drop).
- R05: the Hex test-name and boundary-triad lines (C-32 rests on Chronicler's own state).
- R06: the workflow path filters and the CI durations (C-21 rests on Chronicler's own state).
- R01: the 92 lines with a negation in the Invariants range.
- The size of Hex: the findings asset says 263k Rust lines in `src/` and `crates/`, and R05 says 190k lines. The difference is likely `crates/` (inferred). No proposal depends on it.
- The `to_have_screenshot` builder in the `playwright-rs` 0.9 source. Only the closed map's note was checked (K-29). G4 should check it first.

## 5. Depth: the gaps that the assets name

Each asset ends with "What I could not find or did not read". This part groups those gaps by theme and says whether a gap could change a proposal.

| Theme | Gap | Asset | Could it change a proposal? |
|---|---|---|---|
| A | `~/code/control/AGENTS.md` is private, so Hex's real workflow rules are unknown. | R01, R03, R04, R07 | **No.** C-04 is a drop for a Chronicler reason. C-19 turns on `AGENTS.md:415`, not on Hex's private rules. |
| A | R01 read 15 of 225 `AGENTS.md` diffs for the reason behind a rule. | R01 | **No.** C-03 is covered by `docs/AGENTS.md:38`. |
| A | R01 read about 55 lines of the 662-line Invariants range. | R01 | **No.** K-11 counted all 73 bullets mechanically. |
| A | No one measured which Chronicler files a session loads. | R01 | **Yes, for C-01 in G2.** The cost of one spec file depends on the size that every session already loads. A known lower bound: `AGENTS.md` is 29,524 B, and Hex's is 100,349 B. |
| B | R04 read 5 of 10 feature files in part and skipped the other 5. | R04 | **No.** The shape is clear from `README.md`, `recovery.md` and `dictation.md`. |
| B | R04 read one Chronicler spec in part. | R04 | **No.** A search of all 31 specs found no proof or gap records (K-26). |
| B | R03 did not read `docs/research/` or `docs/features/`. | R03 | **No.** R04 covers `docs/features/`. |
| C | No record proves that a person looks at each changed golden image. | R05 | **No** for keep. **Yes** for the G4 prototype design: it must say who looks at a changed image. |
| C | R04 did not read the golden images or the recorded scenes. | R04 | **No.** |
| C | R03 found no specification of the `.sigil` format. | R03 | **No.** C-16 is a drop. |
| C | The `to_have_screenshot` builder was not checked in the crate (this asset). | this asset | **Yes, for the type of G4.** If the builder is missing, G4 becomes research. |
| D | The pipeline code and prompts are not public. The strict reviewer's criteria are unknown. | R02, R07 | **No** for keep. **Yes** for the design of C-18 if G1 keeps it: Chronicler would write its own criteria. |
| D | The required status checks and the land queue program are not in the repo. | R06, R07 | **No.** |
| D | The CI durations of today are unknown. | R06 | **No.** C-23 is a drop for a platform reason. |
| D | R06 did not read the 670 job lines of `release.yml`. | R06 | **No.** C-26 is a drop. |
| D | The meaning of `needs-design` and `needs-repro`, and who closes a failed PR, are inferred. | R02 | **No.** C-28 is a drop. |
| D | R02 did not read the issues before 2026-10-29. | R02 | **No.** They predate the pipeline. |
| D | R07 did not transcribe the podcast episodes of Kit Langton and Dax Raad. | R07 | **Low.** They could describe the pipeline. That adds detail to G1 and changes no keep or drop (inferred). |
| E | R05 did not read `crates/`, `prototypes/`, `design/`, `site/` or the SDK. | R05 | **No.** |
| E | R05 did not run clippy, so the "85 errors left" count is the comment's claim. | R05 | **No.** The ledger method of C-31 does not depend on the count. |
| E | R05 found no Hex review checklist. | R05 | **No.** |
| F | R03 found no tracker for the `tNNNN` ids. | R03, R07 | **No.** C-48 is covered. |
| F | R03 read `ROADMAP.md` from line 98 by headings only. | R03 | **No.** The shape of the file is clear. |
| G | No X access; the quotes come through a newsletter. | R07 | **No.** C-51 is a drop. |

Summary of depth (inferred): three gaps can change a proposal. Two change only the design or the type of a kept practice (G4, C-18). One adds a measure to G2 (C-01). No gap blocks a keep or a drop. No area looks too shallow for its proposal. The owner judges this in the grilling.

## 6. Source index

Each source row and the practice that holds it.

| Source | Rows → practice |
|---|---|
| findings | F-1 → C-17; F-2 → C-01, C-02; F-3 → C-09, C-12; F-4 → C-19; F-5 → C-15; F-6 → C-31; F-7 → C-30; F-8 → C-50. Fo: nextest → C-40; large modules → C-39; capability folders → C-35; `ROADMAP.md` and `## Current Work` → C-42; short commit bodies → C-49 |
| R01 | 1 → C-01; 2 → C-02; 3 → C-03; 4 → C-03; 5 → C-42; 6 → C-30; 7 → C-17; 8 → C-04; 9 → C-05; 10 → C-06 |
| R02 | 1 → C-28; 2 → C-20; 3 → C-07; 4 → C-17; 5 → C-18; 6 → C-20; 7 → C-19; 8 → C-24; 9 → C-28; 10 → C-28; 11 → C-19; 12 → C-20 |
| R03 | 1 → C-43; 2 → C-43; 3 → C-12; 4 → C-43; 5 → C-47; 6 → C-47; 7 → C-47; 8 → C-44; 9 → C-46; 10 → C-42; 11 → C-42; 12 → C-45; 13 → C-16, C-15; 14 → C-48 |
| R04 | 1 → C-09; 2 → C-10; 3 → C-12; 4 → C-12; 5 → C-13; 6 → C-11; 7 → C-14; 8 → C-09; 9 → C-15; 10 → C-03; 11 → C-02; 12 → C-12; 13 → C-09; 14 → C-09; 15 → C-11 |
| R05 | 1 → C-32; 2 → C-32; 3 → C-34; 4 → C-33; 5 → C-35; 6 → C-36; 7 → C-31; 8 → C-31; 9 → C-15; 10 → C-41; 11 → C-30; 12 → C-30; 13 → C-38; 14 → C-38; 15 → C-22; 16 → C-37 |
| R06 | 1 → C-19; 2 → C-17; 3 → C-21; 4 → C-23; 5 → C-22; 6 → C-19, C-25; 7 → C-25; 8 → C-26; 9 → C-27 |
| R07 | 1 → C-04; 2 → C-19; 3 → C-07; 4 → C-48; 5 → C-31; 6 → C-15; 7 → C-19; 8 → C-29; 9 → C-12; 10 → C-15; 11 → C-30; 12 → C-28; 13 → C-50; 14 → C-17, C-18; 15 → C-28; 16 → C-51; 17 → C-02, C-44; 18 → C-08; 19 → C-52 |
| map | NYS (docs as feature maps) → C-09; poteto excerpt → C-09 |

## 7. Corrections for the coordinator

The ticket bars edits to the research assets, the findings asset and `map.md`. These corrections are for the coordinator to move:

1. **Findings asset, Env and clock reads.** Three tests read the clock apart from the code under test, not one: `game_tests.rs:15`, `:23` and `catalogue_tests.rs:162` (K-32). The clock reads are 17 lines in 15 files ("about 20" holds).
2. **Findings asset, practice 7.** The port layer was checked: `AfterNarrationAgent` and `EndOfJobAgent` each have one impl and no test impl (K-31).
3. **Research 02.** 46 `clean` verdicts, not 47 (K-05). 4 subjects start `Revert`, not 5 (K-07).
4. **Research 03.** Commits do name tickets: 56 of the last 300 subjects (K-40). The work list is 34 open tickets in 11 effort folders, not 97 open tickets in 10 maps (K-42).
5. **Research 05.** `build.py` has a filter argument, `test-pattern` (K-38).
6. **Findings asset, practice 1.** `check.ts` has 118 lines (K-02).
