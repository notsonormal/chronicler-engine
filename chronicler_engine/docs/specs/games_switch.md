# Feature Spec: Games Switch

Endpoint: `POST /games/:id/switch`

Behavioural authority for switching the active game. Each "When" is an HTTP
request; each "Then" is an HTTP-observable outcome.

Scenario IDs are `18.x` and stay stable across edits. The pilot dedups by ID
across `docs/specs/*.md`, so IDs must stay unique.

## Scenarios

#### Scenario 18.1: Switching to an existing game returns success and refreshes

```gherkin
Given two created games with ids "id1" and "id2", where "id2" is the active game
When the client POST /games/{id1}/switch
Then the response status is "200 OK"
And the response has an "HX-Refresh: true" header
```

#### Scenario 18.2: Switching to an unknown game id returns 400

```gherkin
When the client POST /games/99999999/switch
Then the response status is "400 BAD_REQUEST"
And the response body mentions "Game not found"
```
