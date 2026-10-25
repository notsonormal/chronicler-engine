# Feature Spec: Failure Display

Endpoints:
- `POST /action`
- `GET /fragment/header`
- `GET /fragment/visual-sidebar`
- `GET /status/busy`

## Scenarios

### Header health

#### Scenario 38.1: A degraded role raises the header banner and clears on recovery

```gherkin
Given an active Game whose newest Quantifier attempt carries an error message
When the client GET /fragment/header
Then the response carries a failure banner naming the Quantifier and its failure effect
And the banner states that role health covers every game
And the banner's Details disclosure lists each role with the backend and model of its newest attempt
And a role whose newest attempt succeeded reads as a success
And a role with no recorded attempt reads as having no calls
And the raw failure text is reachable only inside that disclosure
When a later Quantifier attempt succeeds and the client GET /fragment/header again
Then the response carries no failure banner
```

#### Scenario 38.8: A degraded Trigger role raises the banner and leaves the Connections marker empty

```gherkin
Given an active Game whose newest Trigger attempt carries an error message
When the client GET /fragment/header
Then the response carries a failure banner naming the Trigger and its failure effect
And its Details disclosure lists the Trigger role with that raw failure text
And the Connections sub-tab's warning marker stays empty
```

#### Scenario 38.6: The header poll keeps the Settings role health current

```gherkin
Given a client showing the dashboard, whose Settings panel rendered a health cell per role row
When a role's newest attempt fails
And the client GET /fragment/header
Then the response carries an out-of-band swap for that role's health cell
And the cell reports the role as Degraded
And the response carries an out-of-band swap for the Connections sub-tab's warning marker, which is present
When a later attempt for that role succeeds
And the client GET /fragment/header again
Then the cell reports the role as Healthy
And the marker's swap is empty
```

### Failed requests

#### Scenario 38.2: A failed poll answers non-2xx and asks the client to change nothing

```gherkin
Given an active Game whose visual sidebar cannot render
When the client GET /fragment/visual-sidebar
Then the response is non-2xx
And the response asks the client to swap nothing
```

#### Scenario 38.4: A failed user action is not a failed poll

```gherkin
Given an active Game whose posture auto-save cannot be stored
When the client posts the posture change
Then the response is non-2xx
And the failure text reaches the client for it to render on the action's own surface
And the response does not ask the client to swap nothing
```

### Generation errors

#### Scenario 38.3: A generation error clamps to one line with the raw text in the disclosure

```gherkin
Given a Game whose last generation failed with a raw transport message
When the client GET /status/busy
Then the response carries one short user-facing line
And the raw transport message is reachable only inside the response's disclosure
```

#### Scenario 38.5: A failure kind picks its own clamped line

```gherkin
Given a Game whose last generation failed because its active prompt preset is missing
When the client GET /status/busy
Then the response carries the missing-data line that names the preset, not the line for an unnamed failure
And the raw text naming the missing preset is reachable only inside the response's disclosure
```

#### Scenario 38.7: A spent token budget clamps to its own line with the counts in the disclosure

```gherkin
Given a Game whose last generation failed because the model spent its whole token budget before writing an answer
When the client GET /status/busy
Then the response carries the spent-token-budget line, not the unreadable-answer line or the line for an unnamed failure
And that line names the model's reasoning and only an action the dashboard offers
And the raw text naming the completion and reasoning token counts is reachable only inside the response's disclosure
```

#### Scenario 38.9: An empty answer keeps its own line and saves no text

```gherkin
Given a Game whose narrator call answers with empty text
When the client POST /action with command="look"
And the Job ends
And the client GET /status/busy
Then the status response carries the empty-answer line, not the unreadable-answer line
And the story log holds no Narration of that Action
```

#### Scenario 38.10: A contradiction after the text keeps the text and names its own line

```gherkin
Given a Game whose current state names a Character that the World does not hold
And a narrator backend that answers the narration prompt
When the client POST /action with command="look"
And the Job ends
And the client GET /status/busy
Then the status response carries the contradiction line
And the story log holds that Action's Narration text
And the raw internal message appears once, inside that response's disclosure
```

#### Scenario 38.11: A refused save keeps its own line and saves no Narrations

```gherkin
Given a running Job whose Narration call is held
And the storage refuses the write of a Narration's text while the call is held
When the call is released and the Job ends
And the client GET /status/busy
Then the status response carries the save-failed line, not the line for an unnamed failure
And the raw storage error is reachable only inside the response's disclosure
And the story log holds that Action's Input
And the story log holds no Narration of that Action
```

#### Scenario 38.12: A prompt too long for the connection keeps its own line

```gherkin
Given a Game whose narration connection holds a context window too small for the system prompt and the fixed input budget
When the client POST /action with command="look"
And the Job ends
And the client GET /status/busy
Then the status response carries the prompt-too-long line, not the line for an unnamed failure
And the raw context-overflow message is reachable only inside the response's disclosure
And the story log holds no Narration of that Action
```

#### Scenario 38.13: An unreadable Game while the Job chooses a hook preset ends the Job with a failure line

```gherkin
Given a running Job whose Narration call is held
And the Game's record becomes unreadable while the call is held
When the call is released and the Job ends
And the client GET /status/busy
Then the status response carries the missing-data line that names the Game
And the raw storage message is in the response's disclosure
And the Quantifier is not asked
And the story log holds the Narration of that Action beside the opening
```

#### Scenario 38.14: A read that fails in a Job shows the missing-data line with the name of its record

```gherkin
Given a Game whose Job reads the record of one row of the Examples table below
And the read fails
When the client POST /action with command="look"
And the Job ends
And the client GET /status/busy
Then the status response carries the missing-data line that names the record of that row
And the status response does not carry the save-failed line
And the raw storage error is reachable only inside the response's disclosure
And the story log holds the Narrations of that row

Examples:
| Read that fails | Record in the line | Narrations in the story log |
| the story entries, after the Narration call | the story | the opening and the Narration of the Action |
| the Swipe state of the start, after the Narration call | the start state | the opening only |
| the Swipe state of the saved text, after the analysis call | the saved state | the opening and the Narration of the Action |
| the World, at the first step | the world | the opening only |
```

#### Scenario 38.15: A start state that cannot be read refuses the request with the missing-data line

```gherkin
Given a Game whose start state cannot be read
When the client POST /action with command="look"
Then the response is 500
And the response carries the missing-data line that names the start state
And the response does not carry the raw storage error
When the client GET /status/busy
Then the status response carries the same line
And the raw storage error is reachable only inside the response's disclosure
And the story log is unchanged
```

#### Scenario 38.16: A missing preset of the Narrator or of a hook shows the missing-data line that names the preset

```gherkin
Given a Game whose preset for the kind of one row of the Examples table below is not in the library
When the client POST /action with command="look"
And the Job ends
And the client GET /status/busy
Then the status response carries the missing-data line that names the preset
And the raw text naming the missing preset is reachable only inside the response's disclosure
And the story log holds the Narrations of that row
And the Quantifier is not asked

Examples:
| Missing preset | Narrations in the story log |
| the Narrator's | the opening only |
| the Quantifier's | the opening and the Narration of the Action |
```

#### Scenario 38.17: A refused state update shows the contradiction line and keeps the text

```gherkin
Given a Game whose storage refuses the write of a state update
And a narrator backend that answers the narration prompt
When the client POST /action with command="look"
And the Job ends
And the client GET /status/busy
Then the status response carries the contradiction line, not the save-failed line
And the raw storage error is reachable only inside the response's disclosure
And the story log holds that Action's Narration text
```
