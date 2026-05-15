# Plan: Message+Swipe Storage (Marinara/SillyTavern Model)

## Corrected Understanding

**Delete behavior (unchanged):** Only the last message can be deleted. After deleting it, the previous message becomes the new last message. Repeat to peel back layers.

**Retry behavior (unchanged):** Only the last message can be retried. To retry the narration, first delete the event (now narration is last), then click retry.

**What changes:** Messages are **independent database records** instead of being bundled inside a Turn JSON blob. Each message has its own swipes (alternatives).

## Runtime Analysis

### How Turns Are Created (Current Model)

**Turn structure:** A Turn = { id, input: LogEntry, swipes: Vec<Swipe>, active_swipe_index }. A Swipe = { index, entries: Vec<LogEntry> }. All AI-generated content for a single player input lives inside one Turn, inside one Swipe.

**Creation flow:**
1. **Server (server/fragments/actions.rs process_action):**
   - For async actions, calls game_state.add_log(command, Some(player_name), LogType::Input) BEFORE spawning the background task.
   - add_log with LogType::Input calls add_input(), which creates Turn::new(input) and pushes it to narrative.turns.
   - The server then reads turn_id from narrative.turns.last().id (the Turn just created).
   - Saves a snapshot with (turn_id, swipe_index=0).
   - Spawns game_service.execute_action(ctx, cmd, pname) via tokio::task::spawn_blocking.

2. **Engine (engine/game_service/actions.rs execute_action_impl):**
   - For Action::FreeAction(text), acquires action_lock, loads latest state.
   - Gets turn_id from state.narrative.turns.last().id (same Turn created by server).
   - Calls execute_freeaction_pipeline(service, ctx, state, turn_id, text, 0).

3. **Pipeline (execute_freeaction_pipeline):**
   - Saves committed pre-gen snapshot: save_committed_state(ctx, state, "pre-main:TURN_ID", 0).
   - Calls LLM for main narration.
   - Reloads state, sets phase to Quantifying.
   - Calls execute_freeaction_impl(), which calls next_state.add_log(narration_text, None, LogType::Narration).
   - add_log for non-Input appends the LogEntry to turn.active_swipe_mut().entries of the **last** turn.
   - If trigger continuation exists:
     - Saves committed snapshot: save_committed_state(ctx, next_state, "pre-event:TURN_ID", 0).
     - Calls LLM for trigger continuation.
     - Calls commit_trigger_narration(), which calls state.add_log(continuation_text, None, LogType::Narration) -- again appended to the **same** turn's active swipe.
     - Runs post-trigger quantifier + NPC events.
   - Saves final state: save_state(ctx, next_state, turn_id, swipe_index).

**Key insight:** The entire pipeline (narration + trigger event) appends ALL generated entries into the **same** Turn's active Swipe. There is no per-message turn creation. LogType::Dialogue is defined but never produced by the current pipeline.

### How Retry Works (Current Model)

**Entry point:** retry_last_response_impl loads the latest snapshot, gets turn_id and current_swipe, checks if the last AI response is an event continuation (is_last_ai_response_event_continuation() checks if the last Narration/Dialogue entry has event_header.is_some()).

**Main narration retry (retry_main_narration):**
1. Loads snapshot load_by_turn("pre-main:TURN_ID", 0).
2. Reconstructs GameState from that snapshot (this is the state BEFORE any generation for this turn).
3. Calls turn.create_swipe(current_swipe + 1) on the last turn -- this pushes a NEW **empty** Swipe and sets it as active.
4. Calls execute_freeaction_pipeline(..., current_swipe + 1).
5. The pipeline re-runs from scratch: LLM narration -> quantifier -> trigger -> event. All new entries go into the NEW empty swipe.

**Event continuation retry (retry_event_continuation):**
1. Loads snapshot load_by_turn("pre-event:TURN_ID", 0).
2. Reconstructs GameState from that snapshot (state AFTER main narration but BEFORE trigger event).
3. Calls turn.create_swipe_copying_active(current_swipe + 1) -- this pushes a NEW Swipe that **copies** all entries from the currently active swipe, then sets the new one active.
4. Calls LLM for trigger continuation directly.
5. Calls commit_trigger_narration() to append the new event text to the new swipe.
6. Runs post-trigger quantifier + NPC events.
7. Saves with current_swipe + 1.

**Key insight:** Retry on main narration RE-RUNS THE ENTIRE PIPELINE from pre-main snapshot. Retry on event only re-runs the event part from pre-event snapshot, but copies the main narration entries into the new swipe so they are preserved.

### How History Is Mutated (Current Model)

**Edit (server/fragments/history.rs edit_history_handler):**
- Loads latest snapshot to get (turn_id, swipe_index).
- Loads state via state.load_state().
- Calls guard.edit_log(id, form.text).
- edit_log searches ALL turns -> ALL swipes -> ALL entries for a matching LogEntry.id. If found, updates entry.text.
- Saves snapshot with same (turn_id, swipe_index).

**Delete (server/fragments/history.rs delete_history_handler):**
- Loads state via state.load_state().
- Calls guard.delete_last_turn() which simply does self.narrative.turns.pop().
- This removes the **ENTIRE** last Turn (input + all its swipe entries).
- Calls snapshot_storage.delete_turn_snapshots(removed_turn_id) to delete pre-main/pre-event snapshots too.
- Gets new_turn_id from the new last turn.
- Saves snapshot with (new_turn_id, 0).

**Swipe switch (server/fragments/checkpoint.rs switch_swipe_handler):**
- Loads state.
- Finds turn by turn_id.
- Sets turn.active_swipe_index = swipe_index.
- Saves snapshot with (turn_id, swipe_index).

**Checkpoint restore:**
- Loads checkpoint -> gets (turn_id, swipe_index).
- Loads snapshot load_by_turn(turn_id, swipe_index).
- Reconstructs GameState.
- Sets turn.active_swipe_index = swipe_index on the matching turn.
- Saves as new latest snapshot.

### How State Is Loaded/Saved In Each Handler

| Handler | Load | Save |
|---------|------|------|
| action_handler / process_action | state.load_state() (via AppState::load_state) | GameStateSnapshot::from_game_state + snapshot_storage.save() |
| retry_handler | snapshot_storage.load_latest(None) + GameState::from_snapshot | GameStateSnapshot::from_game_state + snapshot_storage.save() (sets status=Generating) |
| edit_history_handler | snapshot_storage.load_latest(None) + state.load_state() | GameStateSnapshot::from_game_state + snapshot_storage.save() (preserves turn_id/swipe_index) |
| delete_history_handler | state.load_state() | delete_turn_snapshots + GameStateSnapshot::from_game_state + snapshot_storage.save() (new turn_id, swipe=0) |
| switch_swipe_handler | state.load_state() | GameStateSnapshot::from_game_state + snapshot_storage.save() (same turn_id, new swipe_index) |
| create_checkpoint_handler | snapshot_storage.load_latest(None) | snapshot_storage.save_checkpoint() |
| restore_checkpoint_handler | load_checkpoint + load_by_turn | GameStateSnapshot::from_game_state + snapshot_storage.save() |
| reset_handler | N/A (creates fresh GameState::new) | GameStateSnapshot::from_game_state + snapshot_storage.save() |

AppState::load_state pattern: load_latest(None) -> if Some, GameState::from_snapshot(snap, world, map, player, npcs) -> if None, GameState::new(...).

helpers::load_state pattern: Identical to AppState::load_state but used inside the engine.

helpers::save_state pattern: GameStateSnapshot::from_game_state(state, turn_id, swipe_index) -> snapshot_storage.save(snapshot).

helpers::save_committed_state pattern: Same as save_state but sets snapshot.committed = true before saving.

### Key Observations for Refactoring

1. **Snapshot narrative column is a JSON blob** containing the entire NarrativeState (turns, swipes, entries, generation state). The game_state_snapshots table has ON CONFLICT(turn_id, swipe_index) DO UPDATE, meaning each (turn_id, swipe_index) pair stores exactly one snapshot.

2. **Turn IDs are used as snapshot keys:** "pre-main:TURN_ID" and "pre-event:TURN_ID" are synthetic turn IDs for committed snapshots. delete_turn_snapshots deletes all three.

3. **add_log appends to the last turn's active swipe.** This is the central mutation point. In the new message model, add_log would insert a new Message row (or append to the messages Vec) instead.

4. **add_input creates a new Turn.** In the new model, player input is just another message with role=User.

5. **Retry regenerates from snapshots.** The pre-main/pre-event snapshots capture the full GameState at a point in time. In the new model, retry would load the appropriate snapshot and regenerate only the target message, then create a new swipe for that specific message.

6. **The server and engine both construct GameStateSnapshot inline.** There is no central save function in the server layer -- each handler calls from_game_state + save directly. The engine uses helpers::save_state / save_committed_state.

7. **GenerationGuard pattern:** tokio::task::spawn_blocking wraps the engine call. A GenerationGuard (Drop impl on Arc<AtomicBool>) resets is_generating when the block finishes. This prevents concurrent async actions.

---

## Example Flow

```
[1] Player: look around
[2] Narrator: You see a courtyard...
[3] Event: A guard approaches...
[4] Carla: Stay close.
```

Operations:
- **Retry Carla** -> Carla message gets a new swipe (Carla says something else)
- **Delete Carla** -> Carla removed. Event is now last.
- **Retry Event** -> Event gets a new swipe (different event happens)
- **Delete Event** -> Event removed. Narration is now last.
- **Retry Narration** -> Narration gets a new swipe
- **Delete Narration** -> Narration removed. Player input is now last.
- **Delete Player Input** -> Input removed. Chat is empty.

## Why This Is Better Than Turn Model

| Operation | Turn Model | Message Model |
|-----------|-----------|---------------|
| Retry Carla | Cannot -- retry regenerates entire turn | Carla gets a new swipe |
| Delete Event | Deletes entire turn (narration gone too) | Event removed, narration stays |
| Retry narration after deleting event | N/A -- turn was already deleted | Narration is now last, can be retried |

## Target Schema

```sql
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    role TEXT NOT NULL,
    sender TEXT,
    content TEXT NOT NULL,
    active_swipe_index INTEGER NOT NULL DEFAULT 0,
    sequence INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE message_swipes (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    index INTEGER NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(message_id, index)
);
```

## In-Memory Model

```rust
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub sender: Option<String>,
    pub content: String,
    pub active_swipe_index: u32,
    pub swipes: Vec<MessageSwipe>,
    pub sequence: u32,
    pub created_at: DateTime<Utc>,
}

pub struct MessageSwipe {
    pub id: String,
    pub index: u32,
    pub content: String,
    pub created_at: DateTime<Utc>,
}
```

## How the Pipeline Works

1. Player sends look around -> insert Message(role=User, sequence=N)
2. Main LLM generates narration -> insert Message(role=Narrator, sequence=N+1)
3. Trigger fires -> insert Message(role=Narrator, sequence=N+2)
4. NPC speaks -> insert Message(role=Assistant, sender=Carla, sequence=N+3)

Each insert is independent. Each message can later be retried or deleted independently.

## How Retry Works

1. User clicks retry on last message
2. Load game state from pre-generation snapshot
3. Regenerate that specific message
4. Create new MessageSwipe row for that message
5. Update message active_swipe_index to point to new swipe

## How Delete Works

1. User clicks delete on last message
2. Delete that Message row (cascade deletes its swipes)
3. Previous message is now last

## Files to Change

| File | Change |
|------|--------|
| src/model/message.rs | New -- Message, MessageSwipe, MessageRole |
| src/model/state.rs | Refactor -- NarrativeState uses Vec<Message> |
| src/storage/db.rs | New tables -- messages, message_swipes; drop narrative from snapshots |
| src/storage/snapshot_storage.rs | Refactor -- load/save narrative from message tables |
| src/engine/game_service/actions.rs | Refactor -- pipeline inserts individual messages |
| src/engine/game_service/retry.rs | Refactor -- retry creates new swipe on last message |
| src/server/fragments/history.rs | Refactor -- delete last message, edit any message by ID |
| src/server/templates.rs | Update -- render from messages, show swipe nav |
| tests/ | Update -- all tests for new model |

## Validation

```bash
cd chronicler_engine && python build.py
```



~12 files. The game logic (prompts, LLM calls, quantifier) stays the same. Only storage shape changes.

## Investigation Findings

### 1. ALL Source Files Referencing Turn, Swipe, LogEntry, or NarrativeState

| File | What It Uses | Change Needed |
|------|-------------|---------------|
| src/model/turn.rs | Defines `Turn`, `Swipe` | **DELETE** -- replace with `src/model/message.rs` |
| src/model/state.rs | Defines `NarrativeState` with `turns: Vec<Turn>`, `LogEntry`, `add_log`, `add_input`, `edit_log`, `delete_last_log`, `delete_last_turn`, `history()` | **MAJOR** -- refactor `NarrativeState` to use `messages: Vec<Message>`; rewrite all mutation methods |
| src/model/state_snapshot.rs | `GameStateSnapshot` has `turn_id: String`, `swipe_index: u32` | **MAJOR** -- remove turn_id/swipe_index; snapshot becomes a simple state blob keyed by timestamp only |
| src/model/checkpoint.rs | `Checkpoint` has `turn_id`, `swipe_index` | **MODERATE** -- change to `message_id` + `swipe_index` (or remove swipe_index if checkpoints just store full state) |
| src/model/mod.rs | `pub mod turn;` | **MINOR** -- replace with `pub mod message;` |
| src/model/state_tests.rs | Uses `add_log`, `history()`, `edit_log`, `delete_last_log`, `narrative.turns` in prop tests | **MODERATE** -- update tests for new model |
| src/engine/action_processing.rs | `FreeActionContext` takes `history: &[LogEntry]`; `execute_freeaction_impl` calls `add_log` | **MODERATE** -- `history()` still returns `Vec<LogEntry>` so interface stays; `add_log` behavior changes |
| src/engine/game_service/actions.rs | `execute_freeaction_pipeline` takes `turn_id`, `swipe_index`; calls `save_committed_state(ctx, state, "pre-main:TURN_ID", 0)` | **MAJOR** -- remove turn_id/swipe_index params; save committed snapshots by message_id instead |
| src/engine/game_service/retry.rs | `retry_last_response_impl` loads snapshot by turn_id/swipe; calls `turn.create_swipe()` and `turn.create_swipe_copying_active()` | **MAJOR** -- retry operates on last **message** not turn; creates new `MessageSwipe` instead of `Swipe` |
| src/engine/game_service/helpers.rs | `save_state(ctx, state, turn_id, swipe_index)`, `save_committed_state` | **MODERATE** -- remove turn_id/swipe_index params from helpers |
| src/engine/trigger_eval_tests.rs | References `NarrativeState` | **MINOR** -- likely just test data construction |
| src/server/fragments/actions.rs | `process_action` reads `narrative.turns.last().id` for turn_id; saves snapshot with `(turn_id, 0)` | **MAJOR** -- no turn_id needed; save snapshot without turn_id/swipe_index |
| src/server/fragments/history.rs | `edit_history_handler` calls `edit_log(id, text)`; `delete_history_handler` calls `delete_last_turn()` and `delete_turn_snapshots(turn_id)` | **MAJOR** -- delete removes last **message**; edit finds message by id |
| src/server/fragments/checkpoint.rs | `switch_swipe_handler` takes `(turn_id, swipe_index)`; `restore_checkpoint_handler` loads by turn_id/swipe_index | **MAJOR** -- swipe switching is per-message; checkpoint stores message_id |
| src/server/fragments/misc.rs | `retry_handler` uses `snapshot.turn_id`, `snapshot.swipe_index`; `reset_handler` uses `add_log` | **MAJOR** -- retry handler needs new snapshot shape |
| src/server/fragments/renderers.rs | `render_action_area` reads `narrative.turns.last()` for swipe nav data | **MODERATE** -- swipe nav reads from last **message** instead |
| src/server/templates.rs | `StoryLogTemplate` renders from `&[LogEntry]`; `ActionAreaTemplate` takes `turn_data: Option<(String, u32, u32)>` | **MODERATE** -- `LogEntry` can stay as render type; `turn_data` becomes `message_data` |
| src/server/templates_tests.rs | Constructs `LogEntry` directly for template tests | **MINOR** -- can keep `LogEntry` as render DTO |
| src/server/debug.rs | `DebugStateResponse` has `narration_history_tail: Vec<LogEntry>` | **MINOR** -- `history()` still returns `Vec<LogEntry>` |
| src/server/mod_tests.rs | `FailingStorage` implements `SnapshotStorage` trait with `load_by_turn`, `delete_turn_snapshots` | **MODERATE** -- update mock impl for trait changes |
| src/bootstrap/run.rs | `GameStateSnapshot::from_game_state(&state, "initial", 0)`; `add_log` for arrival narration | **MODERATE** -- remove turn_id/swipe_index from snapshot creation |
| src/bootstrap/scenario.rs | `inject_scenario_logs` calls `state.add_log(text, None, LogType::Narration)` | **MINOR** -- `add_log` signature stays, behavior changes |
| src/narrative/prompt/types.rs | `PromptContext` and `PromptBuilder` hold `history: &[LogEntry]` | **MINOR** -- `history()` still returns `Vec<LogEntry>` |
| src/narrative/prompt/context.rs | `make_prompt_context` takes `history: &[LogEntry]` | **MINOR** -- interface unchanged |
| src/narrative/prompt/builder_tests.rs | Constructs `LogEntry` directly for prompt tests | **MINOR** -- can keep `LogEntry` as test DTO |
| src/narrative/agents/quantifier/types.rs | References `LogEntry` | **MINOR** -- likely just history access |
| src/narrative/agents/quantifier/test_support.rs | References `LogEntry` | **MINOR** -- test data |
| src/narrative/agents/quantifier/core_tests.rs | References `LogEntry` | **MINOR** -- test data |
| src/narrative/agents/quantifier/prompt_tests.rs | References `LogEntry` | **MINOR** -- test data |
| src/narrative/llm/mock_tests.rs | References `LogEntry` | **MINOR** -- test data |
| src/test_support/fixtures.rs | `TestGameState::with_npc_raw` constructs `NarrativeState { turns: Vec::new(), ... }` | **MODERATE** -- change `turns` to `messages` |
| src/test_support/in_memory_storage.rs | `InMemorySnapshotStorage` implements `SnapshotStorage` with `load_by_turn`, `delete_turn_snapshots` | **MAJOR** -- update for new trait signatures |

### 2. ALL Storage Files Needing Schema Changes

| File | Change Needed |
|------|---------------|
| src/storage/db.rs | **MAJOR** -- Add `messages` and `message_swipes` table migrations. Option A: Keep `narrative` JSON column in snapshots (simplest migration -- just change what's inside the JSON). Option B: Remove `narrative` from snapshots and load/save messages separately (more work, cleaner schema). |
| src/storage/snapshot_storage.rs | **MAJOR** -- `SnapshotStorage` trait: remove `turn_id` param from `load_latest`, replace `load_by_turn(turn_id, swipe_index)` with `load_by_id(snapshot_id)`, replace `delete_turn_snapshots(turn_id)` with `delete_snapshots_before(message_id)` or similar. All save/load logic changes. |
| src/storage/mod.rs | **MINOR** -- May need new exports for message storage. |
| src/storage/llm_message_storage.rs | **NONE** -- This is for LLM API logging, unrelated to game messages. |

### 3. ALL Test Files Constructing Turn/Swipe/LogEntry

| File | What It Constructs | Change Needed |
|------|-------------------|---------------|
| tests/flow_mock.rs | `add_log`, `narrative.turns.last().id`, `GameStateSnapshot::from_game_state(&state, turn_id, 0)`, `add_input_and_save` helper | **MAJOR** -- `add_input_and_save` helper needs rewrite; no turn_id extraction |
| tests/flow_mock/retry_main.rs | `turn.input.text = ...`, `turn.create_swipe(1)`, `turn.active_swipe_mut().entries`, `narrative.turns.clear()`, `GameStateSnapshot::from_game_state(&state, turn_id, swipe_index)`, `pre-main:test` synthetic turn IDs | **MAJOR** -- all turn/swipe construction becomes message/message_swipe; synthetic snapshot keys change |
| tests/flow_mock/retry_event.rs | Same as retry_main plus `event_header` checks on `history()` entries | **MAJOR** -- same pattern |
| tests/flow_mock/sequence.rs | `turn.active_swipe_mut().entries.retain(|e| e.id != id)`, `narrative.turns.clear()`, `GameStateSnapshot::from_game_state(&state, turn_id, swipe_index)` | **MAJOR** -- same pattern |
| tests/game_service/basic.rs | `narrative.turns.clear()`, `add_log`, `history()` | **MODERATE** -- `turns.clear()` becomes `messages.clear()` |
| tests/game_service/advanced.rs | Heavy use: `turns.clear()`, `add_log`, `history()`, `GameStateSnapshot::from_game_state(&state, turn_id, 0)`, `pre-main:test`, `pre-event:test`, `swipe_index`, `delete_last_turn()`, `event_header`, `turn.input.text`, `turn.active_swipe_mut()`, `turn.swipes.len()` | **MAJOR** -- most heavily affected test file |
| tests/state_snapshot_tests.rs | `GameStateSnapshot::from_game_state(&original, "msg1", 0)`, `history().len()`, `turn_id`, `swipe_index` | **MODERATE** -- remove turn_id/swipe_index from snapshot construction |
| tests/snapshot_storage_tests.rs | `GameStateSnapshot::from_game_state(&state, turn_id, swipe_index)`, `load_by_turn`, `delete_turn_snapshots`, `Checkpoint` with `turn_id`/`swipe_index` | **MAJOR** -- update for new trait and snapshot shape |
| tests/components/misc.rs | `add_log`, `GameStateSnapshot::from_game_state(&state, turn_id, 0)`, `InMemorySnapshotStorage`, `Checkpoint { turn_id, swipe_index }` | **MODERATE** -- update snapshot/checkpoint construction |
| tests/components/fragment.rs | `add_log`, `narrative.turns.last().unwrap().id`, `turn.create_swipe(1)`, `GameStateSnapshot::from_game_state(&state, turn_id, 0)`, `Checkpoint { turn_id, swipe_index }`, `FailingStorage` implementing old trait | **MAJOR** -- heavily affected; constructs turns and swipes directly |
| tests/components/text_check.rs | Likely constructs `LogEntry` | **MINOR** -- check for `LogEntry` usage |
| tests/diagnostic/backends.rs | Likely uses `history()` | **MINOR** -- check |
| tests/diagnostic/scenarios.rs | Likely uses `history()` | **MINOR** -- check |
| tests/flow_llm_tests.rs | Likely uses `add_log`, `history()` | **MODERATE** -- check |
| tests/llm_message_storage_tests.rs | **NONE** -- unrelated to game messages |

### 4. ALL Doc Files Describing the Turn/Swipe Model

| File | Relevance | Change Needed |
|------|-----------|---------------|
| docs/adr/adr-012-turn-swipe-model.md | **Primary** -- Defines the current Turn+Swipe ADR | **Write new ADR-014** superseding this; keep for history |
| docs/adr/adr-008-sqlite-snapshot-persistence.md | Mentions turn/swipe in snapshot schema | **Update** to reflect new snapshot keying (no turn_id/swipe_index) |
| docs/architecture/system.md | Describes `NarrativeState`, turns, swipes, `GameStateSnapshot` | **Update** architecture description |
| docs/system/game_flow.md | Describes turn creation, retry, delete, swipe switch flows | **Update** for message-based flow |
| docs/system/ui_design.md | Mentions swipe navigation UI | **Update** -- swipe nav is now per-message |
| docs/system/dashboard.md | Mentions `LogEntry` in dashboard context | **Minor** -- terminology update |
| docs/system/triggers.md | Mentions `NarrativeState`, `add_log` for trigger continuations | **Minor** -- flow is same, storage shape changes |
| docs/CHANGELOG.md | Mentions Turn, Swipe, LogEntry in past changes | **Add entry** for Message+Swipe migration |
| docs/README.md | Mentions Turn, Swipe in overview | **Update** overview |
| docs/external_applications/sillytavern_chat_window.md | Mentions Swipe in SillyTavern context | **Update** to describe per-message swipes |
| docs/external_applications/sillytavern_chat_window_example.html | HTML example with swipe references | **Update** if swipe UI changes |
| docs/external_applications/marinara_engine.md | Describes Marinara model (target) | **Verify** it already describes message model |
| docs/plans/archived/jericho-huntress-devil-dinosaur-20260513.md | Previous plan, mentions Message model | **Keep** -- already describes target state |
| docs/reviews/defensive-architecture-review.md | Mentions `NarrativeState` | **Minor** |
| docs/reviews/holistic-review-phase1-domain-alignment.md | Mentions Turn/Swipe | **Minor** |
| docs/reviews/holistic-review-phase2-structural-forces.md | Mentions Turn/Swipe | **Minor** |
| docs/reviews/holistic-review-phase3-evolution-stress.md | Mentions Turn/Swipe | **Minor** |
| docs/reviews/holistic-review-phase4-health-metrics.md | Mentions Turn/Swipe | **Minor** |
| docs/reviews/holistic-architectural-review.md | Mentions NarrativeState | **Minor** |
| docs/reviews/agent-scalability-assessment.md | Mentions LogEntry | **Minor** |
| docs/reviews/cross-project-architectural-comparison.md | Likely mentions turns | **Minor** |
| docs/system/llm_processing.md | May mention history format | **Minor** |
| docs/system/narration_engine.md | May mention turn structure | **Minor** |
| docs/system/prompt_system.md | Describes history as `Vec<LogEntry>` | **MINOR** -- `LogEntry` stays as prompt DTO |
| docs/reference/data_schemas.md | May document turn/swipe schema | **Update** for Message/MessageSwipe |
| docs/plans/multi-agent-architecture-overarching-spec.md | Mentions NarrativeState | **Minor** |

---

## Refined Files to Change

### Critical Path (must change for compile)
| # | File | Change |
|---|------|--------|
| 1 | src/model/message.rs | **NEW** -- `Message`, `MessageSwipe`, `MessageRole` enums/structs |
| 2 | src/model/state.rs | **REFACTOR** -- `NarrativeState`: `messages: Vec<Message>` instead of `turns: Vec<Turn>`; rewrite `add_log`, `add_input`, `edit_log`, `delete_last_log`, `delete_last_turn` -> `delete_last_message`, `history()` |
| 3 | src/model/state_snapshot.rs | **REFACTOR** -- Remove `turn_id` and `swipe_index` fields; snapshot is just `id`, `movement`, `narrative`, `scene`, `character_state`, `committed`, `created_at` |
| 4 | src/model/checkpoint.rs | **REFACTOR** -- Change `turn_id` -> `message_id` (or remove if checkpoints just point to snapshot_id) |
| 5 | src/model/mod.rs | **UPDATE** -- `pub mod message;` instead of `pub mod turn;` |
| 6 | src/storage/db.rs | **UPDATE** -- Add `messages` and `message_swipes` tables; keep or remove `narrative` JSON from snapshots |
| 7 | src/storage/snapshot_storage.rs | **REFACTOR** -- Redesign `SnapshotStorage` trait: `load_latest()` without turn_id param, `load_by_id(id)`, remove `delete_turn_snapshots`, update `Checkpoint` CRUD |
| 8 | src/test_support/in_memory_storage.rs | **REFACTOR** -- Update `InMemorySnapshotStorage` for new trait |
| 9 | src/engine/game_service/helpers.rs | **REFACTOR** -- `save_state(ctx, state)` without turn_id/swipe_index; `save_committed_state(ctx, state, label)` where label is just a string tag |
| 10 | src/engine/game_service/actions.rs | **REFACTOR** -- `execute_freeaction_pipeline` takes `state` only (no turn_id/swipe_index); save committed snapshots with message-based labels |
| 11 | src/engine/game_service/retry.rs | **REFACTOR** -- Retry loads pre-generation snapshot by label, regenerates last message, creates new `MessageSwipe` on that message |
| 12 | src/server/fragments/actions.rs | **REFACTOR** -- Remove turn_id extraction from `narrative.turns.last()`; save snapshot without turn_id/swipe_index |
| 13 | src/server/fragments/history.rs | **REFACTOR** -- `delete_last_turn()` -> `delete_last_message()`; `edit_log` searches messages instead of turns/swipes |
| 14 | src/server/fragments/checkpoint.rs | **REFACTOR** -- `switch_swipe_handler` takes `message_id` not `turn_id`; checkpoint stores `message_id` |
| 15 | src/server/fragments/misc.rs | **REFACTOR** -- `retry_handler` uses new snapshot shape; `reset_handler` saves without turn_id |
| 16 | src/server/fragments/renderers.rs | **UPDATE** -- `render_action_area` reads swipe nav from last message |
| 17 | src/server/templates.rs | **UPDATE** -- `ActionAreaTemplate` takes `message_data` instead of `turn_data` |

### Test Updates (must change for tests to pass)
| # | File | Change |
|---|------|--------|
| 18 | src/model/state_tests.rs | Update prop tests that reference `narrative.turns` |
| 19 | src/server/mod_tests.rs | Update `FailingStorage` mock for new trait |
| 20 | src/server/templates_tests.rs | Keep -- `LogEntry` stays as render DTO |
| 21 | src/narrative/prompt/builder_tests.rs | Keep -- `LogEntry` stays as prompt DTO |
| 22 | tests/flow_mock.rs | Rewrite `add_input_and_save` helper; remove turn_id extraction |
| 23 | tests/flow_mock/retry_main.rs | Rewrite all turn/swipe construction to message/message_swipe |
| 24 | tests/flow_mock/retry_event.rs | Same as retry_main |
| 25 | tests/flow_mock/sequence.rs | Same pattern |
| 26 | tests/game_service/basic.rs | `turns.clear()` -> `messages.clear()` |
| 27 | tests/game_service/advanced.rs | Heaviest rewrite -- all snapshot construction, pre-main/pre-event synthetic IDs, swipe logic |
| 28 | tests/state_snapshot_tests.rs | Remove turn_id/swipe_index from snapshot construction |
| 29 | tests/snapshot_storage_tests.rs | Update for new trait methods; update `Checkpoint` construction |
| 30 | tests/components/misc.rs | Update snapshot/checkpoint construction |
| 31 | tests/components/fragment.rs | Heavily rewrite -- turn creation, swipe creation, checkpoint storage |
| 32 | src/test_support/fixtures.rs | `NarrativeState { turns: ... }` -> `NarrativeState { messages: ... }` |

### Documentation Updates
| # | File | Change |
|---|------|--------|
| 33 | docs/adr/adr-014-message-swipe-model.md | **NEW** -- ADR superseding ADR-012 |
| 34 | docs/adr/adr-008-sqlite-snapshot-persistence.md | Update snapshot schema description |
| 35 | docs/architecture/system.md | Update NarrativeState description |
| 36 | docs/system/game_flow.md | Update flow descriptions |
| 37 | docs/system/ui_design.md | Update swipe nav description |
| 38 | docs/CHANGELOG.md | Add migration entry |
| 39 | docs/README.md | Update overview |
| 40 | docs/reference/data_schemas.md | Update schema docs |

---

## Validation

```bash
cd chronicler_engine && python build.py
```

## Scope

~40 files total. The game logic (prompts, LLM calls, quantifier, triggers) stays the same. The changes are concentrated in:
- Model layer (state.rs, message.rs, state_snapshot.rs, checkpoint.rs)
- Storage layer (db.rs, snapshot_storage.rs, in_memory_storage.rs)
- Engine layer (actions.rs, retry.rs, helpers.rs)
- Server layer (actions.rs, history.rs, checkpoint.rs, misc.rs, renderers.rs, templates.rs)
- Tests (flow_mock/, game_service/, components/, snapshot_storage_tests.rs, state_snapshot_tests.rs)