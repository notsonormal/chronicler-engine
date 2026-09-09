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

### Per-game posture, mode, and presets

#### Scenario 20.1: The games panel renders the posture fragment for the active game

```gherkin
Given the dashboard is loaded with an active game in Novel mode (Third person, Past)
When the client opens the Games tab
Then #game-posture-controls is rendered with narrator_mode, narrative_perspective, and narrative_tense selects
And the system_preset_id, quantifier_preset_id, and impersonate_preset_id selects are rendered
And the selects show the game's current posture and preset ids
```

#### Scenario 20.2: Changing perspective or tense auto-saves and re-renders the fragment

```gherkin
Given #game-posture-controls is rendered for the active game
When the client changes the narrative_tense select to "present"
Then the browser POSTs /games/{game_id}/posture with the posture row
And #game-posture-controls is re-rendered with narrative_tense "present" selected
```

#### Scenario 20.3: Switching narrator mode retargets presets and nudges perspective

```gherkin
Given #game-posture-controls is rendered with the game in Novel mode and perspective "third"
When the client changes the narrator_mode select to "interactive_fiction"
Then #game-posture-controls is re-rendered with narrative_perspective "second" selected
And the system_preset_id select's selected option is the Interactive Fiction bundle's system preset ("system_if_default")
```
