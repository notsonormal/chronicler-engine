# Feature Spec: Retry (POST /swipe/new)

Endpoint: `POST /swipe/new`

## Scenarios

### Main retry

#### Scenario 9.1: Main retry replaces the narration with a new swipe

```gherkin
Given a Game whose story log holds only its opening Narration
And a narrator backend that returns "First." then "Second." for successive narration prompts
When the client sends POST /action with command="look"
And the Game is no longer busy
And the client sends POST /swipe/new
Then the response is 200 with an empty body
And the Game is no longer busy
And the story log holds 2 Narration entries (its opening Narration and the Action's)
And the Action's Narration entry keeps its message ID and has 2 swipes (one appended by the retry)
And the active swipe's text is "Second."
And the generation status is Idle
```

#### Scenario 9.2: Main retry re-runs the quantifier and can move the player

```gherkin
Given a fresh game state whose player stands in "room_1"
And a quantifier backend that returns no movement for the first action, then movement to "room2" on the retry prompt
When the client sends POST /action with command="walk around"
And the Game is no longer busy
And the client sends POST /swipe/new
And the Game is no longer busy
Then the player stands in "room2" (the Quantifier re-ran on the redo)
And the story log shows the room "Room room2" on the Narration entry
When the client switches to its first swipe
Then the story log shows no room on the Narration entry (that generation did not move)
```

#### Scenario 9.3: Main retry preserves the input message

```gherkin
Given a fresh game state with an Input message whose text is "walk around"
And a non-event Narration as the last message
When the client sends POST /swipe/new
And the Game is no longer busy
Then the story log contains an Input entry whose text is still "walk around" (unchanged by retry)
```

#### Scenario 9.4: Main retry uses the edited input text

```gherkin
Given a fresh game state with an Input message whose active swipe has been edited to "sprint forward"
And a non-event Narration as the last message
When the client sends POST /swipe/new
And the Game is no longer busy
Then the story log contains a Narration entry whose text reflects "sprint forward", not the original input text
```

#### Scenario 9.5: Main retry re-evaluates triggers

```gherkin
Given a game state with a Character whose trigger fires when the player is in "room2"
And a quantifier backend that returns no movement on the first action, then movement to "room2" on the retry prompt
When the client sends POST /action (no trigger fires — player not in "room2")
And the Game is no longer busy
And the client sends POST /swipe/new
And the Game is no longer busy
Then the story log contains at least one Narration entry that carries an event header (the trigger fired after the retry moved the player)
```

#### Scenario 9.6: Main retry completes when the quantifier returns no movement

```gherkin
Given a fresh game state
And a quantifier backend that returns no movement for both the action and the retry prompt
When the client sends POST /action with command="walk around"
And the Game is no longer busy
And the client sends POST /swipe/new
And the Game is no longer busy
Then the generation status is Idle (retry completes even with no movement)
```

#### Scenario 9.8: A main retry whose redo stays put shows no room

```gherkin
Given a game state whose first Action moved the player into "room2"
And the story log shows the room "Room room2" on the Narration entry
And a quantifier backend that returns no movement for the retry prompt
When the client sends POST /swipe/new
And the Game is no longer busy
Then the Narration entry has 2 swipes
And the story log shows no room on the Narration entry (the redo must not keep its sibling's room)
When the client switches to its first swipe
Then the story log shows the room "Room room2" on the Narration entry
```

A switch to another Swipe gives back the state of that Swipe; `swipe_switch.md` 36.4 covers it.

### Event retry

#### Scenario 10.1: Event retry replaces the event narration with a new swipe

```gherkin
Given a game state where the last message is an event Narration (it carries an event header)
And a narrator backend that returns a different continuation text for the retry prompt
When the client sends POST /swipe/new
And the Game is no longer busy
Then the story log holds 3 Narration entries (its opening Narration, the main narration and the event narration)
And the event narration message has its new swipe as the active swipe
And the generation status is Idle
```

#### Scenario 10.2: Event retry does not move the player

```gherkin
Given a game state where the last message is an event Narration
And the Action that fired the event moved the player into "room2"
When the client sends POST /swipe/new
And the Game is no longer busy
Then the player is still in "room2" (an event retry applies no movement)
```

#### Scenario 10.4: An event retry after a Retrigger adds a swipe to the retriggered event

```gherkin
Given a game state whose last message is a non-event Narration
And a stored event record whose continuation narration failed
When the client sends POST /retrigger
And the Game is no longer busy (the event narration is appended)
And the client sends POST /swipe/new
And the Game is no longer busy
Then the retriggered event Narration has 2 swipes (the retriggered one and the retry's)
And its new swipe is the active swipe
And the generation status is Idle
```

#### Scenario 10.5: An event retry starts from the state of the Swipe it retries

```gherkin
Given a game state whose event Narration meets Character Gabriella in its first Swipe
And a Quantifier backend that leaves Gabriella out of the retry's continuation
When the client sends POST /swipe/new
And the Game is no longer busy
Then Gabriella's encounter count is 0 (the first Swipe's encounter does not carry over)
And no Character is present
And the event Narration has 2 swipes
```

#### Scenario 10.6: An event retry after the later Action's Messages are deleted reuses its own event record

```gherkin
Given a game state whose event Narration carries the event record of the Trigger that fired it
And a later Action that fires another Trigger and appends its own event Narration
And the client deletes that later Action's Messages
When the client sends POST /swipe/new
And the Game is no longer busy
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
And the Game is no longer busy
Then Gabriella's encounter count is 0, as it is without the switch
```

#### Scenario 10.8: An event retry keeps the room name off the event Swipe

```gherkin
Given a game state whose event Narration follows the movement into "room2"
And the story log shows the room "Room room2" on the Narration that moved the player
And the story log shows no room on that event Narration
When the client sends POST /swipe/new
And the Game is no longer busy
Then the event Narration has 2 swipes
And the story log shows no room on the event Narration
When the client switches to its first swipe
Then the story log shows no room on the event Narration
```

### Retry error cases

#### Scenario 11.2: Retry with no game context returns 400

```gherkin
Given no game exists in storage
When the client sends POST /swipe/new
Then the response is 400 Bad Request
```

#### Scenario 11.5: Retry LLM failure sets Failed

```gherkin
Given a game state where the last message is a non-event Narration
And a narrator backend that fails for the narration request
When the client sends POST /swipe/new
And the Game is no longer busy
Then the generation status is Failed
And the original narration's swipes are unchanged (no new swipe appended)
```

#### Scenario 11.6: Retry empty narration sets Failed

```gherkin
Given a game state where the last message is a non-event Narration
And a narrator backend that returns an empty string for the narration prompt
When the client sends POST /swipe/new
And the Game is no longer busy
Then the generation status is Failed, with a failure that contains "empty"
And the original narration's swipes are unchanged (no new swipe appended)
```

#### Scenario 11.7: Retry room not found sets Failed

```gherkin
Given a Game whose state before its last Narration names a room that is not on the map
And the last Message is a non-event Narration
When the client sends POST /swipe/new
And the Game is no longer busy
Then the generation status is Failed, with a failure that names the invalid room
And the original narration's swipes are unchanged (no new swipe appended)
```

#### Scenario 11.8: Event retry trigger narration failure sets Failed and preserves main narration

```gherkin
Given a game state where the last message is an event Narration
And the Action that fired the event moved the player into "room2" and marked the trigger fired
And a narrator backend that fails only for the trigger continuation prompt (main narration succeeds)
When the client sends POST /swipe/new
And the Game is no longer busy
Then the generation status is an error whose text is "The event narration failed."
And the story log still contains the main Narration entry (preserved, not removed)
And the story log gained no entry for the failure
And the player is still in "room2"
And the event's trigger is still marked as fired
```

#### Scenario 11.9: A failed retry does not roll the movement back

```gherkin
Given a game state whose last Action moved the player into "room2"
And the state that the retry starts from has the player in "room_1"
And a narrator backend that fails the narration request
When the client sends POST /swipe/new
And the Game is no longer busy
Then the generation status is an error
And the player is still in "room2"
```

#### Scenario 11.10: A failed retry that wrote its narration keeps the state its analysis saved

```gherkin
Given a game state whose last Action left the player in "room_1"
And a trigger in "room2" that fires when the player enters it
And a quantifier backend that moves the player into "room2" on the retry
And a narrator backend that fails only for the trigger continuation prompt (the narration succeeds)
When the client sends POST /swipe/new
And the Game is no longer busy
Then the generation status is an error whose text is "The event narration failed."
And the player is in "room2" (the state that the retry's analysis saved)
```

### Concurrency

#### Scenario 12.1: Retry while a generation is already in flight returns "Still thinking..."

```gherkin
Given a Game whose Job is running
When the client sends POST /swipe/new
Then the response is 200 with a "Still thinking..." status string
And no Swipe is added
```

### Three-way retry

#### Scenario 22.1: Re-impersonate retry runs the analysis over a new Input swipe

```gherkin
Given a game state whose last message is an Input whose active swipe is an impersonation with the impersonate direction "I look around."
And a narrator backend that returns "I look around cautiously." for the impersonation prompt
And a quantifier backend that moves the player into "room2" on the redo's analysis
When the client sends POST /swipe/new
And the Game is no longer busy
Then the story log contains exactly one Input entry (the original message ID unchanged)
And that Input entry has 2 swipes (one appended by retry)
And the active swipe's text is "I look around cautiously."
And the generation status is Idle
And the player is in "room2" (a re-impersonate runs the analysis)
```

#### Scenario 22.2: Retry of a typed Input generates a new Input swipe

```gherkin
Given a game state whose last message is an Input whose active swipe has no stored inputs
And a narrator backend that returns "I sprint forward." for the prompt of the Retry of a typed Input
When the client sends POST /swipe/new
And the Game is no longer busy
Then the story log contains exactly one Input entry (the original message ID unchanged)
And that Input entry has 2 swipes (one appended by retry)
And the active swipe's text is "I sprint forward."
And the generation status is Idle
And the player has not moved (the quantifier did not re-run)
```

#### Scenario 22.3: Re-impersonate retry preserves the stored inputs

```gherkin
Given a game state whose last message is an Input whose active swipe is an impersonation
And the active swipe's impersonate direction is "Sneak past the guard."
When the client sends POST /swipe/new
And the Game is no longer busy
Then the story log contains an Input entry whose active swipe is an impersonation
And the active swipe's impersonate direction is "Sneak past the guard."
```

#### Scenario 22.4: Re-impersonate redo uses the impersonate preset's current content

```gherkin
Given a game whose impersonate preset's instructions changed after the original generation
And a game state whose last message is an Input whose active swipe is an impersonation
And a narrator backend that returns "I look around cautiously." for the impersonation prompt
When the client sends POST /swipe/new
And the Game is no longer busy
Then a new swipe is appended and its stored inputs re-apply (impersonated with the impersonate direction)
And the narrator generation's recorded prompt contains the preset's current instructions
```

#### Scenario 22.5: A guided retry shows the room its redo arrives in

```gherkin
Given a game state whose Action before a guided Narration moved the player into "room2"
And a quantifier backend that reports no movement for the guided generation and movement back to "room_1" for the redo
When the client sends POST /action with command="/guide look for the key"
And the Game is no longer busy
Then the story log shows no room on the guided Narration (it stayed put)
When the client sends POST /swipe/new
And the Game is no longer busy
Then the guided Narration has 2 swipes
And the story log shows the room "Room room_1" on the guided Narration
When the client switches to its first swipe
Then the story log shows no room on the guided Narration
```

#### Scenario 22.6: Retry of a bare Impersonate Input runs

```gherkin
Given a game state whose last message is an Input whose active swipe is an impersonation with no impersonate direction
And a narrator backend that returns "I greet the room." for the impersonation prompt
When the client sends POST /swipe/new
And the Game is no longer busy
Then the story log contains exactly one Input entry (the original message ID unchanged)
And that Input entry has 2 swipes
And the active swipe's text is "I greet the room."
And the active swipe is an impersonation with no impersonate direction
And the generation status is Idle
```

---

## Properties

These properties hold across every `POST /swipe/new` and are observable
through HTTP. Drift indicates a regression even if all scenarios pass.

- **I.1** A Job always ends: the generation status then reads Idle or an
  error, never a step.
- **I.2** Retry (`POST /swipe/new`) never adds a new message to
  the story log. It appends a swipe to an existing message. Re-impersonate
  and Retry of a typed Input append the new swipe to the existing Input message.
- **I.4** Retry never modifies the text of any prior swipe. Main
  retry uses the input's current active swipe text, which the user may
  have edited before clicking retry, but the retry itself does not
  change it. Re-impersonate and Retry of a typed Input generate a new
  swipe without altering prior swipes.
- **I.5** Each retry appends exactly one swipe. Repeated retries on the
  same message increment the swipe count by one each time.
- **I.6** A retry moves the player nowhere except through the movement its
  own generation reports. A main retry and a re-impersonate retry apply the
  movement their generation reports. An event retry applies no movement, so
  the player stays in the room that event narration reported. Retry of a typed
  Input narrates the replacement message only.
- **I.7** A request whose start state is missing is a data-integrity
  violation: 500 and the failure line "The data for this request is
  missing: …", not a user error (400).
- **I.8** Retry rejects concurrent generation the same way `POST /action`
  does: 200 with a `"Still thinking..."` status, no generation started.
- **I.9** A failed retry that appended no Swipe leaves the room, the
  encounters and the fired triggers as the click found them. A failed retry
  that appended a Swipe keeps the state that its analysis saved.
