# Feature Spec: Games

Endpoints:
- `POST /games`
- `POST /games/:id/switch`
- `POST /games/:id/delete`

## Scenarios

### Create

#### Scenario 17.1: Creating a game with valid world and persona returns success and refreshes

```gherkin
Given a seeded world with key "test" and a persona with key "test_player"
When the client POST /games with "world_key=test&persona_key=test_player"
Then the response status is "200 OK"
And the response has an "HX-Refresh: true" header
```

#### Scenario 17.2: Creating a game with an unknown world key returns 400

```gherkin
Given a seeded persona with key "test_player"
When the client POST /games with "world_key=no_such_world&persona_key=test_player"
Then the response status is "400 BAD_REQUEST"
And the response body mentions "World not found"
```

#### Scenario 17.3: Creating a game with an unknown persona key returns 400

```gherkin
Given a seeded world with key "test"
When the client POST /games with "world_key=test&persona_key=no_such_persona"
Then the response status is "400 BAD_REQUEST"
And the response body mentions "Persona not found"
```

### Switch

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

### Delete

#### Scenario 19.1: Deleting a non-active game returns success

```gherkin
Given two created games with ids "id1" (active) and "id2"
When the client POST /games/{id2}/delete
Then the response status is "200 OK"
```

#### Scenario 19.2: Deleting the active game returns 400

```gherkin
Given one created game that is the active game
When the client POST /games/{active_id}/delete
Then the response status is "400 BAD_REQUEST"
And the response body mentions "Cannot delete the active game"
```

#### Scenario 19.3: Deleting an unknown game id returns success (idempotent)

```gherkin
When the client POST /games/99999999/delete
Then the response status is "200 OK"
```
