# Plan check: P0 to P5

Source: scout `verify-p0-p5` (session b42e8412). Full tables are in that session; this file keeps the result.

## Result

Every Content item, test fate, guardrail rule, doc item and "Done when" check of P0 to P5 is done, superseded by the core step as the plan orders, or changed with a reason recorded in its slice ticket (known). No item is missing with no reason.

Recorded changes (all with reasons in tickets 01 to 06): `--test-target` not validated against the binary list; `unit_test_standards.md` deleted (owner); rule 2 narrowed to `catalogue.rs`; `GameId` newtype; `apply_game_rules` outcome shape; preset and budget reads stay in the Job (rule 11); `OptionsGeneratorAgent` name; `MockBackend` knobs `with_narrations`/`with_prompt_responses` kept for 3 legacy `src/` test files; the prompting tests (ticket 09); the in-memory `Tx` answers `Internal`; the `list_game_names_for_world` cases deleted, not moved; `empty_sqlite_storage` name.

Checkers run by the scout: `validate_seam_cases.py` (5 contracts, 96 cases), `validate_feature_spec.py` (292/292, quarantine 74/74), `generate_guardrails_doc.py --check` — all green. Gate green is known from `logs/build_20261019_213433.log`.

## Findings to carry

| # | Finding | Status | Lead check |
|---|---|---|---|
| P-1 | The stop (`POST /status/reset-generating`) and the status poll answer with no active Game ("failed" / idle, plan §4), but no spec scenario states that answer | open gap, reported in ticket 02, never closed | Known: `docs/specs/dashboard.md` has no "no active Game" scenario (grep) |
| P-2 | Plan §7 "Done when" still names the removed `[System] NPC detection uncertain` assertion | stale plan text, on purpose (ticket 07) | No action; plan is a record |
| P-3 | `validate_feature_spec.py` keeps the `#[ignore]` anchor path (`IgnoredTestAnchorTests`) though no target test is ignored now | vestigial, still used by `tests/llm/` rules? | To check in step 7 |
| P-4 | Lock order C10 (guard → backend mutex → connection mutex) not verified by reading the lock sites | inferred | Hand to reviewers |
