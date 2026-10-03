# Plan: `dashboard-ui-updates` follow-ups

**Date:** 2026-10-03
**Status:** Reviewed against HEAD `19712ddd`. D1, D3, D4, D5, D6, D7, D8, D9 and D10 are decided (see below). D2 moved out of this plan on 2026-10-03 to `old-docs/archived-plans/target-seed-absolute-paths-plan.md`. No decisions are open. Everything else is implemented; see the progress log. This plan lands on `dashboard-ui-updates`, merges into `main`, and the remaining dashboard work (map tickets) continues on a new branch.
**Sources:** `tmp/afterplan/report.md` (after-plan run of `dashboard-ui-updates` vs `main`, plus its verification pass) and `tmp/deletion-audit/report.md` (repo-wide deletion-test audit). `tmp/` is gitignored, so this file is the durable record. Report tags (`#N`, `R`, `N`, `D`) are kept so findings trace back.
**Goal:** Close what the after-plan report left open, as work packages (WP) ordered by cost and dependency.

## How to execute

- One WP = one ticket = one change set. Do the WPs in order unless a "Depends on" line says otherwise.
- Test first (AGENTS.md): for each fix, find or write the failing test, then fix.
- Iterate with `python build.py clippy` and `python build.py test-pattern <pattern>`. End each WP with `python build.py` green.
- Agents sharing one checkout use `python build.py --target-dir target/<name> --no-fmt`.
- Do not commit without user approval; commit through `/commit-and-push`. Review your own diff against this plan before asking.
- `assets/index.html` and `tests/` are also edited by open tickets in `.scratch/dashboard-ui-review/`. Run `git status` before each WP and do not touch other agents' uncommitted edits.

## Progress log

| WP | State | Evidence |
|---|---|---|
| WP0 | **Done** | P0.1, P0.2 landed; P0.6 dropped per D6. `tests/http/prompt_presets.rs` slices the impersonate add-form and fails when a field is renamed out of it. |
| WP1 | **Done** | S1–S6 landed; E4 notes fixed; DC5 notes fixed except two recorded below. Gate `logs/build_20261003_115325.log`: 1528 integration, 137 guardrail, 30 browser, 1 architecture, 0 failed. |
| WP2 | **Done** | B2, B3, B6 landed by `wp2-build-tooling` (own target dir); B5 landed 2026-10-03 after D3 was decided report-only. Gate `logs/build_20261003_144517.log` (18 steps): architecture 1, guardrails 137, integration 1523 (2 skipped), browser 30, 0 failed, with the duplicate summary in the log. B1 moved out of this plan to `old-docs/archived-plans/target-seed-absolute-paths-plan.md`. |
| WP3 | **Done** | 3a (DC1, DC2, D4, visibility) by `wp3-dead-code`; 3b (DC3 merges, A7 #17, DC4 inlines) by this session. |
| WP4 | **Done** | A2 landed; `python build.py browser` green (`logs/build_20261003_121531.log`, 30 passed). |
| WP5 | **Done** | A3, A5, A6 by `wp5-structure` (own target dir). |
| WP6 | **Done** | A1-pre and A1 landed, plus A7 #9 and A7 #19. |
| WP7 | **Done** | E1 coverage, E2 leak hunt and E3 by `wp7-environment` (own target dir). |

Whole-plan gate after WP3–WP6: `logs/build_20261003_123925.log` — fmt, clippy and validation OK; architecture 1, guardrails 137, integration 1523 (2 skipped), browser 30, 0 failed.

Final gate with every work package in the tree (WP7's ENVIRONMENT.md included): `logs/build_20261003_132337.log` — same counts, 0 failed, 117s.

Gate with B5 in the tree (`logs/build_20261003_144517.log`, 18 steps): 0 failed, and the log carries the duplicate summary at lines 310–313.

WP2 deviations and risks:

- **B3** changed the build-signature digest (`build_signature` no longer spawns `rustc`), so an existing warm target dir re-stamps once on its next build. Cargo remains the guard.
- **B2** behaviour change, flagged by the agent: `is_target_locked` no longer has the win32 `msvcrt` branch, so on Windows it reports "not locked" (matching `target_seed`'s documented no-op). The warning is advisory only.
- **B6** finding for WP7/E2: nextest 0.9.146 does emit `LEAK` (non-interactive `--status-level` accepts `leak`; `--final-status-level` does not), and the gate's `NEXTEST_STATUS_LEVEL=fail` suppresses live lines. Naming a leaking test needs `--test-timings` or an explicit `--status-level leak`.
- **B5** landed 2026-10-03 after D3 was decided report-only. `scripts/healthcheck.py` gained changed-file scoping (`resolve_scope`, `list_changed_files`, `is_git_available`, `is_head_detached`, `resolve_base_ref`) that filters the parsed pairs, not jscpd's input; `build.py` gained an additive `duplicates` step in the gate that calls the check with `check=False` and never records a failure; new `scripts/tests/test_healthcheck.py` covers the filter and all three fallback notes. Default scope reports **236 pairs** and keeps the four known branch files; `--all` reports **572** (the plan's "≈577" came from the stale 11:18 report, so treat 572 as current). The manual `python build.py duplicates` still exits non-zero when jscpd is missing — only the gate step is required never to fail. Deviation: the step forwards `--all`/`--ref` as explicit flags rather than `argparse.REMAINDER`, which fails on a leading option in Python 3.13. Reviewed 2026-10-03 by a focused single-axis review (`review-b5`), because the two-axis pass predated the step: no hard findings — report-only, filter semantics, path normalisation and the missing-jscpd path all verified clean. Three judgement fixes applied: the summary prints the in-scope file count and calls out an empty scope or a scope filter that removed every pair, an unresolvable explicit `--ref` no longer claims `origin/main` was tried, and `test_healthcheck.py` gained the test that drives `check_duplicates` with a non-empty changed set (the uncovered wiring). The `chronicler-after-plan-workflow-plus-review` skill copy now matches the main one on reading the gate log instead of re-running the script.

WP7 results:

- **E1 coverage:** `python build.py --coverage --target-dir target/wp7 --no-fmt` is green (`logs/build_20261003_124430.log`, 530s; JSON at `target/wp7/llvm-cov/coverage.json`). **10094/11285 = 89.4%**, up from 8006/9059 = 88.4% on 2026-09-21; counted lines grew 24.6%. This branch's changed areas are all above the 80% line: `http/**` 93.9%, `application/**` 95.1%, `domain/model/template.rs` 100%, `storage/worlds.rs` 86.5%. `llm/transport/utils/client.rs` is still 0/88 — the already-recorded "live LLM HTTP leg has no covering test". No per-file JSON from the earlier run exists, so per-file movement cannot be claimed.
- **E1 documentation:** ENVIRONMENT.md was missing the coverage install steps and now carries them (`rustup component add llvm-tools-preview`, `cargo install cargo-llvm-cov --locked`).
- **E2 not reproducible:** 10 browser runs, all "30 tests run: 30 passed, 0 skipped", no `LEAK` line and no `leaky` segment in any build log (`tmp/wp7/browser_leak_run01..10.log`, `logs/build_20261003_131312.log` through `logs/build_20261003_132148.log`). No teardown fix was possible because no leaking test was ever named. Confirmed the B6 finding: `--status-level` accepts `leak`, `--final-status-level` does not.
- **E3:** no flake across those 10 runs, so the wall-clock-timer item needed no action.

WP3 deviations and notes:

- **DC1**: the audit's "zero in-repo callers" was wrong for four re-exports; the compiler needed them back (`lib.rs` `EngineError`, and three in `agents/quantifier`). Method held: delete, compile, restore what the compiler needs.
- **DC2** `model_swipes_to_db`: its five test uses moved to a test-local `to_db_swipes` helper, so the roundtrip coverage survives the deletion. `EngineError::Render`'s `error_tests.rs` fixture switched to `EngineError::Config`, keeping the non-displayable property under test.
- **DC3** `render_template` merged into `domain/model/template.rs` (7 import sites); `SaveLlmMessageFn` merged into `application/llm_recorder.rs` (4 sites); `name_uniqueness.rs` moved to `application/name_uniqueness.rs`. This also satisfies `.scratch/ponytail-audit-cuts/issues/02`'s file-deletion goal, but by merge rather than by inlining at call sites (D7).
- **DC4** inlined or deleted: `spawn_pipeline_task`, `parse_preset_type`, `render_preset_xml_parts`, and the uncalled `AppState::current_shutdown_token` (with its test). Kept, with reasons: `load_settings` (holds the defaults fallback and its warning log), `render_header_unlocked` (maps the askama error to `EngineError::Template`), `PromptContext::build_narration_prompt` (builds and configures an assembler), and the single-field `::new` constructors (idiomatic, and a struct literal is not clearer).
- **A7 #17** landed with DC3, then was partly reverted by review. `application/utils/` was gone; it is back, holding only `name_is_available` as a free `pub(crate) fn` in `src/application/utils/name_uniqueness.rs`. The review found `NameUniqueness` was a unit struct that existed only to dodge the free-fn guardrail, and `utils` is on that guardrail's allowlist, so the wrapper bought nothing.
- **Guardrail conflict, resolved by the guardrail's own remedy.** Merging `render_template` into `domain/model/template.rs` and moving `name_is_available` out of `application/utils/` put top-level free functions outside the folders `check_free_fn_location` allows (`mappers`, `utils`, `builders`, `test_support`, `bootstrap`, `handlers`), which `guardrails_free_fn_location` caught. `render_template` is now `TemplateVars::render(&self, text)` (17 call sites, the argument order flips) and stays that way; `name_is_available` went back into `utils/` as a free function instead of an associated one. D7's merges stand; A7 #17's folder removal does not.

WP5 deviations and notes:

- **A3** is user-visible: the panel's preset cards now show the truncated 120-character, newline-flattened preview, matching an edit-refreshed card, because both render one Askama partial. The single-card endpoints' names and previews now carry Askama's numeric entities (`&#34;`), which render identically. `preset_card_html` takes the `ModeActiveIds` bundle (two loose ids before review).
- **A5** is internal: `provider_options_html` and `narrator_mode_select_options_html(&str)` became one `SelectOptionsTemplate` over `SelectOptionView`, taking `NarratorMode`; the option lists no longer use `|safe`. New `utils/template_helpers_tests.rs` pins the escaping of `"` and `&`.
- **A6** is internal: `Storage::world_select_sql` derives the SELECT from `WORLD_COLUMNS`, `DbWorld::from_row` reads every column by name, and `update_world` routes through `write_map_row`. The S6 re-seed pair is unchanged and still asserts every persisted column.

WP6 deviations and notes:

- **A1** landed as designed. `error_fragment(message)` (no prefix) and `render_error(message)` (adds `Error: `) sit beside `error_fragment_response(message)` (200 + fragment) and `error_response(error, prefix)` (Validation → 400 + fragment; anything else → 200 + `"{prefix}: {error}"`). Settings passes `"Error"`, presets pass their existing prefixes, so no message text changed.
- `error_span` (17 call sites) and `preset_save_error_response` are gone; `panel_handler` returns `Response<Body>` (P0.5); the create-world refusal routes through `error_response(e, "Failed to create world")`, so scenario 25.7's 400 now carries the canonical refusal message.
- Specs: all 21 "error span" mentions across `prompt_presets.md`, `games.md`, `worlds.md`, `browser_dashboard.md` now say "error fragment" (the games 20.4–20.6 wording was already inaccurate — those handlers rendered a div before this change).
- Tests updated: 13 exact-equality assertions and the S3 check in `tests/http/prompt_presets.rs`, plus `tests/test_utils/server.rs`.
- **Left alone, for ticket 25:** the games template's inline `<div class="error-message">Presets unavailable: …</div>` (already the canonical class), and the bare-string 400s for malformed forms (`worlds.rs` `bad_request(e)`) which carry no fragment at all.
- **A7 #9** landed: `generate_preset_id()` became `generate_storage_id(prefix)`, and connection ids now carry the uuid-v4 suffix (they did not before).
- **A7 #19** landed as documentation, not signature churn: the refusal representation at the service boundary is `ApplicationError::Validation`, and `settings_service` documents that its uniqueness rule runs inside a `Storage` closure, whose error type is `EngineError`, converted at that one boundary. Moving the check out of the closure would reintroduce the check-then-write race that ticket 39 tracks.

WP1 deviations from the plan:

- **S2** added a spec scenario. Scenario 20.9's Given is a single game, so the two-game listing had no spec coverage. Added `docs/specs/games.md` scenario 20.10 and `test_games_fragment_lists_other_saved_games_http`, which builds the app first (the builder re-seeds its own active game) and then adds a second game through the storage seam. 20.9 now also asserts the active card renders.
- **S5** put the shared helper in `src/test_support/http.rs` with `#[allow(clippy::expect_used)]` on the function. Note for new `src/test_support/*.rs` files: arch-lint exempts only `test_*.rs` / `*_tests.rs` / `tests/` paths, so a helper file needs that attribute or an `// arch-lint: allow(no-unwrap-expect) reason="..."` comment.
- **E4** resolved the `structure.rs` TODO about `system.md` by deleting the dead check (`points_to_system_md`, `SYSTEM_MD_EXEMPT`, `is_system_md_exempt` and their call site): an anchor to `docs/architecture/system.md` already fails the "must resolve under `docs/diataxis/reference/`" rule. The `check_mod_purity` TODO is answered in its doc comment (the runner discovers `src` files only).
- **DC5** `TextCheckMode::Disabled`: no change. The service guard is the policy (pinned by `disabled_mode_returns_none` with a stub checker that would otherwise answer), and the adapter guard is what makes its own `Disabled => unreachable!()` arm sound (pinned by `harper_text_checker_tests.rs`). Two layers, both tested.
- **DC5** LLM transport test: the disposition note was left as a dated migration record. The real finding — the live LLM HTTP leg has no test (`call_chat_completions` has no test caller; only the `#[ignore]`d flow tests exist) — is recorded in "Not in scope" below.
- **DC5** port helpers: the doc comments were corrected (behaviour unchanged). `docs/plans/build-py-run-step-plan.md` now decides to delete the helpers instead, so this fix is superseded and will disappear with the file. Owner of that removal: the build-py-run-step session; it must not edit this plan.

## Review outcomes (2026-10-03)

Two-axis review (`code-review`: Standards and Spec) over the uncommitted change set at `HEAD 19712ddd` (113 files, +1526/−1560), plus a focused follow-up review of B5, which landed after the first pass had started.

Fixed from the review:

- **Spec drift.** `docs/specs/prompt_presets.md` (14 literals) and `docs/specs/settings.md` (2) still pinned the removed `<span class='error'>…</span>`, contradicting the tests that assert the `error-message` div. They now pin the real markup, in backticks because the literal contains double quotes.
- **Doc drift.** `guardrails.md`'s hand-written prose claimed the project "intentionally uses `log` instead of `tracing`" (there is no `log` dependency); the CHANGELOG bullet below it claimed a body-level observer re-resolves `#submit-btn`, which A2 removed; `template_helpers.rs` claimed every dashboard select renders through its partial (the game templates build their own).
- **Unspecified behaviour.** Worlds create turned non-`Validation` failures from 400 into 200 + fragment. Kept — the old 400-for-everything contradicted the sibling paths that 500 — and now specified rather than accidental: `docs/specs/worlds.md` scenario 25.9 with a failure-injection test in `tests/http/worlds.rs`.
- **Data clumps.** `preset_card_html`, `preset_card_view` and `PresetCardTemplate::new` take `&ModeActiveIds` instead of two loose ids.
- **Speculative wrapper.** `NameUniqueness` existed only to dodge the free-fn guardrail; see the A7 #17 note above.
- **Duplicated test helper.** Five local body-to-string helpers (caps 4096, 16384 and `usize::MAX`) collapsed onto the shared `test_support::body_text`. The caps were reconciled onto the non-truncating `usize::MAX` deliberately: every body involved is a small fragment and no test depends on truncation.
- **Minors.** `PresetCardTemplate::is_active` was a stored restatement of its two inputs; `preset_card_html` no longer swallows a render error into `""` and returns the shared render path instead.
- **B5 follow-up.** No hard findings. Three judgement items fixed: the summary prints the in-scope file count and calls out an empty scope or a filter that removed every pair, an unresolvable explicit `--ref` no longer claims `origin/main` was tried, and `test_healthcheck.py` gained the test that drives `check_duplicates` with a non-empty changed set. The `chronicler-after-plan-workflow-plus-review` skill copy now matches the main one on reading the gate log.

Deferred, with reasons:

- **Provider list.** `SelectOptionView::providers` repeats three `LlmBackendType` names, but the enum has no canonical iteration and the list deliberately omits `Mock`, so deriving it would mean adding a new list rather than removing one.
- **Left for their own tickets:** the worlds bare-string 400s and the games template's inline error div (ticket 25), and the near-identical test blocks that D3 keeps report-only.

Gate over the frozen tree: `logs/build_20261003_151015.log` (18 steps) — architecture 1, guardrails 137, integration 1524 (2 skipped), browser 30, 0 failed.

## Review notes (what changed from the previous draft)

| Finding | Change |
|---|---|
| The old Phase 0 said "before committing". The tree is committed (`19712ddd`, 11:22). The recorded gate log `logs/build_20261003_023851.log` (02:43) predates it. | Phase 0 is now WP0, "close out the committed tree", and ends with a fresh full gate. |
| P0.3 (permission-config review) | Removed, per user. |
| P0.1, P0.2, P0.5, P0.6 | Re-checked at HEAD; all four still hold. |
| P0.6: existing `Load failed` tests in `prompt_presets_tests.rs` inject a `get_preset` failure, not a settings failure | Confirms the settings branch of `edit_preset_form_handler` is untested. |
| DC2 said `EngineError::{Serialize,Render}` are both dead | Only `Serialize` has zero references. `Render` is built in `src/adapters/driving/http/error_tests.rs` as a fixture; changing that test is part of deleting it. |
| DC2 "delete outright" items | Several have unit tests (`response_tests.rs`, `mappers/message_tests.rs`, `token_budget_tests.rs`) that must be deleted with them. "Zero callers" means zero production callers. |
| DC2 `backend_selector` | Confirmed: no non-test caller. It is implemented on the agents and only called from tests. |
| A7 #13 (merge the two poll pause/resume pairs) | Moved into ticket 38. Inferred, not verified: both helpers keep one module-level `originalTrigger`, so a second `pausePolling()` overwrites it. That would explain "editing a second entry freezes the poll". |
| A1 waited on tickets 08/25/26 | Ticket 08 is blocked by 04 and 05, so this is a long wait. The world-message duplication is split out as a ready item (A1-pre). |
| DC1 "~67 lines" | Not re-counted. The method (delete, compile, restore on error) does not depend on the count. |
| Terminology | "NPC" in new text is "Character", per `CONTEXT.md`. Quoted CHANGELOG text is unchanged. |

## Decisions

The recommendation is mine unless marked. Items I did not verify are marked *inferred*. D1, D4, D5, D6, D7, D8, D9 and D10 were decided by the user on 2026-10-03.

| # | Decision | Outcome |
|---|---|---|
| D1 | A4: one source for action-area markup | **Deferred.** A4 is out of WP4 until the user reopens it. |
| D4 | DC2: `validate_loaded_data` (unwired) | **Delete** the module, its 5 tests and the re-export. |
| D5 | DC2: `Agent::backend_selector()`, `AgentConfig.{backend,phase}` | **Deferred.** Leave in place; not part of WP3. |
| D6 | P0.6: untested settings branch in `edit_preset_form_handler` | **Drop** the `app_state.settings()` call and its comment. |
| D7 | DC3: module merges | **Only two:** `domain/model/utils/template.rs` → `domain/model/template.rs`, and `application/llm_message.rs` → `llm_recorder.rs`. Plus A7 #17. The other single-consumer modules stay. |
| D8 | DC4: wrappers to inline | **Wrappers that only forward a call.** Keep a wrapper that holds logic (for example `load_settings` falls back to defaults and logs). |
| D10 | A1: wait for ticket 08? | **No.** Implement A1 now, with no display-policy change: use the existing `.error-message` class and markup. Tickets 25/26 (new branch) build on it. |
| D9 | DC1: narrowing public re-exports | **Go.** An unused re-export is removed. Record it in the CHANGELOG. |

Open decisions:

| # | Decision | Options | Recommendation | Gates |
|---|---|---|---|---|
| D2 | B1: seeded `build/` output | Rewrite source→destination prefix in copied `output` files, or exclude `build/` | **Moved out of this plan on 2026-10-03** to `old-docs/archived-plans/target-seed-absolute-paths-plan.md`, which carries the measurements and the two options. | — |
| D3 | B5: does the gate fail on clones? | Report-only, or fail above a threshold | **Decided 2026-10-03: report-only.** jscpd is heuristic, and today's top pairs are legitimate near-identical test blocks, so any threshold low enough to catch real clones also reds untouched code. Revisit only after changed-file scoping has run for a while. | WP2 |

## WP0 — Close out the committed tree

**Depends on:** none. **Size:** small.

1. **P0.1** — Scenario 21.1 impersonate inputs are not observed.
   - File: `tests/http/prompt_presets.rs`, `test_prompt_presets_panel_renders_full_surface` (assertions at ~L123–128); spec `docs/specs/prompt_presets.md` 21.1.
   - Now: the five `name="…"` checks run against the whole body, which the system add-form already satisfies.
   - Fix: slice the impersonate add-form (from `value="impersonate"` to its closing `</form>`) and assert the five names inside the slice: `name`, `role`, `instructions`, `writing_style`, `output_format`.
   - Verify: `python build.py test-pattern prompt_presets_panel_renders_full_surface`. Then remove `writing_style` from the impersonate form temporarily and confirm the test fails.
2. **P0.2** — `docs/CHANGELOG.md`, `## 2026-10-03`. Move "rendered NPC portrait names as `.image-label` with a tooltip" out of the "Copy sweep" bullet into its own bullet (ticket 22). "Copy sweep" keeps the label associations only.
3. **P0.6** — dropped (D6): remove the `app_state.settings()` check and its comment in `edit_preset_form_handler` (`prompt_presets/handlers/prompt_presets.rs`). Confirm `python build.py test-pattern prompt_presets` stays green.
4. **P0.5** is not done here. It is `panel_handler` building `<span class='error'>` inline at its settings-load failure (it returns `Html<String>`); fold it into A1.
5. Run `python build.py` and keep the log as the new gate record.

**Done when:** the gate is green on a tree that includes these edits.

## WP1 — Test sharpness and stale notes

**Depends on:** none. **Size:** small, test-only plus comments. All P3; each scenario is also pinned one tier down.

- **S1** — 21.31, `tests/http/prompt_presets.rs` `test_create_preset_same_name_in_another_category_is_allowed` (~L1016). `body.contains("Alpha")` is already true from the system preset. Assert "Alpha" inside the quantifier section slice.
- **S2** — 20.9, `tests/http/games_fragment.rs` `test_games_fragment_lists_only_other_saved_games_http` (~L88). It seeds only the active game, so `!contains(">Current<")` passes even when no active card renders. Seed a second game; assert it is listed under Saved Games and the active game is not.
- **S3** — 21.32, `tests/http/prompt_presets.rs` `test_update_preset_keeps_its_own_name` (~L1044). Assert the posted `Changed.` instructions appear in the response or storage. Replace the broad `!contains("error")` with a check for the error element (`class='error'`, until A1 renames it).
- **S4** — 20.11, `tests/http/settings.rs` (~L392). Drop `body.contains("Alpha")` or scope it to the connection card; `alpha-model-2` already carries the check.
- **S5** — `body_text` is duplicated in `prompt_presets/handlers/prompt_presets_tests.rs` and `settings/handlers/settings_tests.rs`. Move one copy to `src/test_support/` and delete both locals. Check the name does not collide with an existing helper there.
- **S6** — Re-seed keeps the other world columns (report #27). Extend the InMemory/SQLite re-seed pair in `src/adapters/driven/storage/worlds_tests.rs` to assert every bound world column, not only id, card, map and characters.
- **E4** — stale notes:
  - `arch-lint.toml` (~L30) says the project uses `log` + `env_logger`; `src/` uses `tracing`. Fix the comment.
  - `tests/infrastructure/guardrails/structure.rs` L17 and L155 carry stale `TODO`s (deleted `system.md` guardrails; "is this actually catching all problems"). Resolve each or delete it.
  - `src/domain/model/settings.rs` (~L280) DeepSeek carve-out in `check_api_key_available`: leave until the DeepSeek provider lands; the comment already says so.
- **DC5 notes:**
  - `storage/core.rs` doc-comment calls `Storage` a "backend trait"; it is a concrete struct. Fix the comment.
  - `http/utils/port_utils.rs` says "(Windows only)" on two functions but has no `#[cfg(windows)]`, so `netstat` and `taskkill` compile everywhere. Decide: gate with `cfg`, or correct the comment if the platform-agnostic behaviour is intended. Read the callers first. **Resolved 2026-10-03:** the comments were corrected in WP1 (behaviour unchanged). **Superseded:** `docs/plans/build-py-run-step-plan.md` decides to remove the helpers outright, along with `build.py kill_port`/`kill_by_name` and the kill attempt in `bootstrap/port.rs`. When that lands, WP1's comment fix goes with the file; do not treat it as work to preserve.
  - `.scratch/test-strategy-execution/assets/integration-migration-disposition.md:11` cites `tests/integration/adapters/driven/llm/llm_client.rs`, which does not exist. Only the `#[ignore]`d `tests/llm/flow_llm_tests.rs` covers the live LLM HTTP leg. Correct the note; do not restore the test here.
  - `TextCheckMode::Disabled` is asserted in `text_check_service.rs` and `harper_text_checker.rs`. Delete the redundant one.

**Verify:** `python build.py` green. **Done when:** S1–S4 each fail when the behaviour they name is broken (mutate once, then restore).

## WP2 — Build tooling

**Depends on:** D3. **Size:** medium. Order inside the WP matters.

1. **B2 + B3** (one change, shared files).
   - B2: `build.py` `is_target_locked` takes `LOCK_EX`; `scripts/target_seed.py` `_source_is_idle` takes `LOCK_SH`. A seed in progress makes `build.py` report the dir busy. One shared helper, one lock mode.
   - B3: `rustc -vV` is run in `build.py` (~L706) and twice in `target_seed.py` (~L43, ~L74). One helper in `target_seed.py`; `build.py` imports it; drop the discarded third call.
   - Verify: `python build.py py-tests`.
2. **B1** — **moved out of this plan** (2026-10-03) to `old-docs/archived-plans/target-seed-absolute-paths-plan.md`. It is a build-tooling change to a file WP2 had just touched, and the peer build-tooling session owns that area. Do not re-open it here; the new plan holds the measurements (10 `output` files, four C crates), the mechanism (cargo compares the `output` mtime), and the options.
3. **B6** — the gate summary hides a leaky test.
   - Facts: `logs/build_20261003_021636.log:309` reads `30 tests run: 30 passed (1 leaky), 0 skipped`, but the gate line says `nextest: 30 passed, 0 failed`. `_nextest_summary_line` (`build.py` ~L264) drops the leaky count. `_NEXTEST_RESULT_RE` (~L246) matches `PASS|FAIL|SKIP` only. `leak` is not a valid `--final-status-level` value (valid: `none, fail, flaky, slow, skip, pass, all`).
   - Step 1: parse `(\d+) leaky` and render `nextest: 30 passed, 0 failed, 1 leaky`. Test in `NextestSummaryTests` (`scripts/tests/test_build_cli.py`) feeds the recorded line and expects the leaky count. Run with `python build.py py-tests`.
   - Step 2 (naming the test): find out whether nextest emits a `LEAK` line in non-interactive mode at all, and why the browser step's log has no per-test lines. If it does, accept `LEAK` in `_NEXTEST_RESULT_RE`. If it does not, use a structured reporter (`--message-format json` or JUnit via `.config/nextest.toml`) or an explicit `--status-level leak` run. Record the answer in the ticket.
4. **B5** — duplicate check runs in the gate, scoped to changed files (decision made 2026-10-03: changed files are the default scope, the gate runs the check).
   - Files: `scripts/healthcheck.py` (`summarize_report`, the `duplicates` parser), `build.py`, `.agents/skills/chronicler-after-plan-workflow/SKILL.md` step 7.
   - Filter the parsed pairs, not jscpd's input list. A pair is in scope when either side changed; filtering the input would hide new code cloning old code.
   - "Changed" = `git diff --name-only <base>` (two-dot, so uncommitted tracked edits count) plus `git ls-files --others --exclude-standard`.
   - Fall back to whole-repo scope with a printed note when git is unavailable, HEAD is detached, or neither `main` nor `origin/main` resolves.
   - Keep `--all` (whole repo) and `--ref <ref>` (base). Add `python build.py duplicates` and call it from the full gate. Report-only per D3.
   - After this lands, step 7 of the after-plan skill reads the gate output instead of running the script again, so the two scopes cannot disagree.
   - Verify: the default run keeps the 4 known branch pairs (`tests/http/prompt_presets.rs`, `tests/http/settings.rs`, `tests/http/requires_migration/{fragment,connections}.rs`) and drops inherited test-block clones; `--all` restores 577 pairs (report figure); the gate log carries the summary.
5. **B4** — report-only risks. Fix one only if it bites: R6 `build.py` passes `Path.cwd()` to seeding (~L1078, ~L1088); R7 the lld wrapper cache is keyed by uid, so a toolchain bump can reuse an old `ld.lld`; R11 the build-slot lock file is mode `0o666` at a predictable `/tmp` path (dev container only). #23 needs no fix: dropping `image` saved nothing because `playwright-rs` pulls it in; correct the expectation. R10 (`once_cell` → `LazyLock`, duplicate dev-deps) stays with the dependency-audit ticket.

**Done when:** `python build.py` green and the gate log shows the leaky count and the duplicate summary.

## WP3 — Dead code and shallow modules

**Depends on:** none. **Size:** large; split into two commits if the diff is hard to review. All items are `src/` only; each deletion is test-protected by the compiler plus the existing suite.

**3a — Deletions (mechanical first)**

- **DC1** — delete the dead re-export lines, then drop `#![allow(unused_imports)]` in `src/adapters/driven/llm/transport/mod.rs` (the only occurrence in `src/`). Files: `application/mod.rs`, `application/games/mod.rs`, `application/generation/mod.rs`, `games/view_query.rs` (→ plain `use`), `prompting/mod.rs`, `agents/mod.rs`, `agents/options/mod.rs`, `agents/quantifier/mod.rs`, `quantifier/types.rs`, `llm/transport/mod.rs`, `text_check/mod.rs`, `storage/models/mod.rs`, `http/bootstrap/mod.rs`, `http/worlds/templates/mod.rs`, `http/games/templates/mod.rs`, `src/lib.rs`. Keep the live hops (`pipeline::{ActionPipeline, PhaseError}`, `debug::DebugStateView`).
  - Method: delete a file's lines, run `python build.py check`; restore any line the compiler needs. `tests/` also compiles under `check`, so test callers surface.
  - The crate is a library, so "dead" means no in-repo caller; D9 accepts that. `tests/infrastructure/guardrails/structure.rs` permits `pub use`, so this is convention, not repair.
- **DC2** — dead items. Delete each together with its tests:
  - `service_unavailable_generating()` (`http/utils/response.rs`) and its test in `response_tests.rs`.
  - `model_swipes_to_db` (`storage/mappers/message.rs`) and its use in `message_tests.rs`; keep any test coverage that checks real behaviour through another path.
  - `PromptLayer` (`prompting/types.rs`; one reference, ordering is an explicit array in the assembler).
  - `MAX_SYSTEM_TOKENS` (`prompting/token_budget.rs`) and the one assertion in `token_budget_tests.rs`.
  - `EngineError::Serialize` (zero references). `EngineError::Render`: switch the `error_tests.rs` fixture to a live variant first, then delete.
  - `ActionAreaViewModel.{error_message,available_actions}` (`http/view_models.rs`; invisible to rustc because they are `pub`).
  - `ActionPipeline::prompt_assembler()` (`pipeline/action_pipeline/core.rs`).
  - `validate_loaded_data` — delete `src/bootstrap/validate.rs`, `validate_tests.rs` and the re-export in `bootstrap/mod.rs` (D4). Check that `src/bootstrap/mod.rs` and the structure index in `AGENTS.md` no longer list the module.
  - `backend_selector` / `AgentConfig.{backend,phase}` stay (D5 deferred).
  - Narrow visibility: `MessageService::load_messages_into_state` has one in-crate caller (`message_service.rs`); make it private.

**3b — Merges and inlines** (after 3a, so the module map is stable)

- **DC3** — only the two merges below (D7). Strongest: `domain/model/utils/template.rs` (`render_template`, 17 callers) beside `TemplateVars` in `domain/model/template.rs`. Also `application/llm_message.rs` → `llm_recorder.rs`. Left alone (D7), listed for the record: `utils/xml.rs`, `scenario_defaults.rs`, `world_defaults.rs`; `driven/utils/mock.rs` → `providers/mock.rs`; `test_support/quantifier.rs` → `fixtures.rs`; `http/utils/port_utils.rs` → `bootstrap/port.rs`; `http/utils/view_mappers.rs` → its two handlers; `prompting/prompt_merge.rs` → the two provider adapters (also edit the free-function guardrail exemption). Fold **A7 #17** here: `src/application/utils/` holds only `name_uniqueness.rs`; move it up a level.
- **DC4** — inline only wrappers that just forward a call (D8). Candidates: `load_settings` (`utils/settings.rs`), `spawn_pipeline_task` (`pipeline/spawn.rs`), `render_header_unlocked` (`http/builders/headers.rs`), `parse_preset_type` (`http/utils/handler_helpers.rs`), `AppState::current_shutdown_token()`, `render_preset_xml_parts` (`prompting/builders/sections.rs`, verbatim forward), `PromptContext::build_narration_prompt` (`prompting/assembler.rs`), trivial `::new` constructors in `http/templates.rs` and `http/view_models.rs`.
- Note: `arrival_service.rs` re-implements the narration prefix owned by `pipeline/narration_generation.rs`. This is a convergence decision, not a deletion. Open a separate architecture ticket; do not do it here.
- After either sub-step, regenerate indexes with the `scripts/generate_*_index.py` scripts if the pre-commit hook is not installed.

**Done when:** `python build.py` green; the CHANGELOG notes the narrowed public surface.

## WP4 — Frontend shell (`assets/index.html`)

**Depends on:** none (A4 deferred, D1). **Size:** small. Check `git status` for overlap with open map tickets (02, 10, 38) before starting.

- **A2** — one status/Send-button writer. Today `setButtonState` (~L135), `setStatus` (~L148), `onStatusPoll` and a body-wide `statusObserver` (~L696, observing `document.body` at ~L739) write the same display. The observer exists because the action area is replaced (ticket 02). Fix: one `applyStatusDisplay()` called from the existing `hx-on::after-swap="onStatusPoll(this)"` hook (~L83); remove the observer. Verify the browser specs 16.5–16.11 and 30.4–30.9 stay green (`python build.py browser`).
- **A4 — deferred (D1).** Do not start. Action-area markup has one source. Now the shell copy (~L57–88) omits `required minlength="1"` and `disabled` and hardcodes `status ready`, while `ActionAreaTemplate` (`http/templates.rs`) has them. The choice changes submit behaviour: empty submits are blocked from the first load. Add or update a browser or HTTP test that submits an empty command on a fresh page.
- **A7 #13** — not in this WP. Pause/resume pairs (`pausePolling`/`resumePolling`, `pauseLlmPolling`/`resumeLlmPolling`) go to ticket 38, which owns the same code. Add a note to that ticket.

## WP5 — Backend structure (independent tickets)

**Depends on:** none. Each item can ship alone.

- **A3** — preset card has one source. `preset_card_html` (`http/builders/presets.rs`) truncates the preview to 120 chars and flattens newlines; the panel template (`prompt_presets/templates/prompt_presets.rs`, three inline card blocks) prints the full `preview_text()`. An edit-refreshed card looks different from the same card in the panel. Fix: one Askama card partial used by the panel loop and the single-card endpoints. Decide which preview behaviour wins (default: the truncated one), and say so in the CHANGELOG.
- **A5** — typed `<option>` rendering. `provider_options_html` (`http/utils/template_helpers.rs`) and `narrator_mode_select_options_html(selected_mode: &str)` (`http/builders/forms.rs`) build HTML strings that four templates emit with `|safe` (`games.rs`, `worlds.rs`, `settings.rs`). Fix: one option-list view model rendered by Askama; take `NarratorMode`, not `&str`. Test the escaped output of a value containing `"`.
- **A6** — world row SQL in one place (`storage/worlds.rs`). `WORLD_COLUMNS` (~L418) order differs from positional `DbWorld::from_row`; the 14-column `SELECT` is written three times (~L39, ~L56, ~L345); the `UPDATE maps` at ~L324 sits beside `write_map_row` (~L268). Fix: derive the `SELECT` from `WORLD_COLUMNS`, read columns by name, route the update through `write_map_row`. The S6 test covers this refactor; do S6 first.
- **A7 #9** — `conn-{millis}` id (`settings/handlers/settings.rs` ~L110) skips the collision guard in `generate_preset_id` (`utils/handler_helpers.rs`). Pre-existing. Reuse the guarded generator (rename it if "preset" no longer fits).
- **A7 #19** — `PromptPresetService` mixes `Result` and `Result<_, ApplicationError>`; `SettingsService` returns `EngineError::Validation`. Pick one rule for refusals and write it in the module docs. Ties to A1; do it in the same sitting if A1 is open.

## WP6 — One error renderer (A1)

**Depends on:** none (D10). Ticket 08 decides the display policy later; A1 does not preempt it. It keeps the existing `.error-message` markup and class, and only collapses the four owners into one. Tickets 25/26 then change *where* errors show, on top of one renderer. Add a done-when line to ticket 25: "uses the shared error renderer from A1".

- **A1-pre (ready now)** — the `WorldAlreadyExists` sentence is written twice (`src/error.rs` `#[error(...)]` and `application/world_catalogue.rs`). Make `world_catalogue.rs` reuse the `EngineError` message. Test: the existing refusal test in `worlds_tests.rs` still passes unchanged.
- **A1 design (scoped 2026-10-03, read-only pass):**
  - `utils/error.rs` gains `error_fragment(message)` = `<div class="error-message">{escaped}</div>` (no prefix) and keeps `render_error(message)` = `error_fragment("Error: {message}")`, plus one `error_response(error: ApplicationError, prefix: &str) -> Response<Body>`: `Validation` → 400 + `render_error(&message)` (unchanged), anything else → 200 + `error_fragment("{prefix}: {error}")`.
  - Presets: delete `error_span` (17 call sites) and `preset_save_error_response`; call the shared helpers. Text stays as it is today; only the element and class change.
  - `panel_handler` returns `Response<Body>` (P0.5) so it uses the shared path.
  - `worlds.rs` `bad_request(format!("Failed to create world: {e}"))` routes through the shared refusal path once the error type there is `ApplicationError`.
  - One class, one rule: everything emits `.error-message` (already styled, `assets/styles.css`); nothing emits `<span class='error'>`, which had no rule. No CSS change needed.
  - Element shape becomes `<div>`, so the prompt-presets spec scenarios that say "returns an error span" (21.3, 21.5, 21.6, 21.8, 21.11, 21.15, 21.17, 21.19, 21.20, 21.22, 21.24) say "returns an error fragment", and the exact-equality assertions in `tests/http/prompt_presets.rs` (`<span class='error'>…</span>`) become the shared div. The `class="error-message"` assertions in `games_tests.rs`, `worlds_tests.rs` and `response_tests.rs` already match and stay.
  - WP1's S3 assertion `!body.contains("class='error'")` must move to the new class when A1 lands.

- **A1** — four places own error rendering: `http/utils/error.rs`; `settings/handlers/settings.rs` (`application_error_response`); `prompt_presets/handlers/prompt_presets.rs` (`error_span`, `preset_save_error_response`, and the inline span in `panel_handler`, P0.5); `worlds/handlers/worlds.rs`. Settings and games render `<div class="error-message">`; presets render `<span class='error'>`, which has no CSS rule (`assets/styles.css` has `.error-message` and `.status.error` only).
  - Fix: one `ApplicationError` → fragment function in `utils/error.rs`; one error class with a CSS rule; `panel_handler` returns `Response<Body>` like its siblings.
  - Update the assertions that match the old class (`games_tests.rs`, `worlds_tests.rs`, `response_tests.rs`, S3).
  - Side find: `panel_handler` uses `unwrap_or_default()` on the preset lists, so a storage failure renders an empty panel with no error. Decide with ticket 25 whether that stays.

## WP7 — Environment and unknowns

**Depends on:** B6 for E2. **Size:** small, mostly investigation.

- **E1** — coverage. `cargo-llvm-cov` v0.9.1 was installed on 2026-10-03 (`rustup component add llvm-tools-preview`, then `cargo install cargo-llvm-cov --locked`), but nothing has run since. Last known figure: 2026-09-21, 8006/9059 = 88.4%. Run `python build.py --coverage` and review the branch-changed files. Expect a full recompile: `cargo-llvm-cov` overrides `RUSTC_WRAPPER`, so sccache is bypassed. The binary lives in the container's user home, so a rebuild drops it; record both install steps in `ENVIRONMENT.md`, or bake them into the image.
- **E2** — name the leaky browser test. Three direct runs and the 02:38 gate showed 0 leaks; only the 02:16 run showed one (report #25). After B6, run `python build.py browser` repeatedly (the browser step runs three at a time) until the new summary shows a leak, then read the name from the B6 step-2 reporter and fix its teardown. Close as "not reproducible" after a bounded number of runs (suggest 10) with the log paths recorded.
- **E3** — `tests/browser/stub/dashboard.rs` (~L141, ~L219) depends on wall-clock timers (report #26). Stable in four runs. No action unless it flakes.

## Owned elsewhere — do not open a second ticket

| Finding | Owner |
|---|---|
| B1: seeded `build/` output carries absolute paths | `old-docs/archived-plans/target-seed-absolute-paths-plan.md` (moved out on 2026-10-03) |
| Theme 1: errors and health redesign | Tickets 08, 09 (decisions) → 25, 26 (fixes). All open; 08 is blocked by 04 and 05. |
| #3: error fragments replace whole panels | Tickets 25, 26 |
| Text-check-enabled review (blocks 08) | Ticket 04 (blocked by 03) |
| #8: editing a second entry freezes the poll; A7 #13 | Ticket 38 |
| Storage-level preset name uniqueness | Ticket 39 |
| Retrigger driven in a browser test | Ticket 40 |
| N1, N7: frontend doc alignment, new-game `<label for>` coverage | Ticket 23 |
| N9: competing tier-1 rules in `tests/STRATEGY.md` | Tickets 31, 32 |
| Remaining review tickets | 06, 07, 10, 12–16, 18, 19, 21 |
| Final re-review | Ticket 24 (blocked by all others) |

Prior rulings stay closed: `.scratch/ponytail-audit-cuts` "Out of scope" (thin service wrappers, generation gate, empty `http/*` dirs) is confirmed by the deletion audit. `.scratch/architecture-deepening` tickets 06/07 cover interface shape, not deletion.

## Won't do

| Finding | Reason |
|---|---|
| N3: `docs/plans/coverage-run-reliability-plan.md:66` stale thread counts | Plans are historical |
| N4: old CHANGELOG claim about duplicate-ID detection | Historical entry |
| N5: `.scratch/.../test-audit/docs_drift.md` shows fixed items | Dated snapshot by design |
| N6: stale prose in closed tickets 03, 27, 29, 33, 37 and map line 20 | Ticket records are historical |
| N8: 21.1 card assertions bend around the fixture | Pre-existing accommodation |
| R4: browser ×3 memory-pressure flake surface | Accepted by design; documented |
| R5: relative `rustc-wrapper` breaks raw cargo outside the repo root | Documented entry point is `python build.py` |
| #29: ticket 33's "352→250 lines" | Not reproducible; no action possible |

## Not in scope

- The permission-config delta (reviewed by the user).
- Changing how failures display (tickets 08, 25, 26).
- Converging `arrival_service.rs` with `narration_generation.rs` (separate architecture ticket).
- Restoring the missing `llm_client.rs` integration test. **Follow-up worth its own ticket:** the live LLM HTTP leg (`call_chat_completions`) has no covering test; the disposition table that claimed one in `tests/integration/adapters/driven/llm/llm_client.rs` describes a path that no longer exists.
- Any new feature work on `dashboard-ui-updates`.

## Failure modes

| Change | If it fails | Handling |
|---|---|---|
| DC1 re-export removal | Compile error in a file or test that used the hub path | Restore that line; it is not dead |
| DC2 deletions | A deleted item had a hidden caller (macro, `cfg(test)`, doc anchor) | `python build.py check` covers `tests/`; `validate-docs` covers doc anchors |
| B1 prefix rewrite | Rewritten `output` file breaks a build script's cached flags | Moved out of this plan; `old-docs/archived-plans/target-seed-absolute-paths-plan.md` owns it |
| B5 scoping | Git missing or detached HEAD gives an empty change set and a false "no clones" | Whole-repo fallback with a printed note |
| B6 parser | Nextest changes its summary wording | The unit test pins the recorded line; add new wording as a second case |
| A2 observer removal | Send button sticks after a text-check swap | Browser specs 16.5–16.11 and 30.4–30.9 |
| A4 server-rendered shell | First paint differs, or empty commands now blocked | New empty-submit test; D1 |

## Suggested order

1. WP0, then a fresh `python build.py`.
2. WP1 (test-only, cheap), then WP2 (a broken seeded build blocks every worktree; B2/B3 first).
3. WP5 items and WP4 as capacity allows; they do not depend on each other.
4. WP3 as one or two mechanical change sets.
5. WP6 (A1-pre first, then A1).
6. WP7 last; E2 only after B6.
