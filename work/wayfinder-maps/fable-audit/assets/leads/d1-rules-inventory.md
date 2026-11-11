# d1 — Inventory of agent rules and their enforcement

Scope: the rule sources named in the brief, read in full. The generated blocks of `AGENTS.md`
(lines 7–329), `tests/AGENTS.md` (46–199) and `docs/AGENTS.md` (45–98) are skipped.
Every row is `known` unless the row says `inferred` or `guessed`. Rationale sentences, allowance
sentences and facts that give no instruction are not rows.

A **rule** is one sentence that tells an agent to do or not do something. A compound sentence is
split. A row names the strongest enforcement that exists, not every enforcement.

---

## Part 1 — The rules

### AGENTS.md (hand-written part, 1,144 words)

| id | source | rule (shortened) | class | machine-checkable | status |
|---|---|---|---|---|---|
| A01 | AGENTS.md:333 | Be responsible for the overall health of the repository | PROSE-ONLY | no — a judgement of outcome | known |
| A02 | AGENTS.md:333 | Repository health matters more than your task succeeded | PROSE-ONLY | no — a judgement of priority | known |
| A03 | AGENTS.md:333 | Do not arbitrarily delete or revert unknown or unexpected files | PROSE-ONLY | partly — a run's file deletions are observable | known |
| A04 | AGENTS.md:333 | Do not remove a file only because it is not working or in the way | PROSE-ONLY | partly — needs the reason, not the act | known |
| A05 | AGENTS.md:335 | Do not make code changes to fix problems during a review | PROSE-ONLY | no — a review has no artifact trail | known |
| A06 | AGENTS.md:337 | Build code cleanly, modularly, with separation of concerns | PROSE-ONLY | no — a design judgement | known |
| A07 | AGENTS.md:337 | Never bolt on a feature because the user asked for it | PROSE-ONLY | no — a design judgement | known |
| A08 | AGENTS.md:337 | Stop your work ahead of time when no natural build exists | PROSE-ONLY | no | known |
| A09 | AGENTS.md:337 | Say so when you stop | PROSE-ONLY | yes — a message scan | known |
| A10 | AGENTS.md:337 | Recommend the refactor that makes the whole thing clean | PROSE-ONLY | partly — a message names a refactor | known |
| A11 | AGENTS.md:341 | Label epistemic status when it matters: known, inferred, guessed | PROSE-ONLY | partly — an unlabelled claim needs judgement | known |
| A12 | AGENTS.md:343 | Say "I don't know" instead of inventing an answer | PROSE-ONLY | no — needs ground truth | known |
| A13 | AGENTS.md:345 | Say agree or disagree before saying what you changed | PROSE-ONLY | partly — a message starts with a verdict | known |
| A14 | AGENTS.md:349 | Read `docs/AGENTS.md` and `tests/AGENTS.md` when searching | PROSE-ONLY | partly — a tool-call trace shows the read | known |
| A15 | AGENTS.md:349 | Never hand-edit the generated blocks | PROSE-ONLY | partly — the generators overwrite a hand edit, with no error | known |
| A16 | AGENTS.md:349 | Regenerate the indexes with `scripts/generate_*_index.py` without the hook | CHECK — `build.py:1079` `guardrails-doc-check`, `scripts/generate_guardrails_doc.py --check` | yes — a stale generated doc | known |
| A17 | AGENTS.md:351 | Read `CODING_STANDARDS.md` before writing or reviewing code | PROSE-ONLY | partly — a tool-call trace shows the read | known |
| A18 | AGENTS.md:355 | Keep scratch work in `tmp/` | PROSE-ONLY | partly — `.gitignore` ignores `tmp/` | inferred |
| A19 | AGENTS.md:357 | Every build action goes through `build.py` | PROSE-ONLY | partly — `ENVIRONMENT.md:86` says a raw `cargo` build recompiles, but nothing fails | known |
| A20 | AGENTS.md:357 | Run `build.py` in the foreground with the bash timeout at 1200 | PROSE-ONLY | yes — the tool call carries both | known |
| A21 | AGENTS.md:357 | Send the build output to a file of your own | PROSE-ONLY | yes — the command line shows a redirect | known |
| A22 | AGENTS.md:357 | Read your log from the `Full build log:` line | PROSE-ONLY | yes — the next command reads that path | known |
| A23 | AGENTS.md:364-376 | Use the `build.py` subcommand whose scope matches the change | PROSE-ONLY | partly — the command against the diff | inferred |
| A24 | AGENTS.md:378 | A shared-API change is confirmed by `clippy` or the full gate | PROSE-ONLY | partly — the build log names the step | known |
| A25 | AGENTS.md:380 | Read `ENVIRONMENT.md` when a run is slow, waits or runs out of memory | PROSE-ONLY | partly — a tool-call trace shows the read | known |
| A26 | AGENTS.md:386 | Keep issues as local markdown under `.scratch/` with no `gh` dependency | PROSE-ONLY | yes — a scan for `.scratch/<effort>/` shape | known |
| A27 | AGENTS.md:390 | Use the five canonical triage role strings as `Status:` lines | PROSE-ONLY | yes — a fixed vocabulary | known |
| A28 | AGENTS.md:394 | Keep one `CONTEXT.md` glossary at the repo root | PROSE-ONLY | yes — a file count | known |
| A29 | AGENTS.md:398 | Read the permission config to see allowed permissions | PROSE-ONLY | partly — a tool-call trace shows the read | known |
| A30 | AGENTS.md:398 | Do not circumvent the permissions | PERMISSION — `.pi/extensions/pi-permission-system/config.json:2-48` denies the listed commands | yes — the denied commands | known |
| A31 | AGENTS.md:398 | Recommend permission changes, never apply them without approval | PERMISSION — config.json:11 and :15 deny write and edit of that file | yes — a write attempt is denied | known |
| A32 | AGENTS.md:400 | Do not commit without explicit approval | PROSE-ONLY | yes — the commit exists in the log | known |
| A33 | AGENTS.md:400 | Prefer normal git commands; avoid destructive git commands | PROSE-ONLY | yes — a command-line scan | known |
| A34 | AGENTS.md:404 | Delegate only a substantial task; do a short task yourself | PROSE-ONLY | partly — a delegation decision needs judgement | known |
| A35 | AGENTS.md:406 | Use `implementer` by default when the brief names the design, files and checks | PROSE-ONLY | partly — the brief text | known |
| A36 | AGENTS.md:406-410 | Choose `strong-implementer` for open design, cross-layer consistency, or a wrong earlier run | PROSE-ONLY | partly — the brief text | known |
| A37 | AGENTS.md:412 | Give a substantial investigation to a `scout` | PROSE-ONLY | partly — the delegation record | known |
| A38 | AGENTS.md:412 | Name the finding in the brief | PROSE-ONLY | yes — the brief text | known |
| A39 | AGENTS.md:416 | Be strict about the scope of the task | PROSE-ONLY | partly — the diff against the brief | known |
| A40 | AGENTS.md:417 | Ask for proof of work, not a summary | PROSE-ONLY | partly — the brief and the reply | known |
| A41 | AGENTS.md:418 | Do a one-step review of the diff | PROSE-ONLY | partly — the reply cites the diff | known |

### CODING_STANDARDS.md (282 words)

| id | source | rule (shortened) | class | machine-checkable | status |
|---|---|---|---|---|---|
| C01 | CODING_STANDARDS.md:5 | Do not preserve backward compatibility unless the user asks | PROSE-ONLY | yes — a scan for `deprecated`, `legacy`, compat shims | known |
| C02 | CODING_STANDARDS.md:7 | Symbols must use verbose, domain-aligned names that map to the docs | PROSE-ONLY | partly — a name scan needs the glossary | known |
| C03 | CODING_STANDARDS.md:9 | HTTP form structs must deserialize from partial urlencoded bodies | CHECK — `tests/infrastructure/guardrails/layers.rs:100` `check_form_fields_urlencoded_safe` | yes — a field type | known |
| C04 | CODING_STANDARDS.md:9 | Mark an omittable legacy field with `#[serde(default)]` | CHECK — layers.rs:100 | yes — an attribute | known |
| C05 | CODING_STANDARDS.md:9 | Encode a checkbox group as one boolean field per value | CHECK — layers.rs:100 flags a collection-typed field | yes — a field type | known |
| C06 | CODING_STANDARDS.md:13-14 | Every `src/` file carries a `//! [DOC: …]` anchor on line 1 | CHECK — `tests/infrastructure/guardrails/structure.rs:38` `check_doc_standards` | yes — line 1 text | known |
| C07 | CODING_STANDARDS.md:14 | The anchor targets `reference/` only, never `explanation/`, `how-to/` or `tutorials/` | CHECK — structure.rs:38 rejects a path outside `docs/diataxis/reference/` | yes — the path prefix | known |
| C08 | CODING_STANDARDS.md:14 | `src/test_support/*.rs` must not carry a DOC anchor | CHECK — `scripts/validate_docs.py:1003` `TEST_SUPPORT_ANCHOR_FORBIDDEN`; structure.rs:7 `MODULE_DOC_EXEMPTIONS` | yes — line 1 text | known |
| C09 | CODING_STANDARDS.md:15 | Line 2 is a human-readable summary, `src/test_support/*.rs` included | CHECK — structure.rs:38; `validate_docs.py:1021` `TEST_SUPPORT_SUMMARY_REQUIRED` | yes — line 2 text | known |
| C10 | CODING_STANDARDS.md:16 | Do not add function-level DOC anchors | PROSE-ONLY | yes — a comment scan | known |
| C11 | CODING_STANDARDS.md:18 | Never write a comment that paraphrases what the code does | SKILL — `.agents/skills/chronicler-comment-fixer/SKILL.md:3` | partly — paraphrase needs a semantic test | known |
| C12 | CODING_STANDARDS.md:18 | Rename the symbol rather than comment when the code is unclear | PROSE-ONLY | partly — a comment plus an unchanged name | known |
| C13 | CODING_STANDARDS.md:18 | Comment the WHY only when non-obvious | SKILL — chronicler-comment-fixer/SKILL.md:3 | partly | known |
| C14 | CODING_STANDARDS.md:20 | Never reference the task in a code comment | PROSE-ONLY | yes — a token scan (`issue`, `added for`, `TODO from review`) | known |
| C15 | CODING_STANDARDS.md:24 | Do not compile, build or test during a review | PROSE-ONLY | yes — the command trace | known |
| C16 | CODING_STANDARDS.md:26 | Avoid vague architectural critique; focus on actionable changes | PROSE-ONLY | no — a quality judgement | known |
| C17 | CODING_STANDARDS.md:30 | Read `tests/AGENTS.md` and `tests/STRATEGY.md` when writing tests | PROSE-ONLY | partly — a tool-call trace | known |
| C18 | CODING_STANDARDS.md:30 | Follow the integration-test patterns in `integration_test_standards.md` | PROSE-ONLY | no — the doc states no checkable rule | known |
| C19 | CODING_STANDARDS.md:32 | Tautological tests are harmful | PROSE-ONLY | partly — a test that asserts the expression under test | known |
| C20 | CODING_STANDARDS.md:36 | Read `docs/AGENTS.md` when writing or reviewing documentation | PROSE-ONLY | partly — a tool-call trace | known |

### .pi/APPEND_SYSTEM.md (111 words)

| id | source | rule (shortened) | class | machine-checkable | status |
|---|---|---|---|---|---|
| S01 | .pi/APPEND_SYSTEM.md:3 | Write prose in ASD-STE100 Simplified Technical English | CHECK — `build.py:372` `vale` step, `.vale.ini:11` `BasedOnStyles = Chronicler` | yes — a sentence rule | known |
| S02 | .pi/APPEND_SYSTEM.md:5 | Use short sentences, one action or statement each | CHECK — `styles/Chronicler/SentenceLength.yml:4` max 25 words | yes — a word count | known |
| S03 | .pi/APPEND_SYSTEM.md:6 | Use a clear subject and an active verb; name the actor | PROSE-ONLY | partly — a passive-voice pattern | known |
| S04 | .pi/APPEND_SYSTEM.md:7 | Use the same term for the same thing | PROSE-ONLY | partly — a repeated-synonym scan | known |
| S05 | .pi/APPEND_SYSTEM.md:8 | Use familiar words with one precise meaning | PROSE-ONLY | no — a vocabulary judgement | known |
| S06 | .pi/APPEND_SYSTEM.md:9 | Keep noun groups short; use prepositions for relations | PROSE-ONLY | partly — a run of nouns | known |
| S07 | .pi/APPEND_SYSTEM.md:10 | Preserve code, commands, identifiers, paths and errors verbatim | PROSE-ONLY | partly — a diff of a quoted string | known |
| S08 | .pi/APPEND_SYSTEM.md:11 | Answer in the first sentence | PROSE-ONLY | partly — a reply's opening line | known |
| S09 | .pi/APPEND_SYSTEM.md:12 | Weigh two or more options in an unfenced markdown table | PROSE-ONLY | yes — a reply scan | known |
| S10 | .pi/APPEND_SYSTEM.md:14 | Ground language in the project's terms, architecture and constraints | SKILL — `.agents/skills/domain-modeling/SKILL.md`; `docs/agents/domain.md:26` | partly — needs the glossary | known |

### ENVIRONMENT.md (1,232 words)

| id | source | rule (shortened) | class | machine-checkable | status |
|---|---|---|---|---|---|
| E01 | ENVIRONMENT.md:3 | Read this when a run is slow, waits or runs out of memory | PROSE-ONLY | partly — a tool-call trace | known |
| E02 | ENVIRONMENT.md:20 | Check the build slot with `python scripts/build_slot.py --status` | PROSE-ONLY | yes — the command is in the trace | known |
| E03 | ENVIRONMENT.md:55 | Give a coverage run a dedicated `--target-dir` | PROSE-ONLY | yes — the command line | known |
| E04 | ENVIRONMENT.md:91 | Start the dev server with `python build.py run` | PROSE-ONLY | yes — the command line | known |
| E05 | ENVIRONMENT.md:93 | Start the server from the worktree itself | PROSE-ONLY | yes — the working directory | known |

### tests/STRATEGY.md (2,459 words)

| id | source | rule (shortened) | class | machine-checkable | status |
|---|---|---|---|---|---|
| T01 | tests/STRATEGY.md:9-11 | A tier is defined by what is faked; every `tests/` directory has one row | PROSE-ONLY | partly — an unlisted directory | known |
| T02 | tests/STRATEGY.md:30 | A `#[tokio::test]` with fakes is a unit test | PROSE-ONLY | partly — the fakes it constructs | known |
| T03 | tests/STRATEGY.md:34 | A unit test lives in a seam contract, a `domain/model/` file, or the legacy list | CHECK — `tests/infrastructure/guardrails/unit_test_allowlist.rs:178`, `scripts/check_test_structure.py` | yes — the file path | known |
| T04 | tests/STRATEGY.md:42 | Every test in a declaring file carries one `CASE` tag | CHECK — `scripts/validate_seam_cases.py` (`build.py:1075` `seam-cases`) | yes — the tag | known |
| T05 | tests/STRATEGY.md:43 | A unit test in a declaring file calls only that box's entry point | PROSE-ONLY | partly — a call-graph diff against production | known |
| T06 | tests/STRATEGY.md:48 | One clause per case: one input shape, result or failure | PROSE-ONLY | no — a semantic split | known |
| T07 | tests/STRATEGY.md:49 | Build the box as production does; call no helper production never calls | PROSE-ONLY | partly — a helper call graph | known |
| T08 | tests/STRATEGY.md:51 | Assert the declared outcome, not the internal steps | PROSE-ONLY | partly — an assertion on a private value | known |
| T09 | tests/STRATEGY.md:53 | A storage failure goes through `Storage::with_failure` | PROSE-ONLY | yes — the call shape | known |
| T10 | tests/STRATEGY.md:56 | A call-recording box has a good-call case and a failed-call case | PROSE-ONLY | partly — two cases with those labels | known |
| T11 | tests/STRATEGY.md:59 | A pure `domain/model/` function's own branches are its cases | CHECK — unit_test_allowlist.rs:178 checks the folder | yes — the file path | known |
| T12 | tests/STRATEGY.md:64 | The legacy list only shrinks, one module at a time | CHECK — unit_test_allowlist.rs:178 fails a stale entry | partly — a shrink needs a human diff | known |
| T13 | tests/STRATEGY.md:65 | A mechanically broken legacy test changes in place | PROSE-ONLY | no | known |
| T14 | tests/STRATEGY.md:66 | A behaviour-broken legacy test moves to its seam or scenario | PROSE-ONLY | partly — the new path | known |
| T15 | tests/STRATEGY.md:70 | A player-visible behaviour is a scenario, not a unit test | PROSE-ONLY | partly — the spec link | known |
| T16 | tests/STRATEGY.md:71 | A module that is neither a box nor a pure domain function has no unit test | CHECK — unit_test_allowlist.rs:178 | yes — the file path | known |
| T17 | tests/STRATEGY.md:81 | The coverage percentage stays a diagnostic; no step fails on it | CHECK — `build.py:1154-1162` runs coverage report-only | yes — the step setting | known |
| T18 | tests/STRATEGY.md:85 | A tier-1 test asserts domain outcomes only | PROSE-ONLY | partly — a `Storage` call in an assertion | known |
| T19 | tests/STRATEGY.md:90-94 | A read seam is an HTTP GET or a listed application read method | PROSE-ONLY | partly — a call on `Storage` | known |
| T20 | tests/STRATEGY.md:96 | A read seam no driving adapter calls needs a doc comment that names the fact | PROSE-ONLY | partly — an undocumented `pub fn` only tests call | known |
| T21 | tests/STRATEGY.md:105 | Judge the read inside a helper's body | PROSE-ONLY | partly — needs the same rule at one hop | known |
| T22 | tests/STRATEGY.md:107 | A test may seed state through `Storage` and fixtures | PROSE-ONLY | no — an allowance | known |
| T23 | tests/STRATEGY.md:108 | A synchronization wait may read state | PROSE-ONLY | no — an allowance | known |
| T24 | tests/STRATEGY.md:111 | Read a model call's prompt and settings from `ScriptedModel::calls()` | PROSE-ONLY | partly — the call shape | known |
| T25 | tests/STRATEGY.md:116 | A persistence fact with no read seam is a driven-adapter or unit assertion | PROSE-ONLY | partly — the file path | known |
| T26 | tests/STRATEGY.md:123 | An HTTP scenario maps to a tier-1 test; a DOM scenario to tier 2 or 3 | CHECK — `scripts/validate_feature_spec.py` (`build.py:1074` `spec-coverage`) | yes — the tag and the file | known |
| T27 | tests/STRATEGY.md:133 | A Given or Then names a domain thing with a `CONTEXT.md` term | PROSE-ONLY | partly — a glossary lookup | known |
| T28 | tests/STRATEGY.md:135 | Reword a scenario that names a Rust identifier, field path or module | PROSE-ONLY | yes — a code-shaped token in a spec | known |
| T29 | tests/STRATEGY.md:139 | Name markup only as an element `id` or `class` that a test selects on | PROSE-ONLY | partly — a selector scan | known |
| T30 | tests/STRATEGY.md:143 | A spec covers each distinct response its route can return | PROSE-ONLY | partly — the route table against the spec | known |
| T31 | tests/STRATEGY.md:145 | Fine-grained UI behaviour inside one response stays in the code and its tests | PROSE-ONLY | no | known |
| T32 | tests/STRATEGY.md:146 | Specs describe the system, not a change to it | PROSE-ONLY | partly — future and past tense (Vale warns) | known |
| T33 | tests/STRATEGY.md:152 | Overlap across tiers is expected | PROSE-ONLY | no — an allowance | known |
| T34 | tests/STRATEGY.md:156 | Within one tier, one behaviour has one test | PROSE-ONLY | partly — two tests that assert one thing | known |
| T35 | tests/STRATEGY.md:166 | A domain outcome goes to tier 1 | PROSE-ONLY | partly — the assertion target | known |
| T36 | tests/STRATEGY.md:170 | If faking the server changes the behaviour, it is tier 2 | PROSE-ONLY | partly — the file path under `stub/` | known |
| T37 | tests/STRATEGY.md:173 | A full-stack hop is tier 3, in `tests/browser/<surface>.rs` | PROSE-ONLY | partly — the file path | known |
| T38 | tests/STRATEGY.md:177 | When in doubt, file the test a tier down | PROSE-ONLY | no | known |
| T39 | tests/STRATEGY.md:180 | A tier-2 test is an `async fn check_*(page, stub)` passed to `StubRunner::run` | CHECK — validate_feature_spec.py fails an unwired stub check | yes — the function shape | known |
| T40 | tests/STRATEGY.md:182 | The SCENARIO tag goes above the `check_*` function | CHECK — validate_feature_spec.py (`run_*` exempt) | yes — the tag position | known |
| T41 | tests/STRATEGY.md:184 | Split a runner when it becomes the longest browser test by a wide margin | PROSE-ONLY | yes — the timing report | known |
| T42 | tests/STRATEGY.md:186 | A layout-only check goes in `tests/browser/stub/layout.rs` | PROSE-ONLY | partly — the file path | known |
| T43 | tests/STRATEGY.md:211 | A stub-tier test asserts client behaviour, not canned fragment content | PROSE-ONLY | partly — the assertion target | known |
| T44 | tests/STRATEGY.md:217 | Full-stack interaction goes through the htmx-settle helpers | CHECK — `tests/infrastructure/guardrails/structure.rs:259` `check_browser_interactions_use_htmx_settle` | yes — a raw interaction call | known |
| T45 | tests/STRATEGY.md:222 | A `// settle-guard-exempt: <why>` line exempts a flagged line | CHECK — structure.rs:259 | yes — the marker | known |
| T46 | tests/STRATEGY.md:225 | Wait on an htmx-settle helper or a DOM condition | CHECK — structure.rs:259 (the raw call is flagged) | partly — a bare poll is not flagged | known |
| T47 | tests/STRATEGY.md:226 | `networkidle` is banned at every tier | PROSE-ONLY | yes — a token scan in `tests/` | known |
| T48 | tests/STRATEGY.md:231 | Every stub check and tier-3 check ends with the convergence check | PROSE-ONLY | partly — the runner runs it | known |
| T49 | tests/STRATEGY.md:244 | Use `run_without_convergence` only for a page that cannot match, with the reason | PROSE-ONLY | partly — the call plus a reason string | known |
| T50 | tests/STRATEGY.md:250 | A `SCENARIO:` tag goes only on a test in `tests/http/` or `tests/browser/` | CHECK — validate_feature_spec.py scans those two trees | yes — the file path | known |
| T51 | tests/STRATEGY.md:251 | At the unit and driven tiers the test name describes the behaviour | PROSE-ONLY | partly — a `test_` prefix | known |
| T52 | tests/STRATEGY.md:254 | A DOM scenario lives in `browser_<feature>.md`; an HTTP one in `<feature>.md` | CHECK — validate_feature_spec.py (surface mismatch) | yes — the tag target | known |
| T53 | tests/STRATEGY.md:256 | A scenario has one observation surface, one spec file, one tag | CHECK — validate_feature_spec.py | yes — the tag | known |
| T54 | tests/STRATEGY.md:257 | A test file mirrors its spec file | CHECK — validate_feature_spec.py (partly; mirroring only) | partly — the name pair | known |
| T55 | tests/STRATEGY.md:259 | Dashboard-chrome scenarios live in `browser_dashboard.md` | PROSE-ONLY | partly — the spec path | known |
| T56 | tests/STRATEGY.md:270 | The quarantine count may only go down | CHECK — validate_feature_spec.py:84 `REQUIRES_MIGRATION_TEST_COUNT = 67` | yes — a count | known |
| T57 | tests/STRATEGY.md:277 | `layout.rs` and `htmx_events.rs` carry no tag | CHECK — validate_feature_spec.py:71 `TAG_EXEMPT_FILES` | yes — the file name | known |

### tests/AGENTS.md (hand-written part, 362 words)

| id | source | rule (shortened) | class | machine-checkable | status |
|---|---|---|---|---|---|
| G01 | tests/AGENTS.md:5 | Read `STRATEGY.md` before placing a test or writing a scenario | PROSE-ONLY | partly — a tool-call trace | known |
| G02 | tests/AGENTS.md:11 | Show the actual test output and quote the failure verbatim | PROSE-ONLY | partly — the quoted message | known |
| G03 | tests/AGENTS.md:12 | Read the test code before explaining why it failed | PROSE-ONLY | partly — a tool-call trace | known |
| G04 | tests/AGENTS.md:13 | Verify an assumption before you state it | PROSE-ONLY | no | known |
| G05 | tests/AGENTS.md:14 | Never rationalize a failure away | PROSE-ONLY | partly — the reply's claim | known |
| G06 | tests/AGENTS.md:15 | Investigate pre-existing and flaky failures too | PROSE-ONLY | partly — the reply addresses them | known |
| G07 | tests/AGENTS.md:17 | Say so and investigate when you are unsure; do not invent an explanation | PROSE-ONLY | no | known |
| G08 | tests/AGENTS.md:21 | Keep an LLM test `#[ignore]`d by default | CHECK — `tests/infrastructure/guardrails/test_hygiene.rs:12` `check_no_ignored_tests` exempts `tests/llm/` | yes — the attribute and the path | known |
| G09 | tests/AGENTS.md:25 | Read `tmp/test_server_logs/{port}_{stream}.log` to diagnose a run | PROSE-ONLY | partly — a tool-call trace | known |
| G10 | tests/AGENTS.md:29 | A test path mirrors its `src/` subpath within the binary | PROSE-ONLY | partly — the name pair | known |
| G11 | tests/AGENTS.md:39 | Keep API details in the exemplar file, not in this list | PROSE-ONLY | no | known |

### docs/AGENTS.md (hand-written part, 1,261 words)

| id | source | rule (shortened) | class | machine-checkable | status |
|---|---|---|---|---|---|
| D01 | docs/AGENTS.md:7 | Write documentation in Simplified Technical English | CHECK — `build.py:372` `vale`, `.vale.ini:11` | yes — three structural rules | known |
| D02 | docs/AGENTS.md:19 | Every spec has an associated integration test | CHECK — `scripts/validate_feature_spec.py` | yes — the coverage gap list | known |
| D03 | docs/AGENTS.md:25 | Put each fact in exactly one layer | PROSE-ONLY | partly — a duplicate sentence between two docs | known |
| D04 | docs/AGENTS.md:29 | A purpose, name or invariant lives in Diátaxis Reference | PROSE-ONLY | no — a classification | known |
| D05 | docs/AGENTS.md:30 | A reason for the design lives in Diátaxis Explanation | PROSE-ONLY | no | known |
| D06 | docs/AGENTS.md:31 | An event rule or a per-route property lives in a spec | PROSE-ONLY | no | known |
| D07 | docs/AGENTS.md:32 | An ordering rule between concurrent parts lives in the verify skill's TLA+ model | PROSE-ONLY | partly — the model file | known |
| D08 | docs/AGENTS.md:33 | Fine-grained behaviour, values and markup live in the code and tests | PROSE-ONLY | partly — a column or constant list in a doc | known |
| D09 | docs/AGENTS.md:35 | One home per subject: Reference states it, Explanation gives only the reason | PROSE-ONLY | partly — a repeated paragraph | known |
| D10 | docs/AGENTS.md:37 | An invariant must survive the next fix, restyle or copy edit | PROSE-ONLY | no — a prediction | known |
| D11 | docs/AGENTS.md:39 | A behaviour change edits its spec and its test | PROSE-ONLY | partly — the diff touches a spec | known |
| D12 | docs/AGENTS.md:39 | A Diátaxis doc changes only for a part, invariant or name change | PROSE-ONLY | partly — the diff against a code change | known |
| D13 | docs/AGENTS.md:39 | Rewrite a doc that needs an edit for most changes in its area | PROSE-ONLY | no | known |
| D14 | docs/AGENTS.md:41 | List only the paired doc and the docs the body hands a point to | CHECK — `scripts/validate_docs.py:663` `STANDARD_DOC_BODY_REFERENCE` | partly — the check covers body prose only | known |
| D15 | docs/AGENTS.md:41 | Cite no spec scenario numbers in a Diátaxis doc | PROSE-ONLY | yes — a numeric token after a spec name | known |
| D16 | docs/AGENTS.md:102 | Write in the declared Diátaxis mode, never lift content from elsewhere | SKILL — `.agents/skills/chronicler-docs-hygiene/SKILL.md:3` | partly — a mode judgement | known |
| D17 | docs/AGENTS.md:108 | Every doc in `docs/diataxis/` is in exactly one Diátaxis mode | CHECK — `validate_docs.py:883` `FRONTMATTER_INVALID_MODE` | yes — the front-matter value | known |
| D18 | docs/AGENTS.md:117 | Drop content that solves no reader problem; split content that mixes modes | SKILL — chronicler-docs-hygiene/SKILL.md:3 | no | known |
| D19 | docs/AGENTS.md:121-124 | Every doc under `docs/diataxis/` carries front-matter with `diataxis:` and `title:` | CHECK — `validate_docs.py:834` `MISSING_FRONTMATTER`, and the key rules | yes — the front-matter keys | known |
| D20 | docs/AGENTS.md:130 | Create `tutorials/` and `how-to/` only when a doc earns the mode | PROSE-ONLY | partly — an empty directory | known |
| D21 | docs/AGENTS.md:136 | Cite a symbol, not a `path:line` | CHECK — `validate_docs.py:760` `LINE_CITATION` | yes — a backticked citation | known |
| D22 | docs/AGENTS.md:140 | Use Mermaid only for diagrams | PROSE-ONLY | yes — a fence-language scan | known |
| D23 | docs/AGENTS.md:140 | Draw one diagram per cluster | PROSE-ONLY | partly — a count per doc | known |
| D24 | docs/AGENTS.md:140 | Each diagram fits on one screen | PROSE-ONLY | partly — a node count | known |
| D25 | docs/AGENTS.md:143 | Do not use `erDiagram` | PROSE-ONLY | yes — a token scan in a Mermaid fence | known |
| D26 | docs/AGENTS.md:152 | A Reference doc describes only; move the why to Explanation and link it | PROSE-ONLY | partly — a why paragraph in reference/ | known |
| D27 | docs/AGENTS.md:156 | State the design, its reason and its tradeoff directly | PROSE-ONLY | no | known |
| D28 | docs/AGENTS.md:156 | Name an alternative only where the reader needs it as context | PROSE-ONLY | no | known |
| D29 | docs/AGENTS.md:158 | Name a section by what it explains, not as a defence | PROSE-ONLY | yes — a heading pattern (`Why X?`) | known |
| D30 | docs/AGENTS.md:159 | Do not narrate a reader, stage a contrast, editorialize or speculate | SKILL — chronicler-docs-hygiene/SKILL.md:3 | partly — a phrase list | known |
| D31 | docs/AGENTS.md:160 | Do not re-list mechanism; link the Reference doc | PROSE-ONLY | partly — a repeated mechanism list | known |
| D32 | docs/AGENTS.md:164 | A doc does not explain the code by listing modules, types and methods | PROSE-ONLY | partly — a module-list scan | known |
| D33 | docs/AGENTS.md:166 | Values stay in the code: no column, field, signature, constant or cap list | PROSE-ONLY | partly — a value-list pattern | known |
| D34 | docs/AGENTS.md:168 | Keep a name a reader would grep for; drop a mechanics label | PROSE-ONLY | partly — a Rust type in prose | known |
| D35 | docs/AGENTS.md:174 | State what a thing is; do not describe it by what it is not | PROSE-ONLY | partly — a negation pattern | known |
| D36 | docs/AGENTS.md:174 | Do not disclaim scope in body prose | PROSE-ONLY | partly — an out-of-scope sentence | known |
| D37 | docs/AGENTS.md:176 | State a negative constraint as a positive one | PROSE-ONLY | partly — a negation pattern | known |

### docs/agents/*.md (601 words)

| id | source | rule (shortened) | class | machine-checkable | status |
|---|---|---|---|---|---|
| DM01 | docs/agents/domain.md:7 | Read `AGENTS.md` and `CONTEXT.md` before exploring the codebase | PROSE-ONLY | partly — a tool-call trace | known |
| DM02 | docs/agents/domain.md:14 | Proceed silently when a referenced `CONTEXT.md` does not exist | PROSE-ONLY | partly — the reply mentions it | known |
| DM03 | docs/agents/domain.md:26 | Use the `CONTEXT.md` term for a domain concept in your output | PROSE-ONLY | partly — a glossary lookup | known |
| DM04 | docs/agents/domain.md:26 | Do not drift to a synonym the glossary explicitly avoids | PROSE-ONLY | partly — a synonym table | known |
| DM05 | docs/agents/domain.md:28 | Note a missing glossary term for `/domain-modeling` | PROSE-ONLY | partly — the reply | known |
| IT01 | docs/agents/issue-tracker.md:7 | Keep one effort per directory `.scratch/<effort-slug>/` | PROSE-ONLY | yes — a path shape | known |
| IT02 | docs/agents/issue-tracker.md:8 | The PRD or effort map is `.scratch/<effort-slug>/map.md` | PROSE-ONLY | yes — the file name | known |
| IT03 | docs/agents/issue-tracker.md:9 | A ticket is `.scratch/<effort-slug>/issues/NN-<slug>.md`, from `01` | PROSE-ONLY | yes — the path shape | known |
| IT04 | docs/agents/issue-tracker.md:10 | Put linked assets in `.scratch/<effort-slug>/assets/` | PROSE-ONLY | yes — the path | known |
| IT05 | docs/agents/issue-tracker.md:11 | Keep a machine-only file in `assets/local-only/` and cite it by path | PROSE-ONLY | yes — the path | known |
| IT06 | docs/agents/issue-tracker.md:12 | Record triage state as a `Status:` line near the top of a ticket | PROSE-ONLY | yes — the line | known |
| IT07 | docs/agents/issue-tracker.md:13 | Append a comment under `## Comments` or `## Decisions so far` | PROSE-ONLY | yes — the heading | known |
| IT08 | docs/agents/issue-tracker.md:17 | Publish by creating a file under `.scratch/<effort-slug>/issues/` | PROSE-ONLY | yes — the path | known |
| IT09 | docs/agents/issue-tracker.md:21 | Fetch a ticket by reading the referenced path | PROSE-ONLY | partly — a tool-call trace | known |
| IT10 | docs/agents/issue-tracker.md:30 | The frontier is the first open, unblocked, unclaimed ticket by number | PROSE-ONLY | yes — a file scan | known |
| IT11 | docs/agents/issue-tracker.md:31 | Set `Status: claimed` and save before any work | PROSE-ONLY | yes — the line and the order | known |
| IT12 | docs/agents/issue-tracker.md:32 | Resolve: answer under `## Answer`, `Status: resolved`, then the map pointer | PROSE-ONLY | yes — the three edits | known |
| TL01 | docs/agents/triage-labels.md:3 | Use one of the five canonical role strings as the `Status:` value | PROSE-ONLY | yes — a fixed vocabulary | known |

### .pi/agents/*.md (117 words)

| id | source | rule (shortened) | class | machine-checkable | status |
|---|---|---|---|---|---|
| PA01 | .pi/agents/implementer.md:3-4 | The `implementer` agent runs on `claude-sonnet-5-5` with `thinking: high` | CODE — the harness reads the front-matter | yes — the key values | known |
| PA02 | .pi/agents/reviewer.md:3-5 | The `reviewer` runs on opus with `thinking: high` and read-only tools | CODE — the harness reads the front-matter | yes — the key values | known |
| PA03 | .pi/agents/scout.md:3-5 | The `scout` runs on `deepseek-v4.1-flash` with read-only tools | CODE — the harness reads the front-matter | yes — the key values | known |
| PA04 | .pi/agents/judge-*.md:5 | A judge agent sees only `read`, `ls`, `find`, `grep`, `bash` | CODE — the harness reads the `tools` key | yes — the list | known |
| PA05 | .pi/agents/judge-glm.md:2 | The `name:` key matches the file name | PROSE-ONLY | yes — the pair | inferred |

---

## Part 2 — Enforcements with no written rule behind them

The reverse direction. Each row is a check that fails a commit or a run, and no sentence among the
rule sources tells an agent to avoid what it catches. Status is `known` for the code fact and
`inferred` for the absence.

### Guardrail tests

| enforcement | what it fails | written rule? |
|---|---|---|
| `structure.rs:309` `check_file_length` (2000 non-blank lines) | a file over 2000 non-blank lines | none |
| `nesting.rs:12` `check_nesting_depth`, `MAX_NESTING_DEPTH = 3` (`nesting.rs:9`) | control-flow nesting deeper than 3 | none |
| `enums.rs:119` `check_enum_variant_docs` | an enum variant without `///` or a `[TRIVIAL_ENUM]` marker | none |
| `free_fn.rs:31` `check_free_fn_location` | a top-level free fn outside six folder names | none |
| `inherent_impl.rs:15` `check_inherent_impl_locality` | an inherent impl outside the type's file or folder | none |
| `test_hygiene.rs:37` `check_sleep_has_reason` | a `sleep(..)` in a test with no comment above | none |
| `style.rs:9` `check_import_ordering` | an import order other than std, external, crate | none. `rustfmt.toml:10` sets `reorder_imports = false`, so rustfmt will not fix it |
| `style.rs:125` `check_long_comment_runs` | five or more consecutive countable comment lines | none. The count 5 is unwritten |
| `style.rs:245` `check_separator_comments` | a `// === … ===` separator | none |
| `style.rs:260` `check_single_letter_vars` | a single-letter variable in a fn over 10 statements | C02 asks for verbose names but sets no threshold and no statement count |
| `style.rs:166` `check_template_raw_strings` | an `r#"` literal in an askama template source | none |
| `structure.rs:204` `check_empty_rust_file` | a `.rs` file of comments and blanks only | none |
| `structure.rs:128` `check_mod_purity` | an item in `mod.rs` other than a module, import or doc | none |
| `structure.rs:238` `check_no_std_thread_all` | `std::thread` in `src/` | none |
| `location.rs:7` `check_test_file_naming`, `location.rs:42` `check_test_file_pairing` | a `_test.rs` file, and an unpaired `_tests.rs` file | tests/AGENTS.md:29 gives the mirror convention only |
| `layers.rs:9` `check_wiredapp_scope` | a `WiredApp` import outside four scopes | none |
| `layers.rs:58` `check_handler_return_type` | a handler that returns `(StatusCode, String)` | none |
| `layers.rs:166` `check_server_layer_boundaries`, `layers.rs:203` `check_http_storage_leak` | an HTTP file that names a whole-state type or `Storage` | none among the sources. `arch-lint.toml` holds the scope rows |
| `answer_words.rs:12, :40, :86, :132` (four checks) | a `post` route with no words row, a stale row, an unmatched request path, a `Html<String>` post handler | none |
| `screen.rs:48` … `:766` (13 checks) | a part outside the shell, a script that writes a part or `disabled`, an `hx-swap-oob`, a draft control outside a dialog form, and more | only `guardrails.md` (generated) describes them |
| `structure.rs:7` `MODULE_DOC_EXEMPTIONS` (`lib.rs`, `main.rs`, `test_support/`) | nothing — it silences C06 for three paths | C06 says "every file in `src/`" |
| `mod.rs:183` `dead_scope_violation` | a path-limited guardrail whose scope matches no file | none |
| `registry_tests.rs` | a rule with no detection test, an unregistered walker table | none |

### Design-rule walkers

`walker_rows.rs:13, :20, :42, :51, :61, :82, :95` cite design rules 1, 2, 3, 6, 7, 13 and 18.
No file among the rule sources defines those numbers. `rg "Rule (1|2|3|6|7|13|18)"` over the six
rule sources returns nothing; the only definitions are the generated `guardrails.md` and
`.scratch/` research notes. `inferred`: the design rule set lives outside the agent's rule sources.

### Python checks and gate steps

| enforcement | what it fails | written rule? |
|---|---|---|
| `scripts/check_python_docstrings.py` | a Python file with no module docstring | none |
| `scripts/extract_http_routes.py --check` (`build.py:1078`) | a stale `http_routes.md` | none. AGENTS.md:349 names only the index generators |
| `scripts/precommit_regenerate.py:1-10` | a commit that would sweep a bystander's prose into a generated block | none |
| `scripts/check_test_structure.py` | an inline `#[cfg(test)] mod` in `src/`, an unregistered `*_tests.rs` | none |
| `scripts/validate_docs.py:502` `BROKEN_MARKDOWN_LINK` | a link to a missing file | none |
| `scripts/validate_docs.py:546` `STANDARD_PLAN_LINK` | a link from a standard doc into `docs/plans/` | D14 covers the reference list only |
| `scripts/validate_docs.py:810` `BROKEN_SOURCE_PATH` | a backticked source path that does not exist | none |
| `scripts/validate_docs.py:1099` `UNKNOWN_ROUTE` | a doc that names a route the router does not have | none |
| `scripts/healthcheck.py duplicates` (`build.py:1146`) | nothing — report-only | none |
| `src/lib.rs:4-25` nine `#![deny(clippy::…)]` | `unwrap`, `expect`, `panic`, `todo`, `unimplemented`, `print_stdout`, `print_stderr`, `dbg_macro`, a dropped `#[must_use]` | none among the sources. `guardrails.md` states the rationale, generated from this list |
| `arch-lint.toml:52+` layer scopes and `[rules.no-unwrap-expect]` | a cross-layer import, an unwrap in production | none among the sources |
| `cargo fmt` (`rustfmt.toml:5`, `max_width = 100`) | unformatted code | `AGENTS.md:366` names the format check, not the width |
| `styles/Chronicler/ForumLinks.yml:9` | a forum or community URL in a doc | none |
| `styles/Chronicler/FutureTense.yml:9` | future tense in a doc | D32 covers "not a change to it" only |
| `styles/Chronicler/PastDeletion.yml:8` | past-tense deletion history | none |
| `.config/nextest.toml:1-5` retries 0, `test-threads = 3` | a flake, via no retry | none. The reason is a comment in the config |
| permission `grep * . *` (`config.json:27`), `grep * ./ *` (:31), `grep * /workspace/chronicler-engine *` (:35) | a repo-root `grep` | the rule exists only as the deny reason text |
| permission `find / *` (`config.json:43`) | `find /` | the rule exists only as the deny reason text |
| permission `timeout` and `timeout *` (`config.json:19-25`) | the `timeout` command | the rule exists only as the deny reason text |
| permission path `*.env`, `*.env.*` (`config.json:5-6`) | reading or writing a `.env` file | none |

---

## Part 3 — Contradictions and duplicate statements

| # | pair | verdict |
|---|---|---|
| P1 | CODING_STANDARDS.md:13 "Every file in `src/` (excluding `*_tests.rs` unit tests and `src/test_support/*.rs`) has:" vs CODING_STANDARDS.md:15 "required for `src/test_support/*.rs` too" | contradiction in one paragraph. One sentence excludes the folder, the next requires line 2 there. Known: the guardrail (`structure.rs:7`) exempts the folder entirely, so line 15 has no check |
| P2 | CODING_STANDARDS.md:13 "Every file in `src/`" vs `structure.rs:7` `MODULE_DOC_EXEMPTIONS: &[&str] = &["lib.rs", "main.rs", "test_support/"]` | contradiction. Two files that the rule names are exempt from the check, and no rule states the exemption |
| P3 | CODING_STANDARDS.md:14 "Line 1: `//! [DOC: …]`" vs `src/lib.rs:1` `// [DOC: docs/diataxis/reference/coding_standards/guardrails.md]` | contradiction in the repository itself. Line 1 of `lib.rs` is a plain `//` comment, not a `//!` doc anchor, so C06 fails on its own example file |
| P4 | `style.rs:124` "Warns when five or more countable comment lines appear consecutively" and the `Violation::warn` calls in `style.rs` vs `mod.rs:154-168` `assert_violations`, which panics on any non-empty list: "panic!(… {} violation(s) found for {} …)" | contradiction between the documented severity and the behaviour. A "warning" fails the guardrails binary, so `Warning` is not a warning |
| P5 | tests/STRATEGY.md:3-5 "Each rule ends with its **Check**: … or **review-only**" vs tests/STRATEGY.md:30 "A `#[tokio::test]` with fakes is a unit test." and :152 "Overlap across tiers is expected" | contradiction of the document's own frame. These two rules carry no Check line, so the reader cannot tell whether they are checked or review-only |
| P6 | AGENTS.md:398 "These restrictions exist to prevent the agent from touching git without supervision" vs AGENTS.md:400 "Normal git commands are fine" and `config.json:18` `"*": "allow"` under `bash` | contradiction between the stated purpose, the sentence, and the config. No `git` entry denies a command, so nothing stops a commit |
| P7 | AGENTS.md:400 "Don't commit without explicit approval, even if committing is allowed in the permissions config" vs `config.json:18` `"*": "allow"` | the same fact stated twice with different force. The rule is honest about the gap (A32), and the config is the reason |
| P8 | .pi/APPEND_SYSTEM.md:3 "Prose: ASD-STE100 Simplified Technical English." vs docs/AGENTS.md:7 "Write documentation using Simplified Technical English (STE)." | duplicate rule, two files, two wordings, one enforcement (`.vale.ini:11`). The second is the docs-only restatement |
| P9 | tests/AGENTS.md:21 "LLM tests are `#[ignore]`d by default." vs tests/STRATEGY.md:23 "provider flows, `#[ignore]`d and run with `--llm-only`" | duplicate rule, two files. One enforcement: `test_hygiene.rs:12` with the `tests/llm/` exemption |
| P10 | CODING_STANDARDS.md:18 "Never write comments that paraphrase what the code does." vs docs/AGENTS.md:164 "A doc does not explain how the code works by listing its modules, types and methods." | duplicate intent in two scopes, one for code and one for docs. Both are SKILL or PROSE-ONLY, so a reader cannot tell which skill owns it |
| P11 | docs/AGENTS.md:136 "Cite a symbol … not a `path:line`." vs CODING_STANDARDS.md:7 "Symbols … must use verbose, domain-aligned names that map 1-to-1 with concepts in the `docs/`" | circular: the doc must cite a symbol, and the symbol must map to a doc. Neither states which direction wins when a symbol has no doc |
| P12 | `.pi/agents/judge-deepseek.md:2` `name: judge-glm`, `.pi/agents/judge-opus.md:2` `name: judge-glm`, `.pi/agents/judge-glm.md:2` `name: judge-glm` | contradiction. Three files declare one agent name, and two file names disagree with their `name:` key. `implementer.md:2`, `scout.md:2` and `reviewer.md:2` match their file names, so PA05 holds everywhere else |

---

## Part 4 — Counts

### Rules per source

| source | hand-written words | rules |
|---|---|---|
| AGENTS.md (hand-written part) | 1,144 | 41 |
| AGENTS.md (generated Structure block, lines 7–329) | 2,808 | 0 — generated, no rule |
| CODING_STANDARDS.md | 282 | 20 |
| .pi/APPEND_SYSTEM.md | 111 | 10 |
| ENVIRONMENT.md | 1,232 | 5 |
| tests/STRATEGY.md | 2,459 | 57 |
| tests/AGENTS.md (hand-written part) | 362 | 11 |
| tests/AGENTS.md (generated block, lines 46–199) | 3,309 | 0 — generated, no rule |
| docs/AGENTS.md (hand-written part) | 1,261 | 37 |
| docs/AGENTS.md (generated index, lines 45–98) | 147 | 0 — generated, no rule |
| docs/agents/*.md (three files) | 601 | 18 |
| .pi/agents/*.md (eight files) | 117 | 5 |
| **total** | **13,833** | **204** |

The totals count the hand-written words of the three files that carry a generated block:
1,144 + 282 + 111 + 1,232 + 2,459 + 362 + 1,261 + 601 + 117 = 7,569 hand-written words. The
generated blocks add 6,264 words and carry no rule.

### Rules per enforcement class

| class | rules | share |
|---|---|---|
| PROSE-ONLY | 157 | 77 % |
| CHECK | 35 | 17 % |
| SKILL | 6 | 3 % |
| CODE | 4 | 2 % |
| PERMISSION | 2 | 1 % |
| **total** | **204** | 100 % |

`known` reading: the CHECK share concentrates in two sources. tests/STRATEGY.md holds 18 of the
35 CHECK rules, because each of its rules names its own step. Its other 39 rules are PROSE-ONLY,
declared **review-only**. The other ten sources hold 17 CHECK rules and 118 PROSE-ONLY ones.

---

## Part 5 — The 10 PROSE-ONLY rules worth a check

Chosen for cost when broken, and for a check that a machine can make. Each check is one sentence.

| # | rule | why it costs | the check to write |
|---|---|---|---|
| 1 | D25 "`erDiagram` — do not use" (docs/AGENTS.md:143) | a diagram that cannot show the invariant it exists for, and `validate_docs.py` passes it | Fail when a ```` ```mermaid ```` fence in `docs/diataxis/` contains the token `erDiagram`. |
| 2 | C14 "Never reference the task in code comments" (CODING_STANDARDS.md:20) | the comment rots and the next reader cannot resolve it | Fail when a comment in `src/` matches `used by|added for|issue #|TODO from review|from the review`. |
| 3 | C19 "Tautological tests are considered harmful" (CODING_STANDARDS.md:32) | a green test that pins nothing, which the gate cannot see | Fail when an `assert_eq!` in a `*_tests.rs` file compares a call with the same call written again. |
| 4 | D29 "Name a section by what it explains, not as a defence" (docs/AGENTS.md:158) | it frames the doc as a rebuttal and Vale passes it | Fail when an `##` heading in `docs/diataxis/explanation/` matches `^#+ *(Why|What this .* is not)`. |
| 5 | T47 "`networkidle` … is banned at every tier" (tests/STRATEGY.md:226) | a wait that never settles, so the test times out or passes late | Fail when `networkidle` appears in any file under `tests/`. |
| 6 | C01 "Do not preserve backward compatibility unless the user asks" (CODING_STANDARDS.md:5) | a shim that no rule removes, and every later change carries it | Fail when `src/` holds `#[deprecated]`, a `_legacy` identifier, or a `backward_compat` module. |
| 7 | A33 "destructive git commands should be avoided" (AGENTS.md:400) | an unrecoverable loss of another agent's work | Fail when a session's bash trace runs `git reset --hard`, `git clean -fd`, `git checkout -- .`, or `git restore .` |
| 8 | D35 "State what a thing is; do not describe it by what it is not" (docs/AGENTS.md:174) | the reader learns the boundary and not the behaviour | Fail when a body sentence in `docs/diataxis/reference/` starts with `It is not`, `This is not`, or `There is no`. |
| 9 | D15 "Cite no spec scenario numbers in a Diátaxis doc" (docs/AGENTS.md:41) | the number moves on the next spec edit and the citation lies | Fail when a `docs/diataxis/**` file holds a `\b\d{1,2}\.\d{1,2}\b` token on a line that names a file under `docs/specs/`. |
| 10 | T34 "Within one tier, one behaviour has one test" (tests/STRATEGY.md:156) | a duplicate test doubles the cost of every refactor and hides which one owns the regression | Fail when two tests in one binary carry the same SCENARIO tag, or the same name after the module path is removed. |

`guessed` on the ranking of these ten: no run data records which rule agents break. The order
follows the cost of the failure, not a measured frequency.

---

## Closing notes

- `known`: the CHECK class is under-counted for rules that a skill also restates. A rule is assigned
  the strongest enforcement, so a CHECK rule that a skill repeats appears once.
- `known`: `AGENTS.md` states 41 rules in 1,144 hand-written words, and `tests/STRATEGY.md` states
  57 in 2,459. The build commands table (AGENTS.md:364-376) gives one rule (A23) and eleven options.
- `not recorded`: no file among the sources says who reviews a PROSE-ONLY rule, or how often.
