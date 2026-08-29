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

## Notes for the session

- Read before implementing: the map's Decisions-so-far (tickets 01, 02, 04 — the three designs), `docs/diataxis/reference/narrative/ai_steering.md` (the spec pattern to mirror), the steering integration tests under `tests/`, `scripts/validate_feature_spec.py`.
- LLM-dependent scenarios use the mock LLM provider (`src/adapters/driven/utils/mock.rs`), not real backends; real-LLM verification is the `--llm-only` lane, not this ticket.
- Build-green check: `python build.py` (fast suite) + `python scripts/validate_feature_spec.py` on the new specs.
