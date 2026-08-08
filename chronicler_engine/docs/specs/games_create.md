# Feature Spec: Games Create

Endpoint: `POST /games`

Behavioural authority for creating a game. Each "When" is an HTTP request;
each "Then" is an HTTP-observable outcome.

Scenario IDs are `9.x` and stay stable across edits. The pilot dedups by ID
across `docs/specs/*.md`, so IDs must stay unique.

## Scenarios

#### Scenario 9.1: Creating a game with valid world and persona returns success and refreshes

```gherkin
Given a seeded world with key "test" and a persona with key "test_player"
When the client POST /games with "world_key=test&persona_key=test_player"
Then the response status is "200 OK"
And the response has an "HX-Refresh: true" header
```

#### Scenario 9.2: Creating a game with an unknown world key returns 400

```gherkin
Given a seeded persona with key "test_player"
When the client POST /games with "world_key=no_such_world&persona_key=test_player"
Then the response status is "400 BAD_REQUEST"
And the response body mentions "World not found"
```

#### Scenario 9.3: Creating a game with an unknown persona key returns 400

```gherkin
Given a seeded world with key "test"
When the client POST /games with "world_key=test&persona_key=no_such_persona"
Then the response status is "400 BAD_REQUEST"
And the response body mentions "Persona not found"
```
