# Close the four-review findings on `dashboard-ui-issues-2`

## Summary

Fix every verified finding from the four reviews of `main...HEAD` (HEAD `b291dd82`). The work happens on the current branch, in nine ordered phases. The decisions below are locked:

- **Role health:** it stays engine-wide, and the UI labels it so.
- **Ticket-64 deferred paths:** the three paths are fixed now.
- **Generation failures:** they persist a typed kind and the raw text. The HTTP layer owns the sentence.
- **Old snapshots:** there is no compatibility shim.

Two review findings were corrected during verification:

- **TN F7 / AP 14 is invalid.** `effective_enable_auto_check()` reads the first assignment.
- **The TN F6 fix changed.** The test builder keeps the "slot busy, persisted Idle" case, because Task 4.1 tests that case.

## Key Changes

| Area | Change |
| --- | --- |
| Failure surfaces | A failed Text Check save keeps its card. The three deferred paths answer non-2xx. `error_response` is deleted. |
| Role health | One domain `Role` type. One HTTP mapping for the label and the failure effect. The stub renders the production header. The header says "engine-wide". |
| Generation errors | `GenerationStatus::Error(GenerationFailure { kind, raw })` is classified from the typed `EngineError`. The substring matcher is deleted. |
| "Is generating" | The dock and `/debug` read the live registry. |
| Options presets | `AppSettings` slot accessors and one card renderer replace the parallel Options code path. |
| Size | Four files over 1000 lines are split. The inline script moves to `assets/dashboard.js`. |

## Implementation

### Phase 0: Preparation

- [ ] #### Task 0.1: Clean the build state (1 SP)
  - Make sure that no other checkout is testing. Then run `python build.py --cleanup`. This deletes the target dir and the port locks. The next build is cold.

### Phase 1: Failure surfaces

- [ ] #### Task 1.1: A failed Text Check save keeps its card (3 SP)
  - `save_text_check_handler` returns `Response<Body>`: the card as 200 on success, `action_failure_response(format!("Save failed: {e}"))` on failure.
  - Add `<div class="inline-error-slot" data-error-slot="text-check-card" hidden></div>` as a direct child of the Text Check form.
  - In `tests/http/settings.rs`:
    - Inject the failure with `Storage::new_in_memory().with_failure("update_settings", TestOverride::internal(...))`.
    - Assert a 500, the failure text, and no card markup in the body.
    - Also cover the panel-load and edit-form-load failure arms.
  - Add scenario **20.22** to `docs/specs/settings.md` and tag the test.
- [ ] #### Task 1.2: Fix the three deferred failure paths (5 SP)
  - `create_world_handler`: use `action_error_response(e, "Failed to create world")`. Add an explicit `data-error-slot` to the world form.
  - `duplicate_preset_handler`: use `action_error_response` at both `error_response` sites. A missing preset gives a 400.
  - `activate_preset_handler`: send every failure arm to `action_failure_response`.
  - Split the load macro: `require_preset!` stays for the GET fragments, and `require_preset_action!` (400 refusal) serves the POST handlers.
  - Delete `error_response`.
  - Rewrite scenarios **21.22** (`docs/specs/prompt_presets.md`) and **25.9** (`docs/specs/worlds.md`). Update `test_duplicate_missing_preset_returns_error` and `test_world_create_storage_failure_renders_error_fragment_http`, together with its comment.

### Phase 2: Role health

- [ ] #### Task 2.1: One role identity (5 SP)
  - In the domain, add `Role { Narrator, Quantifier, Options, Trigger }` to `domain/model/agent.rs`, with `ALL`, `agent_name()` and `from_agent_name()`. Define the `AGENT_*` constants from it.
  - `RoleHealth` carries `role: Role`.
  - In the HTTP layer (`builders/headers.rs`), add one `role_label(Role)` and one `role_failure_effect(Role)`.
  - Delete the duplicate label tables in `view_query.rs`, `settings/templates/settings.rs` and `settings/handlers/settings.rs`. Also delete `role_effect` and the Settings template's import of `banner_message`.
- [ ] #### Task 2.2: The stub renders the production header (3 SP)
  - Expose `pub fn header_fragment_html(game_name, roles)` from `builders/headers.rs`.
  - `tests/test_utils/stub_server.rs` calls it. Delete the hand-copied OOB wrapper and the copied banner sentence.
- [ ] #### Task 2.3: Label role health as engine-wide (3 SP)
  - Only the header composer appends `" (engine-wide role health)"` to the banner summary.
  - The title of the Settings sub-tab marker becomes `"A role is degraded (engine-wide role health)"`.
  - Revise these files: `docs/diataxis/reference/frontend/dashboard.md`, `ui_design.md`, and the Degraded wording in `docs/specs/settings.md`, `failure_display.md` and `browser_dashboard.md`. Use this sentence: the newest attempt per role covers all games, so a Degraded marker can come from an earlier game until that role's next call succeeds.
  - Update the assertions that name the old sentence.

### Phase 3: Classified generation failures

- [ ] #### Task 3.1: Persist a typed failure (5 SP)
  - Domain types:
    - `GenerationFailureKind { Unreachable, PromptTooLong, UnreadableAnswer, SaveFailed, SceneMissing, Other }`.
    - `GenerationFailure { kind, raw: String }`.
    - `GenerationStatus::Error(GenerationFailure)`.
  - There is no legacy deserializer. This follows CODING_STANDARDS.md, and the user accepted it.
  - Update `view_query.rs` and every `GenerationStatus::Error(ref msg)` test site to use `.raw` or `.kind`.
- [ ] #### Task 3.2: Classify at the funnel (5 SP)
  - Add `GenerationFailure::from_engine_error(&EngineError)` beside `PhaseError`:

    | `EngineError` variant | `kind` |
    | --- | --- |
    | `Llm(Timeout \| Network \| Http)` | `Unreachable` |
    | `ContextOverflow`, `Narrative(PromptBuild)` | `PromptTooLong` |
    | `Llm(ParseError \| EmptyResponse)`, `Narrative(Generation)` | `UnreadableAnswer` |
    | `Database`, `Io` (persistence) | `SaveFailed` |
    | `RoomNotFound` | `SceneMissing` |
    | all other variants | `Other` |

  - Send the typed failure through these paths:
    - `pipeline_run.rs:95, 106`: these currently do `Err(e.llm_error_string())`.
    - `run.set_error` and `persist_snapshot_or_err`.
    - The `"LLM Error: empty response"` writer, which becomes `UnreadableAnswer`.
    - `narration_generation.rs`, `core.rs` and `arrival_service.rs`.
  - `llm_recorder.rs` keeps `llm_error_string()` for forensics.
  - Replace `generation_error_summary` with one HTTP `match` from kind to sentence. `endpoints.rs` renders the sentence plus `raw_error_detail(&failure.raw)`. Update the stub copy of this rendering.
  - Add a unit test for each mapping row.

### Phase 4: State ownership

- [ ] #### Task 4.1: One "is generating" source (3 SP)
  - The dock (`app_state.rs`) and `/debug` use `current_game_id()` + `generation_gate.is_busy(id)`.
  - Test both cases where the two sources disagree.
- [ ] #### Task 4.2: Test builder separates claim from record (3 SP)
  - Replace `is_generating(bool)` with `claim_generation_slot()`. Delete the `Idle` fallback. Update the three call sites.
- [ ] #### Task 4.3: Derive `display_name` once (3 SP)
  - `NewGame` derives it. Delete the four storage calls in `storage/games.rs` and `storage/db.rs`. The v25 migration keeps its backfill.
- [ ] #### Task 4.4: Small deduplication (3 SP)
  - One `EngineError → ApplicationError` conversion.
  - `bootstrap/wiring.rs` calls `pipeline.heal_stale_status(&generation_gate)`.
  - Move the `llm_message` fixture into `src/test_support/fixtures.rs`.

### Phase 5: Options presets and dead code

- [ ] #### Task 5.1: One slot accessor, one card renderer (13 SP)
  - [ ] ##### SubTask 5.1.1: `AppSettings` owns the active slots (5 SP)
    - Add `active_preset_id(&self, PresetType, NarratorMode) -> &str` and `set_active_preset(&mut self, PresetType, NarratorMode, String)`. The Options type ignores the mode.
    - Delete `PresetType::bundle_slot` and `set_bundle_slot`.
    - The delete guard uses the accessor.
  - [ ] ##### SubTask 5.1.2: One card context (5 SP)
    - `PresetCardTemplate` gets `preset`, `preview`, `active_badges: Vec<String>` and `activate_buttons: Vec<(String, String)>`. The card has no `is_options` branches.
    - `PromptPresetsTemplate` derives its cards from `AppSettings`.
    - Delete these items: `new_options`, `options_card_view`, `options_preset_card_html` and the `ModeActiveIds` `Default`.
  - [ ] ##### SubTask 5.1.3: Collapse the handler branches (3 SP)
    - The four handlers call one `card_response(preset, settings)`. Only the activate handler keeps its own write to the settings.
- [ ] #### Task 5.2: Dead code and readability (3 SP)
  - Delete the unused `.connection-card` badge and action CSS rules.
  - Delete the two `data-subtab` attributes, which have no reader.
  - Delete the unused `WorldFormTemplate` perspective and tense fields.
  - `actions.rs` calls `effective_enable_auto_check()`.

### Phase 6: Test hygiene

- [ ] #### Task 6.1: Make the weak tests able to fail (3 SP)
  - **Discarded result:** delete the `let _ = expect(...)` block in `tests/browser/dashboard.rs`.
  - **Bare sleep:** remove or justify the 200 ms sleep after `stub.stop()` in `tests/browser/stub/dashboard.rs`.
  - **Missing assertion:** add `assert_eq!(game.name, …)` to the SQLite parity arm in `games_tests.rs`.
  - **Hidden query error:** `swipes.rs` must propagate the DOM-query error instead of calling `unwrap_or_default()`.
- [ ] #### Task 6.2: Connection-edit failure gets its own scenario (3 SP)
  - Add scenario **16.35** to `docs/specs/browser_dashboard.md`. Move the second tag off 16.29.
- [ ] #### Task 6.3: Guardrail scanners (3 SP)
  - `registry_tests.rs` uses the `syn` visitor.
  - Add a comment in `http_assertions.rs` that pins the `visible_text` assumption.

### Phase 7: File size

- [ ] #### Task 7.1: Split `tests/browser/stub/dashboard.rs` (5 SP)
  - Split it into these modules:
    - `failure_display.rs`: 16.18–16.22, 16.25–16.27, 16.33, 16.34.
    - `form_failures.rs`: 16.29–16.32, 16.35.
    - `announcements.rs`: 16.23, 16.24.
    - `dashboard.rs`: the remaining scenarios.
    - `support.rs`: the shared helpers.
  - Register the modules in `stub/mod.rs`. Keep every SCENARIO tag.
- [ ] #### Task 7.2: Split `tests/browser/stub/story_log.rs` (3 SP)
  - Move the edit flow, the locks and the failure slots to `story_log_edit.rs`.
- [ ] #### Task 7.3: Move the inline script to `assets/dashboard.js` (3 SP)
  - Load it with `<script src="/assets/dashboard.js?v=1"></script>` at the same end-of-body position.
  - Update the inline-script sentence in `docs/diataxis/explanation/dashboard_design.md`.
- [ ] #### Task 7.4: Split `scripts/tests/test_build_cli.py` (3 SP)
  - Split it into three modules: registry and arguments; gate plan, tiers and journal; summary, epilogue and stamps. Each module needs a module docstring.

### Phase 8: Records and validation

- [ ] #### Task 8.1: Record the work (1 SP)
  - Add `.scratch/dashboard-ui-review/issues/71-close-review-findings.md`. In ticket 64's Answer, record that the three deferred paths are now closed. Update `map.md`.
- [ ] #### Task 8.2: Regenerate and run the full gate (3 SP)
  - Run these commands in order:
    - `python scripts/precommit_regenerate.py`
    - `python build.py validate-docs`
    - `python build.py spec-coverage`
    - `python build.py`

## Test Plan

- **Unit:** one test per classifier mapping row, plus tests for the `AppSettings` slot accessors, the `Role` name mapping, and the `NewGame` display name.
- **HTTP:** the Text Check failure (500, inline slot, no card swap), the rewritten scenarios 21.22 and 25.9, the Settings failure arms, and the dock and `/debug` registry reads.
- **Browser:** the full tier after the markup, CSS, stub-header and `dashboard.js` changes. The split modules run the same scenarios.
- **Spec coverage:** zero gaps, orphans or duplicate tags.
- **Coverage:** `python build.py --coverage` stays at 80% or more.

## Per Task/Sub Task Validation Steps

| Phase | After each task | Phase gate |
| --- | --- | --- |
| 0 | — | `--cleanup` completes |
| 1–4 | `python build.py test-pattern "<test or module>"` | `python build.py --no-browser` |
| 5 | `python build.py test-pattern "prompt_presets"` | `python build.py` |
| 6 | `python build.py test-pattern "<test>"`, `python build.py guardrails` | `python build.py` |
| 7 | `python build.py guardrails`; `python build.py browser` after 7.1–7.3 | `python build.py` |
| 8 | — | full gate + `validate-docs` + `spec-coverage` |

Read the results with `tail -n 10 "$(ls -t logs/build_*.log | head -1)"`. Use a bash timeout of 1200 s.

## Assumptions

- **Branch and fixed point:** the fixed point is `main...HEAD` at `b291dd82`. The work lands on `dashboard-ui-issues-2`.
- **Target dirs:** this is the main checkout, not a worktree. The lead uses the default target dir. Each delegated implementer uses `--target-dir target/<name> --no-fmt`.
- **Commits:** one commit per phase, and only with your approval.
- **Saved games:** `--cleanup` does not touch saved games. A game saved in the `Error` state loads as fresh state after Phase 3. You accepted this.
- **Excluded:** TN F7 / AP 14 (invalid), TP 10 (already on `main`), AP 9 (the dedicated query is the single implementation), CR STD-4 (vendored JS licences need upstream text), and the `ConnectionTestSurface` note (not a finding).
- **New `src/` modules:** none, so no new DOC anchors are needed.
