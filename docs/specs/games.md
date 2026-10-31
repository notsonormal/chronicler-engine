# Feature Spec: Games

Endpoints:
- `GET /fragment/games/new`
- `GET /fragment/games/:id/rename`
- `POST /games`
- `POST /games/:id/switch`
- `POST /games/:id/rename`
- `POST /games/:id/delete`

## Scenarios

### Create

#### Scenario 17.1: Creating a game with valid world and persona answers nothing and makes the new Game active

```gherkin
Given a seeded world with key "test" and a persona with key "test_player"
When the client POST /games with "world_key=test&persona_key=test_player"
Then the response status is "200 OK"
And the response body is empty
And the response has no "HX-Refresh" header
And the new Game is the active Game
And the new Game's story log holds its opening Narration
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

#### Scenario 18.1: Switching to an existing game answers nothing and makes that Game active

```gherkin
Given two created games with ids "id1" and "id2", where "id2" is the active game
When the client POST /games/{id1}/switch
Then the response status is "200 OK"
And the response body is empty
And the response has no "HX-Refresh" header
And the active game is "id1"
```

#### Scenario 18.2: Switching to an unknown game id returns 400

```gherkin
When the client POST /games/99999999/switch
Then the response status is "400 BAD_REQUEST"
And the response body mentions "Game not found"
```

#### Scenario 18.4: A Job goes on for its own Game while another Game is active

```gherkin
Given a Game A whose Job is held at its narrator call
And a Game B created from Game A's World, whose Job is held at its narrator call
When both Jobs' model answers arrive
And the client switches back to Game A
Then Game A's story log holds its own Narration and no Narration of Game B
And Game B's story log holds its own Narration and no Narration of Game A
```

#### Scenario 18.5: A switch that the storage rejects answers 500 and the active Game stays

```gherkin
Given two created games with ids "id1" and "id2", where "id2" is the active game
And the storage rejects the write that records a switch to a Game
When the client POST /games/{id1}/switch
Then the response status is "500 INTERNAL_SERVER_ERROR"
And the response body is an error fragment naming the storage failure
And the active game is still "id2"
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

#### Scenario 19.4: A delete answers nothing, and the Games list shows the empty state once the last saved game is gone

```gherkin
Given three created games, one of them active, and two saved games "id2" and "id3"
When the client POST /games/{id2}/delete
Then the response status is "200 OK"
And the response body is empty
And a GET /screen?parts=games lists "id3" and shows no empty state

When the client POST /games/{id3}/delete
Then the response status is "200 OK"
And the response body is empty
And a GET /screen?parts=games renders the Saved Games empty state
```

### Rename

#### Scenario 17.4: Renaming a game answers nothing and sets its display name, shown in the header and the Games tab

```gherkin
Given a seeded world with key "test" and an active game
When the client POST /games/{active_id}/rename with display_name="The Long Road"
Then the response status is "200 OK"
And the response body is empty
And the response has no "HX-Refresh" header
And the active game's display name is "The Long Road"
And the active game's stable name is unchanged
And a GET /screen?parts=header renders "The Long Road" as the game name
And a GET /screen?parts=games renders "The Long Road" as the active game name
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

#### Scenario 20.7: A perspective or tense auto-save answers nothing and the fragment renders the new value selected

```gherkin
Given #game-posture-controls is rendered for the active game
When the client POST /games/{game_id}/posture with narrative_perspective="third" and narrative_tense="present"
Then the response status is "200 OK"
And the response body is empty
And game_catalogue.current_game() reports tense "present" and perspective "third"
And a GET /screen?parts=games renders the narrative_tense select with "present" selected
And it renders the narrative_perspective select with "third" selected
```

#### Scenario 20.8: The games fragment renders the posture controls for the active game

```gherkin
Given a seeded world with key "test" and an active game in Novel mode (Third person, Past)
When the client GET /screen?parts=games
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
And the response body is empty
And the active game's Options preset is the chosen one
And a GET /screen?parts=games renders the Options select with that Options preset selected
```

#### Scenario 20.9: The games fragment lists only the games other than the active one

```gherkin
Given a seeded world with key "test" and an active game in Novel mode
When the client GET /screen?parts=games
Then the response status is "200 OK"
And the Saved Games section says there are no other saved games (the active game is the only game)
And the active game card carries no status badge repeating the section heading
```

#### Scenario 20.10: The games fragment lists the other saved games and excludes the active one

```gherkin
Given a seeded world with key "test", an active game, and a second saved game
When the client GET /screen?parts=games
Then the response status is "200 OK"
And the Saved Games section lists the second game
And the Saved Games section does not list the active game
```

#### Scenario 20.12: A config auto-save with a value the engine rejects answers 400 with the reason

```gherkin
Given #game-posture-controls is rendered for the active game
When the client POST /games/{game_id}/mode with narrator_mode="no_such_mode"
Then the response status is "400 BAD_REQUEST"
And the response body is an error fragment naming the rejected value
And the active game's Narrator Mode is unchanged
```

#### Scenario 20.13: The games fragment is the content of the Games panel, with a stable id on every Game row

```gherkin
Given a seeded world with key "test", an active game, and a second saved game
When the client GET /screen?parts=games
Then the response status is "200 OK"
And the body does not hold an element with the class "games-panel"
And the active game row and the row of the second game each carry the id "game-row-{game_id}"
And each Game row has a Rename control that loads `/fragment/games/{game_id}/rename` into `#edit-dialog-body`
And the fragment has a New Game control that loads `/fragment/games/new` into `#edit-dialog-body`
And the fragment holds no form that waits for a Save and no disclosure
```

#### Scenario 20.37: The Rename form is a dialog fragment with the Game's display name

```gherkin
Given a seeded Game with the display name "The Long Road"
When the client GET /fragment/games/{game_id}/rename
Then the response status is "200 OK"
And the body is a dialog form (`data-dialog-form`) with the title `edit-dialog-title`
And the body has a form that posts to /games/{game_id}/rename
And the form has a display_name field holding "The Long Road"
And the form has an error slot and a Cancel control that closes the dialog
```

#### Scenario 20.38: The Rename form of an unknown Game answers 404

```gherkin
Given a fresh app state
When the client GET /fragment/games/9999/rename
Then the response status is "404 NOT_FOUND"
And the body is an error fragment that names the missing Game
```

#### Scenario 20.39: The New Game form lists the Worlds and the Personas

```gherkin
Given a seeded world with key "test" and a persona with key "test_player"
When the client GET /fragment/games/new
Then the response status is "200 OK"
And the body is a dialog form (`data-dialog-form`) with the title `edit-dialog-title`
And the body has a form that posts to /games
And the form has a World select with the option "test" and a Persona select with the option "test_player"
And the form has an error slot and a Cancel control that closes the dialog
```

#### Scenario 20.40: The New Game form has no form when no World exists

```gherkin
Given a fresh app state with no World
When the client GET /fragment/games/new
Then the response status is "200 OK"
And the body says that no World is available
And the body has no form
```

### Pages of saved Games

The Saved Games section lists 50 Games for each page, newest `updated_at` first. A Game with the same `updated_at` as another lists after the Game with the greater id. The active Game is not in the list: it shows in the Active Game section above. The page is asked for with `games_page` on `GET /screen`. A request without it asks for page 1. A section with more than one page shows the pager with Previous, Next and the page position.

#### Scenario 20.41: The first page lists 50 saved Games and offers the next page

```gherkin
Given an active Game and 51 saved Games
When the client GET /screen?parts=games
Then the response status is "200 OK"
And the Saved Games section lists 50 Games, the newest first
And the Active Game section lists the active Game, and the Saved Games section does not
And the pager (`games-pager`) shows "Page 1 of 2"
And the Previous control is inert (`aria-disabled="true"`)
And the Next control is enabled and goes to page 2
```

#### Scenario 20.42: The last page lists the Games that remain and offers the previous page

```gherkin
Given an active Game and 51 saved Games
When the client GET /screen?parts=games&games_page=2
Then the response status is "200 OK"
And the Saved Games section lists one Game, the oldest
And the pager shows "Page 2 of 2"
And the Next control is inert (`aria-disabled="true"`)
And the Previous control is enabled and goes to page 1
```

#### Scenario 20.43: A Game moves to the first page when a write updates it

```gherkin
Given an active Game and 51 saved Games, where the oldest is on page 2
When the client POST /games/{oldest_id}/rename with display_name="Renamed Oldest"
And the client GET /screen?parts=games
Then the first Game of the Saved Games section is "Renamed Oldest"
When the client GET /screen?parts=games&games_page=2
Then the Saved Games section lists the second-oldest Game
```

#### Scenario 20.44: A page past the end shows the last page

```gherkin
Given an active Game and 51 saved Games
When the client GET /screen?parts=games&games_page=9
Then the response status is "200 OK"
And the pager shows "Page 2 of 2"
And the Saved Games section lists one Game
```

#### Scenario 20.45: A list of 50 saved Games is one page with no pager

```gherkin
Given an active Game and 50 saved Games
When the client GET /screen?parts=games
Then the Saved Games section lists 50 Games
And the body has no pager (`games-pager`)
```
