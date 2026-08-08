# Feature Spec: Games Delete

Endpoint: `POST /games/:id/delete`

Behavioural authority for deleting a game. Each "When" is an HTTP request;
each "Then" is an HTTP-observable outcome.

Scenario IDs are `19.x` and stay stable across edits. The pilot dedups by ID
across `docs/specs/*.md`, so IDs must stay unique.

## Scenarios

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
