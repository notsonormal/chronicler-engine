# Feature Spec: Actions

Endpoints:
- `POST /action`
- `GET /fragment/llm-messages` (the LLM call records)

## Scenarios

### Normal flow

#### Scenario 1.1: Successful action produces exactly one narration and returns to Idle

```gherkin
Given a fresh game state with the default test world loaded
And the message history is empty
And an LLM backend that returns "You see a small wooden room." for any narration prompt
When the client POST /action with command="look"
And the pipeline returns to idle (status not generating)
Then the message history gained exactly 2 messages:
  - one player-input message whose text is "look"
  - one Narration message whose text is "You see a small wooden room."
And the generation status is Idle
```

#### Scenario 1.2: User input is persisted in history before the narration message

```gherkin
Given a fresh game state with an empty message history
And an LLM backend that returns "It is a small wooden room." for the narration prompt
When the client POST /action with command="examine the room"
And the pipeline returns to idle
Then the message history gained 2 messages
And the player-input message appears in the message history before the Narration message
And the player-input message's text is "examine the room"
And the Narration message's text is "It is a small wooden room."
And the generation status is Idle
```

#### Scenario 1.4: Action whose quantifier detects an NPC fires the trigger

```gherkin
Given a game state with an NPC "npc_1" whose Trigger fires on the first meeting
And the current room's NPCs include "npc_1"
And a quantifier backend that responds with {"npcs_in_room": ["npc_1"]} for the quantifier prompt
And a narrator backend that returns valid narration for both main and trigger prompts
When the client POST /action with command="enter the shop"
And the pipeline returns to idle
Then the message history holds at least one Narration message carrying an event header (the trigger fired)
And the message history holds at least 2 Narration messages in total (main + trigger continuation)
And the generation status is Idle
```

#### Scenario 1.5: Empty input produces a continuation narration without adding a player-input message

```gherkin
Given a fresh game state with an empty message history
And a narrator backend that returns "The scene continues." for the continuation prompt
When the client POST /action with command="" (empty)
And the pipeline returns to idle
Then the message history gained exactly one Narration message (a continuation, with no player-input message)
And that Narration message's text is "The scene continues."
And the generation status is Idle
```

#### Scenario 1.6: Action trigger continuation re-runs the quantifier and detects newly-present NPCs

```gherkin
Given a game state with an NPC "gabriella" present in the current room
And a quantifier backend that returns [] for the first quantifier call then ["gabriella"] for the second
And a narrator backend that returns valid narration for both main and trigger prompts
When the client POST /action with command="enter shop"
And the pipeline returns to idle
Then the scene's NPCs include "gabriella"
And the player has met "gabriella" once
And the player is currently meeting "gabriella"
```

#### Scenario 1.7: NPC without triggers produces narration with no event header

```gherkin
Given a game state with an NPC "bartender" in the current room
And the NPC has no triggers
And a quantifier backend that returns {"npcs_in_room": ["bartender"]} for the quantifier prompt
And a narrator backend that returns "The bartender nods." for the main narration prompt
When the client POST /action with command="talk to bartender"
And the pipeline returns to idle
Then the message history holds at least one Narration message
And no Narration message carries an event header (no trigger fired)
And the generation status is Idle
```

#### Scenario 1.8: Repeat action against a one-shot trigger does not refire

```gherkin
Given a game state with an NPC "shopkeeper" whose Trigger fires on the first meeting
And the current room's NPCs include "shopkeeper"
And a quantifier backend that returns {"npcs_in_room": ["shopkeeper"]} for every quantifier call
And a narrator backend that returns valid narration for both main and trigger prompts
When the client POST /action with command="talk to shopkeeper"
And the pipeline returns to idle
And the client POST /action with command="talk to shopkeeper"
And the pipeline returns to idle
Then at most one Narration message carries an event header (the trigger fires at most once)
And the generation status is Idle
```

#### Scenario 1.14: A Narration that moves the player reports the room it arrives in

```gherkin
Given a game state whose player stands in "room_1"
And a map whose rooms are "room_1" and "room2"
And a quantifier backend that moves the player to "room2" for the first Action and reports no movement for the second
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="walk north"
And the pipeline returns to idle
Then the Narration message reports the room "Room room2"
And the player-input message reports no room
When the client POST /action with command="look"
And the pipeline returns to idle
Then the second Action's Narration message reports no room
And that Action's player-input message reports no room
```

### Error recovery

#### Scenario 2.1: Action in a non-existent room reports an error

```gherkin
Given a game state where the player stands in a room that does not exist
And the message history is empty
When the client POST /action with command="look"
And the pipeline returns to idle
Then the generation status is an error whose text indicates the room is invalid
And the message history gained no Narration message
```

#### Scenario 2.2: LLM transport failure reports an error

```gherkin
Given a fresh game state with an empty message history
And an LLM backend configured to fail (returns an error for the narration request)
When the client POST /action with command="look"
And the pipeline returns to idle
Then the generation status is an error whose text is non-empty
```

#### Scenario 2.3: Empty LLM response reports an error without persisting an empty narration

```gherkin
Given a fresh game state with an empty message history
And an LLM backend that returns an empty string for the narration prompt
When the client POST /action with command="examine the room"
And the pipeline returns to idle
Then the generation status is an error whose text contains "empty"
And the message history gained no Narration message (no empty narration persisted)
```

#### Scenario 2.4: Trigger narration failure preserves the main narration and reports an error

```gherkin
Given a game state with an NPC "npc_1" whose Trigger fires on the first meeting
And a quantifier backend that returns {"npcs_in_room": ["npc_1"]}
And a narrator backend that succeeds for the main prompt but fails for the trigger prompt
When the client POST /action with command="examine the npc"
And the pipeline returns to idle
Then the generation status is an error whose text mentions "Trigger narration failed"
And the message history gained no entry for the failure and still ends in the main Narration
And the story-log fragment still offers the `.retrigger-btn` control on that Narration
```

#### Scenario 2.5: A failed or unparseable quantifier answer still completes the action with the prior NPC set

```gherkin
Given a game state with an NPC "npc_1" in the current room
And a narrator backend that returns a non-empty narration for any prompt
And a quantifier backend that either fails every call or answers on both attempts with text that is not a room report and names no known NPC
When the client POST /action with command="look"
And the pipeline returns to idle
Then the generation status is Idle
And the scene's NPCs still include "npc_1"
```

### Concurrency

#### Scenario 3.3: Delayed LLM response does not deadlock the pipeline

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
Then the response is 200 carrying the "Still thinking..." status
And the message history gained no player-input message for "listen"
And the message history gained no Narration message for "listen"
```

#### Scenario 3.5: Two clients POST the same Action at the same moment and one Action is accepted

```gherkin
Given a Game that is idle
When two clients POST /action with command="look" at the same moment
Then one Action is accepted and the other is refused with the "Still thinking..." status
And the message history holds exactly one player-input message for "look"
And the message history holds exactly one Narration message
```

### Sequencing

#### Scenario 6.1: Three actions in sequence produce three player-input and three Narration messages

```gherkin
Given a fresh game state with an empty message history
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the pipeline returns to idle
And the client POST /action with command="look around"
And the pipeline returns to idle
And the client POST /action with command="check inventory"
And the pipeline returns to idle
Then the message history gained exactly 3 player-input messages
And the message history holds at least 3 Narration messages (one per action)
And each player-input message's text matches its submitted command, in order
```

#### Scenario 6.2: Execute → retry → execute produces two inputs and at least two narrations

```gherkin
Given a fresh game state with an empty message history
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the pipeline returns to idle
And the client POST /swipe/new (retry)
And the pipeline returns to idle
And the client POST /action with command="look around"
And the pipeline returns to idle
Then the message history gained exactly 2 player-input messages
And the message history holds at least 2 Narration messages
```

#### Scenario 6.3: Async action sequence then retry completes with two inputs persisted

```gherkin
Given a fresh game state with an empty message history
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="hello"
And the pipeline returns to idle
And the client POST /action with command="examine room"
And the pipeline returns to idle
And the client POST /swipe/new (retry)
And the pipeline returns to idle
Then the message history gained exactly 2 player-input messages
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
Given a fresh game state with an empty message history
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="/impersonate hello"
And the pipeline returns to idle
Then the message history gained exactly one player-input message
And no player-input message's text is "/impersonate hello"
And the generation status is Idle
```

#### Scenario 1.10: `/guide` dispatches as guided generation and does not persist a player-input message

```gherkin
Given a fresh game state with an empty message history
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="/guide look around"
And the pipeline returns to idle
Then the message history gained at least one Narration message
And the message history gained no player-input message
And the generation status is Idle
```

#### Scenario 1.12: Recognized engine commands bypass the player-input text check

```gherkin
Given a fresh game state with an empty message history
And text check mode is Spell with auto-check enabled
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action/check with command="/guide look at the casle" (a deliberate misspelling the spell check would flag)
Then the response contains no text-check preview
And the guide action is dispatched directly
And the pipeline returns to idle
And a narration message has created scene history
When the client POST /action/check with command="/options" (an engine command the dispatch layer handles)
Then the response contains no text-check preview
And the options action is dispatched directly
And the pipeline returns to idle
When the client POST /action/check with command="look at the casle" (same words, plain input)
Then the response contains a text-check preview
```

Engine commands are the slash commands the dispatch layer itself handles —
`/guide`, `/impersonate` and `/options`. An unknown slash command stays free
text and is checked like ordinary player input.

### Confirm

#### Scenario 1.13: Confirming a preview retargets the status display

```gherkin
Given a game state with a command held in the text-check preview
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action/confirm with that command
Then the response is the status fragment alone, not the action area or the command form
And the response retargets #status-display
And the pipeline returns to idle
```

The confirm path is the preview's send: it dispatches the checked command
without swapping a fresh action area back in, so the status display alone
reflects the running generation.

## Properties

These properties hold across every `POST /action` and are observable
through HTTP. Drift indicates a regression even if all scenarios pass.

- **I.1** After the pipeline returns to idle, the generation status is
  Idle or an error — never stuck Generating.
- **I.2** A player-input message always appears in the message history
  before its associated Narration message (the input is persisted first).
- **I.3** No scenario produces more than one main Narration message
  per `POST /action` call (trigger continuations are separate
  Narration messages; no duplicates from streaming, retry, or
  retrigger).
- **I.4** Failed narration never adds a Narration message to history
  (no empty/broken narrations persisted).
