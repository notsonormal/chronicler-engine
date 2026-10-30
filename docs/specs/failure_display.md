# Feature Spec: Failure Display

Endpoints:
- `POST /action`
- `GET /screen?parts=header`
- `GET /screen?parts=failure-banner`
- `GET /screen?parts=settings`
- `GET /screen?parts=visual-sidebar`
- `GET /screen?parts=status-display`

## Scenarios

### Header health

#### Scenario 38.1: A degraded role raises the failure banner and clears on recovery

```gherkin
Given an active Game whose newest Quantifier attempt carries an error message
When the client GET /screen?parts=failure-banner
Then the response carries a failure banner naming the Quantifier and its failure effect
And the banner states that role health covers every game
And the banner's Details disclosure lists each role with the backend and model of its newest attempt
And a role whose newest attempt succeeded reads as a success
And a role with no recorded attempt reads as having no calls
And the raw failure text is reachable only inside that disclosure
When a later Quantifier attempt succeeds and the client GET /screen?parts=failure-banner again
Then the response carries no failure banner
```

#### Scenario 38.8: A degraded Trigger role raises the banner and leaves the Connections marker empty

```gherkin
Given an active Game whose newest Trigger attempt carries an error message
When the client GET /screen?parts=failure-banner
Then the response carries a failure banner naming the Trigger and its failure effect
And its Details disclosure lists the Trigger role with that raw failure text
When the client GET /screen?parts=settings
Then the Connections sub-tab's warning marker stays empty
```

#### Scenario 38.6: The header carries the game name only, and the Settings panel renders its own role health

```gherkin
Given a client showing the dashboard, with a role whose newest attempt failed
When the client GET /screen?parts=header
Then the response carries the game name
And the response carries no failure banner and no health cell
And the response carries no out-of-band swap
When the client GET /screen?parts=settings
Then the response carries a health cell for each role row, with no out-of-band swap
And the failed role's cell reports it as Degraded
And the response carries the Connections sub-tab's warning marker, which is present
When a later attempt for that role succeeds
And the client GET /screen?parts=settings again
Then the cell reports the role as Healthy
And the marker is empty
```

### Failed requests

#### Scenario 38.2: A failed load answers non-2xx and asks the client to change nothing

```gherkin
Given an active Game whose visual sidebar cannot render
When the client GET /screen?parts=visual-sidebar
Then the response is non-2xx
And the response asks the client to swap nothing
```

#### Scenario 38.18: A panel load that cannot read its data answers non-2xx with an error fragment

```gherkin
Given the settings, the Games or the Worlds cannot be read from storage
When the client GETs /screen?parts=settings, /screen?parts=prompt-presets, /screen?parts=worlds or /screen?parts=games
Then each response is non-2xx
And the body of each response is an error fragment
And no response carries HX-Reswap
```

`GET /screen` fails the same way when a part cannot be read: Scenario 45.11 (`docs/specs/screen.md`).

#### Scenario 38.4: A failed request is not a failed load

```gherkin
Given an active Game whose posture auto-save cannot be stored
When the client posts the posture change
Then the response is non-2xx
And the failure text reaches the client for it to render on the surface of the request
And the response does not ask the client to swap nothing
```

### Generation errors

#### Scenario 38.3: A generation error clamps to one line with the raw text in the disclosure

```gherkin
Given a Game whose last generation failed with a raw transport message
When the client GET /screen?parts=status-display
Then the response carries data-status "failed" and one short user-facing line
And the raw transport message is reachable only inside the response's disclosure
```

#### Scenario 38.5: A failure kind picks its own clamped line

```gherkin
Given a Game whose last generation failed because its active prompt preset is missing
When the client GET /screen?parts=status-display
Then the response carries the missing-data line that names the preset, not the line for an unnamed failure
And the raw text naming the missing preset is reachable only inside the response's disclosure
```

#### Scenario 38.7: A spent token budget clamps to its own line with the counts in the disclosure

```gherkin
Given a Game whose last generation failed because the model spent its whole token budget before writing an answer
When the client GET /screen?parts=status-display
Then the response carries the spent-token-budget line, not the unreadable-answer line or the line for an unnamed failure
And that line names the model's reasoning and only a remedy the dashboard offers
And the raw text naming the completion and reasoning token counts is reachable only inside the response's disclosure
```

#### Scenario 38.9: An empty answer keeps its own line and saves no text

```gherkin
Given a Game whose narrator call answers with empty text
When the client POST /action with command="look"
And the Job ends
And the client GET /screen?parts=status-display
Then the status response carries the empty-answer line, not the unreadable-answer line
And the story log holds no Narration of that Action
```

#### Scenario 38.10: A contradiction after the text keeps the text and names its own line

```gherkin
Given a Game whose current state names a Character that the World does not hold
And a narrator backend that answers the narration prompt
When the client POST /action with command="look"
And the Job ends
And the client GET /screen?parts=status-display
Then the status response carries the contradiction line
And the story log holds that Action's Narration text
And the raw internal message appears once, inside that response's disclosure
```

#### Scenario 38.11: A failed save keeps its own line and saves no Narrations

```gherkin
Given a running Job whose Narration call is held
And the storage fails the write of a Narration's text while the call is held
When the call is released and the Job ends
And the client GET /screen?parts=status-display
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
And the client GET /screen?parts=status-display
Then the status response carries the prompt-too-long line, not the line for an unnamed failure
And the raw context-overflow message is reachable only inside the response's disclosure
And the story log holds no Narration of that Action
```

#### Scenario 38.13: An unreadable Game while the Job chooses a hook preset ends the Job with a failure line

```gherkin
Given a running Job whose Narration call is held
And the Game's record becomes unreadable while the call is held
When the call is released and the Job ends
And the client GET /screen?parts=status-display
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
And the client GET /screen?parts=status-display
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

#### Scenario 38.15: A start state that cannot be read fails the request with the missing-data line

```gherkin
Given a Game whose start state cannot be read
When the client POST /action with command="look"
Then the response is 500
And the response carries the missing-data line that names the start state
And the response does not carry the raw storage error
When the client GET /screen?parts=status-display
Then the status response carries the same line
And the raw storage error is reachable only inside the response's disclosure
And the story log is unchanged
```

#### Scenario 38.16: A missing preset of the Narrator or of a hook shows the missing-data line that names the preset

```gherkin
Given a Game whose preset for the kind of one row of the Examples table below is not in the library
When the client POST /action with command="look"
And the Job ends
And the client GET /screen?parts=status-display
Then the status response carries the missing-data line that names the preset
And the raw text naming the missing preset is reachable only inside the response's disclosure
And the story log holds the Narrations of that row
And the Quantifier is not asked

Examples:
| Missing preset | Narrations in the story log |
| the Narrator's | the opening only |
| the Quantifier's | the opening and the Narration of the Action |
```

#### Scenario 38.17: A failed state update shows the contradiction line and keeps the text

```gherkin
Given a Game whose storage fails the write of a state update
And a narrator backend that answers the narration prompt
When the client POST /action with command="look"
And the Job ends
And the client GET /screen?parts=status-display
Then the status response carries the contradiction line, not the save-failed line
And the raw storage error is reachable only inside the response's disclosure
And the story log holds that Action's Narration text
```
