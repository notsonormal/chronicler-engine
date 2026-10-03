# Plan: `dashboard-ui-updates` follow-ups

**Date:** 2026-10-03
**Status:** Planning
**Source:** `tmp/afterplan/report.md` (after-plan run on `dashboard-ui-updates` vs `main`) plus its verification pass. A second, independent source feeds Phase 5: the repo-wide deletion-test audit at `tmp/deletion-audit/report.md`.
**Goal:** One readable list of everything the after-plan report left open, grouped by kind and ordered by cost. Item IDs are stable; `#N`, `D`, `N` and `R` tags point back into the report.

## How to read this

- **Phase 0** blocks the commit of the current uncommitted tree. Everything else is follow-up work.
- **Phases 1–4** are new work with no current owner. Each item is self-contained.
- **Owned elsewhere** lists findings that an open map ticket in `.scratch/dashboard-ui-review/` already covers. Do not open a second ticket for them.
- **Won't do** lists findings the report deliberately left alone, with the reason.

## Phase 0 — Before committing the current tree

### P0.1: Scenario 21.1 impersonate inputs are not observed
- **Files:** `tests/http/prompt_presets.rs` (`test_prompt_presets_panel_renders_full_surface`), `docs/specs/prompt_presets.md` 21.1.
- **Current state:** D7 added "the impersonate add-form contains inputs named name, role, instructions, writing_style, output_format". The test checks `name="name"` and the rest against the whole body, which the system add-form already satisfies.
- **Fix:** slice the impersonate add-form (the `<form>` after `value="impersonate"`) and assert the five input names inside that slice.
- **Verification:** `python build.py test-pattern prompt_presets_panel_renders_full_surface`; temporarily drop `writing_style` from the impersonate form and confirm the test fails.

### P0.2: CHANGELOG "Copy sweep" bullet mixes in ticket 22
- **File:** `docs/CHANGELOG.md`, `## 2026-10-03`.
- **Fix:** move "rendered NPC portrait names as `.image-label` with a tooltip" into its own bullet; keep "Copy sweep" to the label associations.

### P0.3: Review the permission-config delta
- **File:** `.pi/extensions/pi-permission-system/config.json` (report #28).
- **Current state:** changed on the branch, outside both workstreams, not reviewed.
- **Fix:** read `git diff main -- .pi/extensions/pi-permission-system/config.json` and confirm every change was intended. User decision.

### P0.4: Commit
- User-gated via `/commit-and-push`. The final gate (`logs/build_20261003_023851.log`) covers the tree; re-run `python build.py` if P0.1 or P0.2 land first.

### P0.5 (optional): Remaining inline error span
- **File:** `src/adapters/driving/http/prompt_presets/handlers/prompt_presets.rs:105` (`panel_handler`).
- **Current state:** still builds `<span class='error'>` inline because `panel_handler` returns `Html<String>`, not `Response<Body>`. Fold into A1 instead of fixing alone.

### P0.6 (optional): Untested settings-failure branch in the edit form
- **File:** same handler file, `edit_preset_form_handler`.
- **Current state:** calls `app_state.settings()` only to return `Load failed: …` on failure. No test pins that branch.
- **Decision:** either add a failing-settings unit test in `prompt_presets_tests.rs`, or drop the call if nothing needs it. Behaviour today is unchanged from `main`.

## Phase 1 — Test sharpness (test-only, cheap)

All four are P3 and each scenario is also pinned one tier down. Fix them together in one ticket.

### S1: 21.31 never sees the quantifier preset
- **File:** `tests/http/prompt_presets.rs:1016-1018` (`test_create_preset_same_name_in_another_category_is_allowed`).
- **Current state:** `body.contains("Alpha")` is already true from the system preset created first.
- **Fix:** assert "Alpha" inside the quantifier section slice of the panel.

### S2: 20.9 never renders a non-active game
- **File:** `tests/http/games_fragment.rs:88-95` (`test_games_fragment_lists_only_other_saved_games_http`).
- **Current state:** seeds only the active game; `!contains(">Current<")` passes even if no active card renders.
- **Fix:** seed a second game; assert it is listed under Saved Games and the active game is not.

### S3: 21.32 does not read the edit back
- **File:** `tests/http/prompt_presets.rs:1044-1050` (`test_update_preset_keeps_its_own_name`).
- **Fix:** assert the posted `Changed.` instructions appear in the response (or storage); replace the broad `!contains("error")` with a check for the error element.

### S4: 20.11 has one vacuous assertion
- **File:** `tests/http/settings.rs:392`.
- **Fix:** drop `body.contains("Alpha")` or scope it to the connection card; `alpha-model-2` already carries the check.

### S5: Promote `body_text`
- **Files:** `src/adapters/driving/http/prompt_presets/handlers/prompt_presets_tests.rs:32`, `src/adapters/driving/http/settings/handlers/settings_tests.rs:33` (report #22).
- **Fix:** one shared helper in `src/test_support/`; delete both copies.

### S6: Re-seed keeps the other world columns
- **Files:** storage tests for `seed_world` (report #27).
- **Fix:** extend the InMemory/SQLite re-seed pair to assert every bound world column, not just id, card, map and characters.

## Phase 2 — Code structure (one ticket each)

### A1: One error renderer
- **Files:** `src/adapters/driving/http/utils/error.rs`, `settings/handlers/settings.rs:23` (`application_error_response`), `prompt_presets/handlers/prompt_presets.rs` (`error_span`, `preset_save_error_response`), `worlds/handlers/worlds.rs:131`, `assets/styles.css` (report #2, P0.5).
- **Current state:** four homes for the refusal/error policy. Settings renders `<div class="error-message">`; presets render `<span class='error'>`, which has no CSS rule. The `WorldAlreadyExists` sentence is written twice (`src/error.rs:127`, `src/application/world_catalogue.rs:48`).
- **Fix:** one `ApplicationError` → fragment function in `utils/error.rs`, one error class with a CSS rule; `world_catalogue.rs` reuses the `EngineError` message.
- **Sequencing:** ticket 08 decides how failures display. Do A1 after 08, or fold it into tickets 25/26.

### A2: Status/Send-button state machine
- **File:** `assets/index.html` — `setButtonState` (`:135`), `setStatus` (`:148`), `onStatusPoll`, `statusObserver` (`:696`, observes `document.body` at `:739`) (report #1).
- **Current state:** three writers plus a body-wide `MutationObserver` that works around the action-area detach (ticket 02).
- **Fix:** one `applyStatusDisplay()` called from the existing `hx-on::after-swap="onStatusPoll(this)"` hook (`:83`); remove the observer.
- **Verification:** browser specs 16.5–16.11 and 30.4–30.9 stay green.

### A3: Preset card has one source
- **Files:** `src/adapters/driving/http/builders/presets.rs:121` (`preset_card_html`), `prompt_presets/templates/prompt_presets.rs` (three inline card blocks) (report #4).
- **Current state:** the builder truncates the preview to 120 chars and flattens newlines (`:176-180`); the panel prints `preview_text()` in full. An edit-refreshed card looks different from the same card in the panel.
- **Fix:** one Askama card partial used by the panel loop and the single-card endpoints.

### A4: Action-area markup has one source
- **Files:** `assets/index.html:57-88`, `src/adapters/driving/http/templates.rs:79` (`ActionAreaTemplate`) (report #6).
- **Current state:** the shell copy omits `required minlength="1"` and `disabled` and hardcodes `status=ready`.
- **Decision needed** (changes submit behaviour):

| Option | Effect | Risk |
|---|---|---|
| Shell renders the template on first load | One source; empty submits blocked from the start | Startup page becomes server-rendered for this region |
| Copy the template's attributes into the shell by hand | Smallest change | Two copies remain; can drift again |

### A5: Typed `<option>` rendering
- **Files:** `src/adapters/driving/http/utils/template_helpers.rs:4-17`, `builders/forms.rs:10` (`narrator_mode_select_options_html(selected_mode: &str)`), the `|safe` sites in `games/templates/games.rs:130`, `worlds/templates/worlds.rs:86`, `settings/templates/settings.rs:48` (report #10, #15).
- **Fix:** one option-list view model rendered by Askama; take `NarratorMode`, not `&str`.

### A6: World row SQL in one place
- **File:** `src/adapters/driven/storage/worlds.rs` (report #16, #18).
- **Current state:** `WORLD_COLUMNS` order differs from positional `DbWorld::from_row`; the 14-column SELECT is hand-written three times; `UPDATE maps` is inline at `:324` beside `write_map_row` (`:268`).
- **Fix:** derive the SELECT from `WORLD_COLUMNS`, read by name, route the update path through `write_map_row`.

### A7: Small code-shape items
- **#9** `conn-{millis}` id at `settings/handlers/settings.rs:110` skips the collision guard in `generate_preset_id` (`utils/handler_helpers.rs:35`). Pre-existing. Reuse the guarded generator.
- **#13** two near-identical poll pause/resume pairs (`assets/index.html:216-232`, `:779-796`). Merge into one helper taking the element.
- **#17** `src/application/utils/` only re-exports `name_is_available`. Move `name_uniqueness.rs` up a level or leave until a second helper arrives.
- **#19** `PromptPresetService` mixes `Result` and `Result<_, ApplicationError>`; `SettingsService` returns `EngineError::Validation`. Pick one rule for refusals and write it in the module docs.

## Phase 3 — Build tooling

### B1: Seeded target dirs link against the source checkout
- **File:** `scripts/target_seed.py` (`_COPIED_DIRS` includes `build`) (report #5 / R1).
- **Current state:** copied `build/<pkg>/output` files keep absolute `cargo:rustc-link-search` paths into the source checkout. Fingerprints stay fresh, so a seeded dir breaks only after the sibling's target dir is removed. Documented in `ENVIRONMENT.md` as a known limitation.
- **Decision needed:**

| Option | Cost | Benefit |
|---|---|---|
| Rewrite the source→destination prefix in copied `output` / `root-output` | Small text rewrite; must match cargo's file format | Keeps the full seed speed-up |
| Exclude `build/` from the seed | One-line change | Re-runs C builds (`aws-lc-sys`, `libsqlite3-sys`) on every cold dir |

- **Verification:** seed a fresh dir, delete the source target dir, relink, confirm the build succeeds.

### B2: One target-dir lock probe
- **Files:** `build.py:991` (`is_target_locked`, `LOCK_EX`), `scripts/target_seed.py:205` (`_source_is_idle`, `LOCK_SH`) (report #7).
- **Current state:** a seed holding `LOCK_SH` makes `build.py` report the dir as busy.
- **Fix:** one shared helper with one lock mode.

### B3: One `rustc -vV` probe
- **Files:** `build.py:706-707`, `scripts/target_seed.py:43`, `:74-75` (report #14).
- **Fix:** one helper in `target_seed.py`; `build.py` imports it; drop the discarded third call.

### B4: Report-only tooling risks
Fix only if one bites. Each is low.
- **R6** `build.py:1078,1088` pass `Path.cwd()` to seeding; elsewhere the script uses `Path(__file__)`.
- **R7** the lld wrapper cache is keyed by uid only, so a toolchain bump can reuse an old `ld.lld`.
- **R11** the build-slot lock file is mode `0o666` at a predictable `/tmp` path. Dev-container only.
- **#23** dropping `image` saved nothing; `playwright-rs` still pulls it in (`Cargo.lock:2108,2117`). Nothing to fix; correct the expectation.
- **R10** `once_cell` → `LazyLock` and duplicate dev-deps stay with the dependency-audit ticket.

### B5: Duplicate check runs in the gate, scoped to changed files

- **Files:** `scripts/healthcheck.py` (`summarize_report` `:204`, the `duplicates` parser `:272`), `build.py` (gate step + a `duplicates` subcommand), `.agents/skills/chronicler-after-plan-workflow/SKILL.md` step 7.
- **Current state:** the check always reports the whole repo — 577 clone pairs over 237 file pairs — so the after-plan step cannot separate branch-introduced clones from inherited ones (only 4 of the top 25 pairs touch branch files). `build.py` never calls it, so it runs only when someone remembers. Cost is not the obstacle: measured 2026-10-03, the full run takes about 1 s (`jscpd` re-ran; `report/jscpd-report.json` rewritten).
- **Decision (user, 2026-10-03):** changed files are the default scope, and the gate runs the check so no manual run is needed.
- **Fix:**
  - Scope by filtering the **parsed pairs**, not jscpd's input list. Pointing jscpd only at changed files would hide the case that matters most: new code cloning pre-existing code elsewhere. A pair is in scope when either side changed.
  - "Changed" is `git diff --name-only <base>` — two-dot, so uncommitted tracked edits are included — **plus** untracked files from `git ls-files --others --exclude-standard`. New files are where new duplication appears, and `git diff` alone misses them.
  - Fall back to whole-repo scope, with a printed note, when git is unavailable, HEAD is detached, or no `main`/`origin/main` resolves.
  - Keep `--all` for forced whole-repo scope and `--ref <ref>` to choose the base.
  - Add `python build.py duplicates` (AGENTS.md's convention that every gate step is also a subcommand) and call it from the full gate.
- **Decision needed — does the gate fail on clones?** jscpd pairs are heuristic; today's top pairs are legitimate near-identical test blocks.

| Option | Effect |
|---|---|
| Report-only (recommended) | The gate prints the changed-file summary; judgement stays with the reviewer |
| Fail above a line threshold | Needs a threshold that survives the inherited test-block pairs, or the gate goes red on untouched code |

- **Sequencing:** once the gate prints the summary, workflow step 7 reads the gate output instead of running the script again, so the two scopes cannot disagree.
- **Verification:** the default run keeps the 4 known branch pairs (`tests/http/prompt_presets.rs`, `tests/http/settings.rs`, `tests/http/requires_migration/{fragment,connections}.rs`) and drops the inherited test-block clones; `--all` restores 577; the gate log carries the summary.

### B6: The gate summary hides a leaky test

- **Files:** `build.py` (`_nextest_summary_line` `:264-285`, `_NEXTEST_RESULT_RE` `:245`, `--final-status-level pass` `:314`), `scripts/tests/test_build_cli.py` (`NextestSummaryTests` `:316`), `.config/nextest.toml`.
- **Current state:** nextest reports a leak only as a parenthetical in its Summary line — `logs/build_20261003_021636.log:309` reads `30 tests run: 30 passed (1 leaky), 0 skipped`, and `logs/build_20261002_171811.log:293` matches. No `LEAK` line appears in any log, so nothing names the test.
  - `build.py` renders its own epilogue from that line and extracts only passed/failed/skipped, so the condensed summary an agent reads says `nextest: 30 passed, 0 failed`. The leak stays invisible once anything condenses the raw output.
  - `_NEXTEST_RESULT_RE` matches `PASS|FAIL|SKIP` only, so a `LEAK` status line would also be dropped from the timing report.
  - `leak` is not a valid `--final-status-level` value (accepted: `none, fail, flaky, slow, skip, pass, all`); it is only a `--status-level` value. Naming the test is therefore not a one-flag change.
- **Fix:**
  - Parse `(\d+) leaky` in `_nextest_summary_line` and render it — `nextest: 30 passed, 0 failed, 1 leaky` — so no condensed summary can hide a leak.
  - Then find the name. First establish whether nextest emits a `LEAK` line in this run mode at all: the browser step's log carries no per-test lines, only the run banner and the Summary. If it does, accept `LEAK` in `_NEXTEST_RESULT_RE` and surface it in the timing report. If it does not, read the name from a structured reporter (`--message-format json`, JUnit) or an explicit `--status-level leak` browser run.
- **Unknown:** why the browser step's log holds no per-test lines, and whether nextest emits `LEAK` at all in non-interactive mode. Open the ticket with that question.
- **Sequencing:** before E2, which cannot name the test until this lands.
- **Verification:** a unit test in `NextestSummaryTests` feeds the recorded line `30 tests run: 30 passed (1 leaky), 0 skipped` and expects the leaky count in the one-liner, run via `python build.py py-tests`. No live leak needed.

## Phase 4 — Environment and unknowns

### E1: Coverage
- **Current state:** the tooling was restored on 2026-10-03, but coverage is still unproven. `cargo-llvm-cov` was never installed in this container (it lived in the pre-migration environment), and restoring it takes two steps, not one: `rustup component add llvm-tools-preview` (no `llvm-profdata` existed before) then `cargo install cargo-llvm-cov --locked` (v0.9.1 → `/home/node/.cargo/bin`). Nothing has run since the install, so the last real coverage number is 2026-09-21 (8006/9059 = 88.4%).
- **Fix:** `python build.py --coverage` and review branch-changed files. Expect a full recompile — under a coverage run `cargo-llvm-cov` overrides `RUSTC_WRAPPER`, so the sccache wrapper is bypassed and no instrumented artifact is a cache hit.
- **Durability:** the binary lands in the container's user home, so a container rebuild drops it again. Record both steps in `ENVIRONMENT.md`, or bake them into the image.

### E2: Name the leaky browser test
- **Current state:** nextest's Summary line in `logs/build_20261003_021636.log:309` reads `30 tests run: 30 passed (1 leaky), 0 skipped`; three direct runs and the final gate showed 0. No line names the leaking test, so the root cause is unknown (report #25).
- **Fix:** run the browser binary with nextest's leak status reporting enabled until a run names the test; then fix its teardown.
- **Sequencing:** B6 first — it makes the leak visible in the gate summary and settles whether nextest can name the test at all.

### E3: Watch the timed stub tests
- `tests/browser/stub/dashboard.rs:141` and `:219` depend on wall-clock timers (report #26). Stable in four runs. No action unless they flake.

### E4: Stale code notes
- `arch-lint.toml:30` says the project uses `log` + `env_logger`; `src/` uses `tracing` (report #21). Fix the comment.
- `tests/infrastructure/guardrails/structure.rs:17,155` carry stale `TODO`s about deleted `system.md` guardrails. Resolve or delete.
- `src/domain/model/settings.rs:280-285` DeepSeek carve-out (report #20). Delete when the DeepSeek provider lands.

## Phase 5 — Repo-wide deletion candidates (from the deletion-test audit)

**Source:** `tmp/deletion-audit/report.md` (resolution detail) plus raw evidence in `tmp/deletion-audit/data/`. `tmp/` is gitignored, so the summary below is the durable record — do not rely on the file surviving.
**Scope:** `src/` code shape across the whole repo, independent of the `dashboard-ui-updates` workstream. Nothing here was changed by the audit; land as its own change set, each item test-protected.
**Method:** the deletion test (delete the module; if complexity vanishes it was a pass-through, if it reappears across N callers it earns its keep). Verdicts: `dead`, `delete-inline`, `merge`, `keep`.
**Overlap — fold, do not duplicate:** DC3's `application/utils/mod.rs` hop is A7 #17; DC5's error-surface notes sit next to A1. Prior `.scratch/ponytail-audit-cuts` "Out of scope" rulings (thin service wrappers, generation gate, empty `http/*` dirs) are confirmed by the audit and stay closed; `.scratch/architecture-deepening` tickets 06/07 cover interface shape, not deletion.

### DC1: Dead re-export lines (~67, mechanical)
- **Files:** `application/mod.rs:26-32`, `application/games/mod.rs:12-13`, `application/generation/mod.rs:8-9`, `games/view_query.rs:14` (→ plain `use`), `prompting/mod.rs:12-14`, `agents/mod.rs:12-14`, `agents/options/mod.rs:9-11`, `agents/quantifier/mod.rs:7-16`, `quantifier/types.rs:9-11`, `llm/transport/mod.rs:8-13`, `text_check/mod.rs:11`, `storage/models/mod.rs:16-26`, `http/bootstrap/mod.rs:7`, `http/worlds/templates/mod.rs:6`, `http/games/templates/mod.rs:6`, `src/lib.rs` (`EngineError`, `AppSettings`, `AppState`, `TestData`).
- **Current state:** each line re-exports a symbol no in-repo caller imports through the hub path; several files are kept warning-free only because of `#![allow(unused_imports)]`.
- **Fix:** delete the lines; drop the `#![allow(unused_imports)]` in `llm/transport/mod.rs`; keep the live hops (`pipeline::{ActionPipeline, PhaseError}`, `debug::DebugStateView`).
- **Caveats:** the crate is a library, so "dead" is in-repo only — this narrows a public surface and is a deliberate decision. `tests/infrastructure/guardrails/structure.rs` permits `pub use`, so removals are convention, not repair.

### DC2: Dead functions, variants and fields
- **`validate_loaded_data` is unwired.** `src/bootstrap/validate.rs` (~70 LOC) + re-export `bootstrap/mod.rs:14` have **zero production callers**; `load.rs::seed_game_data` never invokes it. Decide intent before deleting: either wire the boot check or remove the module and its 5 tests.
- **Delete outright:** `service_unavailable_generating()` (`http/utils/response.rs:41-43`), `model_swipes_to_db` (`storage/mappers/message.rs:74-91`), `PromptLayer` enum (`prompting/types.rs`; ordering is an explicit array in the assembler), `MAX_SYSTEM_TOKENS` (`prompting/token_budget.rs:11`), `EngineError::{Serialize,Render}` (`src/error.rs`), `ActionAreaViewModel.{error_message,available_actions}` (`http/view_models.rs:136,139` — invisible to rustc because pub), `ActionPipeline::prompt_assembler()` (`pipeline/action_pipeline/core.rs:106-108`).
- **Unused seam:** `Agent::backend_selector()` + `AgentConfig.{backend,phase}` have no production reader (`registry.rs:42-53` builds nothing from them). Confirm intent, then drop or wire.
- **Narrow visibility:** `MessageService::load_messages_into_state` is `pub` with 0 external callers — make it private.

### DC3: Split concepts to merge (module boundary is shallow; behaviour stays)
- `domain/model/utils/template.rs` (`render_template`, 17 callers) belongs beside `TemplateVars` in `domain/model/template.rs`. Strongest merge candidate.
- Single-consumer modules: `domain/model/utils/xml.rs`, `scenario_defaults.rs`, `world_defaults.rs`; `driven/utils/mock.rs` → `providers/mock.rs`; `test_support/quantifier.rs` → `fixtures.rs`; `http/utils/port_utils.rs` → `bootstrap/port.rs`; `http/utils/view_mappers.rs` → its two handlers (flips to `keep` if a second consumer appears).
- Lone-alias module: `application/llm_message.rs` → `llm_recorder.rs`. Config helper: `prompting/prompt_merge.rs` → the two provider adapters (also edit the free-fn guardrail exemption).

### DC4: One-call-site wrappers to inline
`load_settings` (`utils/settings.rs`), `spawn_pipeline_task` (`pipeline/spawn.rs`), `render_header_unlocked` (`http/builders/headers.rs`), `parse_preset_type` (`http/utils/handler_helpers.rs`), `AppState::current_shutdown_token()` (test-only caller), `render_preset_xml_parts` (`prompting/builders/sections.rs`, verbatim forward), `PromptContext::build_narration_prompt` (`prompting/assembler.rs`), and the trivial `::new` constructors in `http/templates.rs` / `http/view_models.rs`.

### DC5: Adjacent health notes (not deletions, but surfaced by the audit)
- `storage/core.rs` doc-comment calls `Storage` a "backend trait"; it is a concrete struct. Fix the comment.
- `http/utils/port_utils.rs` says "(Windows only)" but has no `#[cfg(windows)]` gate — `netstat`/`taskkill` compile on every platform. Latent cross-platform defect.
- The live LLM HTTP leg is effectively untested: `tests/integration/adapters/driven/llm/llm_client.rs` (cited by `.scratch/test-strategy-execution/assets/integration-migration-disposition.md:11`) does not exist; only the `#[ignore]`d `tests/llm/flow_llm_tests.rs` covers it. Correct the disposition note or restore the test.
- `TextCheckMode::Disabled` is asserted twice (`text_check_service.rs:24-27`, `harper_text_checker.rs:56-58`); one of the two is redundant.
- `arrival_service.rs` re-implements the narration prefix owned by `pipeline/narration_generation.rs` — convergence candidate, architecture decision.

## Owned elsewhere — do not duplicate

| Finding | Owner |
|---|---|
| Theme 1: errors and health redesign | map tickets 08, 09 (decisions) → 25, 26 (fixes) |
| #3: error fragments replace whole panels | tickets 25, 26 |
| Text-check-enabled review (blocks 08) | ticket 04 |
| #8: editing a second entry freezes the poll | ticket 38 |
| Storage-level preset name uniqueness | ticket 39 |
| Retrigger driven in a browser test | ticket 40 |
| N1, N7: frontend doc alignment, new-game `<label for>` coverage | ticket 23 |
| N9: competing tier-1 rules in `tests/STRATEGY.md` | tickets 31, 32 |
| Remaining review tickets | 06, 07, 10, 12–16, 18, 19, 21 |
| Final re-review | ticket 24 (blocked by all others) |

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

## Suggested order

1. Phase 0 (P0.1–P0.4), then commit.
2. Phase 1 as one test-only ticket.
3. B1 decision and B2, since a broken seeded build blocks every worktree.
4. A2 and A3, which are self-contained.
5. A1 together with tickets 08 / 25 / 26.
6. DC1 and DC2 as one mechanical ticket, folding A7 #17 into DC3.
7. Everything else as capacity allows.
