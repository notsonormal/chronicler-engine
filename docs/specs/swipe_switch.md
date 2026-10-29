# Feature Spec: Swipe Switching

Endpoint: `POST /message/:id/swipe/:index`

## Scenarios

### Restoring a swipe

#### Scenario 36.1: Switching to another swipe leaves the game ready

```gherkin
Given a game whose last Message has two Swipes
When the client switches to the other Swipe
Then the response is 200 with an empty body
And the generation status is idle
```

#### Scenario 36.2: Switching to another swipe drops the offered options

```gherkin
Given the dock shows a generated option set
And the last Message has two Swipes
When the client switches to the other Swipe
Then the dock renders no options strip
```

#### Scenario 36.3: A swipe switch while a Job runs is refused

```gherkin
Given a Game whose Message has two Swipes
And a Job of that Game held at its narrator call
When the client POST /message/:id/swipe/:index for the other Swipe of that Message
Then the response is 409 carrying the "Still thinking..." line
And the active Swipe of that Message does not change
```

#### Scenario 36.4: Switching to another swipe gives back the state of that Swipe

```gherkin
Given a Game whose Action moved the player into "room2"
And a Retry of that Action's Narration that stayed in "room_1"
And the Action's Narration has two Swipes
When the client switches to the first Swipe
Then the player stands in "room2"
And the story log shows the room "Room room2" on that Narration
When the client switches to the second Swipe
Then the player stands in "room_1"
And the story log shows no room on that Narration
```

#### Scenario 36.5: A swipe index past the last Swipe answers 400

```gherkin
Given a Game whose last Message has two Swipes
When the client POST /message/:id/swipe/2
Then the response is 400 Bad Request carrying "Swipe index out of bounds"
And the last Message keeps the active Swipe it had
And the last Message keeps its two Swipes
```
