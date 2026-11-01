# Research 04: the Hex feature map and its proof records

Status: reference

Ticket: [04 Research the Hex feature map and its proof records](../issues/04-research-feature-map.md)

Sources: [anomalyco/hex](https://github.com/anomalyco/hex) at `7609574ca92c` (2026-10-31), clone in `tmp/hex-compare/anomalyco-hex/`; this repo for the Chronicler state.

## Summary

- Hex keeps one feature map in `docs/features/`: 10 files, 61,371 words. `README.md` is the index and holds the product map (known: `wc -w` on the clone).
- The map is organized by what the user can do. Each behavior has a stable dotted id, written as a `//` comment inside a `ts` tree diagram (`AGENTS.md:49-70`).
- The map keeps three states apart: the behavior is built, a check exists, and someone saw it work. Checks are "locators" until a dated executed result sits beside them (`docs/features/README.md:10-11`).
- No code or test refers to a behavior id. The map names source paths, test function names and UI scene names instead (known: search of `src/`, `crates/`, `tests/`, `scripts/`, `.github/`).
- Nothing checks the map. `scripts/check` and the CI workflows run no doc check, and `docs/features/` is not a flagged path in `.github/agent-policy/check.ts` (known).
- The map drifts. 3 concrete drifts are in [Drift that no check catches](#drift-that-no-check-catches), and no check catches any of them.
- Chronicler covers the spec-to-test link by a gate step, and it covers this map's placement rule in `docs/AGENTS.md`. It has no record of what someone saw work, no proof limit beside a test, and no dated evidence (known: read of `docs/specs/`, `docs/diataxis/`, `tests/STRATEGY.md`).
- 15 practices below. 3 are new, not in the [findings asset](./findings.md).

## Practices

| # | Practice | Hex evidence | Chronicler state | Candidate |
|---|---|---|---|---|
| 1 | Map goes by user capability, not by module or route | `AGENTS.md:60-62`; product tree `docs/features/README.md:13-39` | partly covered: `docs/specs/*.md` go by feature and route, and `docs/AGENTS.md:23-32` places each fact by layer | 3 |
| 2 | Stable behavior id per behavior, inline in the diagram | `AGENTS.md:63-65`; 142 ids in 16 namespaces; `setup.model-card` at `README.md:262` and `dictation.md:602`; 12 ids died with their feature since `cb547de4` | gap: a scenario number (`41.1`) is stable and never reused, but it names a test case, not a behavior | 3, 2 |
| 3 | Three states kept apart: built, a check exists, someone saw it work | `AGENTS.md:66-70`; `README.md:10-11` | gap: a green `python build.py` is the only proof; no place records a manual run | 3 |
| 4 | Dated observed record: machine, build, and what was not exercised | `README.md:246`, `README.md:406-411`, `README.md:471-474`; 50 dates in `docs/features/`, 24 "not yet observed" | gap: no dated verification record anywhere in `docs/` | 3 |
| 5 | A "Proof limit" sentence beside each check | 17 occurrences, 15 in `recovery.md` (`recovery.md:60`, `:88`, `:129`) | gap: `tests/STRATEGY.md` states a tier for each test, and no doc states what a test cannot show | new (smaller habit) |
| 6 | Checks named by test function | 611 test names in the map; 600 resolve to code, 11 do not | partly covered: `tests/STRATEGY.md` `## SCENARIO tags` plus `scripts/validate_feature_spec.py`; a spec cites a number, not a test name | 3 |
| 7 | A "Gaps And Constraints" section with "Missing evidence:" lines and issue links | `recovery.md:606-672`; `dictation.md:1327-1343`; `README.md:501` | gap: known gaps live in closed wayfinder tickets and `tmp/`, not in a durable place | 3, new (smaller habit) |
| 8 | Entry points and prerequisites per feature | `recovery.md:19-33` "How To Get To It"; `dictation.md:13` | partly covered: each spec opens with an `Endpoints:` list, and `http_routes.md` is generated from `router.rs` | 3 |
| 9 | Named UI fixture scenes and golden names as proof locators | `README.md:278` (`--scene picker`), `README.md:602`; scenes resolve at `src/rune_ui/playground.rs:102` | partly covered: browser tests select on DOM ids and classes (`tests/STRATEGY.md` `## SCENARIO tags`); no named scene fixtures | 3, 5 |
| 10 | Update the map in the same change | `AGENTS.md:57-59`; 159 of 241 source commits in the last 300 also touch `docs/features/` (66%) | partly covered: the `spec-coverage` gate step (`build.py:347-349`) forces a spec and a tagged test for a behavior change; a Diátaxis edit is review-only | 3, 1 |
| 11 | One invariants section with exact values | `AGENTS.md:527-1188` (662 lines) | partly covered: `docs/AGENTS.md:29` and `:36` put an invariant in Reference and give its durability test; the docs state cardinality, not thresholds | 2 |
| 12 | A durable recipe for an evidence record | `README.md:1851-1853` | gap | new (smaller habit) |
| 13 | The map is not a roadmap | `README.md:3-5`; `AGENTS.md:70-71` | covered: `docs/AGENTS.md:23-32` fact table, and `docs/plans/` holds plans | 3 |
| 14 | The index stays current when a file splits | `AGENTS.md:60-62`; 10 files under one `README.md` index | covered: the `<!-- AUTO-INDEX -->` block in `docs/AGENTS.md` is generated by `scripts/generate_docs_index.py` | 3 |
| 15 | Nothing checks the map, so drift survives | no doc step in `scripts/check`; no `docs` step in `.github/workflows/`; `docs/features/` is not in `.github/agent-policy/check.ts` | gap: this repo checks docs for structure only (`scripts/validate_docs.py`, `scripts/vale_lint.py`) | 1, 3 |

## Detail

### 1. Shape of the map and its rule

Known. The rule is `AGENTS.md:49-70`. It says the map indexes "existing user-facing capabilities, entry points, checks, and gaps", and it is "not a substitute for source inspection or a second roadmap". It gives five bullets: update in the same change, organize by user capability, prefer `ts` diagrams with Unicode tree characters, keep references and proof limits beside the behavior, and keep three states apart.

Known. `README.md:1-11` states the same in the index's own words: "What can the user do, how do they reach it, and what would prove it works?" It adds that "Listed checks are locators unless an executed result is explicitly recorded".

Known. The map has 10 files and 61,371 words (`wc -w docs/features/*.md`). `README.md` is 1,854 lines and 16,426 words. The other files run from 64 lines (`transcript-cli.md`) to 1,348 lines (`dictation.md`).

Inferred. The size tells the map is a product document, not a test index. `README.md` carries product trees for setup, models, MCP, and every OS. `dictation.md` and `recovery.md` carry the failure and recovery contracts.

### 2. Behavior ids: shape, stability, and reuse

Known. A behavior id is a `//` comment inside a `ts` fence, for example:

- `docs/features/recovery.md:43` — `recover.cancel`: Escape targets the active capture first.
- `docs/features/dictation.md:230` — `dictate.hold`.
- `docs/features/capture.md:20` — `capture.gate`.
- `docs/features/README.md:157` — `setup.practice`.

Known. There are 142 distinct ids (search `// <name>.<name>` across `docs/features/`). The namespaces are `dictate` 30, `meetings` 27, `capture` 22, `setup` 21, `vocabulary` 9, `maintain` 8, `linux` 6, `recover` 4, `settings` 3, `labs` 3, `dev` 3, and 6 others.

Known. An id is reusable across files, and that reuse works as a cross-file anchor. `setup.model-card` is defined at `README.md:262` ("shared by every desktop root") and referenced at `dictation.md:602`. `setup.delete-model`, `meetings.detect`, `meetings.calendar` and `dictate.stop-tone` do the same. No convention marks which site is the definition.

Known. An id dies with its feature and is not renamed for a refactor. At the baseline commit `cb547de4` (2026-09-01, "docs: add diagram-first feature maps") the map had 22 ids. 12 of them are gone today, for example the whole `voice-action.*` group, removed by `33430157` ("chore: remove Voice Action (t733) (#423)"). 132 ids arrived since the baseline.

Known. No code and no test refers to an id. I searched `src/`, `crates/`, `tests/`, `scripts/`, `.github/` and `site/` for each of the 142 ids. The only hits are settings keys that share the dotted shape: `labs.playground` at `src/control/settings.rs:271`, `meetings.record` at `src/control/settings.rs`. Those are `settings.json` keys, not map references.

Inferred. The overlap between behavior ids and settings keys is a hazard. A reader cannot tell an id reference from a settings key by shape alone.

### 3. The three states, and the dated observed record

Known. `AGENTS.md:66-70` is the rule: "Distinguish implemented behavior, available checks, and observed verification. A test's existence is not a passing run, and a preview is not native end-to-end proof. Record missing coverage and known defects explicitly."

Known. The "someone saw it work" state is written as a dated paragraph that names the machine, the build, and the paths that were not exercised. Three examples:

- `README.md:246`: "Observed on apollo (Vulkan, 2026-10-31, LibriSpeech test-clean), when they were still hints: with Qwen3-ASR 0.6B, 'Kaffar, Rodolfo, Bragelonne, …' fixed Caffer, Rudolpho, Redolfo, Rudolph, and Bragolon in 10 clips…".
- `README.md:406-411`: "**Observed September 4, 2026 (2.1.16), existing model picker:** all 455 Rust tests passed… Native menu interaction, installed-app switching, and Linux runtime behavior were not exercised."
- `README.md:471-474`: "**Observed October 30, 2026, X11 observer:** on a headless Xvnc with `setxkbmap -layout us,ru,de…`".

Known. `docs/features/` holds 50 ISO dates, 85 uses of the word "observed", and 24 sentences that say a path is "not yet observed" or "has not been observed".

Known. The negative state is as explicit as the positive one. `recovery.md:36-38` says of its own checks: "This is a map of existing checks, not a verified end-to-end recovery driver. Unless an executed result is dated below, checks are locators rather than passing runs."

Inferred. The date names the machine and the build, not a person. A record therefore proves that a run happened on that machine, and it does not name who ran it.

### 4. Proof limits

Known. 17 "Proof limit" sentences sit beside the checks they limit. 15 are in `recovery.md`, for example `recovery.md:88`: "Proof limit: synthetic events and sampled-state fixtures, not every physical keyboard or remapping." One is in `dictation.md`, one is in `README.md`.

Known. The same idea appears without the label. `README.md:228-231` says of a picker check: "Fixture renders and recorded requests only; a live model switch with a fallback has not been observed."

Inferred. This is the single most reusable part of the map for Chronicler. It costs one sentence per check, and it stops a locator from reading as a proof.

### 5. Known gaps

Known. Two feature files own a `## Gaps And Constraints` section.

- `recovery.md:606-672` lists five gaps, each with a `ts` sketch, a source link, an issue link, and a "Missing evidence:" line. Example: "Missing evidence: affected-keyboard metadata showing whether Quartz preserves enough identity to distinguish the remapped key."
- `dictation.md:1327-1343` lists six gaps as bullets, each naming what must still be observed.

Known. A gap can also sit inside a behavior, next to it. `README.md:501`: "**Known gap:** `is_installed` checks size, not checksum." `dictation.md:703`: "**Known defect, observed October 28, 2026 (same Mac and model):**…". `README.md:1411`: "**Known defect, outside Hex:**…".

Known. The rule forbids the opposite: `AGENTS.md:68-69` says "never rewrite an expected outcome merely to hide a regression".

### 6. How the map names its checks

Known. The map names a check by its test function, in backticks: `try_it_waits_for_the_model_until_it_is_ready` (`README.md:157`, defined at `src/rune_ui/onboarding.rs:1272`). It also names source paths as relative links.

Known. I extracted 611 distinct test-name strings from `docs/features/`. 600 match a string in `src/` or `crates/`; 11 match nothing in `src/`, `crates/`, `tests/` or `scripts/`.

Known. The 11 stale names are: `a_new_shortcut_s_keys_stay_inside_their_tray`, `adopting_saved_settings_restarts_only_for_listener_settings`, `captures_default_off_and_refuse_a_shortcut_in_use`, `command_preparation_does_not_block_the_listener_or_propagate_failure`, `developer_panes_show_only_in_developer_builds`, `model_switches_remember_language_and_hints_without_changing_other_settings`, `only_parakeet_based_models_use_the_vocabulary`, `parses_brave_tab_url_and_title_output`, `the_host_is_absent_while_the_experiment_is_off`, `the_meeting_tab_query_reads_only_the_active_tab`, `voice_filter_needs_labs_or_hex_microphone`.

Inferred. Most stale names name the old `experiments` module (`the_host_is_absent_while_the_experiment_is_off`, `voice_filter_needs_labs_or_hex_microphone`). The `experiments` module became `labs` in `8b885e42` (#527) and `5b7df639` (#534) on 2026-10-30. The docs were updated in those commits, and these names were missed.

Inferred. A 1.8% stale rate over 611 names is the cost of a prose link with no check. The link still works well enough to read.

### 7. How the map stays in step with the code

Known. The only mechanism is the agent rule at `AGENTS.md:57-59`: "Update the affected map in the same change when behavior, entry points, defaults, prerequisites, platform availability, recovery, or verification coverage changes. Do not churn unrelated maps for internal-only refactors."

Known. Compliance, last 300 commits: 241 touch `src/`. 168 touch `docs/features/`. 159 touch both. So 66% of source commits also update the map, and 95% of map commits also change source.

Known. Of the 82 source commits that do not touch the map, 25 have a subject that the rule exempts: module folds ("Fold dictation_* into dictation/", 14 of them, whose doc updates land in separate "Update docs for …" commits), `fix(ci)`, Rune version bumps, and doc-only commits.

Known. No gate enforces the rule. `scripts/check` runs `cargo fmt`, `clippy` and `nextest` only. No workflow in `.github/workflows/` runs a doc check or a link check. `.github/agent-policy/check.ts` flags `AGENTS.md` ("agent instructions or config") but not `docs/features/`.

Inferred. The map is correct because an agent reads it and edits it, not because a machine checks it. The measure above bounds how well that works.

### 8. Drift that no check catches

All known. Three concrete drifts at `7609574ca92c`:

1. **Stale Labs model in the index.** `README.md:31` says "[Labs](labs.md) is one switch, on by default, for every unfinished feature." `docs/features/labs.md:1-20` and `AGENTS.md:34-46` say the opposite: "There is no master switch", and each row is off by default. `8b885e42` (2026-10-30, "Labs: one switch", #527) added the sentence to `README.md`. `5b7df639` ("Labs: one flat list of per-feature toggles, each off by default", #534) replaced the model and updated `labs.md` and `AGENTS.md`, and left `README.md:31` in place.
2. **A check name that does not exist.** `README.md:1789-1790` names `developer_panes_show_only_in_developer_builds` in `rune_ui/tests/panes.rs`. `each_os_lists_only_the_panes_it_can_run` is at `src/rune_ui/tests/panes.rs:25`; the other name appears nowhere.
3. **10 more stale check names**, listed in section 6.

### 9. The maintenance section

Known. `README.md:1837-1854` is "Grow The Map". It gives a 4-part template:

```ts
One feature file
├── Sub-features -> Small action/outcome diagrams with stable IDs
├── Entry points -> Every user route and its prerequisites
├── Existing checks -> Source/test links and what they do not prove
└── Gotchas -> Exceptions, known defects, missing native proof
```

Known. It also gives the evidence recipe: "Future evidence should name the behavior ID, entry point, revision/binary, platform, prerequisites, outcome, and artifact location. Report skipped paths. Never publish tokens, personal transcripts, or unredacted diagnostics as proof."

Known. The template is a convention, not a rule that holds. `capture.md`, `control-cli.md`, `transcript-cli.md` and `vocabulary.md` have no `##` heading at all. `labs.md` has one (`## Features`). `recovery.md` and `dictation.md` follow the 4 parts closely.

Known. `README.md:1847` states the workflow's own limit: "This is the map-only stage of the verification workflow, not a working driver." I found no later stage in the repo.

### 10. The Chronicler side, fact by fact

Known. **The spec-to-test link is enforced.** `scripts/validate_feature_spec.py` reads `docs/specs/` and `tests/http/` plus `tests/browser/`. `build.py:347-349` runs it as the `spec-coverage` step. It fails on a declared scenario with no test, a tag with no scenario, an untagged test, a surface mismatch, or a quarantine count above the pin. `tests/STRATEGY.md` `## SCENARIO tags` states the rule, and the tag carries the spec path and the scenario number: `// [spec-path] SCENARIO: N.N`.

Known. **The corpus is 31 spec files with 554 scenarios.** `wc -w docs/specs/*.md` gives 49,230 words. For comparison, the Hex map is 61,371 words for a tree of 263k Rust lines, against Chronicler's 42.6k lines in `src/`.

Known. **A Chronicler spec holds behavior, not proof state.** I read `docs/specs/autoplay.md:1-40`. A spec opens with an `Endpoints:` list, one or two paragraphs of rules, then Gherkin scenarios. It records no observed run, no proof limit, and no gap. The words "observed", "proof" and "manual" do not occur in `docs/specs/`.

Known. **Placement has a rule.** `docs/AGENTS.md:23-32` gives the "Where a fact lives" table. An invariant goes to Diátaxis Reference, a reason to Explanation, an event rule at feature level to a spec scenario, and fine-grained behavior to the code and its tests. `docs/AGENTS.md:36` gives the durability test for a Reference sentence.

Known. **Invariants are per subject, not in one list.** `docs/diataxis/reference/game_flow.md:15-16` states cardinality rules ("A Game has at most one Job", "A Job runs at most two generations"). `storage.md:14` states a decorator invariant. The whole Reference set holds one numeric timing value: the 1 s poll at `docs/diataxis/reference/frontend/dashboard.md:85`. Every other rule it states is a cardinality ("at most one Job", "exactly one story per game").

Known. **No place records a manual run.** A search of `docs/` for "observed", "manually verified" and "verified by hand" returns nothing. A green `python build.py` is the whole proof. `tests/AGENTS.md` and `tests/STRATEGY.md` describe tiers, the domain-outcome rule, and the SCENARIO tags, and they state no observed-verification state.

Known. **The paired-rule-id idea is open, not built.** `.scratch/statechart-models/map.md` (Decisions so far) records ticket 01 (39 gate rule sentences: 20 pinned, 16 partly pinned, 3 not pinned) and ticket 02 (first method: one tier-1 test per rule, paired by rule id, "the skill's method plus the `validate_feature_spec.py` idiom"). The map's "Not yet specified" holds the open question of whether a rule pairs to a spec scenario or a doc sentence. No rule id exists in `docs/` yet.

Known. **The spec map was reorganized once, by hand, with the validator as the check.** `.scratch/narrator-modes-and-options/map.md`, ticket 18 and ticket 20: per-surface spec files (`browser_<feature>.md` for DOM-observed scenarios), test files that mirror them, and a validator surface rule. Ticket 19 pinned the rule; ticket 20 shipped it. Scenario numbers stayed, and gaps were allowed to stay behind.

Known. **A known gap of a feature lives in a closed ticket.** Ticket 12 of that map ends with "flagged as-is: `/options` empty-history validation surfaces as HTTP 500, unknown world-posture values silently fall back to the domain default…". Ticket 10 ends with "Known gaps recorded in the ticket: arrival narration never fires always-on options, swipe-switch rewind restores point-in-time sets, options-preset delete-check still open". Those tickets sit in `.scratch/_closed/`.

Inferred. That is the largest Chronicler gap against this Hex area. The gap records exist, and they sit in a place a reader of the behavior does not go.

## What I could not find or did not read

Known, unless labelled.

- I did not read `docs/features/` in full. The ticket allowed a sample. I read `README.md:1-330`, `:1745-1854`, `recovery.md:1-200` and `:595-672`, `release-verification.md:1-140`, `labs.md:1-60`, `dictation.md:1327-1343`. I read headings of all 10 files. I did not read `capture.md`, `meetings.md`, `control-cli.md`, `transcript-cli.md` or `vocabulary.md`.
- I did not read the pipeline prompts that make an agent update the map. They are private (`~/code/control/AGENTS.md`), and the findings asset says the same.
- I did not read a PR body to see whether a review step checks the map. I called no GitHub API, per the ticket.
- I did not read the golden images or the recorded scenes. I confirmed the scene names resolve in `src/rune_ui/playground.rs:102` and `src/rune_ui.rs:2005`.
- The 11 stale check names in section 6 come from a scan of all 10 files, so the stale rate covers the whole map. I read none of the surrounding text in the 5 files I skipped, so I cannot say what that drift hides.
- I did not check whether a later Hex commit fixes the 3 drifts in section 8. The clone stops at `7609574ca92c`.
- I did not read `docs/specs/` in full. I read `docs/specs/autoplay.md:1-40` only, and I searched all 31 files for 3 words. I did not read `docs/diataxis/` in full. I read `docs/AGENTS.md:1-60` and searched the Reference set by pattern.
- I did not run `python build.py`, and I did not run the validator. I read the gate wiring at `build.py:347-349` and `build.py:1195`.
- Chronicler sampling: one spec file in part, `docs/diataxis/reference/game_flow.md` and `storage.md` by search hit only, and `tests/STRATEGY.md` in part. The 554-scenario and 49,230-word counts come from `grep` and `wc` over the whole directory.

## How I sampled

- Hex map: full read of `README.md` lines 1-330 and 1745-1854, which hold the product map, the rule's own statement, the setup contracts and the maintenance section. Full read of `recovery.md:1-200` and `:595-672`, because the ticket named `recovery.md` and it is the densest proof-record file. Heading scan of all 10 files to check the 4-part template.
- Hex code: targeted searches only, for 5 named test functions, 142 behavior ids, 611 test names, and 3 drift claims. No file read beyond 80 lines.
- Hex history: `git log` over `docs/features/`, a per-commit file-set join over the last 300 commits, and `git log -S` for 5 strings.
- Chronicler: the files the ticket named, plus `docs/AGENTS.md`, `tests/STRATEGY.md`, `scripts/validate_feature_spec.py:1-80` and `build.py` wiring.
