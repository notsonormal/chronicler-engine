# Ticket 17: Speaker-axis type rework + user-regen

## Summary

Implement the code changes resolved by ticket 15 (grilling, resolved): `MessageType` encodes speaker/role, not function. Impersonate output becomes `MessageType::Input` (overturning ticket 09's `Dialogue`). The `sender` field is deleted entirely (full removal + storage migration v17). The `Dialogue` variant is removed. `Input` gains swipe support, and retry becomes a three-way disambiguation on the last message + steering record: re-narrate (Narration), re-impersonate (Input + record), user-regen (Input, no record). Re-impersonate and user-regen append an alternate swipe and stop — no auto-narration, no quantifier/commit/trigger. User-regen uses a hardcoded rewrite instruction over the system preset, is retry-only, and earns no `PresetType` (decision 6).

## Key Changes

- **Type axis = speaker/role.** Impersonate output `Dialogue` → `Input` (`pipeline_run.rs:198-205`). `sender` deleted from `Message`, `MessageEntry`, `DbMessage`, `MessageEntryView`, the template, CSS, storage, and all `add_message`/`Message::new` call sites. `last_input_text` returns `Option<String>`.
- **`Dialogue` removed.** Enum variant, view-model arm, `last_ai_response_index`/`is_last_ai_response_event_continuation`/`resolve_retry_target`/`retrigger` filter arms, `push_message` swipe-append condition, the `.log-entry.dialogue` CSS rules + `--color-log-dialogue`, and all tests/specs.
- **Storage migration v17** drops `messages.sender` (column, INSERT/SELECT SQL, `DbMessage::from_row` indices). Irreversible — follows the existing `DROP COLUMN` pattern from v13/v14.
- **History rendering without `sender`** (recommendation B, confirmed conservative): `render_history_layer` (assembler) and `<RecentHistory>` (quantifier prompt) switch to type-derived labels — Narration→`Narrator:`, Input→`{persona_name}:`, System→`System:`, Narrator→bare. Production Input already carries `sender = persona.sheet.name` (`action.rs:60-64`), so this preserves current behavior; the only change is System `Narrator:`→`System:` (fixes a pre-existing quirk).
- **Three-way retry.** `resolve_retry_target` sets `old_target = messages.last()` and derives the mode from type + `Swipe.replay`. `reconstruct_retry_state` truncates to `anchor_idx` (exclusive) when the Input is the target, `anchor_idx + 1` (inclusive, existing) for Narration/event. `push_message` swipe-append condition adds `Input`. Swipe `replay` inheritance is automatic via `target.replay().cloned()` (`game_state.rs:138`) — no `pending_replay` manipulation needed.
- **Generate-swipe-and-stop path.** New `ActionPipeline` retry paths for re-impersonate and user-regen: build mode-specific prompt → `call_narrator` helper (assembled + `recorder.complete` + empty-check, extracted from `phase_narrate`) → `add_message(Input)` appends a swipe to `retry_target` (replay inherited automatically) → `save_message_and_snapshot` → `retry_target.take()` → `history.append` → `phase_finalize`. Forensics recorded via `recorder.complete` (writes `llm_messages`); errors propagate via `set_error`/`PhaseError`/`finalize_phase_error`, plugged into `log_cancellation`. No quantifier, no engine commit, no trigger.
- **Hardcoded user-regen instruction** (verbatim from ticket, Marinara-derived) over the system preset. No `PresetType`, no parser entry, no auto-suggestion.
- **Template swipe gating** shows controls when `loop.last && (log_type == "narration" || log_type == "input")`. `show_retrigger` `is_narration` becomes `log_type == "narration"` only.
- **Specs updated** to match the new types: `browser.md` 17.8, `actions.md` 1.9, `retrigger.md` 14.3 (Dialogue→Input wording); new `swipe_new.md` scenarios 13.1–13.3 for re-impersonate/user-regen retry.

## Implementation

### Phase 1: Speaker-axis type rework (mechanical, build-green)

- [ ] #### Task 1.1: Impersonate output `Dialogue` → `Input` + spec lockstep (1 SP)
  - `src/application/pipeline/pipeline_run.rs:198-205`: change the impersonate branch from `(Some(persona.sheet.name), MessageType::Dialogue)` to `MessageType::Input` (sender goes in 1.2).
  - `tests/http/actions.rs:679` (`test_slash_impersonate_produces_dialogue_http`, SCENARIO 1.9): expect `Input`, not `Dialogue`; rename fn to drop "dialogue".
  - `tests/browser/behaviour.rs:558` (`test_slash_impersonate_produces_dialogue_entry`, SCENARIO 17.8): assert `.log-entry.input` count increases (not `.dialogue`); rename fn.
  - **Spec lockstep:** `docs/specs/actions.md` Scenario 1.9 — "contains exactly one Dialogue entry" → "one Input entry"; `docs/specs/browser.md` Scenario 17.8 — title "produces a Dialogue log entry" → "produces an Input log entry", body `.log-entry.dialogue` → `.log-entry.input`.
  - `python build.py` green.

- [ ] #### Task 1.2: Delete `sender` (full removal + storage migration v17) (6 SP)
  - [ ] ##### SubTask 1.2.1: Domain types and signatures (2 SP)
    - `src/domain/model/message.rs`: remove `sender` from `Message`, `Message::new`, `Message::from_db`.
    - `src/domain/model/state/message_types.rs`: remove `sender` from `MessageEntry`, `Default`, `From<&Message>`.
    - `src/domain/model/state/game_state.rs`: `push_message`/`add_message` lose the `sender` arg; `Message::new` call at `:144` drops it.
    - `last_input_text` (`message_history.rs:117`) returns `Option<String>` (text only); update `retry.rs:29`, `retry.rs:73`, `core.rs:501`.
    - Update all `add_message` call sites: `action.rs:62` (Input), `action.rs:141` (Narrator), `pipeline_run.rs:206` (narration/impersonate), `pipeline_run.rs:251`/`:299` (System), `game_state.rs:175/206/301` (scenario/movement/System), `arrival_service.rs:171`, `message_service.rs:129`.
  - [ ] ##### SubTask 1.2.2: Storage migration v17 + SQL (2 SP)
    - `src/adapters/driven/storage/utils/plumbing.rs`: add `if version < 17 { if column_exists(conn, "messages", "sender") { exec("ALTER TABLE messages DROP COLUMN sender") } ... }` (reuses the v13/v14 `column_exists` + `DROP COLUMN` pattern).
    - `src/adapters/driven/storage/models/message.rs`: remove `sender` from `DbMessage`; `from_row` reads indices 0-4 (`id, game_id, message_type_json, timestamp, active_swipe_index`), `is_deleted: 0`.
    - `src/adapters/driven/storage/messages.rs`: `insert_message` SQL drops `sender`; `load_message_rows` SELECT drops `sender`.
    - `src/adapters/driven/storage/mappers/message.rs`: both `TryFrom` impls drop `sender`.
  - [ ] ##### SubTask 1.2.3: View model, template, CSS, prompt rendering, src unit tests (2 SP)
    - `view_models.rs:28,49`: remove `MessageEntryView.sender`; `From<&MessageEntry>` drops the field.
    - `templates.rs:25`: remove the `{% if entry.sender != "" %}<span class="sender">...</span>{% endif %}` from the story-log template.
    - `assets/styles.css`: drop `.log-entry .sender`, `.log-entry.narration .sender`, `.log-entry.input .sender`, `--font-size-sender` (dialogue rules removed in 1.3).
    - **History rendering (recommendation B, confirmed conservative):** `assembler.rs:360` `render_history_layer` — Narrator stays bare; Narration → `Narrator: {text}`; Input → `{self.persona.sheet.name}: {text}` (persona available on `PromptContext`); System → `System: {text}`. `quantifier/prompt.rs:85` `<RecentHistory>` — same scheme using the player name for Input. Preserves current behavior (Input already uses player name); only changes System `Narrator:`→`System:`.
    - `test_support/quantifier.rs`: drop `sender` from `make_history`.
    - **src unit tests (complete list):**
      - `assembler_tests.rs` — drop `sender` from `MessageEntry` constructions; re-assert the type-derived labels (Narration→`Narrator:`, Input→player name, Narrator bare).
      - `templates_tests.rs` — drop `sender` from all `MessageEntry` constructions and `entry.sender` assertions.
      - `message_tests.rs` (domain) — drop `sender` arg + assertions.
      - `message_history_tests.rs` — `make_message` drops `sender`; `test_last_input_text` asserts `Option<String>` not a tuple.
      - `game_state_tests.rs` — drop `sender` from `add_message`/`Message::new` calls and `message.sender` assertions (impersonate-Dialogue test rewritten in 1.3.2).
      - `mappers/message_tests.rs` — drop `sender` arg + `original.sender == back.sender` assertion.
      - `messages_tests.rs` — remove the three `test_message_with_sender_*` tests; drop `sender` from `Message::new` in remaining tests.
      - `models/message_tests.rs` — drop `sender` from `CREATE TABLE`/`SELECT` SQL and `msg.sender` assertions; `from_row` indices shift.
      - `db_tests.rs:176` — INSERT SQL drops `sender`.
      - `action_tests.rs` (pipeline) `:544` — remove `narrator.sender.is_none()` assertion.
      - **`retry_tests.rs` (23 matches), `retrigger_tests.rs`, `core_tests.rs:728`, `message_service_tests.rs:88/105/122`** — drop `sender` from every `add_message`/`Message::new` call.
  - [ ] ##### SubTask 1.2.4: Test helpers + integration tests (sender removal) (2 SP)
    - `tests/helpers/sqlite_test_app_builder.rs:98` — `log()` drops the `speaker` param; update all `.log(...)` call sites.
    - `tests/helpers/application_ext.rs:71` — drop `sender` from `add_message`.
    - `tests/http/retrigger.rs` (7 matches) — update `.log(..., Some("Player"), ...)` and `Message::new` sender constructions.
    - `tests/http/swipe_new.rs` (9 existing matches at 429/465/502/507/563/568/630/647/750) — drop `sender` (the 2.5 additions use the new signature).
    - `tests/http/requires_migration/fragment.rs` (6 matches) — update `.log` and `Message::new` sender constructions.
    - `tests/storage/message_storage.rs` (13 matches) — drop `sender` from all `Message::new`.
    - `tests/storage/snapshot_storage.rs` (4 matches) — drop `sender` from all `Message::new`.
  - **Mechanical gate (both subtasks):** `cargo build --all-targets` clean; `grep -rn "add_message\(\|Message::new\(\|\.log\("` over `src/`+`tests/` — every remaining hit uses the new no-sender signature (no `Some(...)`/`None` in the sender position). `python build.py` green.

- [ ] #### Task 1.3: Remove the `Dialogue` variant + dead CSS + spec (3 SP)
  - [ ] ##### SubTask 1.3.1: Variant + filter arms + CSS (1 SP)
    - `message_types.rs`: remove `Dialogue` from `MessageType`.
    - `view_models.rs:54`: remove the `MessageType::Dialogue => "dialogue"` arm.
    - `message_history.rs:107,132`: drop `Dialogue` from `last_ai_response_index`/`is_last_ai_response_event_continuation` filters.
    - `retry.rs:108` (`resolve_retry_target` `old_target` filter): drop `Dialogue` (becomes `Narration` only for Phase 1; replaced by three-way branching in 2.1).
    - `retrigger.rs:28`: `is_narration` becomes `MessageType::Narration` only.
    - `game_state.rs:122` (`push_message` swipe-append): drop `Dialogue` (becomes `Narration | Narrator`; `Input` added in 2.1).
    - `assets/styles.css`: drop `.log-entry.dialogue`, `.log-entry.dialogue .text`, `.log-entry.dialogue .sender`, `--color-log-dialogue`.
  - [ ] ##### SubTask 1.3.2: Tests + spec (2 SP)
    - `message_history_tests.rs:155` (`test_last_ai_response_index`): replace the `Dialogue` message with `Narrator`; re-derive the expected index.
    - `game_state_tests.rs` `test_push_message_stages_impersonate_replay_on_player_voiced_dialogue`: rewrite to `MessageType::Input`; drop `sender`; rename fn.
    - `game_state_tests.rs` `log_type_strategy`: remove `Dialogue` from the proptest strategy.
    - `mappers/message_tests.rs:66` (`test_message_log_type_json_serialization`): `Dialogue` → `Input`; assert `"\"Input\""`.
    - **Spec:** `docs/specs/retrigger.md` Scenario 14.3 — "the last message is an Input message (not a Narration or Dialogue)" → "(not a Narration)".
    - Grep `MessageType::Dialogue` across `src/`+`tests/` — zero remaining.
  - `python build.py` green.

### Phase 2: Three-way retry + user-regen (architectural)

- [ ] #### Task 2.1: `Input` swipe support + three-way retry disambiguation (5 SP)
  - [ ] ##### SubTask 2.1.1: `Input` in `push_message` swipe-append (1 SP)
    - `game_state.rs:117-145`: add `MessageType::Input` to the swipe-append condition (now `Narration | Input | Narrator`). Replay inheritance is automatic via `target.replay().cloned()` (`game_state.rs:138`) — no `pending_replay` logic needed. A normal (non-retry) Input add still creates a new message (`retry_target` is `None` outside retry). Unit test: with `retry_target = Some(Input)`, `add_message(text, Input)` appends a swipe inheriting the Input's replay.
  - [ ] ##### SubTask 2.1.2: `resolve_retry_target` three-way branching (2 SP)
    - `retry.rs:88-114`: `old_target = messages.last().cloned()`. Derive a mode enum from `old_target`: `Narration` (±event_header) → re-narrate; `Input` + `replay.impersonate == true` → re-impersonate; `Input` + no record → user-regen. Keep `is_event` for the Narration-event branch. `find_retry_anchor`/`find_retry_anchor_msg` unchanged (still the last `Input`).
  - [ ] ##### SubTask 2.1.3: `reconstruct_retry_state` mode-aware truncation (2 SP)
    - Re-narrate/event: `truncate(anchor_idx + 1)` (keep through anchor); `retry_target = old_target` (Narration).
    - Re-impersonate/user-regen: `truncate(anchor_idx)` (remove the Input — it lives in `retry_target`); `retry_target = old_target` (Input). No duplication when `retry_target.take()` re-appends.
    - Unit tests for both truncation modes.
  - `cargo test --lib` green; `python build.py` green.

- [ ] #### Task 2.2: Re-impersonate retry path (generate swipe + stop) (3 SP)
  - Extract `call_narrator(&self, state, context, preset, response_length) -> Result<String, PhaseError>` from `phase_narrate` (`pipeline_run.rs:155-175`): does `prompt_assembler.assemble` → `recorder.complete(AGENT_NARRATOR, ...)` (forensics) → empty-check → returns text. `phase_narrate` keeps room/impersonate/context building; the new paths build their own context and call the helper.
  - New `ActionPipeline` method called from `retry_last_response` when mode = re-impersonate: load world bundle (`load_world_bundle`, reuse from `retry_event_continuation`); `retry_target` = the Input (from reconstruction); resolve impersonate steering from `retry_target.replay()`; load the impersonate preset (`load_impersonate_preset_and_response_length`); build `PromptContext` with `with_impersonate(true)` and `impersonate_direction` as `user_message`; `call_narrator` → text; `add_message(text, Input)` appends a swipe (replay inherited automatically via `target.replay()`); `save_message_and_snapshot`; `retry_target.take()` → `history.append`; `phase_finalize`.
  - **Failure handling (mirrors `retry_event_continuation`, `core.rs:493-551`):** on `load_world_bundle`/preset/assemble/LLM error → `set_error` + `finalize_phase_error(&run, Some(state), e)`; return `Result<(), PhaseError>` so `retry_last_response`'s `log_cancellation` handles `Cancelled`. No `phase_post_generation`/`phase_engine_commit`/trigger.
  - Unit tests in `retry_tests.rs`: re-impersonate appends a swipe to the Input (not a new message); swipe inherits the impersonate record (via `target.replay()`); status Idle; no narration follows; LLM failure → `GenerationStatus::Error` + no swipe appended.

- [ ] #### Task 2.3: User-regen path + hardcoded instruction (3 SP)
  - [ ] ##### SubTask 2.3.1: Hardcoded user-regen instruction builder (1 SP)
    - New Rust fn `build_user_regen_instruction(original: &str) -> String` producing, verbatim from the ticket:
      ```
      Regenerate the user's previous message as an alternate swipe.
      Write only the replacement user message text.
      Do not answer as the assistant, continue the assistant side, or describe what the assistant does next.

      <original_user_message>
      {original Input text}
      </original_user_message>
      ```
    - `original` = the active swipe's text of the Input being retried (`retry_target.text()`). Unit test the builder output. Add a `// WHY` comment: hardcoded (not a `PresetType`) per ticket 15 decision 6.
  - [ ] ##### SubTask 2.3.2: User-regen retry path (2 SP)
    - New `ActionPipeline` method sharing the skeleton with 2.2: load world bundle; `retry_target` = the Input; load the **system preset**; build `PromptContext` (no `with_impersonate`, keeps `<PlayerCharacter>`) with the hardcoded instruction as `user_message`; `call_narrator` → text; `add_message(text, Input)` appends a swipe (no record inherited — the Input's swipe has no `replay`, so `target.replay()` is `None`); `save_message_and_snapshot`; `retry_target.take()` → `history.append`; `phase_finalize`.
    - **Failure handling:** same pattern as 2.2 (`set_error`/`finalize_phase_error`/`log_cancellation`). No quantifier/commit/trigger.
    - Unit tests in `retry_tests.rs`: user-regen appends a swipe; new swipe has no record (further retry stays user-regen); status Idle; no narration; original Input text appears in the LLM user prompt (assert via `MockBackend` capture).
  - `python build.py` green.

- [ ] #### Task 2.4: Template swipe-control gating + stop-after-swipe (2 SP)
  - `templates.rs` `NarrativeLogTemplate` source: `{% if loop.last && (entry.log_type == "narration" || entry.log_type == "dialogue") %}` → `{% if loop.last && (entry.log_type == "narration" || entry.log_type == "input") %}`.
  - `templates.rs` `NarrativeLogTemplate::new`: `is_narration = last.log_type == "narration"` (drop `dialogue`); `show_retrigger = has_last_trigger && is_narration && !is_event_continuation`.
  - Confirm stop-after-swipe: 2.2/2.3 paths do not call `phase_narrate` again after the swipe. Add an integration assertion that after re-impersonate/user-regen the last message is the Input with >1 swipe and no Narration follows.
  - `templates_tests.rs`: update swipe-control assertions; add a case for a last `input` entry showing swipe controls.

- [ ] #### Task 2.5: Integration tests + new swipe_new.md scenarios (3 SP)
  - `docs/specs/swipe_new.md`: add a new section "Impersonate and user-regen retry" with:
    - **Scenario 13.1:** Re-impersonate retry appends a swipe to the impersonate Input (last message = Input with impersonate record → POST /swipe/new → Input gains a 2nd swipe, no new message, no Narration, status Idle).
    - **Scenario 13.2:** User-regen retry appends a swipe to a plain Input (last message = Input with no record, e.g. after delete-last → POST /swipe/new → Input gains a 2nd swipe, no Narration, status Idle).
    - **Scenario 13.3:** Re-impersonate retry preserves the impersonate steering record on the new swipe (further retry re-impersonates again).
    - Update invariant I.6 to note re-impersonate/user-regen do not re-run the quantifier.
  - `tests/http/swipe_new.rs`: add the three tests with `// [docs/specs/swipe_new.md] SCENARIO: 13.1/13.2/13.3` tags. User-regen test setup: action → delete-last narration → POST /swipe/new. Update existing retry tests if Input-as-target changes their assertions.
  - `tests/http/actions.rs` Scenario 1.9 test: add an assertion the impersonate `Input` carries the impersonate `replay` record.
  - Do NOT commit `docs/specs/steering.md` or `tests/http/steering.rs` (ticket 14's NEW spec, out of scope).
  - `python build.py` green (109s target).

## Test Plan

- `cargo fmt`; `cargo clippy --all-targets -- -D warnings`.
- `cargo test --lib` — all `Dialogue`/`sender` references gone; new `push_message` swipe-append, reconstruction, and history-rendering tests pass.
- `cargo nextest run --tests` — impersonate tests assert `Input` + no `sender`; three-way retry + user-regen tests pass; existing retry/swipe tests green.
- `python scripts/validate_feature_spec.py` — every updated/new spec scenario has a covering test; no orphan SCENARIO tags.
- `python build.py` green after each phase and at the end.
- UI (browser/manual): swipe controls render on a retried `Input`; no `sender:` prefix on any message; impersonate renders as an `input`-styled line; re-impersonate and user-regen stop after the swipe.

## Per Task/Sub Task Validation Steps

- After 1.1: `cargo nextest run --test http` (impersonate SCENARIO 1.9) + browser (17.8) + `validate_feature_spec.py` — spec and test IDs match, bodies agree on `Input`.
- After 1.2: `cargo build --all-targets` clean (the mechanical gate — every `add_message`/`Message::new`/`.log` hit uses the no-sender signature); `cargo test --lib` (all affected unit-test files) + `cargo nextest run --test storage`; `PRAGMA user_version == 17`; `messages.sender` absent; `assembler_tests` history-rendering assertions match type-derived labels.
- After 1.3: `cargo build --all-targets` + `cargo test --lib` — zero `MessageType::Dialogue`; `validate_feature_spec.py` green for retrigger 14.3.
- After 2.1: `cargo test --lib` (retry_tests, game_state_tests) — three-way branching + truncation modes correct.
- After 2.2: `cargo test --lib` (retry_tests) — re-impersonate appends a swipe, inherits record, stops; LLM failure → `Error` status, no swipe.
- After 2.3: `cargo test --lib` (retry_tests) — user-regen appends a swipe, no record, stops; instruction builder output exact; LLM failure → `Error` status.
- After 2.4: `cargo test --lib` (templates_tests) + browser — swipe controls on last `input`/`narration`.
- After 2.5: `cargo nextest run --tests` + `validate_feature_spec.py` (13.1–13.3 covered) + `python build.py` green.

## Assumptions

- **History rendering (recommendation B, confirmed conservative).** Production Input messages already carry `sender = persona.sheet.name` (`action.rs:60-64`), so type-derived labels (Narration→`Narrator:`, Input→`{persona_name}:`, Narrator→bare) preserve current behavior. The only change is System `Narrator:`→`System:`, which fixes a pre-existing quirk (System sender was `None`, falling back to `Narrator:`). Option A (bare text for all non-Narrator) would lose the player/narrator distinction and regress generation. The ticket scope does not list `assembler.rs`/`quantifier/prompt.rs`, but `entry.sender` is read there so they must change to compile.
- **Replay inheritance is automatic.** `push_message` inherits swipe `replay` via `target.replay().cloned()` (`game_state.rs:138`). Adding `Input` to the swipe-append condition (2.1.1) is sufficient for the new paths; no `pending_replay` manipulation. Re-impersonate's new swipe carries the impersonate record automatically; user-regen's carries none (the Input swipe has no replay).
- **Forensics + failure handling for new paths.** The new paths use `recorder.complete` (writes `llm_messages`) and the `set_error`/`PhaseError`/`finalize_phase_error`/`log_cancellation` machinery — mirroring `phase_narrate` and `retry_event_continuation`. No silent LLM failures.
- **Shared helper.** Extract `call_narrator(context, preset, response_length) -> Result<String, PhaseError>` (assemble + recorder + empty-check) from `phase_narrate`; both new paths reuse it. Each path builds its own context. Avoids 3 copies of the recorder/error logic.
- **User-regen uses the system preset** (decision 6: no new `PresetType`). The hardcoded instruction overrides `role`/`instructions`; the system preset supplies `writing_style`/`output_format`.
- **Template swipe-control gating infers from `log_type` + `loop.last`** (no new view-model flag). Every last `Input` is retryable.
- **`old_target = messages.last()`** in `resolve_retry_target` (decision 9: retry is last-message-only).
- **Phase 1 retry-of-impersonate is misrouted, not broken.** After 1.1, impersonate emits `Input` but `resolve_retry_target` still scans `Narration | Dialogue`; retry-of-impersonate misroutes to `run_from_input` (re-narrates using the impersonate text as input) until 2.1. No test covers it (build stays green). Accepted because phases are one ticket/branch and don't ship alone; fixed by 2.1's three-way branching.
- **Migration v17 is irreversible** (`DROP COLUMN sender`), consistent with v13 (`player_key`) and v14 (`starting_room_id`). Data loss accepted.
- **Spec scope.** The NEW `docs/specs/steering.md` + `tests/http/steering.rs` commit is ticket 14 (out of scope). UPDATES to existing specs this ticket changes (browser 17.8, actions 1.9, retrigger 14.3, swipe_new 13.1–13.3) ARE in scope — `validate_feature_spec.py` couples specs to tests by scenario ID.
- **`GenerationReplay` → `SteeringRecord` rename is deferred** (decision 10, separate ticket).
- **CONTEXT.md unchanged.** Ticket 15 already updated the Impersonate entry; the `Replay Blob` rename is deferred.
- **Phasing.** Phase 1 (type rework) lands green before Phase 2's architectural work. The two phases are one ticket.
