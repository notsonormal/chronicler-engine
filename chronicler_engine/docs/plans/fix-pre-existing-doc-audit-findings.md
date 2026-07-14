# Fix Pre-Existing Doc Audit Findings

## Summary

After completing the `STANDARD_DOC_BODY_REFERENCE` migration (0 violations across 82 STANDARD docs), the `chronicler-docs-hygiene` skill surfaced ~65 pre-existing semantic issues across 23 files: behavior mismatches (wrong code paths, wrong signatures, wrong paths), ghost features (UI elements + capability claims not in `src/`), schema drift (fictional JSON shapes), sediment (timeline history in body prose), and cross-doc drift (inconsistent layer counts, retention scope). Mechanical rule already passes; this plan addresses the semantic layer. No new code or schema changes — only doc edits to align prose with actual `src/` behavior.

## Key Changes

- 23 files modified, one task per file
- Primary agent does all edits inline (no subagents); one `validate_docs.py` pass at the end
- No new tools, no new validators, no new docs

## Constraints (vs prior plan)

- **No subagents.** Primary agent reads each file, applies edits, moves to next.
- **No per-file `validate_docs.py`.** One full pass at end of all 23 edits. Per-file spot-check by grep against `src/` for Error-severity findings only.

## Implementation

### Phase 1: Apply all 23 file fixes (sequential, no subagents)

- [ ] #### Task 1.1: Fix `docs/system/character_state.md` (1 SP)
  - Collapse 3-bullet field enumeration (`times_met`/`trigger_fired`/`currently_meeting`) into one contract statement (Phase 4).
- [ ] #### Task 1.2: Fix `docs/system/dynamic_rooms.md` (1 SP)
  - Path `state.dynamic_rooms` → `state.movement.dynamic_rooms`; replace "generic or LLM-derived description" with the static placeholder; clarify persistence (lost on world re-seed, not per game session).
- [ ] #### Task 1.3: Fix `docs/system/message_model.md` (1 SP)
  - `MessageHistory` path: `src/domain/model/message.rs` → `src/domain/model/message_history.rs`.
- [ ] #### Task 1.4: Fix `docs/system/narration_engine.md` (1 SP)
  - Rename "Arrival Logic Flow" → "Per-Action Flow"; drop or mark "Arrival Instruction" as planned (literal text not in active preset).
- [ ] #### Task 1.5: Fix `docs/reference/data_layer.md` (1 SP)
  - Rename "Key invariant" label (RESERVED word) to "Storage rule".
- [ ] #### Task 1.6: Fix `docs/reference/persona_system.md` (1 SP)
  - `EngineError::Config` → `EngineError::PersonaNotFound(key)` for boot error.
- [ ] #### Task 1.7: Fix `docs/reference/testing.md` (1 SP)
  - Drop "execute_action() wrapper" reference (phantom symbol); replace with the direct pipeline rule.
- [ ] #### Task 1.8: Fix `docs/architecture/system.md` (1 SP)
  - Drop or correct `GameId` type alias reference (`u64` is used); `Arc<RwLock<HashMap<u64, GenerationSlot>>>`; `DefaultApplicationService` shutdown_token detail acceptable as-is.
- [ ] #### Task 1.9: Fix `docs/diagnostics/DEBUGGING.md` (1 SP)
  - Append `_impl` suffix to `execute_action`, `retry_last_response`, `retrigger_event` in instrumented function list (lines 67-71).
- [ ] #### Task 1.10: Fix `docs/external_applications/sillytavern_chat_window.md` (1 SP)
  - Cross-doc drift: "Layer 5: Chat History" → "Layer 6"; "Layer 6: User Input" → "Layer 7" (to match `system_prompt.md`); update Document References bullet.
- [ ] #### Task 1.11: Fix `docs/system/action_pipeline.md` (3 SP)
  - 5 findings: cancel-checkpoints prose → one-line data-flow; name the four `persist_snapshot_or_err` sites + which propagates vs swallows; clarify retry path (`save_retry_error` does not call `phase_finalize`); drop `pub(crate)` modifier + `reconcile_post_trigger_npcs` symbol; `state.narrative.retry_target.take()` timing.
- [ ] #### Task 1.12: Fix `docs/system/agent_system.md` (3 SP)
  - 5 findings: downgrade `backend_selector()` to "Reserved; not consulted"; note PreGeneration has no dispatcher; replace JSON example with one-line contract; remove step 2 PreGeneration agents in pipeline flow OR mark reserved; quantifier backend bound at wiring time.
- [ ] #### Task 1.13: Fix `docs/system/llm_processing.md` (3 SP)
  - 6 findings: replace "CancellationToken" with "current_game_id() alpha-check"; `GameService` is a struct; `{{variable}}` → `[FILTERED]`; sanitizer path `application/llm_sanitizer.rs`; retention 50 rows globally not per-game; drop `save_message_and_snapshot` hand-off if present.
- [ ] #### Task 1.14: Fix `docs/system/prompt_system.md` (3 SP)
  - 4 findings: drop "goals" from NPC card field list (not rendered); persona `scenario` rendered as `Background:` label; drop "Trigger: Keyword matching" ghost; `{{user}}` is the only template variable.
- [ ] #### Task 1.15: Fix `docs/system/text_check.md` (3 SP)
  - 5 findings: `TextCheckSettings` stored in SQLite `settings` row not `settings.json`; `HarperTextChecker` constructed once per service not per check; test path `text_check.rs` not `text_check_tests.rs`; fix `TextChecker::check` signature to match live (`fn check(&self, &str, TextCheckMode, &[String]) -> Result<Option<CheckResult>, EngineError>`); drop "~8MB stripped" FST size.
- [ ] #### Task 1.16: Fix `docs/system/triggers.md` (3 SP)
  - 5 findings: triggers run after main narration AND quantification (not "between"); `evaluate_triggers` iterates NPC map parameter (not `state.npcs`); add `Lt` operator to docs; trigger continuation uses stored-prompt build (not "7-layer with splice"); persistence-of-movement re-evaluation clarification.
- [ ] #### Task 1.17: Fix `docs/system/ui_design.md` (3 SP)
  - 5 findings: drop "Game Selector Button" / `#games-dropdown` (ghost UI); drop "Action Hints" (no CSS class); retrigger button uses `.retrigger-btn` cyan styling not edit-btn; swipe controls qualifier "last narration or dialogue message"; remove "Game Dropdown" + "Responsive Breakpoints" sections (not in code).
- [ ] #### Task 1.18: Fix `docs/reference/data_schemas.md` (3 SP)
  - 6 findings: rewrite Trigger JSON to `{requirement, narration, repeat, room_id?}`; rename "WorldCard Schema" → "WorldManifest Schema" + full field list (`id`, `map_file`, `characters_dir`, `default_scenario_id`); add `key` to PersonaCard JSON; mark `image_path` and `sender` as optional; drop Swipe + Message JSON/field paste (keep accessor-methods paragraph).
- [ ] #### Task 1.19: Fix `docs/reference/quantifier_prompt.md` (3 SP)
  - 4 findings: drop `AppSettings.active_quantifier_prompt` caching claim; fix path `assemble_prompt_text` → `src/application/narrative_prompt/assembler.rs`; replace `prompt_preset_storage.rs` with `src/application/agents/quantifier/parser.rs`; fold or drop "Code references" section (private symbols + wrong paths).
- [ ] #### Task 1.20: Fix `docs/reference/system_prompt.md` (3 SP)
  - 5 findings: fix path `assemble_prompt_text` → `src/application/narrative_prompt/assembler.rs`; replace `prompt_preset_storage.rs`; remove `<writing_style>`/`<output_format>` from system-prompt XML (they live in post-history prompt); fix Layer 5/6 → Layer 6/7 numbering in prose; fold or drop "Code references" section.
- [ ] #### Task 1.21: Fix `docs/diagnostics/error_catalog.md` (3 SP)
  - 5 findings: replace `docs/reference/prompt_budget.md` (nonexistent) with `quantifier_prompt.md` or drop; replace 5-item InternalError Common Causes list with actual `internal_error(...)` strings from `src/`; mark `NarrativeFailure::PromptBuild` as test-fixture only; `NarrativeFailure::Generation` "mock" stages includes `mock_trigger`; attribute error sources by file (not lumped under `client.rs`).
- [ ] #### Task 1.22: Fix `docs/system/dashboard.md` (5 SP)
  - 8 findings (heaviest): `chat-event` CSS class → `event-header`; retrigger availability qualifier (excludes event continuations, includes dialogue); swipe controls qualifier (last narration/dialogue); empty-input button label "Stop" vs status "Thinking..."; game name format uses spaces not underscores; LLM retention is 50 rows global not per-backend; collapse layout bullets (Code-indexer); inactive tab color reference to stylesheet.
- [ ] #### Task 1.23: Fix `docs/architecture/guardrails.md` (5 SP)
  - 8 findings (heaviest): rewrite "5 exempted files" line to match reality (3 files carry markers, 2 paths wrong); test path `tests/infrastructure/invariant_contract.rs` + test name `test_inv001_generation_guard_resets_on_panic`; remove `generation_state` struct reference (field is on `InputBuffer`); INV-002 mutation order wording; INV-004 only transport-level cancellation exists (no `before/after backend calls`); INV-003 also flags `std::thread::sleep`; deferred-rules row `server→storage` is enforced (split from `server→narrative`); move Phase 1.7/2.x/T2 timeline references into Document References or `docs/plans/`; extend §3 with missing rules from `tests/infrastructure/guardrails/mod.rs` (≥17 rules vs 9 listed); correct `tests/guardrails.rs` path → `tests/infrastructure/guardrails/`.

## Test Plan

After all 23 tasks complete (single end-of-plan pass):
1. `python scripts/validate_docs.py` → expect 0 errors, 0 warnings (mechanical rule still passes)
2. For Error-severity findings only: grep `src/` to spot-check the corrected claim matches live code (≈10 grep checks)
3. Diff stat per file: confirm targeted edits, no unrelated changes
4. Optional: re-run hygiene pass on heaviest files (dashboard.md, guardrails.md) to confirm findings cleared

## Per Task Validation Steps

Inline per task (no separate `validate_docs.py` call):
- Read file before edit to confirm exact line content
- Apply targeted edit
- For Error-severity findings: grep `src/` immediately after edit to verify the corrected claim
- Move to next task

End-of-plan gate:
- One full `python scripts/validate_docs.py` pass

## Assumptions

- Primary agent executes all 23 tasks sequentially in one session; no `general-purpose` subagent spawned
- Each task is self-contained: the audit already documented each finding with file:line + recommended fix, so primary agent doesn't need to re-derive the contract from `src/` for trivial path/signature corrections
- For Error-severity findings where the fix isn't obvious from the audit recommendation, primary agent greps `src/` once to confirm the corrected claim
- No new `validate_docs.py` rules added; this plan is pure prose correction
- Story points dropped from per-task execution check (no subagent SP budgeting needed since primary does it all)
- Files with no audit findings (game_flow, navigation, storage, marinara_engine, sillytavern_prompt_system) are skipped from this plan
- Cross-doc drift fixes are sequenced to avoid double-editing: when two docs disagree, the wrong one is fixed in its own task; no cross-task coordination needed because the audit already identified which is wrong
- Plan does not introduce new docs (e.g., no `CONTEXT.md` entries, no new ADR)
- `python build.py` not required (no Rust/Python code changes)
