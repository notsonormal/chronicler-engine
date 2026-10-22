# Feature Spec: Autoplay

Endpoints:
- `POST /autoplay` (the start; form field `cycles`)
- `POST /status/reset-generating` (the stop)
- `GET /status/generating` (the progress)

An Autoplay **run** plays the Persona by itself. It starts ordinary Jobs one after the other: an `/impersonate` Job, then a Continue Job that answers the Input the first one wrote. The run holds its Game from its start to its end, also between two Jobs. A **cycle** ends when a Job of the run ends without a failure line and leaves the story on a Narration. So an Input is answered by a Continue Job, or by the event Narration of a Trigger that fires in the Job that wrote the Input. The **budget** is the number of cycles: it counts the answered Inputs of the run.

The next Job of a run is a Continue when the last Message is an Input, and an Impersonate in every other case. This rule also picks the first Job.

A run stops for one of three reasons: a stop, the budget, or a Job that ends with a failure line. A failed Options run and a fallback of the Quantifier do not stop it.

The status poll of a Game that a run holds answers `autoplay <cycle>/<budget> <step>` while a Job runs, and `autoplay <cycle>/<budget>` between two Jobs. The cycle is the number of the cycle that is in progress. The step names are the ones of Scenario 39.1.

## Scenarios

#### Scenario 41.1: A run on a fresh Game plays its cycles and the Game is then idle

```gherkin
Given a Game whose story log holds only its opening Narration
When the client POST /autoplay with cycles=2
Then the response body carries "Thinking..."
When the run ends
Then the story log holds, after the opening Narration, an Impersonate Input, a Narration, an Impersonate Input and a Narration, in this order
And the Game is idle
And the narrator call of each Continue Job carries the Impersonate Input that it answers
```

#### Scenario 41.2: A run whose last Message is an unanswered Input starts with a Continue

```gherkin
Given a Game whose last Message is an Impersonate Input with no Narration after it
When the client POST /autoplay with cycles=1
And the run ends
Then the first Job of the run is a Continue Job, with no Impersonate call
And the story log holds a Narration after that Input
And the Game is idle
```

#### Scenario 41.3: A Trigger that fires in a Continue Job adds its event Narration to the cycle

```gherkin
Given a Game whose analysis of a Continue Job fires a Trigger
When the client POST /autoplay with cycles=2
And the run ends
Then the story log holds the Impersonate Input, the Narration and the event Narration of the Trigger of the first cycle
And the second Impersonate Input and the second Narration follow the event Narration
And the run made two Impersonate Jobs and two Continue Jobs
And the Game is idle
```

#### Scenario 41.4: A failed narrator call ends the run and its failure line stays

```gherkin
Given a Game whose narrator call of the Continue Job fails
When the client POST /autoplay with cycles=3
And the run ends
Then the story log holds the Impersonate Input and no Narration of the run
And the status poll answers with the failure line of that Job
And the Game is not busy
And no further Job of the run starts
```

#### Scenario 41.5: A failed Options run writes its Notice and the run goes on

```gherkin
Given a Game whose World has the always-on Options toggle
And an Options run that fails at the end of each Continue Job
When the client POST /autoplay with cycles=2
And the run ends
Then the story log holds both cycles
And the dock shows the Notice of the failed Options run
And the Game is idle
```

#### Scenario 41.6: A stop during a Job of a run ends the run

```gherkin
Given a run whose first Job is held at its narrator call
When the client POST /status/reset-generating
Then the stop answers "reset"
And the Game is idle at once
When the Job's model answer arrives after the stop
Then the story log holds no Message of the run
And no further Job of the run starts
And the client can start a Job on the Game
```

#### Scenario 41.7: A stop between two Jobs of a run ends the run

```gherkin
Given a Game that a run holds, with no Job running
When the client POST /status/reset-generating
Then the stop answers "reset"
And the status poll answers "idle"
And the story log is as it was
And no model call is made
And the client can start a Job on the Game
```

#### Scenario 41.8: A start on a busy Game, or on a Game that a run holds, answers "Still thinking..."

```gherkin
Given a Game whose Job is held at its narrator call
When the client POST /autoplay with cycles=1
Then the response status is 200
And the response body carries "Still thinking..."
And no run holds the Game after that Job ends

Given a Game that a run holds
When the client POST /autoplay with cycles=1
Then the response status is 200
And the response body carries "Still thinking..."
And the first run goes on with its own budget
```

#### Scenario 41.9: A budget that is 0, above 100, or no whole number answers 400

```gherkin
Given a Game with no Job
When the client POST /autoplay with cycles=0
Then the response status is 400
And the Game is idle

When the client POST /autoplay with cycles=101
Then the response status is 400
And the Game is idle

When the client POST /autoplay with cycles=many
Then the response status is 400
And the Game is idle

When the client POST /autoplay with cycles=100
Then the status poll answers "autoplay 1/100 narrating" while the first Job is at its narrator call
```

#### Scenario 41.10: A start with no `cycles` field runs with a budget of 10

```gherkin
Given a Game with no Job
When the client POST /autoplay with no cycles field
Then the status poll answers "autoplay 1/10 narrating" while the first Job is at its narrator call
```

#### Scenario 41.11: While a run holds the Game, every story write is refused, also between two Jobs

```gherkin
Given a Game whose last Message is a Narration with two Swipes
And a run that holds the Game with no Job running
When the client POST /action with command="look"
And the client POST /swipe/new
And the client switches to the other Swipe of the last Message
And the client POST /history/:id with a new text
And the client POST /history/delete
Then each response status is 200
And each response body carries "Still thinking..."
And the story log is as it was
```

#### Scenario 41.12: While a run holds the Game, the config writes are refused and the config stays

```gherkin
Given a Game with a Narrator Mode, a posture and four preset selections
And a run that holds the Game with no Job running
When the client POST /games/:id/mode with the other mode
And the client POST /games/:id/posture with another perspective and tense
And the client POST /games/:id/presets with another system preset
Then each response status is 200
And each response body carries "Still thinking..."
And the Game's config is as it was
When the run ends
And the client POST /games/:id/posture with another perspective and tense
Then the Game's posture is the new one
```

#### Scenario 41.13: The status poll answers with the progress while a run holds the Game

```gherkin
Given a run with cycles=2 on a Game
When a Job of the first cycle is at its narrator call
Then the status poll answers "autoplay 1/2 narrating"
When the Continue Job of the first cycle is at its narrator call
Then the status poll answers "autoplay 1/2 narrating"
When the Impersonate Job of the second cycle is at its narrator call
Then the status poll answers "autoplay 2/2 narrating"

Given a Game that a run holds, with no Job running
When the client requests GET /status/generating and GET /debug/is_generating
Then the status poll answers "autoplay 1/2"
And the debug response body is "true"
```

#### Scenario 41.14: A run goes on when the client switches the active Game

```gherkin
Given a run whose first Job is held at its narrator call
When the client creates a second Game, which becomes the active Game
And the Job's model answer arrives
And the run ends
Then the story log of the first Game holds the Messages of the run
And the story log of the second Game holds its opening Narration only
```

#### Scenario 41.15: A reset of the Game ends its run

```gherkin
Given a run whose first Job is held at its narrator call
When the client POST /reset
And the Job's model answer arrives after the reset
And the run ends
Then the new Game's story log holds its opening Narration only
And the status poll answers "idle"
And no further Job of the run starts
```

#### Scenario 41.16: A delete of the Game ends its run

```gherkin
Given a run on a Game that is not the active Game, whose first Job is held at its narrator call
When the client POST /games/:id/delete for that Game
And the Job's model answer arrives after the delete
And the run ends
Then no further Job of the run starts
```

#### Scenario 41.17: The end of a stopped run does not release the run that started after it

```gherkin
Given a run A whose first Job is held at its narrator call
And the client stopped run A
And the client started a run B on the same Game, whose first Job is held at its narrator call
When the model answer of run A arrives
And run A ends
Then the status poll answers with the progress of run B
And a typed action answers "Still thinking..."
When the model answer of run B arrives
And run B ends
Then the story log holds the Messages of run B's budget
And the Game is idle
```

#### Scenario 41.18: While a run holds the Game, the story log and the options dock render their story-changing controls disabled

```gherkin
Given a Game whose last Message is a Narration with two Swipes, the second one active
And an offered set of options in the dock
When the client GET /fragment/story-log and GET /fragment/options-dock
Then no Edit, Delete, Previous swipe or Retry control carries the `disabled` attribute
And no option button carries the `disabled` attribute

Given a run that holds the Game with no Job running
When the client GET /fragment/story-log and GET /fragment/options-dock
Then every Edit, Delete, Previous swipe and Retry control carries the `disabled` attribute
And every option button carries the `disabled` attribute
And the entries and their text are as before

When the client POST /status/reset-generating
And the client GET /fragment/story-log and GET /fragment/options-dock
Then none of those controls carries the `disabled` attribute
```

#### Scenario 41.19: The Games fragment marks a Game that a run holds

```gherkin
Given two Games, with the second one active
And a run that holds the first Game with no Job running
When the client GET /fragment/games
Then the item of the first Game carries a `.run-badge`
And the item of the active Game carries no `.run-badge`

Given a run that holds the active Game with no Job running
When the client GET /fragment/games
Then the item of the active Game carries a `.run-badge`
```

#### Scenario 41.20: While a run holds the Game, its config controls render disabled

```gherkin
Given a run that holds the active Game with no Job running
When the client GET /fragment/games
Then the Narrator Mode, Perspective, Tense and four preset selects of the active Game carry the `disabled` attribute

When the client POST /status/reset-generating
And the client GET /fragment/games
Then none of those selects carries the `disabled` attribute
```

#### Scenario 41.21: A Trigger that fires in an Impersonate Job answers the Input and ends the cycle

```gherkin
Given a Game whose analysis of an Impersonate Job fires a Trigger
When the client POST /autoplay with cycles=2
And the run ends
Then the story log holds, after the opening Narration, the first Impersonate Input, the event Narration of the Trigger, the second Impersonate Input and a Narration, in this order
And the event Narration ends the first cycle, so the run ends after two cycles
And the run made three Jobs: two Impersonate Jobs and one Continue Job
And the Game is idle
```

#### Scenario 41.22: While a run holds the Game, the Retrigger and Next swipe controls render disabled

```gherkin
Given a Game whose last Message is an Impersonate Input whose event Narration failed
And a run that holds the Game with no Job running
When the client GET /fragment/story-log
Then the Retrigger control carries the `disabled` attribute

Given a Game whose last Message is a Narration with two Swipes, the first one active
And a run that holds the Game with no Job running
When the client GET /fragment/story-log
Then the Next swipe control carries the `disabled` attribute

When the client POST /status/reset-generating
And the client GET /fragment/story-log
Then the Next swipe control does not carry the `disabled` attribute
```
