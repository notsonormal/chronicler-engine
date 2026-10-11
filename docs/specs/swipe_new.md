# Feature Spec: Retry (POST /swipe/new)

Endpoint: `POST /swipe/new`

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
And the Action that fired the event moved the player into "room2"
When the client sends POST /swipe/new
And the pipeline returns to idle
Then the player is still in "room2" (the Quantifier does not re-run)
```

#### Scenario 10.4: An event retry after a Retrigger adds a swipe to the retriggered event

```gherkin
Given a game state whose last message is a non-event Narration
And a stored trigger context whose continuation narration failed
When the client sends POST /retrigger
And the pipeline returns to idle (the event narration is appended)
And the client sends POST /swipe/new
And the pipeline returns to idle
Then the retriggered event Narration has 2 swipes (the retriggered one and the retry's)
And its new swipe is the active swipe
And the generation status is Idle
```

#### Scenario 10.5: An event retry starts from the Snapshot of the Swipe it retries

```gherkin
Given a game state whose event Narration meets Character Gabriella in its first Swipe
And a Quantifier backend that leaves Gabriella out of the retry's continuation
When the client sends POST /swipe/new
And the pipeline returns to idle
Then Gabriella's encounter count is 0 (the first Swipe's encounter does not carry over)
And the scene holds no Character
And the event Narration has 2 swipes
```

#### Scenario 10.6: An event retry after the later Action's Messages are deleted reuses its own trigger context

```gherkin
Given a game state whose event Narration carries the trigger context of the Trigger that fired it
And a later Action that fires another Trigger and appends its own event Narration
And the client deletes that later Action's Messages
When the client sends POST /swipe/new
And the pipeline returns to idle
Then the event Narration has 2 swipes
And its new swipe's event header names the first event's own Trigger
And the generation status is Idle
```

#### Scenario 10.7: A prior swipe switch does not change what an event retry starts from

```gherkin
Given a game state whose event Narration meets Character Gabriella in its first Swipe
And a Quantifier backend that leaves Gabriella out of the retry's continuation
And the client re-selects that Message's first Swipe
When the client sends POST /swipe/new
And the pipeline returns to idle
Then Gabriella's encounter count is 0, as it is without the switch
```

#### Scenario 10.8: An event retry keeps the Swipe's room name

```gherkin
Given a game state whose event Narration was written after the movement into "room2"
And that event Narration's Swipe reports the room "Room room2"
When the client sends POST /swipe/new
And the pipeline returns to idle
Then the event Narration has 2 swipes
And its new active swipe reports the room "Room room2"
And the first swipe still reports the room "Room room2"
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
And the Action that fired the event moved the player into "room2" and marked the trigger fired
And a narrator backend that fails only for the trigger continuation prompt (main narration succeeds)
When the client sends POST /swipe/new
And the pipeline returns to idle
Then the generation status is an error whose text mentions "Trigger narration failed"
And the message history still contains the main Narration entry (preserved, not removed)
And the message history gained no entry for the failure
And the player is still in "room2"
And the event's trigger is still marked as fired
```

#### Scenario 11.9: A failed retry does not roll the movement back

```gherkin
Given a game state whose last Action moved the player into "room2"
And the Snapshot that the retry restores has the player in "room_1"
And a narrator backend that fails the narration request
When the client sends POST /swipe/new
And the pipeline returns to idle
Then the generation status is an error
And the player is still in "room2"
```

#### Scenario 11.10: A failed retry that wrote its narration keeps the world it wrote

```gherkin
Given a game state whose last Action left the player in "room_1"
And a trigger in "room2" that fires when the player enters it
And a quantifier backend that moves the player into "room2" on the retry
And a narrator backend that fails only for the trigger continuation prompt (the narration succeeds)
When the client sends POST /swipe/new
And the pipeline returns to idle
Then the generation status is an error whose text mentions "Trigger narration failed"
And the player is in "room2" (the world the retry's narration was written from)
```

### Concurrency

#### Scenario 12.1: Retry while a generation is already in flight returns "Still thinking..."

```gherkin
Given a game state where a generation is already in flight (the generation gate is busy)
When the client sends POST /swipe/new
Then the response is 200 with a "Still thinking..." status string
And no retry is started (the gate is not claimed)
```

### Three-way retry

#### Scenario 22.1: Re-impersonate retry generates a new Input swipe

```gherkin
Given a game state whose last message is an Input whose active swipe is an impersonation with the steering instruction "I look around."
And a narrator backend that returns "I look around cautiously." for the impersonation prompt
When the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_messages() contains exactly one Input entry (the original message ID unchanged)
And that Input entry has 2 swipes (one appended by retry)
And the active swipe's text is "I look around cautiously."
And message_service.load_or_fresh().narrative.input_buffer.status is Idle
And the player has not moved (the quantifier did not re-run)
```

#### Scenario 22.2: User-regen retry generates a new Input swipe

```gherkin
Given a game state whose last message is an Input whose active swipe has no stored inputs
And a narrator backend that returns "I sprint forward." for the user-regen prompt
When the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_messages() contains exactly one Input entry (the original message ID unchanged)
And that Input entry has 2 swipes (one appended by retry)
And the active swipe's text is "I sprint forward."
And message_service.load_or_fresh().narrative.input_buffer.status is Idle
And the player has not moved (the quantifier did not re-run)
```

#### Scenario 22.3: Re-impersonate retry preserves the stored inputs

```gherkin
Given a game state whose last message is an Input whose active swipe is an impersonation
And the active swipe's steering instruction is "Sneak past the guard."
When the client sends POST /swipe/new
And the pipeline returns to idle
Then message_service.load_messages() contains an Input entry whose active swipe is an impersonation
And the active swipe's steering instruction is "Sneak past the guard."
```

#### Scenario 22.4: Re-impersonate redo uses the impersonate preset's current content

```gherkin
Given a game whose impersonate preset's instructions changed after the original generation
And a game state whose last message is an Input whose active swipe is an impersonation
And a narrator backend that returns "I look around cautiously." for the impersonation prompt
When the client sends POST /swipe/new
And the pipeline returns to idle
Then a new swipe is appended and its stored inputs re-apply (impersonated with the steering instruction)
And the narrator generation's recorded prompt contains the preset's current instructions
```

---

## Properties

These properties hold across every `POST /swipe/new` and are observable
through HTTP. Drift indicates a regression even if all scenarios pass.

- **I.1** After the pipeline returns to idle, `input_buffer.status` is
  `Idle` or `Error(msg)` — never stuck `Generating`.
- **I.2** Retry (`POST /swipe/new`) never adds a new message to
  `narrative.history`. It appends a swipe to an existing message. Re-impersonate
  and user-regen retries append the new swipe to the existing Input message.
- **I.4** Retry never modifies the text of any prior swipe. Main
  retry uses the input's current active swipe text, which the user may
  have edited before clicking retry, but the retry itself does not
  change it. Re-impersonate and user-regen retries generate a new
  swipe without altering prior swipes.
- **I.5** Each retry appends exactly one swipe. Repeated retries on the
  same message increment the swipe count by one each time.
- **I.6** Main retry re-runs the quantifier; event retry does not.
  Re-impersonate and user-regen retries do not re-run the quantifier.
  They generate one swipe and stop.
- **I.7** If a message exists in history, it must have a `snapshot_id`
  pointing to a restorable snapshot. A message without a snapshot (or with
  a dangling snapshot id) is a data-integrity violation (500), not a user
  error (400).
- **I.8** Retry rejects concurrent generation the same way `process_action`
  does: 200 with a `"Still thinking..."` status, no generation started.
- **I.9** A failed retry that appended no Swipe leaves the room, the
  encounters and the fired triggers as the click found them. A failed retry
  that appended a Swipe keeps the game state that Swipe was written from.
