# Feature Spec: Retrigger (POST /retrigger)

Endpoint: `POST /retrigger`, with the form field `message`: the id of the Message that the page rendered the control on

## Scenarios

### Retrigger

#### Scenario 13.1: Retrigger creates a new event narration message and does not roll back state

```gherkin
Given a Game whose current state holds an event record
And the last Message is a non-event Narration (the main narration)
And the story log holds N Messages
And a narrator backend that returns a non-empty continuation for the trigger prompt
When the client sends POST /retrigger
Then the response is 200 with an empty body
And the Game is no longer busy
And the story log holds N+1 Messages (the original history is preserved; the new event narration is appended, not a replacement)
And the new Message is an event Narration (it carries an event header)
And the generation status is Idle
```

#### Scenario 13.2: Retrigger does not move the player

```gherkin
Given a Game whose current state holds an event record
And the last Message is a non-event Narration
And the player stands in "room2"
When the client sends POST /retrigger
And the Game is no longer busy
Then the player still stands in "room2" (a retrigger applies no movement)
```

#### Scenario 13.3: Retrigger sends the rendered direction to the model

```gherkin
Given a Game whose current state holds an event record
And the last Message is a non-event Narration
And the Trigger's direction names the player with the placeholder {{user}}
When the client sends POST /retrigger
And the Game is no longer busy
Then the event narration call carries the Trigger's direction with the Persona's name in place of {{user}}
And the call carries no unreplaced {{user}}
```

#### Scenario 13.4: Retrigger on an Impersonate Input creates a new event narration message

```gherkin
Given a Game whose last Message is an Impersonate Input
And the state of that Input's Swipe holds an event record
And a narrator backend that returns a non-empty continuation for the trigger prompt
When the client sends POST /retrigger
And the Game is no longer busy
Then the story log gained one Message, an event Narration
And the generation status is Idle
```

### Retrigger error cases

#### Scenario 14.1: Retrigger with no event record returns 400

```gherkin
Given a Game whose current state holds no event record
When the client sends POST /retrigger
Then the response is 400 Bad Request
And no generation is started
```

#### Scenario 14.3: Retrigger when the last message is a typed Input returns 400

```gherkin
Given a Game whose last Message is a typed Input
And the state of that Input's Swipe holds an event record
When the client sends POST /retrigger
Then the response is 400 Bad Request
And no generation is started
And the story-log fragment offers no `.retrigger-btn` control
```

#### Scenario 14.4: Retrigger when the last message is an event continuation returns 400

```gherkin
Given a Game whose last Message is an event Narration that a Trigger appended
When the client sends POST /retrigger
Then the response is 400 Bad Request
And the story-log fragment offers no `.retrigger-btn` control
```

#### Scenario 14.5: Retrigger trigger narration failure sets Failed

```gherkin
Given a Game whose current state holds an event record
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

#### Scenario 15.1: Retrigger while a generation is already in flight answers 409 "Still thinking..."

```gherkin
Given a Game whose Job is running
When the client sends POST /retrigger
Then the response is 409 with the "Still thinking..." line
And no event Narration is added
```

#### Scenario 15.2: A Retrigger that names a Message that is not the last one is refused

```gherkin
Given a Game whose story holds an Input and the Narration that answers it
When the client sends POST /retrigger with the field message set to the id of the Input
Then the response is 409
And the first line of the body is "The last Message changed, so the retrigger was not started."
And the second line of the body is "The page showed a Message that is no longer the last one."
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
  does: 409 with the `"Still thinking..."` line, no generation started.
- **I.9** A failed event Narration leaves the event record of the last
  Narration in place. Retrigger stays available as a recovery path after
  the failure, with no new Action.
