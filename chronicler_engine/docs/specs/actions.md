# Feature Spec: Actions

Endpoint: `POST /action`

Behavioural authority for the action endpoint — what a client observes
through HTTP. Each "When" is an HTTP request; each "Then" is an
HTTP-observable outcome asserted via `message_service.load_messages()`
(the same data `/fragment/story-log` renders) or the generation status
exposed through the `/status/generating` endpoint. Internal-state seams
(cancellation timing, mid-flight streaming flags, `last_trigger` field,
phase transitions, snapshots, call sequencing) are not asserted here —
they live at the unit and driven-adapter tiers.

Scenario IDs are `1.x` through `6.x` and stay stable across edits. The
pilot dedups by ID across `docs/specs/*.md`, so IDs must stay unique.

## Scenarios

### Normal flow

#### Scenario 1.1: Successful action produces exactly one narration and returns to Idle

```gherkin
Given a fresh game state with the default test world loaded
And narrative.history is empty
And an LLM backend that returns "You see a small wooden room." for any narration prompt
When the client POST /action with command="look"
And the pipeline returns to idle (status not generating)
Then message_service.load_messages() contains exactly 2 new entries:
  - one Input entry whose text is "look"
  - one Narration entry whose text is "You see a small wooden room."
And message_service.load_or_fresh().narrative.input_buffer.status is Idle
```

#### Scenario 1.2: User input is persisted in history before the narration entry

```gherkin
Given a fresh game state with narrative.history empty
And an LLM backend that returns "It is a small wooden room." for the narration prompt
When the client POST /action with command="examine the room"
And the pipeline returns to idle
Then message_service.load_messages() contains 2 new entries
And the Input entry appears at a smaller history index than the Narration entry
And the Input entry's text is "examine the room"
And the Narration entry's text is "It is a small wooden room."
And message_service.load_or_fresh().narrative.input_buffer.status is Idle
```

#### Scenario 1.4: Action whose quantifier detects an NPC fires the trigger

```gherkin
Given a game state with an NPC "npc_1" whose trigger fires when times_met == 0
And the current room's NPC list contains "npc_1"
And a quantifier backend that responds with {"npcs_in_room": ["npc_1"]} for the quantifier prompt
And a narrator backend that returns valid narration for both main and trigger prompts
When the client POST /action with command="enter the shop"
And the pipeline returns to idle
Then message_service.load_messages() contains at least one Narration whose event_header is Some(...) (the trigger fired)
And message_service.load_messages() contains at least 2 Narration entries total (main + trigger continuation)
And message_service.load_or_fresh().narrative.input_buffer.status is Idle
```

#### Scenario 1.5: Empty input produces a continuation narration without adding an Input message

```gherkin
Given a fresh game state with narrative.history empty
And a narrator backend that returns "The scene continues." for the continuation prompt
When the client POST /action with command="" (empty)
And the pipeline returns to idle
Then message_service.load_messages() contains exactly one new Narration entry (continuation, no Input entry)
And the new Narration entry's text is "The scene continues."
And message_service.load_or_fresh().narrative.input_buffer.status is Idle
```

#### Scenario 1.6: Action trigger continuation re-runs the quantifier and detects newly-present NPCs

```gherkin
Given a game state with an NPC "gabriella" present in the current room's NPC list
And a quantifier backend that returns [] for the first quantifier call then ["gabriella"] for the second
And a narrator backend that returns valid narration for both main and trigger prompts
When the client POST /action with command="enter shop"
And the pipeline returns to idle
Then message_service.load_or_fresh().scene.npcs_in_area contains "gabriella"
And message_service.load_or_fresh().npc_encounter_log.npcs["gabriella"].times_met is 1
And message_service.load_or_fresh().npc_encounter_log.npcs["gabriella"].currently_meeting is true
```

#### Scenario 1.7: NPC without triggers produces narration with no event header

```gherkin
Given a game state with an NPC "bartender" in the current room's NPC list
And the NPC has triggers: vec![] (no trigger conditions)
And a quantifier backend that returns {"npcs_in_room": ["bartender"]} for the quantifier prompt
And a narrator backend that returns "The bartender nods." for the main narration prompt
When the client POST /action with command="talk to bartender"
And the pipeline returns to idle
Then message_service.load_messages() contains at least one Narration entry
And every Narration entry has event_header() == None (no trigger fired)
And message_service.load_or_fresh().narrative.input_buffer.status is Idle
```

#### Scenario 1.8: Repeat action against a one-shot trigger does not refire

```gherkin
Given a game state with an NPC "shopkeeper" whose trigger fires when times_met == 0
And the current room's NPC list contains "shopkeeper"
And a quantifier backend that returns {"npcs_in_room": ["shopkeeper"]} for every quantifier call
And a narrator backend that returns valid narration for both main and trigger prompts
When the client POST /action with command="talk to shopkeeper"
And the pipeline returns to idle
And the client POST /action with command="talk to shopkeeper"
And the pipeline returns to idle
Then message_service.load_messages() Narration entries with event_header() == Some(...) count is <= 1 (the trigger fires at most once)
And message_service.load_or_fresh().narrative.input_buffer.status is Idle
```

### Error recovery

#### Scenario 2.1: Action in a non-existent room sets GenerationStatus to Error

```gherkin
Given a game state with movement.current_room_id == "non_existent_room"
And narrative.history empty
When the client POST /action with command="look"
And the pipeline returns to idle
Then message_service.load_or_fresh().narrative.input_buffer.status is GenerationStatus::Error(msg) for some non-empty msg indicating the room is invalid
And message_service.load_messages() contains no new Narration entries
```

#### Scenario 2.2: LLM transport failure sets GenerationStatus to Error

```gherkin
Given a fresh game state with narrative.history empty
And an LLM backend configured to fail (returns an error for the narration request)
When the client POST /action with command="look"
And the pipeline returns to idle
Then message_service.load_or_fresh().narrative.input_buffer.status is GenerationStatus::Error(msg) for some non-empty msg
And message_service.load_or_fresh().narrative.input_buffer.status.error_message() is Some(_)
```

#### Scenario 2.3: Empty LLM response sets Error without persisting an empty narration

```gherkin
Given a fresh game state with narrative.history empty
And an LLM backend that returns an empty string for the narration prompt
When the client POST /action with command="examine the room"
And the pipeline returns to idle
Then message_service.load_or_fresh().narrative.input_buffer.status is GenerationStatus::Error(msg) where msg contains "empty"
And message_service.load_messages() contains zero new Narration entries (no empty narration persisted)
```

#### Scenario 2.4: Trigger narration failure preserves main narration, logs the failure, and sets Error

```gherkin
Given a game state with an NPC "npc_1" whose trigger fires when times_met == 0
And a quantifier backend that returns {"npcs_in_room": ["npc_1"]}
And a narrator backend that succeeds for the main prompt but fails for the trigger prompt
When the client POST /action with command="examine the npc"
And the pipeline returns to idle
Then message_service.load_or_fresh().narrative.input_buffer.status is GenerationStatus::Error(msg) where msg mentions "Trigger narration failed"
And message_service.load_messages() contains at least one Narration entry from the main narration (preserved)
And message_service.load_messages() contains at least one System entry whose text mentions "Trigger narration failed"
```

### Concurrency

#### Scenario 3.3: Delayed LLM response does not deadlock the pipeline

```gherkin
Given a fresh game state
And an LLM backend that takes 200 ms to respond
When the client POST /action with command="look around"
Then the response is not 500 INTERNAL_SERVER_ERROR
And within 2 s, message_service.load_or_fresh().narrative.input_buffer.status.is_generating() is false
```

### Sequencing

#### Scenario 6.1: Three actions in sequence produce three Input and three Narration entries

```gherkin
Given a fresh game state with narrative.history empty
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the pipeline returns to idle
And the client POST /action with command="look around"
And the pipeline returns to idle
And the client POST /action with command="check inventory"
And the pipeline returns to idle
Then message_service.load_messages() contains exactly 3 Input entries
And message_service.load_messages() contains at least 3 Narration entries (one per action)
And each Input entry's text matches the submitted command in order
```

#### Scenario 6.2: Execute → retry → execute produces two inputs and at least two narrations

```gherkin
Given a fresh game state with narrative.history empty
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the pipeline returns to idle
And the client POST /swipe/new (retry)
And the pipeline returns to idle
And the client POST /action with command="look around"
And the pipeline returns to idle
Then message_service.load_messages() contains exactly 2 Input entries
And message_service.load_messages() contains at least 2 Narration entries
```

#### Scenario 6.3: Async action sequence then retry completes with two inputs persisted

```gherkin
Given a fresh game state with narrative.history empty
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="hello"
And the pipeline returns to idle
And the client POST /action with command="examine room"
And the pipeline returns to idle
And the client POST /swipe/new (retry)
And the pipeline returns to idle
Then message_service.load_messages() contains exactly 2 Input entries
```

## Invariants

These properties hold across every `POST /action` and are observable
through HTTP. Drift indicates a regression even if all scenarios pass.

- **I.1** After the pipeline returns to idle, `input_buffer.status` is
  `Idle` or `Error(msg)` — never stuck `Generating`.
- **I.2** User `Input` messages always appear in history at a smaller
  index than their associated `Narration` (Input is persisted first).
- **I.3** No scenario produces more than one main `Narration` message
  per `POST /action` call (trigger continuations are separate
  `Narration` entries; no duplicates from streaming, retry, or
  retrigger).
- **I.4** Failed narration never adds a `Narration` entry to history
  (no empty/broken narrations persisted).
