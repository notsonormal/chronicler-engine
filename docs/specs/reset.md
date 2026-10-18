# Feature Spec: Reset

Endpoint: `POST /reset`

## Scenarios

#### Scenario 7.1: Reset clears the previous game's story-log history

```gherkin
Given a Game whose story log holds an Input and its Narration
When the client POST /reset
Then the active Game's story log holds no Input
And it holds only its opening Narration
```

#### Scenario 7.2: Action after reset produces a fresh input

```gherkin
Given a fresh game state with narrative.history empty
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the pipeline returns to idle
And the client POST /reset
And the client POST /action with command="look around"
And the pipeline returns to idle
Then message_service.load_messages() contains exactly 1 Input entry
And the Input entry's text is "look around"
```

#### Scenario 7.3: A reset while a Job runs leaves the new Game with its opening only

```gherkin
Given a Game whose Job is held at its narrator call
When the client POST /reset
And that Job's model answer arrives after the reset
Then the new Game's story log holds its opening Narration
And the new Game's story log holds no Input and no Narration of the Job the reset ended
```
