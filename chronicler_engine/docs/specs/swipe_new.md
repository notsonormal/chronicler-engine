# Feature Spec: Retry (POST /swipe/new)

Endpoint: `POST /swipe/new`

Behavioural authority for the retry endpoint — what a client observes
through HTTP. Each "When" is an HTTP request; each "Then" is an
HTTP-observable outcome asserted via `message_service.load_messages()`
(the same data `/fragment/story-log` renders) or the generation status
exposed through the `/status/generating` endpoint. Internal-state seams
(cancellation timing, mid-flight streaming flags, `last_trigger` field,
phase transitions, snapshots, call sequencing) are not asserted here —
they live at the unit and driven-adapter tiers.

Retry auto-detects main retry or event retry from the last message. The
endpoint returns immediately with a status string; the generation runs
in a background task. The client polls or refreshes to observe the
outcome.

Scenario IDs are `9.x` through `12.x` (continuing the cross-spec
sequence owned by `actions.md` 1.x–6.x, `reset.md` 7.x, `story_log.md`
8.x). The pilot dedups by ID across `docs/specs/*.md`, so IDs must stay
unique.

## Scenarios

### Main retry

#### Scenario 9.1: Main retry replaces the narration with a new swipe

```gherkin
Given a fresh game state with one non-event Narration in history
And a narrator backend that returns "First." then "Second." for successive narration prompts
When the client sends POST /action with command="look"
And the pipeline returns to idle
And the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_messages() contains exactly one Narration entry (the original message ID unchanged)
And that Narration entry has 2 swipes (one appended by retry)
And the active swipe's text is "Second."
And message_service.load_or_fresh().narrative.input_buffer.status is Idle
```

#### Scenario 9.2: Main retry re-runs the quantifier and can move the player

```gherkin
Given a fresh game state with movement.current_room_id == "room1"
And a quantifier backend that returns no movement for the first action, then movement to "room2" on the retry prompt
When the client sends POST /action with command="walk around"
And the pipeline returns to idle
And the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_or_fresh().movement.current_room_id is "room2" (the quantifier re-ran with a potentially different result)
```

#### Scenario 9.3: Main retry preserves the input message

```gherkin
Given a fresh game state with an Input message whose text is "walk around"
And a non-event Narration as the last message
When the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_messages() contains an Input entry whose text is still "walk around" (unchanged by retry)
```

#### Scenario 9.4: Main retry uses the edited input text

```gherkin
Given a fresh game state with an Input message whose active swipe has been edited to "sprint forward"
And a non-event Narration as the last message
When the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_messages() contains a Narration entry whose text reflects "sprint forward", not the original input text
```

#### Scenario 9.5: Main retry re-evaluates triggers

```gherkin
Given a game state with an NPC whose trigger fires when the player is in "room2"
And a quantifier backend that returns no movement on the first action, then movement to "room2" on the retry prompt
When the client sends POST /action (no trigger fires — player not in "room2")
And the pipeline returns to idle
And the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_messages() contains at least one Narration entry whose event_header is Some(...) (the trigger fired after the retry moved the player)
```

#### Scenario 9.6: Main retry completes when the quantifier returns no movement

```gherkin
Given a fresh game state
And a quantifier backend that returns no movement for both the action and the retry prompt
When the client sends POST /action with command="walk around"
And the pipeline returns to idle
And the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_or_fresh().narrative.input_buffer.status is Idle (retry completes even with no movement)
```

### Event retry

#### Scenario 10.1: Event retry replaces the event narration with a new swipe

```gherkin
Given a game state where the last message is an event Narration (has event_header)
And a narrator backend that returns a different continuation text for the retry prompt
When the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_messages() contains exactly 2 Narration entries (the main narration and the event narration)
And the event narration message has its new swipe as the active swipe
And message_service.load_or_fresh().narrative.input_buffer.status is Idle
```

#### Scenario 10.2: Event retry does not re-run the quantifier

```gherkin
Given a game state where the last message is an event Narration
And movement.current_room_id is any valid room
When the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_or_fresh().movement.current_room_id is still "room2" (the quantifier does not re-run; world state is frozen at the pre-event snapshot)
```

### Retry error cases

#### Scenario 11.1: Retry with no input to retry returns 400

```gherkin
Given a fresh game state with no Input message in history
When the client sends POST /swipe/new
Then the response is 400 Bad Request
And no generation is started
```

#### Scenario 11.2: Retry with no game context returns 400

```gherkin
Given no game exists in storage
When the client sends POST /swipe/new
Then the response is 400 Bad Request
```

#### Scenario 11.3: Retry when the anchor message has no snapshot returns 500

```gherkin
Given a game state where the last Input message exists
And that message's snapshot_id is None (no snapshot was saved alongside it)
When the client sends POST /swipe/new
Then the response is 500 Internal Server Error
And message_service.load_or_fresh().narrative.input_buffer.status is GenerationStatus::Error(msg) indicating the snapshot is missing
```

#### Scenario 11.4: Retry when the anchor snapshot was deleted returns 500

```gherkin
Given a game state where the last Input message has a snapshot_id
And that snapshot row has been removed from the database
When the client sends POST /swipe/new
Then the response is 500 Internal Server Error
And message_service.load_or_fresh().narrative.input_buffer.status is GenerationStatus::Error(msg) indicating the snapshot was not found
```

#### Scenario 11.5: Retry LLM failure sets Error

```gherkin
Given a game state where the last message is a non-event Narration
And a narrator backend that fails for the narration request
When the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_or_fresh().narrative.input_buffer.status is GenerationStatus::Error(msg)
And message_service.load_or_fresh().narrative.input_buffer.status.is_generating() is false
And the original narration's swipes are unchanged (no new swipe appended)
```

#### Scenario 11.6: Retry empty narration sets Error

```gherkin
Given a game state where the last message is a non-event Narration
And a narrator backend that returns an empty string for the narration prompt
When the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_or_fresh().narrative.input_buffer.status is GenerationStatus::Error(msg) where msg contains "empty"
And the original narration's swipes are unchanged (no new swipe appended)
```

#### Scenario 11.7: Retry room not found sets Error

```gherkin
Given a game state where movement.current_room_id refers to a non-existent room
And the last message is a non-event Narration
When the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_or_fresh().narrative.input_buffer.status is GenerationStatus::Error(msg) where msg indicates the room is invalid
And the original narration's swipes are unchanged (no new swipe appended)
```

#### Scenario 11.8: Event retry trigger narration failure sets Error and preserves main narration

```gherkin
Given a game state where the last message is an event Narration
And a narrator backend that fails only for the trigger continuation prompt (main narration succeeds)
When the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_or_fresh().narrative.input_buffer.status is GenerationStatus::Error(msg) where msg contains "Trigger narration failed"
And message_service.load_messages() contains the main Narration entry (preserved, not removed)
And message_service.load_messages() contains a System entry whose text mentions "Trigger narration failed"
```

### Concurrency

#### Scenario 12.1: Retry while a generation is already in flight returns "Still thinking..."

```gherkin
Given a game state where a generation is already in flight (the generation gate is busy)
When the client sends POST /swipe/new
Then the response is 200 with a "Still thinking..." status string
And no retry is started (the gate is not claimed)
```

---

## Invariants

These properties hold across every `POST /swipe/new` and are observable
through HTTP. Drift indicates a regression even if all scenarios pass.

- **I.1** After the pipeline returns to idle, `input_buffer.status` is
  `Idle` or `Error(msg)` — never stuck `Generating`.
- **I.2** Retry (`POST /swipe/new`) never adds a new message to
  `narrative.history`. It appends a swipe to an existing message.
- **I.4** Retry never modifies an existing Input message's text. (Main
  retry uses the input's current active swipe text, which the user may
  have edited before clicking retry, but the retry itself does not
  change it.)
- **I.5** Each retry appends exactly one swipe. Repeated retries on the
  same message increment the swipe count by one each time.
- **I.6** Main retry re-runs the quantifier; event retry does not.
- **I.7** If a message exists in history, it must have a `snapshot_id`
  pointing to a restorable snapshot. A message without a snapshot (or with
  a dangling snapshot id) is a data-integrity violation (500), not a user
  error (400).
- **I.8** Retry rejects concurrent generation the same way `process_action`
  does: 200 with a `"Still thinking..."` status, no generation started.
