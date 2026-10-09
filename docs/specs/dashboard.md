# Feature Spec: Dashboard

Endpoints:
- `GET /status/generating` (the dashboard chrome's status poll)
- `GET /fragment/options-dock`
- `GET /debug/is_generating`
- `GET /` (the shell that defines the icons)

## Scenarios

#### Scenario 39.1: The generating poll answers from the live generation registry

```gherkin
Given a game whose persisted status is Generating with no live generation slot
When the client requests GET /status/generating
Then the response body is "idle"

Given a game with a live generation slot in phase Narrating
When the client requests GET /status/generating
Then the response body is "narrating"

Given a game with a live generation slot in phase Quantifying
When the client requests GET /status/generating
Then the response body is "quantifying"
```

#### Scenario 39.2: The shell defines each referenced icon exactly once
```gherkin
Given a game with at least 2 Messages and a stored trigger
When the client requests GET / and GET /fragment/story-log, /fragment/games and /fragment/connections/new
Then the shell defines each icon exactly once
And every icon the shell or a fragment named above references is one the shell defines
And each fragment named above shows at least one icon
```

#### Scenario 39.3: The dock and the debug endpoint answer from the live registry

```gherkin
Given a game with a live generation slot and a persisted status of Idle
When the client requests GET /fragment/options-dock and GET /debug/is_generating
Then the dock's controls are disabled
And the debug response body is "true"

Given a game whose persisted status is Generating with no live generation slot
When the client requests GET /fragment/options-dock and GET /debug/is_generating
Then the dock's controls are enabled
And the debug response body is "false"
```
