# Feature Spec: Games

Endpoints:
- `POST /games`
- `POST /games/:id/switch`
- `POST /games/:id/rename`
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

#### Scenario 18.3: Switching to a game heals a stale generating status

```gherkin
Given two created games with ids "id1" (active) and "id2"
And "id2" carries a persisted "Generating" status with no live generation slot
When the client POST /games/{id2}/switch
Then the response status is "200 OK"
And the response has an "HX-Refresh: true" header
And the active game's persisted status is "idle"
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

#### Scenario 19.4: Deleting the last saved game answers with the list's empty state

```gherkin
Given two created games with ids "id1" (active) and "id2"
When the client POST /games/{id2}/delete
Then the response status is "200 OK"
And the response body renders the Saved Games empty state
Given a third created game with id "id3"
When the client POST /games/{id3}/delete
Then the response status is "200 OK"
And the response body is empty
```

### Rename

#### Scenario 17.4: Renaming a game sets its display name, shown in the header and the Games tab

```gherkin
Given a seeded world with key "test" and an active game
When the client POST /games/{active_id}/rename with display_name="The Long Road"
Then the response status is "200 OK"
And the response has an "HX-Refresh: true" header
And the active game's display name is "The Long Road"
And the active game's stable name is unchanged
And a GET /fragment/header renders "The Long Road" as the game name
And a GET /fragment/games renders "The Long Road" as the active game name
```

#### Scenario 17.5: Renaming a game to a blank display name is rejected

```gherkin
Given a seeded world with key "test" and an active game
When the client POST /games/{active_id}/rename with display_name="   "
Then the response status is "400 BAD_REQUEST"
And the response body mentions "Display name cannot be empty"
```

### Per-game posture, mode, and presets

#### Scenario 20.4: A posture auto-save that fails storage surfaces a 500 error fragment

```gherkin
Given #game-posture-controls is rendered for the active game
And the storage rejects game-config writes
When the client POST /games/{game_id}/posture with a valid posture row
Then the response status is "500 INTERNAL_SERVER_ERROR"
And the response body is an error fragment naming the storage failure
```

#### Scenario 20.5: A presets auto-save that fails storage surfaces a 500 error fragment

```gherkin
Given the per-game preset picker is rendered for the active game
And the storage rejects game-config writes
When the client POST /games/{game_id}/presets with a valid preset selection
Then the response status is "500 INTERNAL_SERVER_ERROR"
And the response body is an error fragment naming the storage failure
```

#### Scenario 20.6: A mode switch that fails storage surfaces a 500 error fragment

```gherkin
Given the active game is in Novel mode
And the storage rejects game-config writes
When the client POST /games/{game_id}/mode with narrator_mode="interactive_fiction"
Then the response status is "500 INTERNAL_SERVER_ERROR"
And the response body is an error fragment naming the storage failure
```

#### Scenario 20.7: A perspective or tense auto-save re-renders the fragment with the new value selected

```gherkin
Given #game-posture-controls is rendered for the active game
When the client POST /games/{game_id}/posture with narrative_perspective="third" and narrative_tense="present"
Then the response status is "200 OK"
And the response body contains id="game-posture-controls" (the re-rendered fragment)
And the narrative_tense select renders "present" selected
And the narrative_perspective select renders "third" selected
And game_catalogue.current_game() reports tense "present" and perspective "third"
```

#### Scenario 20.8: The games fragment renders the posture controls for the active game

```gherkin
Given a seeded world with key "test" and an active game in Novel mode (Third person, Past)
When the client GET /fragment/games
Then the response status is "200 OK"
And the body contains id="game-posture-controls"
And the narrator_mode select renders "novel" selected
And the narrative_perspective select renders "third" selected
And the narrative_tense select renders "past" selected
And the system_preset_id, quantifier_preset_id, impersonate_preset_id, and options_preset_id selects are rendered
And the selects auto-save to /games/{active_game_id}/mode, /posture, and /presets
```

#### Scenario 20.11: The per-game Options selector persists the chosen Options preset

```gherkin
Given #game-posture-controls is rendered for the active game
When the client POST /games/{game_id}/presets with options_preset_id set to a different Options preset
Then the response status is "200 OK"
And the response body re-renders id="game-posture-controls" with that Options preset selected
And the active game's Options preset is the chosen one
```

#### Scenario 20.9: The games fragment lists only the games other than the active one

```gherkin
Given a seeded world with key "test" and an active game in Novel mode
When the client GET /fragment/games
Then the response status is "200 OK"
And the Saved Games section says there are no other saved games (the active game is the only game)
And the active game card carries no status badge repeating the section heading
```

#### Scenario 20.10: The games fragment lists the other saved games and excludes the active one

```gherkin
Given a seeded world with key "test", an active game, and a second saved game
When the client GET /fragment/games
Then the response status is "200 OK"
And the Saved Games section lists the second game
And the Saved Games section does not list the active game
```
