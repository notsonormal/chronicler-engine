# Feature Spec: Retrigger (POST /retrigger)

Endpoint: `POST /retrigger`

## Scenarios

### Retrigger

#### Scenario 13.1: Retrigger creates a new event narration message and does not roll back state

```gherkin
Given a Game whose current state holds a trigger context
And the last Message is a non-event Narration (the main narration)
And the story log holds N Messages
And a narrator backend that returns a non-empty continuation for the trigger prompt
When the client sends POST /retrigger
And the Game is no longer busy
Then the story log holds N+1 Messages (the original history is preserved; the new event narration is appended, not a replacement)
And the new Message is an event Narration (it carries an event header)
And the generation status is Idle
```

#### Scenario 13.2: Retrigger does not move the player

```gherkin
Given a Game whose current state holds a trigger context
And the last Message is a non-event Narration
And the player stands in "room2"
When the client sends POST /retrigger
And the Game is no longer busy
Then the player still stands in "room2" (a retrigger applies no movement)
```

### Retrigger error cases

#### Scenario 14.1: Retrigger with no trigger context returns 400

```gherkin
Given a Game whose current state holds no trigger context
When the client sends POST /retrigger
Then the response is 400 Bad Request
And no generation is started
```

#### Scenario 14.3: Retrigger when the last message is not a narration returns 400

```gherkin
Given a Game whose last Message is an Input message (not a Narration)
When the client sends POST /retrigger
Then the response is 400 Bad Request
```

#### Scenario 14.4: Retrigger when the last message is an event continuation returns 400

```gherkin
Given a Game whose last Message is an event Narration that a Trigger appended
When the client sends POST /retrigger
Then the response is 400 Bad Request
```

#### Scenario 14.5: Retrigger trigger narration failure sets Error

```gherkin
Given a Game whose current state holds a trigger context
And the last Message is a non-event Narration
And a narrator backend that fails for the trigger continuation prompt
When the client sends POST /retrigger
And the Game is no longer busy
Then the generation status is an error whose text is "The event narration failed."
And the story log gained no new event Narration entry (the failed continuation is not persisted)
And the story log still ends in the non-event Narration the failure was reported on
And the story-log fragment still offers the `.retrigger-btn` control on that Narration
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
Given a Game whose Job is running
When the client sends POST /retrigger
Then the response is 200 with a "Still thinking..." status string
And no event Narration is added
```

---

## Properties

These properties hold across every `POST /retrigger` and are observable
through HTTP. Drift indicates a regression even if all scenarios pass.

- **I.3** Retrigger (`POST /retrigger`) adds exactly one new Message to
  the story log (the event Narration). It does not add a Swipe.
- **I.7** A request whose start state is missing is a data-integrity
  violation: 500 and the failure line "The data for this request is
  missing: …", not a user error (400).
- **I.8** Retrigger rejects concurrent generation the same way an Action
  does: 200 with a `"Still thinking..."` status, no generation started.
- **I.9** A failed event Narration leaves the trigger context of the last
  Narration in place. Retrigger stays available as a recovery path after
  the failure, with no new Action.
