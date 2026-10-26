# Feature Spec: Dashboard

Endpoints:
- `GET /status/busy` (the status display)
- `POST /status/reset-busy` (the stop)
- `GET /fragment/options-dock`
- `GET /debug/is_busy`
- `GET /debug/state`
- `GET /` (the shell that defines the icons)

## Scenarios

#### Scenario 39.1: The status display answers with the label of a running Job, else the failure line, else Ready

The answer is one element. Its `data-status` attribute holds the machine state: `idle`, `thinking`, `autoplay` or `failed`. The label inside it is the text the status display shows.

```gherkin
Given a Game whose Job is at the narrator call of a Narration
When the client requests GET /status/busy
Then the response body is a status element with data-status "thinking" and the label "Generating narration..."

Given a Game whose Job is at its analysis
When the client requests GET /status/busy
Then the response body is a status element with data-status "thinking" and the label "Quantifying..."

Given a Game with no Job and a failure line
When the client requests GET /status/busy
Then the response body carries data-status "failed"
And the response body carries that failure line

Given a Game with no Job and no failure line
When the client requests GET /status/busy
Then the response body is a status element with data-status "idle" and the label "Ready"
```

#### Scenario 39.2: The shell defines each referenced icon exactly once
```gherkin
Given a game with at least 2 Messages and a stored trigger
When the client requests GET / and GET /fragment/story-log and /fragment/games
Then the shell defines each icon exactly once
And every icon the shell or a fragment named above references is one the shell defines
And each fragment named above shows at least one icon
```

#### Scenario 39.3: The dock and the debug endpoint answer from the busy state

```gherkin
Given a Game with an offered set and a Job
When the client requests GET /fragment/options-dock and GET /debug/is_busy
Then the dock's controls are disabled
And the debug response body is "true"

Given a Game with an offered set and no Job
When the client requests GET /fragment/options-dock and GET /debug/is_busy
Then the dock's controls are enabled
And the debug response body is "false"
```

#### Scenario 39.4: A stop at each model call leaves the records of the step it reached

```gherkin
Given a Game whose Job is held at one model call of the Examples table below
And the status display answers with the label of the step of that row
When the client POST /status/reset-busy
Then the stop answers "reset"
And the Game is idle
And the status display answers with no failure line
When the Job's model answer arrives after the stop
Then the story log holds exactly the records of that row

Examples:
| Kind of Job | Model call | Status while held | Records in the story log after the stop |
| Narration | the narrator call | Generating narration... | the Input of the Job, and no Narration |
| Narration | the analysis call | quantifying | the Input of the Job and the Narration, with no room on the Narration and no event Narration |
| Narration | the Options call | options | the Input of the Job, the Narration, and the room the Narration arrived in |
| Impersonate | the narrator call | narrating | no Message of the Job |
| Impersonate | the analysis call | quantifying | the Input of the Job and no Narration |
| Retry of a typed Input | the narrator call | narrating | the Messages of the story before the Retry |
| event Narration | the event narrator call | generating-event | the Narration that fired the Trigger, and no event Narration |
| event Narration | the analysis call | quantifying | the Input of the Job, the Narration that fired the Trigger and the event Narration, with the one-shot Trigger fired in the state of the event Narration |

A stop inside a transaction is not observable: a transaction takes milliseconds, so a stop just before one leaves the records of the model call before it.
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
Then the response reports the status "Busy" and the Step "Narrating"

Given a Game with no Job and a failure line
When the client requests GET /debug/state
Then the response reports the status "Failed" with the kind of that failure line, and no Step

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
When the client POST /status/reset-busy
Then the stop answers "failed"
And the status display still answers with the step of the Job
When the Job's model answer arrives
Then the Job saves its Narration and the Game is idle
```

#### Scenario 39.9: A status display that cannot read the Game answers with a failure line

```gherkin
Given a Game with no Job
And a storage that rejects the read of the Game
When the client requests GET /status/busy
Then the response body carries the line "The data for this request is missing: the game."
And the response body is not a status element with data-status "idle"
```

#### Scenario 39.10: With no active Game, the stop answers "failed"

```gherkin
Given no active Game
When the client POST /status/reset-busy
Then the stop answers "failed"
```

#### Scenario 39.11: With no active Game, the status display answers Ready

```gherkin
Given no active Game
When the client requests GET /status/busy
Then the response body is a status element with data-status "idle" and the label "Ready"
```
