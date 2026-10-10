# Feature Spec: Failure Display

Endpoints:
- `GET /fragment/header`
- `GET /fragment/visual-sidebar`
- `GET /status/generating`

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
When the client GET /status/generating
Then the response carries one short user-facing line
And the raw transport message is reachable only inside the response's disclosure
```

#### Scenario 38.5: A failure kind picks its own clamped line

```gherkin
Given a Game whose last generation failed because its active prompt preset is missing
When the client GET /status/generating
Then the response carries the missing-preset line, not the line for an unnamed failure
And the raw text naming the missing preset is reachable only inside the response's disclosure
```
