# Component Spec: Retry & Retrigger

## Endpoints

- **`POST /swipe/new`** — Retry. Auto-detects main retry or event retry from the
  last message. Fires async; the client observes the result by re-reading
  state (page refresh / htmx swap).
- **`POST /retrigger`** — Retrigger. Re-fires the trigger continuation from
  the current state. Fires async; same observation model.

Both endpoints return immediately with a status string. The generation
runs in a background task. The client polls or refreshes to observe the
outcome.

---

## Scenarios

### Main retry

#### Scenario R1.1: Main retry replaces the narration with a new swipe
**Given** a game state where the last message is a non-event Narration  
**And** the narrator backend returns `"A different take on the scene."` for the narration prompt  
**When** the client sends `POST /swipe/new`  
**Then** the response is 200 with a "Retrying..." status string  
**And** after generation completes, `narrative.input_buffer.status` is `GenerationStatus::Idle`  
**And** the narration message has one additional swipe (the new narration)  
**And** the new swipe is the active swipe  
**And** the message ID is unchanged (same message, new swipe)  
**And** `narrative.history` contains no new messages (swipe, not a new entry)  

#### Scenario R1.2: Main retry re-runs the quantifier and can move the player
**Given** a game state where the last message is a non-event Narration  
**And** `movement.current_room_id == "room1"`  
**And** a quantifier backend that returns movement to `"room2"` for the retry prompt  
**When** the client sends `POST /swipe/new`  
**Then** after generation completes, `movement.current_room_id` is `"room2"` (the quantifier re-ran with a potentially different result)  

#### Scenario R1.3: Main retry preserves the input message
**Given** a game state where the last message is a non-event Narration  
**And** an Input message exists in history with text `"look around"`  
**When** the client sends `POST /swipe/new`  
**Then** after generation completes, the Input message text is still `"look around"` (unchanged by retry)  

#### Scenario R1.4: Main retry uses edited input text
**Given** a game state where the last message is a non-event Narration  
**And** the Input message's active swipe has been edited to `"sprint forward"`  
**When** the client sends `POST /swipe/new`  
**Then** the new narration reflects the edited input `"sprint forward"`, not the original input text  

#### Scenario R1.5: Main retry re-evaluates triggers
**Given** a game state where the last message is a non-event Narration  
**And** no trigger fired on the original action (player was not in the trigger's room)  
**And** the quantifier returns movement to `"room2"` where an NPC with a trigger is present  
**When** the client sends `POST /swipe/new`  
**Then** after generation completes, the trigger fires (an event narration with an `event_header` appears in history)  

### Event retry

#### Scenario R2.1: Event retry replaces the event narration with a new swipe
**Given** a game state where the last message is an event Narration (has  
`event_header`)
**And** the narrator backend returns a different continuation text  
**When** the client sends `POST /swipe/new`  
**Then** the response is 200 with a "Retrying..." status string  
**And** after generation completes, `narrative.input_buffer.status` is  
`GenerationStatus::Idle`
**And** the event narration message has one additional swipe (the new  
continuation)
**And** the new swipe is the active swipe  
**And** `narrative.history` contains no new messages (swipe, not a new entry)  

#### Scenario R2.2: Event retry does not re-run the quantifier
**Given** a game state where the last message is an event Narration  
**And** `movement.current_room_id == "room2"`  
**When** the client sends `POST /swipe/new`  
**Then** after generation completes, `movement.current_room_id` is still  
`"room2"` (the quantifier does not re-run; world state is frozen at the
pre-event snapshot)

### Retrigger

#### Scenario R3.1: Retrigger creates a new event narration message
**Given** a game state where `narrative.last_trigger` is set (a trigger  
context exists)
**And** the last message is a non-event Narration (the main narration)  
**And** the narrator backend returns `"The shopkeeper greets you warmly."`  
for the trigger prompt
**When** the client sends `POST /retrigger`  
**Then** the response is 200 with a "Retriggering..." status string  
**And** after generation completes, `narrative.input_buffer.status` is  
`GenerationStatus::Idle`
**And** `narrative.history` contains one additional Narration message with an `event_header` (a new message, not a swipe)  

#### Scenario R3.2: Retrigger does not roll back state
**Given** a game state where `narrative.last_trigger` is set  
**And** the last message is a non-event Narration  
**And** `narrative.history` contains N messages  
**When** the client sends `POST /retrigger`  
**Then** after generation completes, `narrative.history` contains N+1 messages (the original history is preserved; the new event narration is appended, not a replacement)  

#### Scenario R3.3: Retrigger does not re-run the quantifier
**Given** a game state where `narrative.last_trigger` is set  
**And** the last message is a non-event Narration  
**And** `movement.current_room_id == "room2"`  
**When** the client sends `POST /retrigger`  
**Then** after generation completes, `movement.current_room_id` is still `"room2"` (no quantifier re-run)  

### Retry error cases

#### Scenario R4.1: Retry with no input to retry returns 400
**Given** a game state with no Input message in history  
**When** the client sends `POST /swipe/new`  
**Then** the response is 400 Bad Request  
**And** no generation is started  

#### Scenario R4.2: Retry with no game context returns 400
**Given** no game exists in storage  
**When** the client sends `POST /swipe/new`  
**Then** the response is 400 Bad Request  

#### Scenario R4.3: Retry when the anchor message has no snapshot returns 500
**Given** a game state where the last Input message exists  
**And** that message's `snapshot_id` is `None` (no snapshot was saved alongside it)  
**When** the client sends `POST /swipe/new`  
**Then** the response is 500 Internal Server Error  
**And** `narrative.input_buffer.status` is `GenerationStatus::Error(msg)` indicating the snapshot is missing  

#### Scenario R4.4: Retry when the anchor snapshot was deleted returns 500
**Given** a game state where the last Input message has a `snapshot_id`  
**And** that snapshot row has been removed from the database  
**When** the client sends `POST /swipe/new`  
**Then** the response is 500 Internal Server Error  
**And** `narrative.input_buffer.status` is `GenerationStatus::Error(msg)` indicating the snapshot was not found  

#### Scenario R4.5: Retry LLM failure sets Error
**Given** a game state where the last message is a non-event Narration  
**And** a narrator backend that fails for the narration request  
**When** the client sends `POST /swipe/new`  
**Then** after generation completes, `narrative.input_buffer.status` is `GenerationStatus::Error(msg)`  
**And** `narrative.input_buffer.status.is_generating()` is `false`  
**And** no new swipe is added (the old narration's swipes are unchanged)  

#### Scenario R4.6: Retry empty narration sets Error
**Given** a game state where the last message is a non-event Narration  
**And** a narrator backend that returns an empty string for the narration prompt  
**When** the client sends `POST /swipe/new`  
**Then** after generation completes, `narrative.input_buffer.status` is `GenerationStatus::Error(msg)` where `msg` contains `"empty"`  
**And** no new swipe is added  

#### Scenario R4.7: Retry room not found sets Error
**Given** a game state where `movement.current_room_id` refers to a  
non-existent room
**And** the last message is a non-event Narration  
**When** the client sends `POST /swipe/new`  
**Then** after generation completes, `narrative.input_buffer.status` is `GenerationStatus::Error(msg)` where `msg` indicates the room is invalid  
**And** no new swipe is added  

#### Scenario R4.8: Event retry trigger narration failure sets Error and preserves main narration
**Given** a game state where the last message is an event Narration  
**And** a narrator backend that fails only for the trigger continuation prompt (main narration succeeds)  
**When** the client sends `POST /swipe/new`  
**Then** after generation completes, `narrative.input_buffer.status` is `GenerationStatus::Error(msg)` where `msg` contains `"Trigger narration failed"`  
**And** the main narration is preserved in history (not removed)  
**And** a System message is persisted in history with text mentioning `"Trigger narration failed"`  

### Retrigger error cases

#### Scenario R4.9: Retrigger with no trigger context returns 400
**Given** a game state where `narrative.last_trigger` is `None`  
**When** the client sends `POST /retrigger`  
**Then** the response is 400 Bad Request  
**And** no generation is started  

#### Scenario R4.10: Retrigger with no messages returns 400
**Given** a game state with no messages in history  
**When** the client sends `POST /retrigger`  
**Then** the response is 400 Bad Request  

#### Scenario R4.11: Retrigger when the last message is not a narration returns 400
**Given** a game state where `narrative.last_trigger` is set  
**And** the last message is an Input message (not a Narration or Dialogue)  
**When** the client sends `POST /retrigger`  
**Then** the response is 400 Bad Request  

#### Scenario R4.12: Retrigger when the last message is an event continuation returns 400
**Given** a game state where `narrative.last_trigger` is set  
**And** the last message is an event Narration (has `event_header`)  
**When** the client sends `POST /retrigger`  
**Then** the response is 400 Bad Request  

#### Scenario R4.13: Retrigger trigger narration failure sets Error
**Given** a game state where `narrative.last_trigger` is set  
**And** the last message is a non-event Narration  
**And** a narrator backend that fails for the trigger continuation prompt  
**When** the client sends `POST /retrigger`  
**Then** after generation completes, `narrative.input_buffer.status` is `GenerationStatus::Error(msg)` where `msg` contains `"Trigger narration failed"`  
**And** no new event narration message is added  

#### Scenario R4.14: Retrigger with no game context returns 400
**Given** no game exists in storage  
**When** the client sends `POST /retrigger`  
**Then** the response is 400 Bad Request  

### Cancellation

#### Scenario R5.1: Retry cancelled mid-flight resets to Idle
**Given** a game state where the last message is a Narration  
**And** the application's `shutdown_token` is cancelled before the retry starts  
**When** the client sends `POST /swipe/new`  
**Then** `narrative.input_buffer.status` is `GenerationStatus::Idle`  
**And** `narrative.input_buffer.phase` is `GenerationPhase::default()`  
**And** no new swipe or message is added  

#### Scenario R5.2: Retrigger cancelled mid-flight resets to Idle
**Given** a game state where `narrative.last_trigger` is set  
**And** the application's `shutdown_token` is cancelled before the retrigger starts  
**When** the client sends `POST /retrigger`  
**Then** `narrative.input_buffer.status` is `GenerationStatus::Idle`  
**And** `narrative.input_buffer.phase` is `GenerationPhase::default()`  
**And** no new message is added  

#### Scenario R5.3: Retry while a generation is already in flight returns "Still thinking..."
**Given** a generation is already in flight (`is_generating()` is true)  
**When** the client sends `POST /swipe/new`  
**Then** the response is 200 with a "Still thinking..." status string  
**And** no retry is started  

#### Scenario R5.4: Retrigger while a generation is already in flight returns "Still thinking..."
**Given** a generation is already in flight (`is_generating()` is true)  
**When** the client sends `POST /retrigger`  
**Then** the response is 200 with a "Still thinking..." status string  
**And** no retrigger is started  

---

## Invariants

These properties must hold across every retry or retrigger operation.
Drift here indicates a regression even if all scenarios pass.

- **I.1** After a retry or retrigger returns, `input_buffer.status` is
  `Idle` or `Error(msg)` — never stuck `Generating`.
- **I.2** Retry (`POST /swipe/new`) never adds a new message to
  `narrative.history`. It appends a swipe to an existing message.
- **I.3** Retrigger (`POST /retrigger`) adds exactly one new message to
  `narrative.history` (the event narration). It does not append a swipe.
- **I.4** Retry and retrigger never modify an existing Input message's text.
  (Main retry uses the input's current active swipe text, which the user
  may have edited before clicking retry, but the retry itself does not
  change it.)
- **I.5** Each retry appends exactly one swipe. Repeated retries on the
  same message increment the swipe count by one each time.
- **I.6** Main retry re-runs the quantifier; event retry and retrigger do
  not.
- **I.7** If a message exists in history, it must have a `snapshot_id`
  pointing to a restorable snapshot. A message without a snapshot (or with
  a dangling snapshot id) is a data-integrity violation (500), not a user
  error (400).
- **I.8** Retry and retrigger reject concurrent generation the same way
  `process_action` does: 200 with a "Still thinking..." status, no
  generation started.
- **I.9** `last_trigger` is set before the trigger LLM call, so it survives
  a trigger-narration failure. Retrigger is available as a recovery path
  after trigger failure without re-running the full pipeline.
