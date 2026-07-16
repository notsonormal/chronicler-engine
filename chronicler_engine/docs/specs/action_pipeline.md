# Component Spec: Action Pipeline

## Scenarios

### Normal flow

#### Scenario 1.1: Successful action produces exactly one narration and returns to Idle

**Given** a fresh game state with the default test world loaded
**And** `narrative.history` is empty
**And** an LLM backend that returns `"You see a small wooden room."` for any narration prompt
**When** the user submits the action `"look"` via `action_pipeline::execute_action_impl`
**Then** execute_action_impl returns `Ok(...)`
**And** `narrative.input_buffer.status` is `GenerationStatus::Idle` (not `Generating` and not `Error`)
**And** `narrative.history` contains exactly 2 new entries since the action:
  - one entry with `message_type == Input` and `text == "look"`
  - one entry with `message_type == Narration` and `text == "You see a small wooden room."`
**And** `narrative.input_buffer.input` is empty

#### Scenario 1.2: User input is persisted in history before the narration entry

**Given** a fresh game state with `narrative.history` empty
**And** an LLM backend that returns `"It is a small wooden room."` for the narration prompt
**When** the user submits the action `"examine the room"`
**Then** `narrative.history` contains 2 new entries
**And** the `Input` entry appears at a smaller history index than the `Narration` entry
**And** the `Input` entry's text is `"examine the room"`
**And** the `Narration` entry's text is `"It is a small wooden room."`
**And** `narrative.input_buffer.status` is `GenerationStatus::Idle`

#### Scenario 1.3: Action whose quantifier detects movement moves the player

**Given** a game state with `movement.current_room_id == "room1"`
**And** a connected destination room exists with id `"village_square"`
**And** a quantifier backend that responds with movement `{"type": "Entering", "destination": "village_square"}` for the quantifier prompt
**And** a narrator backend that returns `"You walk into the village square."` for the narration prompt
**When** the user submits the action `"walk to the village square"`
**Then** execute_action_impl returns `Ok(...)` within 500 ms
**And** `movement.current_room_id` is `"village_square"`
**And** `narrative.input_buffer.status` is `GenerationStatus::Idle`

#### Scenario 1.4: Action whose quantifier detects an NPC fires the trigger

**Given** a game state with an NPC `"shopkeeper"` whose trigger fires when `times_met == 0`
**And** the current room's NPC list contains `"shopkeeper"`
**And** a quantifier backend that responds with `{"npcs_in_room": ["shopkeeper"]}` for the quantifier prompt
**And** a narrator backend that returns valid narration for both main and trigger prompts
**When** the user submits the action `"enter the shop"`
**Then** `narrative.input_buffer.status` is `GenerationStatus::Idle`
**And** `narrative.history` contains at least one `Narration` whose `event_header` is `Some(...)` (the trigger fired)
**And** `narrative.history` contains at least 2 `Narration` entries total (main + trigger continuation)

#### Scenario 1.5: Empty input produces a continuation narration without adding an Input message

**Given** a fresh game state with `narrative.history` empty
**And** a narrator backend that returns `"The scene continues."` for the continuation prompt
**When** the user submits an empty string (`""`) via `execute_action_impl`
**Then** execute_action_impl returns `Ok(...)`
**And** `narrative.input_buffer.status` is `GenerationStatus::Idle`
**And** `narrative.history` contains exactly one new `Narration` entry (continuation, no `Input` entry)
**And** the new `Narration` entry's text is `"The scene continues."`

### Error recovery

#### Scenario 2.1: Action in a non-existent room sets GenerationStatus to Error

**Given** a game state with `movement.current_room_id == "non_existent_room"`
**And** `narrative.history` empty
**When** the user submits any action (e.g. `"look"`)
**Then** execute_action_impl returns `Ok(...)` (does not propagate as an error)
**And** `narrative.input_buffer.status` is `GenerationStatus::Error(msg)` for some non-empty `msg` indicating the room is invalid
**And** `narrative.input_buffer.status.is_generating()` is `false`
**And** `narrative.history` contains no new `Narration` entries

#### Scenario 2.2: LLM transport failure sets GenerationStatus to Error

**Given** a fresh game state with `narrative.history` empty
**And** an LLM backend configured to fail (returns an error for the narration request)
**When** the user submits the action `"look"`
**Then** execute_action_impl returns `Ok(...)`
**And** `narrative.input_buffer.status` is `GenerationStatus::Error(msg)` for some non-empty `msg`
**And** `narrative.input_buffer.status.is_generating()` is `false`
**And** `narrative.input_buffer.status.error_message()` is `Some(_)`

#### Scenario 2.3: Empty LLM response sets Error without persisting an empty narration

**Given** a fresh game state with `narrative.history` empty
**And** an LLM backend that returns an empty string for the narration prompt
**When** the user submits the action `"examine the room"`
**Then** execute_action_impl returns `Ok(...)`
**And** `narrative.input_buffer.status` is `GenerationStatus::Error(msg)` where `msg` contains `"empty"`
**And** `narrative.history` contains zero new `Narration` entries (no empty narration persisted)

#### Scenario 2.4: Trigger narration failure preserves main narration and logs the failure

**Given** a game state with an NPC `"shopkeeper"` whose trigger fires when `times_met == 0`
**And** a quantifier backend that returns `{"npcs_in_room": ["shopkeeper"]}`
**And** a narrator backend that succeeds for the main prompt but fails for the trigger prompt
**When** the user submits the action `"examine the shopkeeper"`
**Then** `narrative.input_buffer.status` is `GenerationStatus::Idle` (not `Generating`)
**And** `narrative.history` contains at least one `Narration` entry from the main narration (preserved)
**And** `narrative.history` contains at least one `System` entry whose `text` mentions `"Trigger narration failed"`

### State hygiene

#### Scenario 3.1: Action clears a stale `last_trigger` from a prior session

**Given** a game state with `narrative.last_trigger = Some(<non-default StoredTriggerContext>)`
**And** a fresh game session otherwise (no in-flight generation)
**When** the user submits the action `"look"`
**Then** execute_action_impl returns `Ok(...)`
**And** `narrative.input_buffer.status` is `GenerationStatus::Idle`
**And** `narrative.last_trigger` is `None` (cleared)

#### Scenario 3.2: Streaming narration produces exactly one persisted Narration entry (no duplicates)

**Given** a fresh game state with `narrative.history` empty
**And** a streaming narrator backend that emits the full narration incrementally and signals completion
**And** a quantifier backend with a 500 ms delay before responding
**When** the user submits the action `"look around"`
**Then** execute_action_impl returns `Ok(...)`
**And** by the time the quantifier completes (within 1 s of submission), exactly one `Narration` entry exists in history
**And** after the full pipeline returns, `narrative.history` contains exactly one `Narration` entry (no duplicate from quantifier-triggered re-save)

#### Scenario 3.3: Delayed LLM response does not deadlock the pipeline

**Given** a fresh game state with `narrative.input_buffer.status == GenerationStatus::Generating` (entered via previous action)
**And** an LLM backend that takes 200 ms to respond
**When** the user submits the action `"look around"`
**Then** execute_action_impl returns `Ok(...)` within 2 s
**And** `narrative.input_buffer.status` is `GenerationStatus::Idle`
**And** `narrative.input_buffer.phase` is the default phase (reset after completion)

#### Scenario 3.4: GenerationPhase transitions to default after success, stays Narrating after failure

**Success case**

**Given** a fresh game state
**And** an LLM backend that returns valid narration
**When** the user submits the action `"look"`
**Then** `narrative.input_buffer.phase` equals `GenerationPhase::default()` (reset)

**Failure case**

**Given** a fresh game state
**And** an LLM backend that fails for the narration request
**When** the user submits the action `"look"`
**Then** `narrative.input_buffer.phase` is `GenerationPhase::Narrating` (preserved from where the failure occurred)

### Async and cancellation

#### Scenario 4.1: Action submitted when the cancel token is already cancelled resets status to Idle

**Given** a fresh game state with `narrative.history` empty
**And** the application's `cancel_token` is cancelled before `execute_action_impl` is called
**When** the user submits the action `"look"`
**Then** `narrative.input_buffer.status` is `GenerationStatus::Idle`
**And** `narrative.history` contains zero new `Narration` entries (generation did not run)

#### Scenario 4.2: Cancellation after main narration preserves the narration and returns Idle

**Given** a fresh game state
**And** a narrator backend with a 50 ms delay before responding
**And** the test waits until the narrator backend reports narration has started
**When** `cancel_token.cancel()` is called during the main narration phase
**And** `execute_action_impl("look around")` is awaited to completion
**Then** `narrative.input_buffer.status` is `GenerationStatus::Idle`
**And** `narrative.history` contains at least one `Narration` entry (main narration preserved before the cancellation checkpoint)

#### Scenario 4.3: Cancellation during trigger continuation preserves the main narration

**Given** a game state with NPC `"shopkeeper"` whose trigger fires when `times_met == 0`
**And** a narrator backend with a 50 ms delay before the trigger continuation call
**And** a quantifier that returns `{"npcs_in_room": ["shopkeeper"]}`
**And** the test waits until the narrator backend reports trigger narration has started
**When** `cancel_token.cancel()` is called during the trigger continuation phase
**And** `execute_action_impl("enter the shop")` is awaited to completion
**Then** `narrative.input_buffer.status` is `GenerationStatus::Idle`
**And** `narrative.history` contains at least one `Narration` entry (main narration preserved)

### Snapshots (visible via subsequent retry/delete)

#### Scenario 5.1: A pre-main snapshot is saved before narration begins

**Given** a fresh game state with `narrative.history` empty
**And** a narrator backend that takes 50 ms before responding
**When** the user submits the action `"examine the room"`
**And** the pipeline completes within 1 s
**Then** `storage.load_latest_snapshot()` returns `Some(snapshot)` with `snapshot.db_id.is_some()`

#### Scenario 5.2: A pre-event snapshot is saved before trigger continuation begins

**Given** a game state with NPC `"shopkeeper"` whose trigger fires when `times_met == 0`
**And** `narrative.history` empty
**And** a quantifier that returns `{"npcs_in_room": ["shopkeeper"]}`
**And** a narrator backend that responds successfully
**When** the user submits the action `"examine the shopkeeper"`
**And** the pipeline completes within 1 s
**Then** `storage.load_latest_snapshot()` returns `Some(snapshot)` with `snapshot.db_id.is_some()`

## Invariants

These properties must hold across every `execute_action_impl` call. Drift here
indicates a regression even if all scenarios pass.

- **I.1** After `execute_action_impl` returns, `input_buffer.status` is `Idle`
  or `Error(msg)` — never stuck `Generating`.
- **I.2** User Input messages always appear in history at a smaller index
  than their associated Narration (Input is persisted first).
- **I.3** No scenario produces more than one Narration message per
  `execute_action_impl` call (no duplicates from streaming, retry, or
  trigger continuation).
- **I.4** Failed narration never adds a Narration entry to history (no
  empty/broken narrations persisted).
- **I.5** Trigger continuation runs only when the quantifier detects an
  NPC in the current room with a matching trigger condition.