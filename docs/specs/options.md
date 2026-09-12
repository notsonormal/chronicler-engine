# Feature Spec: Options

Endpoints:
- `POST /action`
- `GET /fragment/options-dock`

## Scenarios

### On-demand generation

#### Scenario 24.1: The /options command renders the generated set in the dock

```gherkin
Given a game with scene history and an options agent
When the client POST /action with command="/options"
And the pipeline returns to idle
Then GET /fragment/options-dock renders the options strip with one button per generated option
```

#### Scenario 24.2: /options with no scene history surfaces an error and an empty dock

```gherkin
Given a fresh game with an empty story log
When the client POST /action with command="/options"
Then the response is a 500 whose body names the validation failure ("No scene to generate options for yet")
And GET /fragment/options-dock renders no options strip
```

### Using the offered set

#### Scenario 24.4: Regenerating replaces the set and adds no history entries

```gherkin
Given a game with scene history and an options agent whose first response yields set A and whose second yields set B
When the client POST /action with command="/options" twice, waiting for idle between calls
Then the dock renders set B after the second call and no longer renders set A
And the story-log message count is unchanged by the two /options calls
```

#### Scenario 24.5: A failed regeneration keeps the previous set

```gherkin
Given the dock shows set A
And the options agent's next responses fail to parse
When the client POST /action with command="/options"
And the pipeline returns to idle
Then the dock still renders set A
And a System message naming the options-generation failure was added to the story log
```

### Always-on generation

#### Scenario 24.6: The always-on toggle generates options after a narration turn

```gherkin
Given a world with options_always_on enabled
When the client POST /games creates a game from that world (the toggle is inherited at creation)
And the client POST /action with command="look"
And the pipeline returns to idle
Then the dock renders a fresh option set without any /options command
```

#### Scenario 24.7: The always-on toggle never fires after an impersonate turn

```gherkin
Given a game with the always-on toggle on
When the client POST /action with command="/impersonate hello"
And the pipeline returns to idle
Then the dock renders no options strip
And the offered set is empty
```

#### Scenario 24.12: A narration turn with the toggle off generates no options

```gherkin
Given a game whose always-on toggle is off
When the client POST /action with command="look"
And the pipeline returns to idle
Then the dock renders no options strip
And the offered set is empty
```

### Response shapes

#### Scenario 24.10: A numbered-list response renders parsed options

```gherkin
Given an options agent whose response is a numbered list ("1. Search the desk" per line)
When the client POST /action with command="/options"
And the pipeline returns to idle
Then the dock renders the option texts without their list numbering
```

### Mode neutrality

#### Scenario 24.11: /options works in an Interactive Fiction game

```gherkin
Given an active game in Interactive Fiction mode with scene history and an options agent
When the client POST /action with command="/options"
And the pipeline returns to idle
Then the dock renders the generated option set
```
