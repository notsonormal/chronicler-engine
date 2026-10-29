# Feature Spec: Options

Endpoints:
- `POST /action`
- `GET /fragment/options-dock`
- `POST /games/{game_id}/mode`

## Scenarios

### On-demand generation

#### Scenario 24.1: The /options command renders the generated set in the dock

```gherkin
Given a game with scene history and an options agent
When the client POST /action with command="/options"
And the Game is no longer busy
Then GET /fragment/options-dock renders the options strip with one button per generated option
```

#### Scenario 24.2: /options on a Game with no story text answers 400

```gherkin
Given a Game whose story holds no text (its opening Narration has none)
When the client POST /action with command="/options"
Then the response is 400 Bad Request
And the response body names the missing scene ("No scene to generate options for yet")
And GET /fragment/options-dock renders no options strip and no Options failure
```

#### Scenario 24.18: /options with no options agent reports the agent is unavailable and keeps the set

```gherkin
Given a game with scene history, an offered set of set A, and no options agent
When the client POST /action with command="/options"
And the Game is no longer busy
Then the dock shows the Options failure saying the options agent is not available
And the dock renders no options strip
When the client POST /status/reset-busy
Then the dock renders set A
```

### Using the offered set

#### Scenario 24.4: New Options replace the set and add no history entries

```gherkin
Given a game with scene history and an options agent whose first response yields set A and whose second yields set B
When the client POST /action with command="/options" twice, waiting for idle between calls
Then the dock renders set B after the second call and no longer renders set A
And the story-log message count is unchanged by the two /options calls
```

#### Scenario 24.5: A failed Options run keeps the previous set and shows the Options failure in its place

```gherkin
Given the dock shows set A
And the options agent's next responses fail to parse
When the client POST /action with command="/options"
And the Game is no longer busy
Then the dock shows the Options failure naming the failed Options run
And the dock renders no options strip
When the client POST /status/reset-busy
Then the dock renders set A
```

#### Scenario 24.13: Using an offered option submits it as the player's input and narrates

```gherkin
Given a game with scene history and an options agent whose response yields set A
When the client POST /action with command="/options"
And the Game is no longer busy
And the client POST /action/check with command="Search the desk" (the form submit the Use button performs)
And the Game is no longer busy
Then the story log holds one more player-input message than before
And the newest player-input message's text is "Search the desk"
And a Narration message was added after it
```

### Always-on generation

#### Scenario 24.6: The always-on toggle generates options after a narration message

```gherkin
Given a world with the always-on toggle enabled
When the client POST /games creates a game from that world (the toggle is inherited at creation)
And the client POST /action with command="look"
And the Game is no longer busy
Then the dock renders a fresh option set without any /options command
```

#### Scenario 24.15: An impersonate message leaves the offered set unchanged

```gherkin
Given a game with the always-on toggle on
And the offered set is set A
When the client POST /action with command="/impersonate hello"
And the Game is no longer busy
Then the offered set still holds set A
And the dock still renders set A
```

#### Scenario 24.16: A narration message with the toggle off clears the offered set

```gherkin
Given a game whose always-on toggle is off
And the offered set is set A
When the client POST /action with command="look"
And the Game is no longer busy
Then the offered set is empty
And the dock renders no options strip
```

#### Scenario 24.17: A failed always-on Options run empties the offered set and reports the failure

```gherkin
Given a world with the always-on toggle on and an options agent whose first response yields set A and whose later responses fail to parse
When the client POST /games creates a game from that world
And the client POST /action with command="look"
And the Game is no longer busy
Then the offered set is set A
When the client POST /action with command="look" again
And the Game is no longer busy
Then the offered set is empty
And the dock renders no options strip
And the dock shows the Options failure naming the failed Options run
```

#### Scenario 24.20: With no options agent, an always-on narration message empties the offered set and reports the agent unavailable

```gherkin
Given a world with the always-on toggle on, no options agent, and an offered set of set A
When the client POST /action with command="look"
And the Game is no longer busy
Then the offered set is empty
And the dock renders no options strip
And the dock shows the Options failure saying the options agent is not available
```

#### Scenario 24.22: An Options run follows a Narration whose event narration fails

```gherkin
Given a world with the always-on toggle on and an options agent
And a Character whose Trigger fires on the next Action
When the client POST /games creates a game from that world
And the client POST /action with command="examine the character"
And the Game is no longer busy
Then the dock renders a fresh option set
And the dock shows no Options failure
And the status names the event narration failure
```

#### Scenario 24.23: An Options run follows a Narration whose state update fails

```gherkin
Given a world with the always-on toggle on and an options agent
And storage that fails the write of a state update
When the client POST /games creates a game from that world
And the client POST /action with command="look"
And the Game is no longer busy
Then the dock renders a fresh option set
And the dock shows no Options failure
And the status carries the contradiction line
```

#### Scenario 24.24: An Options run follows a Narration whose event text save fails

```gherkin
Given a world with the always-on toggle on and an options agent
And a Character whose Trigger fires on the next Action
And storage that fails the write of a new Message once the Narration that fired the Trigger is saved
When the client POST /games creates a game from that world
And the client POST /action with command="examine the character"
And the Game is no longer busy
Then the dock renders a fresh option set
And the dock shows no Options failure
And the status names the event narration failure
And the raw text in the status disclosure holds the save-failed line
```

### Response shapes

#### Scenario 24.10: A numbered-list response renders parsed options

```gherkin
Given an options agent whose response is a numbered list ("1. Search the desk" per line)
When the client POST /action with command="/options"
And the Game is no longer busy
Then the dock renders the option texts without their list numbering
```

### Mode neutrality

#### Scenario 24.11: /options works in an Interactive Fiction game

```gherkin
Given an active game in Interactive Fiction mode with scene history and an options agent
When the client POST /action with command="/options"
And the Game is no longer busy
Then the dock renders the generated option set
```

#### Scenario 24.19: A narrator-mode switch leaves the offered set alone

```gherkin
Given an active game in Novel mode whose offered set is set A
And an options agent whose next response yields set B
When the client POST /games/{game_id}/mode with narrator_mode="interactive_fiction"
Then the offered set still holds set A
```

### Concurrency

#### Scenario 24.21: /options is refused while the Game is busy

```gherkin
Given a Game with scene history whose Job is running, held inside its narrator call
When a client POST /action with command="/options"
Then the response is 409 carrying the "Still thinking..." line
And the offered set is unchanged
And GET /fragment/options-dock renders no options strip
```

### Dock controls

#### Scenario 24.14: The populated dock's controls reference icons the shell defines

```gherkin
Given a game with scene history and an options agent
When the client POST /action with command="/options"
And the Game is no longer busy
Then GET /fragment/options-dock shows at least one icon
And every icon it references is one GET / defines
```
