# Research: map the test coverage of generation states and retry forms

Type: research
Status: resolved
Blocked by: —

## Question

Which tests cover each generation state, each Snapshot pointer, each retry form and each game-state writer? Which tests assert the restored game state, and which assert only the display? Which test doubles hide a defect?

## Scope

1. Integration tests (`tests/`, read `tests/AGENTS.md` first) and unit tests (`*_tests.rs` beside `src/application/pipeline/`, `message_service`, `generation/`) that touch: a normal generation, a Trigger continuation, movement, each retry form, Retrigger, swipe switch, Options rewrite, arrival.
2. For each test: the spec scenario it covers (`validate_feature_spec.py` annotations), and whether it asserts the restored state (room, NPCs, Snapshot id) or only the HTML.
3. Test doubles: the mock LLM provider and the mock Quantifier (`src/test_support/quantifier.rs`, `adapters/driven/llm/providers/mock.rs`). Which outputs are fixed (for example "no movement"), and which seed defects they hide. Seed §2 names scenario 22.1.
4. Harness gaps: arrival narration never runs in `src/test_support/`; list any other path that no test reaches.
5. The tests named in the seed as "would go red": do they exist now?

## Context

- Seed: [three pipeline defects with one root](../assets/seed-evidence.md).
- The repo rule: tests are the source of truth for behaviour (`AGENTS.md`, The Test-First Philosophy).

## Output

An asset `../assets/research-10-test-coverage.md`: a coverage matrix (rows: states, pointers, retry forms, writers; columns: test, kind, asserts state or display), the list of masking test doubles, and the list of gaps. Label each fact known, inferred, or tested. Cite test names, not line numbers.

## Feeds

The map's fog patch "Fix or rewrite", and the plan that the map ends in.

## Answer

The full matrix, the masking doubles and the gap list are in the asset [research-10-test-coverage.md](../assets/research-10-test-coverage.md). The answers in brief, all **known** from the cited tests unless marked otherwise:

1. **The matrix holds 79 rows** (16 generation states, 18 Snapshot states and pointers, 19 retry form entries, 26 writer rows). **66 of the 79** have at least one test that asserts restored or persisted state through a read seam. The 13 that do not: the claim-time `Generating` status and `GenerationPhase::Options` (`Options` has no test at all), `S0`/`S4`/`S2b` (asserted only weakly — see point 3), the `Idle`-after-every-generation row and the error-endpoint row (display only), `Si`'s pointer, the `init_game::load_game_state` boot writer, the concurrent-writer row, and the Retrigger guard after a main Retry.
2. **The seed's two "would go red" tests do not exist.** The nearest for the swipe switch is `test_retry_swipe_without_movement_reports_no_room_http` (9.8): it does switch after a retry, but its retry quantifier answers "no movement" and its post-switch assertions read `Message::location_header` only. The nearest for the event retry is `test_event_retry_does_not_carry_the_previous_swipes_encounter_http` (10.5), which asserts the *opposite*: that the event redo must not inherit the main quantifier's NPC. Only 9.2 (main Retry) and 13.2 (Retrigger) seed a tripwire quantifier answer, so they are the only two HTTP tests that can fail for a wrongly-run quantifier.
3. **No test reads a Snapshot pointer back.** `S1`, `S1'`, `S5`, `S5'`, `Si`, `Sa` and `Ss` are pointers, and no test loads the pointed-at Snapshot and compares a restored world field; `S5` is read once, for history only. `S0` and `S4` are not pointers (research 04, Correction), and the two tests named after them assert only `load_latest_snapshot().db_id.is_some()`, which passes for any later save. `test_switch_swipe_restores_a_settled_snapshot` hand-seeds both Swipes with the *same* Snapshot, so it cannot see a wrong-world restore.
4. **The masking double is the absent quantifier, not a permissive one.** `app_with_narrator` passes `AgentRegistry::default()`, so the pipeline runs every turn against `QuantifierResult::default()`: no movement, no NPCs, `Low` confidence, plus a mandatory trailing `[System] NPC detection uncertain` row. That is stronger than the seed's inference for `swipe_new.md` I.6 / 22.1 — there is no quantifier double to answer, and scenario 22.1's "the player has not moved" line has **no assertion behind it at all**. The behaviour is pinned in the opposite direction only by the untagged unit test `test_retry_flow_impersonate_mode_runs_full_tail`, so `validate_feature_spec.py` cannot see the conflict. The second double is the unseeded `MockBackend` non-narrator branch, which returns `[Continuation: …]` — not JSON — and lands in the same `default()`. The hand-seeded pointer fixtures (`seed_main_narration_flow`, `seed_event_flow`, `seed_swipe_with_stored_inputs`, `insert_message_with_swipe`) reproduce the pipeline's own save order, so no test using them can detect a pointer-setting defect.
5. **Arrival never runs in the harness — the seed's claim is confirmed.** No file under `tests/` or `src/test_support/` mentions arrival at all; only `src/application/arrival_service_tests.rs` calls `ArrivalTaskContext::run` directly, single-threaded, so arrival's own outcome is covered but the interleaving is not. Other unreached paths: the `/status/reset-generating` HTTP route (asserted only to contain the string "reset" in the quarantine), `handle_cancellation`'s flipped-to-game target, `check_retry_anchor`'s event-branch mismatch, `/debug/state` values, `S0`/`S4` content, `delete_last`'s orphan Snapshot rows, editing or deleting a `System` notice, a main Retry followed by a Retrigger, and eight of research 05's nine concurrent-writer pairs (only `handle_cancellation` is touched, and only incidentally).

**What this means for the map.** No test asserts the world a Swipe restores, so the seed's §1 pointer defect and its descendants are invisible to the suite by construction — the two-state channel is asserted only where display and state happen to agree. The "Fix or rewrite" decision must therefore weigh that a model-level test does not exist either: the unit tier asserts phases and rows, the HTTP tier asserts markup and text, and the seam between them — which world a pointer restores — has no tier.

Two items stayed **undetermined** and a targeted run would settle them: whether any HTTP `fetch_generating_status == "idle"` assertion could ever observe the persisted status rather than the registry (read as no, from `generating_status_handler`, not executed), and whether `should_fail()`'s effect on the quantifier branch is intentional or incidental.
