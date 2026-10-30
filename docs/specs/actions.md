# Feature Spec: Actions

Endpoints:
- `POST /action`
- `GET /screen?parts=llm-messages` (the LLM call records)

## Scenarios

### Normal flow

#### Scenario 1.1: Successful action produces exactly one narration and returns to Idle

```gherkin
Given a Game whose story log holds only its opening Narration
And an LLM backend that returns "You see a small wooden room." for any narration prompt
When the client POST /action with command="look"
Then the response is 200 with an empty body
And the Game is no longer busy (status not generating)
And the story log gained exactly 2 messages:
  - one player-input message whose text is "look"
  - one Narration message whose text is "You see a small wooden room."
And the generation status is Idle
```

#### Scenario 1.2: User input is persisted in history before the narration message

```gherkin
Given a Game whose story log holds only its opening Narration
And an LLM backend that returns "It is a small wooden room." for the narration prompt
When the client POST /action with command="examine the room"
And the Game is no longer busy
Then the story log gained 2 messages
And the player-input message appears in the story log before the Narration message
And the player-input message's text is "examine the room"
And the Narration message's text is "It is a small wooden room."
And the generation status is Idle
```

#### Scenario 1.4: Action whose quantifier detects a Character fires the trigger

```gherkin
Given a game state with a Character "character_1" whose Trigger fires on the first meeting
And the current room's Characters include "character_1"
And a quantifier backend that responds with {"characters_present": ["character_1"]} for the quantifier prompt
And a narrator backend that returns valid narration for both main and trigger prompts
When the client POST /action with command="enter the shop"
And the Game is no longer busy
Then the story log holds at least one Narration message carrying an event header (the trigger fired)
And the story log holds at least 2 Narration messages in total (main + trigger continuation)
And the generation status is Idle
```

#### Scenario 1.5: Empty input produces a continuation narration without adding a player-input message

```gherkin
Given a Game whose story log holds only its opening Narration
And a narrator backend that returns "The scene continues." for the continuation prompt
When the client POST /action with command="" (empty)
And the Game is no longer busy
Then the story log gained exactly one Narration message (a continuation, with no player-input message)
And that Narration message's text is "The scene continues."
And the generation status is Idle
```

#### Scenario 1.6: Action trigger continuation re-runs the quantifier and detects newly-present Characters

```gherkin
Given a game state with a Character "gabriella" present in the current room
And a quantifier backend that returns [] for the first quantifier call then ["gabriella"] for the second
And a narrator backend that returns valid narration for both main and trigger prompts
When the client POST /action with command="enter shop"
And the Game is no longer busy
Then the Characters present include "gabriella"
And the player has met "gabriella" once
And the player is currently meeting "gabriella"
```

#### Scenario 1.7: Character without triggers produces narration with no event header

```gherkin
Given a game state with a Character "bartender" in the current room
And the Character has no triggers
And a quantifier backend that returns {"characters_present": ["bartender"]} for the quantifier prompt
And a narrator backend that returns "The bartender nods." for the main narration prompt
When the client POST /action with command="talk to bartender"
And the Game is no longer busy
Then the story log holds at least one Narration message
And no Narration message carries an event header (no trigger fired)
And the generation status is Idle
```

#### Scenario 1.8: Repeat action against a one-shot trigger does not refire

```gherkin
Given a game state with a Character "shopkeeper" whose Trigger fires on the first meeting
And the current room's Characters include "shopkeeper"
And a quantifier backend that returns {"characters_present": ["shopkeeper"]} for every quantifier call
And a narrator backend that returns valid narration for both main and trigger prompts
When the client POST /action with command="talk to shopkeeper"
And the Game is no longer busy
And the client POST /action with command="talk to shopkeeper"
And the Game is no longer busy
Then at most one Narration message carries an event header (the trigger fires at most once)
And the generation status is Idle
```

#### Scenario 1.14: A Narration that moves the player shows the room it arrives in

```gherkin
Given a game state whose player stands in "room_1"
And a map whose rooms are "room_1" and "room2"
And a quantifier backend that moves the player to "room2" for the first Action and reports no movement for the second
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="walk north"
And the Game is no longer busy
Then the story log shows the room "Room room2" on the Action's Narration message
And the story log shows no room on the player-input message
When the client POST /action with command="look"
And the Game is no longer busy
Then the story log shows no room on the second Action's Narration message
And the story log shows no room on that Action's player-input message
```

#### Scenario 1.15: A Narration that names the room the player is in shows no room

```gherkin
Given a game state whose player stands in "room_1"
And a map whose rooms are "room_1" and "room2"
And a quantifier backend that names the destination "room_1" for the Action
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="look around"
And the Game is no longer busy
Then the player still stands in "room_1"
And the story log shows no room on the Action's Narration message
```

#### Scenario 1.16: Two added rooms with the same name show one room name

```gherkin
Given a game state whose player stands in an added room named "Misty Glade"
And a quantifier backend that names the destination "Misty Glade" for the Action
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="walk on"
And the Game is no longer busy
Then the player stands in a second added room named "Misty Glade"
And the story log shows no room on the Action's Narration message
```

#### Scenario 1.17: A move between two map rooms with the same name shows no room

```gherkin
Given a game state whose player stands in "room_1"
And a map whose rooms "room_1" and "room2" have the same name
And a quantifier backend that moves the player to "room2" for the Action
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="walk north"
And the Game is no longer busy
Then the player stands in "room2"
And the story log shows no room on the Action's Narration message
```

### Error recovery

#### Scenario 2.1: Action in a non-existent room reports an error

```gherkin
Given a game state where the player stands in a room that does not exist
And the story log holds only its opening Narration
When the client POST /action with command="look"
And the Game is no longer busy
Then the generation status is an error whose text indicates the room is invalid
And the story log gained no Narration message
```

#### Scenario 2.2: LLM transport failure reports an error

```gherkin
Given a Game whose story log holds only its opening Narration
And an LLM backend configured to fail (returns an error for the narration request)
When the client POST /action with command="look"
And the Game is no longer busy
Then the generation status is an error whose text is non-empty
```

#### Scenario 2.4: Trigger narration failure preserves the main narration and reports an error

```gherkin
Given a game state with a Character "character_1" whose Trigger fires on the first meeting
And a quantifier backend that returns {"characters_present": ["character_1"]}
And a narrator backend that succeeds for the main prompt but fails for the trigger prompt
When the client POST /action with command="examine the character"
And the Game is no longer busy
Then the generation status is an error whose text is "The event narration failed."
And the status error's disclosure holds the failure line of the narrator call
And the story log gained no entry for the failure and still ends in the main Narration
And the story-log fragment still offers the `.retrigger-btn` control on that Narration
```

#### Scenario 2.5: A failed or unparseable quantifier answer still completes the action with the prior Character set

```gherkin
Given a game state with a Character "character_1" in the current room
And a narrator backend that returns a non-empty narration for any prompt
And a quantifier backend that either fails every call or answers on both attempts with text that is not a room report and names no known Character
When the client POST /action with command="look"
And the Game is no longer busy
Then the generation status is Idle
And the Characters present still include "character_1"
```

#### Scenario 2.6: An Action whose needed data is missing answers 500 with the missing-data line

```gherkin
Given a Game whose Persona record cannot be read
When the client POST /action with command="look"
Then the response is 500 Internal Server Error
And the response carries the line "The data for this request is missing: the persona."
And the story log gains no Message
When the client GET /screen?parts=status-display
Then the status response carries the same line
And the raw error text is reachable only inside that response's disclosure
```

### Concurrency

#### Scenario 3.3: A delayed LLM response does not leave the Game busy

```gherkin
Given a fresh game state
And an LLM backend that takes 200 ms to respond
When the client POST /action with command="look around"
Then the response is not 500 INTERNAL_SERVER_ERROR
And within 2 s, the generation status is no longer Generating
```

#### Scenario 3.4: An Action is refused while the Game is busy

```gherkin
Given a Game whose Job is running, held inside its narrator call
When a client POST /action with command="listen"
Then the response is 409 carrying the "Still thinking..." line
And the story log gained no player-input message for "listen"
And the story log gained no Narration message for "listen"
```

#### Scenario 3.5: Two clients POST the same Action at the same moment and one Action is accepted

```gherkin
Given a Game that is idle
When two clients POST /action with command="look" at the same moment
Then one Action is accepted and the other is refused with 409 and the "Still thinking..." line
And the story log holds exactly one player-input message for "look"
And the story log holds the opening Narration and exactly one Narration of that Action
```

### Sequencing

#### Scenario 6.1: Three actions in sequence produce three player-input and three Narration messages

```gherkin
Given a Game whose story log holds only its opening Narration
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the Game is no longer busy
And the client POST /action with command="look around"
And the Game is no longer busy
And the client POST /action with command="check inventory"
And the Game is no longer busy
Then the story log gained exactly 3 player-input messages
And the story log holds at least 3 Narration messages (one per action)
And each player-input message's text matches its submitted command, in order
```

#### Scenario 6.2: Execute → retry → execute produces two inputs and at least two narrations

```gherkin
Given a Game whose story log holds only its opening Narration
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the Game is no longer busy
And the client POST /swipe/new (retry)
And the Game is no longer busy
And the client POST /action with command="look around"
And the Game is no longer busy
Then the story log gained exactly 2 player-input messages
And the story log holds at least 2 Narration messages
```

#### Scenario 6.3: Async action sequence then retry completes with two inputs persisted

```gherkin
Given a Game whose story log holds only its opening Narration
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="hello"
And the Game is no longer busy
And the client POST /action with command="examine room"
And the Game is no longer busy
And the client POST /swipe/new (retry)
And the Game is no longer busy
Then the story log gained exactly 2 player-input messages
```

#### Scenario 6.4: The event Narration's call uses the preset that is active when it runs

```gherkin
Given a Game whose analysis fires a Trigger, so the Job narrates an event after its Narration
And the Job's narrator call is held
When the client sets the Game's system preset to a second preset
And the Job's model answer arrives
Then the LLM call record of the event Narration shows the system prompt of the second preset
And the LLM call record of the Narration shows the system prompt of the preset that was active when the Job started
```

### Slash commands

#### Scenario 1.9: `/impersonate` dispatches as impersonate and produces a player-input message

```gherkin
Given a Game whose story log holds only its opening Narration
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="/impersonate hello"
And the Game is no longer busy
Then the story log gained exactly one player-input message
And no player-input message's text is "/impersonate hello"
And the generation status is Idle
```

#### Scenario 1.18: A bare `/impersonate` writes an Impersonate Input

```gherkin
Given a Game whose story log holds only its opening Narration
And a narrator backend that returns a non-empty text for any prompt
When the client POST /action with command="/impersonate"
And the Game is no longer busy
Then the story log gained exactly one player-input message
And that message is an impersonation with no impersonate direction
And the generation status is Idle
```

#### Scenario 1.10: `/guide` dispatches as guided generation and does not persist a player-input message

```gherkin
Given a Game whose story log holds only its opening Narration
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="/guide look around"
And the Game is no longer busy
Then the story log gained at least one Narration message
And the story log gained no player-input message
And the generation status is Idle
```

#### Scenario 1.12: Recognized engine commands bypass the player-input text check

```gherkin
Given a Game whose story log holds only its opening Narration
And text check mode is Spell with auto-check enabled
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action/check with command="/guide look at the casle" (a deliberate misspelling the spell check would flag)
Then the response is 200 with an empty body, and it contains no text-check preview
And the guide action is dispatched directly
And the Game is no longer busy
And a narration message has created scene history
When the client POST /action/check with command="/options" (an engine command the dispatch layer handles)
Then the response is 200 with an empty body, and it contains no text-check preview
And the options action is dispatched directly
And the Game is no longer busy
When the client POST /action/check with command="look at the casle" (same words, plain input)
Then the response contains a text-check preview
```

Engine commands are the slash commands the dispatch layer itself handles —
`/guide`, `/impersonate` and `/options`. An unknown slash command stays free
text and is checked like ordinary player input.

### Confirm

#### Scenario 1.13: Confirming a preview dispatches the command and answers nothing

```gherkin
Given a game state with a command held in the text-check preview
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with that command
Then the response is 200 with an empty body
And the response carries no HX-Retarget, HX-Reswap or HX-Refresh header
And the Game is no longer busy
```

The confirm path is the preview's send: it dispatches the checked command.
The running generation shows in the status display, which loads when the
busy state changes. A confirm that the engine refuses answers the refusal
and writes no part of the screen.

## Properties

These properties hold across every `POST /action` and are observable
through HTTP. Drift indicates a regression even if all scenarios pass.

- **I.1** A Job always ends: the generation status then reads Idle or an
  error, never a step.
- **I.2** A player-input message always appears in the story log
  before its associated Narration message (the input is persisted first).
- **I.3** No scenario produces more than one main Narration message
  per `POST /action` call (trigger continuations are separate
  Narration messages; no duplicates from streaming, retry, or
  retrigger).
- **I.4** Failed narration never adds a Narration message to history
  (no empty/broken narrations persisted).
