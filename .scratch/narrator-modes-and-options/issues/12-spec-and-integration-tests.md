# Task: Specs + integration tests — IF mode and options-autogeneration

Type: task
Status: pending
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
