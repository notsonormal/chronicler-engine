# Task: Specs + integration tests — IF mode and options-autogeneration

Type: task
Status: resolved
Blocked by: 07, 11, 14

## Question

Write the `docs/specs/` spec(s) and `tests/http/` integration tests for the two features on this map — IF/CYOA narrator mode and options-autogeneration — mirroring the steering spec/test pattern (see `docs/diataxis/reference/narrative/ai_steering.md` and its test suite). Blocked until the implementations land (tickets 07/08 for IF mode, 11 for options).

### Scope

1. **IF mode coverage.** Posture inheritance (world → game), the per-mode preset registry retargeting on mode-switch (mode-tagged list per ticket 13), the perspective nudge, the IF preset bundle loading (`system_if_default` seed from ticket 14), and the IF posture end-to-end (narrator elaborates terse player input). Mode-switch action coverage per ticket 07's UI.
2. **Options coverage.** On-demand `/options` generation → dock render → Use (option becomes `MessageType::Input`, narration follows) → Edit (fills input box) → regenerate (set replaced, no history); always-on gating (fires after narration turns only when the game's toggle is on; never after impersonate turns); current-set persistence across a page reload; both prompt-seed shapes parse (tag-wrapped and numbered list).
3. **Specs.** One spec per feature under `docs/specs/`, annotated scenarios per `scripts/validate_feature_spec.py`, every scenario covered by a named integration test.
4. **Cross-feature.** Options available in BOTH modes (map standing preference); steering features available alongside options in both modes (ticket 04 decision 4).
5. **Review-deferred test debt (from ticket 07's two-axis review).** Three items deferred from ticket 07, all unit/test-layer:
   - **Worlds posture/update contract docs + scenarios.** Pin the shipped contract: a post omitting all three posture fields preserves every stored value; a partial post (one or two fields) resets the absent fields to defaults; an invalid value returns an error span into `#world-posture-status` and mutates nothing (decision 2-B; the posture endpoint's per-field preserve is already covered, the generic `update_world_handler` partial-reset is not).
   - **Failure-injection tests for the new game handlers (Cross-cutting A).** `TestOverride::internal` on the storage seam for `update_game_posture_handler`, `update_game_presets_handler`, and `switch_game_mode_handler` — assert each surfaces a 500 / error span on storage failure rather than panicking or silently swallowing (decision 6-B).
   - **`PresetForm` urlencoded parse test.** Add `serde_urlencoded` as a dev-dependency and assert `from_str::<PresetForm>("name=N&instructions=I&preset_type=system&allowed_mode_novel=true")` parses `allowed_mode_novel == true, allowed_mode_if == false`, and an omitted-flags body yields both false — pinning the checkbox grammar that previously needed a browser test to catch (the deleted ticket-07 marker test was vacuous; decision from the review's Finding 3).

## Notes for the session

- Read before implementing: the map's Decisions-so-far (tickets 01, 02, 04 — the three designs), `docs/diataxis/reference/narrative/ai_steering.md` (the spec pattern to mirror), the steering integration tests under `tests/`, `scripts/validate_feature_spec.py`.
- LLM-dependent scenarios use the mock LLM provider (`src/adapters/driven/llm/providers/mock.rs`), not real backends; real-LLM verification is the `--llm-only` lane, not this ticket.
- **Always-on e2e recipe (from ticket 11's verification).** The effective toggle is the GAME row's value, inherited from the world at game creation — `PipelineRun::resolve_options_always_on` is game-first, world fallback only when the row is unreadable (pinned by `test_world_toggle_does_not_enable_always_on_for_running_game` / `test_unreadable_game_row_falls_back_to_world_toggle` in `src/application/pipeline/action_pipeline/options_tests.rs`). The game-row toggle has no UI, so a browser e2e must seed the fixture world's `options_always_on` before the game is created (inheritance happens at creation), or set the game row via storage. The default mock returns canned `<suggestion>` options for unseeded options calls (branch priority pinned in `mock_tests.rs`), so the dock path needs no response seeding.
- `serde_urlencoded` is already a dev-dependency (added by ticket 11 for the WorldForm grammar pins) — the PresetForm parse test below reuses it.
- Build-green check: `python build.py` (fast suite) + `python scripts/validate_feature_spec.py` on the new specs.

## Answer

Shipped: three new specs + one extended spec, their covering tests at the HTTP E2E and browser tiers, and the PresetForm parse unit test. `python build.py` fully green (1612 passed, 0 failed, 2 skipped); `validate_feature_spec.py` 139/139 covered, 0 gaps, 0 orphans. Scope held to specs + tests — no production-code changes.

**Specs and coverage**
- `docs/specs/narrator_mode.md` 23.1–23.5 → `tests/http/narrator_mode.rs` (5 tagged tests): world→game posture+bundle inheritance at creation (prompt-level, all three posture fields resolved), mode-switch retarget (system-prompt marker swap), fragment re-render + perspective nudge (third→second), deliberate-perspective survival, impersonate in an IF game.
- `docs/specs/options.md` 24.1–24.11 → `tests/http/options.rs` (8 tests: 24.1, 24.2, 24.4–24.7, 24.10, 24.11) + `tests/browser/behaviour.rs` (2 tests carrying the 24.3/24.8/24.9 tags): on-demand render, empty-history error, regenerate-replaces (no history), failure-keeps-prior, always-on via world-inherited toggle, never-after-impersonate, numbered-list parsing, options in IF mode, reload persistence, Use-click submit, Edit fill+focus.
- `docs/specs/worlds.md` 25.1–25.4 → `tests/http/worlds.rs` (4 tests) with the shared `world_form_body` helper in `tests/http/test_helpers.rs`: generic-update posture preserve, per-field merge, unknown-value fallback, options-toggle checkbox grammar.
- `docs/specs/games.md` 20.4–20.6 → `tests/http/games_config.rs` (3 tests): storage failure on `/games/:id/posture`, `/presets`, `/mode` each surfaces 500 + error span via `.with_failure("update_game_config", TestOverride::internal(...))`.
- PresetForm urlencoded checkbox grammar → 2 unit tests in `prompt_presets_tests.rs` (no SCENARIO tag, unit tier).
- Scope-4 cross-feature: options-in-IF-mode is 24.11; steering-in-IF-mode is 23.5; always-on×impersonate is 24.7.
- 25.5 (posture-endpoint invalid value → error span) NOT added: already unit-covered in `src/adapters/driving/http/worlds/handlers/worlds_tests.rs` (`test_world_posture_invalid_value_renders_error_and_mutates_nothing`, `test_world_posture_omitted_field_keeps_stored_value`) — overlap rule.

**Shipped-contract notes (pinned as-is, not fixed)**
1. The deferred item's "a partial post resets the absent fields to defaults" is stale: the shipped `update_world_handler` merges posture per field, matching the map's read-modify-write decision. 25.2 pins the merge.
2. `/options` with no scene history surfaces as HTTP 500 with an error span (the action handler maps `EngineError::Validation` to `internal_error`) — pinned in 24.2. A validation rejection rendered as a 500 is a candidate follow-up if it should be user-displayable.
3. The generic world update silently falls back to the domain default for an unknown posture value (`parse_or_default`) — pinned in 25.3; candidate foot-gun.
4. The perspective-nudge skip branch is externally indistinguishable from the nudge branch in every reachable state (the perspective enum has two values and the defaults are those two values), so no test can pin the branch itself; 23.4 pins the user-facing guarantee (a deliberately-set perspective survives a mode switch) instead.
5. Prompt-assertion mechanics for future sessions: posture macros (`{{narrative_perspective}}`/`{{narrative_tense}}`) resolve into the **post-history (user) prompt**, not the system prompt (`WritingStyle`/`OutputFormat` filter, `builders/sections.rs`); the system prompt carries Role+Instructions+GlobalRules. The TestAppBuilder fixture's `system_if_default` is a test preset ("You are a test interactive fiction narrator."), not the production seed JSON — narrator_mode tests re-save it with macro-bearing `writing_style` to make posture observable.

**Test seams used:** `make_test_recorder_with_storage` + `Storage::list_latest_llm_messages` (prompt forensics), `OptionsAgent::with_provider` registry wiring (mirroring `options_tests.rs`), storage-seeded `WorldCard`s for posture/toggle inheritance, `TestOverride::internal` failure injection, `serde_urlencoded` dev-dep for the form-grammar tests.

**Map state after this ticket:** the only substantive fog left toward the destination is the Documentation item (CONTEXT.md terms, a diataxis reference for IF mode + options, DOC anchors) — candidate next ticket; the two small follow-ups (arrival-path always-on options, per-game always-on override UI) remain optional polish.
