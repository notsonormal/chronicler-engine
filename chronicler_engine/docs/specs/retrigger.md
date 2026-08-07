# Feature Spec: Retrigger (POST /retrigger)

Endpoint: `POST /retrigger`

Behavioural authority for the retrigger endpoint — what a client
observes through HTTP. Each "When" is an HTTP request; each "Then" is
an HTTP-observable outcome asserted via `message_service.load_messages()`
or the generation status exposed through the `/status/generating`
endpoint. Internal-state seams (cancellation timing, `last_trigger`
field, phase transitions, snapshots, call sequencing) are not asserted
here — they live at the unit and driven-adapter tiers.

Retrigger re-fires the trigger continuation from the current state.
The endpoint returns immediately with a status string; the generation
runs in a background task. The client polls or refreshes to observe the
outcome.

Scenario IDs are `13.x` through `15.x` (continuing the cross-spec
sequence owned by `actions.md` 1.x–6.x, `reset.md` 7.x, `story_log.md`
8.x, `swipe_new.md` 9.x–12.x). The pilot dedups by ID across
`docs/specs/*.md`, so IDs must stay unique.

## Scenarios

### Retrigger

#### Scenario 13.1: Retrigger creates a new event narration message and does not roll back state

```gherkin
Given a game state where narrative.last_trigger is set (a trigger context exists)
And the last message is a non-event Narration (the main narration)
And narrative.history contains N messages
And a narrator backend that returns a non-empty continuation for the trigger prompt
When the client sends POST /retrigger
And the pipeline returns to idle
Then message_service.load_messages() contains N+1 messages (the original history is preserved; the new event narration is appended, not a replacement)
And the new message is a Narration entry whose event_header is Some(...)
And message_service.load_or_fresh().narrative.input_buffer.status is Idle
```

#### Scenario 13.2: Retrigger does not re-run the quantifier

```gherkin
Given a game state where narrative.last_trigger is set
And the last message is a non-event Narration
And movement.current_room_id == "room2"
When the client sends POST /retrigger
And the pipeline returns to idle
Then message_service.load_or_fresh().movement.current_room_id is still "room2" (no quantifier re-run)
```

### Retrigger error cases

#### Scenario 14.1: Retrigger with no trigger context returns 400

```gherkin
Given a game state where narrative.last_trigger is None
When the client sends POST /retrigger
Then the response is 400 Bad Request
And no generation is started
```

#### Scenario 14.2: Retrigger with no messages returns 400

```gherkin
Given a game state with no messages in history
When the client sends POST /retrigger
Then the response is 400 Bad Request
```

#### Scenario 14.3: Retrigger when the last message is not a narration returns 400

```gherkin
Given a game state where narrative.last_trigger is set
And the last message is an Input message (not a Narration or Dialogue)
When the client sends POST /retrigger
Then the response is 400 Bad Request
```

#### Scenario 14.4: Retrigger when the last message is an event continuation returns 400

```gherkin
Given a game state where narrative.last_trigger is set
And the last message is an event Narration (has event_header)
When the client sends POST /retrigger
Then the response is 400 Bad Request
```

#### Scenario 14.5: Retrigger trigger narration failure sets Error

```gherkin
Given a game state where narrative.last_trigger is set
And the last message is a non-event Narration
And a narrator backend that fails for the trigger continuation prompt
When the client sends POST /retrigger
And the pipeline returns to idle
Then message_service.load_or_fresh().narrative.input_buffer.status is GenerationStatus::Error(msg) where msg contains "Trigger narration failed"
And message_service.load_messages() contains no new event Narration entry (the failed continuation is not persisted)
And message_service.load_messages() contains a System entry whose text mentions "Trigger narration failed"
```

#### Scenario 14.6: Retrigger with no game context returns 400

```gherkin
Given no game exists in storage
When the client sends POST /retrigger
Then the response is 400 Bad Request
```

### Concurrency

#### Scenario 15.1: Retrigger while a generation is already in flight returns "Still thinking..."

```gherkin
Given a game state where a generation is already in flight (the generation gate is busy)
When the client sends POST /retrigger
Then the response is 200 with a "Still thinking..." status string
And no retrigger is started (the gate is not claimed)
```

---

## Invariants

These properties hold across every `POST /retrigger` and are observable
through HTTP. Drift indicates a regression even if all scenarios pass.

- **I.3** Retrigger (`POST /retrigger`) adds exactly one new message to
  `narrative.history` (the event narration). It does not append a swipe.
- **I.7** If a message exists in history, it must have a `snapshot_id`
  pointing to a restorable snapshot. A message without a snapshot (or with
  a dangling snapshot id) is a data-integrity violation (500), not a user
  error (400).
- **I.8** Retrigger rejects concurrent generation the same way
  `process_action` does: 200 with a `"Still thinking..."` status, no
  generation started.
- **I.9** `last_trigger` is set before the trigger LLM call, so it
  survives a trigger-narration failure. Retrigger is available as a
  recovery path after trigger failure without re-running the full
  pipeline.
