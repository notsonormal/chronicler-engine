# Feature Spec: Dashboard

Endpoints:
- `GET /status/generating` (the dashboard chrome's status poll)
- `POST /status/reset-generating` (the stop)
- `GET /fragment/options-dock`
- `GET /debug/is_generating`
- `GET /debug/state`
- `GET /` (the shell that defines the icons)

## Scenarios

#### Scenario 39.1: The status poll answers with the step of a running Job, else the failure line, else idle

```gherkin
Given a Game whose Job is at the narrator call of a Narration
When the client requests GET /status/generating
Then the response body is "narrating"

Given a Game whose Job is at its analysis
When the client requests GET /status/generating
Then the response body is "quantifying"

Given a Game with no Job and a failure line
When the client requests GET /status/generating
Then the response body carries that failure line

Given a Game with no Job and no failure line
When the client requests GET /status/generating
Then the response body is "idle"
```

#### Scenario 39.2: The shell defines each referenced icon exactly once
```gherkin
Given a game with at least 2 Messages and a stored trigger
When the client requests GET / and GET /fragment/story-log, /fragment/games and /fragment/connections/new
Then the shell defines each icon exactly once
And every icon the shell or a fragment named above references is one the shell defines
And each fragment named above shows at least one icon
```

#### Scenario 39.3: The dock and the debug endpoint answer from the busy state

```gherkin
Given a Game with an offered set and a Job
When the client requests GET /fragment/options-dock and GET /debug/is_generating
Then the dock's controls are disabled
And the debug response body is "true"

Given a Game with an offered set and no Job
When the client requests GET /fragment/options-dock and GET /debug/is_generating
Then the dock's controls are enabled
And the debug response body is "false"
```

#### Scenario 39.4: A stop at each model call leaves the records of the step it reached

```gherkin
Given a Game whose Job is held at one model call of the Examples table below
And the status poll answers with the step name of that row
When the client POST /status/reset-generating
Then the stop answers "reset"
And the Game is idle
And the status poll answers with no failure line
When the Job's model answer arrives after the stop
Then the story log holds exactly the records of that row

Examples:
| Kind of Job | Model call | Status while held | Records in the story log after the stop |
| Narration | the narrator call | narrating | the Input of the Job, and no Narration |
| Narration | the analysis call | quantifying | the Input of the Job and the Narration, with no room on the Narration and no event Narration |
| Narration | the Options call | options | the Input of the Job, the Narration, and the room the Narration arrived in |
| Impersonate | the narrator call | narrating | no Message of the Job |
| Impersonate | the analysis call | quantifying | the Input of the Job and no Narration |
| Retry of a typed Input | the narrator call | narrating | the Messages of the story before the Retry |
| event Narration | the event narrator call | generating-event | the Narration that fired the Trigger, and no event Narration |

A stop inside a save unit is not observable: a save unit takes milliseconds, so a stop just before one leaves the records of the model call before it.
```

#### Scenario 39.5: A stopped Job's late answer changes nothing, and the next Job saves its own text

```gherkin
Given a Game whose Job is held at its narrator call
And the client stopped that Job
And a second Job is held at its narrator call
When the stopped Job's model answer arrives
And the second Job's model answer arrives
Then the story log holds the Input of the stopped Job
And the story log holds the Input and the Narration of the second Job
And the story log holds no Narration of the stopped Job
```

#### Scenario 39.6: The debug state names a Step only while a Job runs

```gherkin
Given a Game whose Job is at the narrator call of a Narration
When the client requests GET /debug/state
Then the response reports the status "Generating" and the Step "Narrating"

Given a Game with no Job and a failure line
When the client requests GET /debug/state
Then the response reports the status "Error" with the kind of that failure line, and no Step

Given a Game with no Job and no failure line
When the client requests GET /debug/state
Then the response reports the status "Idle" and no Step
```

#### Scenario 39.7: A storage failure during the debug state read answers a 500

```gherkin
Given a storage that rejects the read of the story
When the client requests GET /debug/state
Then the response status is "500 Internal Server Error"
```

#### Scenario 39.8: A stop that cannot read the dock leaves the Job running

```gherkin
Given a Game whose Job is held at its narrator call
And a storage that rejects the read of the dock record
When the client POST /status/reset-generating
Then the stop answers "failed"
And the status poll still answers with the step of the Job
When the Job's model answer arrives
Then the Job saves its Narration and the Game is idle
```
