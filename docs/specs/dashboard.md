# Feature Spec: Dashboard

Endpoint: `GET /status/generating` (the dashboard chrome's status poll).

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
