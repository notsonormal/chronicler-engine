# Feature Spec: Worlds

Endpoints:
- `POST /worlds`
- `POST /worlds/:key`
- `POST /worlds/:key/posture`
- `POST /worlds/:key/delete`

## Scenarios

### Generic world update posture contract

#### Scenario 25.1: A post omitting all posture fields preserves the stored posture

```gherkin
Given a seeded world in Interactive Fiction mode with perspective "second" and tense "past"
When the client POST /worlds/{key} with the full form but no posture fields
Then the response is a 200 and the panel re-renders
And the stored world still carries mode "interactive_fiction", perspective "second", and tense "past"
```

#### Scenario 25.2: A partial posture post merges per field

```gherkin
Given a seeded world in Interactive Fiction mode with perspective "second" and tense "past"
When the client POST /worlds/{key} with the form carrying only narrative_tense="present"
Then the stored world's tense is "present"
And its mode and perspective still are "interactive_fiction" and "second"
```

#### Scenario 25.3: An unknown posture value falls back to the domain default

```gherkin
Given a seeded world in Interactive Fiction mode
When the client POST /worlds/{key} with the form carrying narrator_mode="warp_drive"
Then the response is a 200
And the stored world's mode is the Novel default
```

### Options toggle grammar

The options toggle is a urlencoded checkbox: a checked box posts "true"; an unchecked box is absent from the post.

#### Scenario 25.4: The options toggle obeys the checkbox grammar

```gherkin
Given a seeded world with options_always_on disabled
When the client POST /worlds/{key} with the form carrying options_always_on="true"
Then the stored world's options_always_on is true
When the client POST /worlds/{key} again with the form omitting options_always_on
Then the stored world's options_always_on is false
```

### World posture auto-save

The edit form's posture selects post their `closest .posture-group` to `POST /worlds/:key/posture`. The handler patches only the fields present in the post and answers with the `#world-posture-status` fragment. This is the contract the ticket-03 flake surfaced: the browser's `change` event races htmx's trigger attachment, so the contract is verified at request level and the browser keeps only a wiring check.

#### Scenario 25.5: The posture endpoint patches and reports per outcome

```gherkin
Given a seeded world in Interactive Fiction mode with tense "past"
When the client POST /worlds/{key}/posture with a valid tense patch
Then the response is a 200 carrying the "Saved" status span
And the stored world's tense is the patched value
When the client POST /worlds/{key}/posture with an invalid narrator_mode
Then the response is a 200 carrying an error fragment
And the stored world is mutated not at all
When the client POST /worlds/{key}/posture for an unknown key
Then the response is a 400
When the storage rejects the world update
Then the response is a 500
```

#### Scenario 25.6: The world edit form renders the posture selects with the stored values

```gherkin
Given a seeded world with key "posture_world" in Interactive Fiction mode (Second person, Past)
When the client GET /worlds/posture_world/edit
Then the response status is "200 OK"
And the body contains "Edit World" (the edit form, not the create form)
And the narrator_mode, narrative_perspective, and narrative_tense selects are rendered
And the narrator_mode select renders "interactive_fiction" selected
And the narrative_perspective select renders "second" selected
And the narrative_tense select renders "past" selected
And the body contains the #world-posture-status target
And the posture selects auto-save to /worlds/posture_world/posture
```

### World creation refuses an existing identifier

#### Scenario 25.7: Creating a World whose identifier already exists is refused

```gherkin
Given a seeded world with identifier "posture_world", a description and a room map
When the client POST /worlds with a second World whose identifier is "posture_world"
Then the response is a 400
And the body names the identifier and says it already exists
And the stored World keeps its name, description, and map
```

### World creation storage failures

#### Scenario 25.9: A storage failure during world creation renders the error fragment with a 500

```gherkin
Given a storage that rejects world creation
When the client POST /worlds with a valid World form
Then the response status is "500 Internal Server Error"
And the body carries the error fragment naming the storage failure
```

### World list

#### Scenario 25.8: The world list pluralises the game count

```gherkin
Given a seeded World with exactly one game
When the client GET /fragment/worlds
Then the response status is "200 OK"
And the World's card shows "1 game" and not "1 games"
When a second game is created in that World
And the client GET /fragment/worlds
Then the World's card shows "2 games"
```

### World delete

#### Scenario 25.10: The refusal of a World delete pluralises the game count

```gherkin
Given a seeded World with exactly one game
When the client POST /worlds/{key}/delete
Then the response is a 200 that holds the refusal "Cannot delete world with 1 game"
And the refusal does not read "1 games"
When a second game is created in that World
And the client POST /worlds/{key}/delete
Then the refusal reads "Cannot delete world with 2 games"
And the World is still stored
```

#### Scenario 25.11: A World delete removes a World with no games, and an unknown key changes nothing

```gherkin
Given a seeded World "posture_world" with no games
When the client POST /worlds/posture_world/delete
Then the response is a 200 with an empty body
And the World is no longer stored
When the client POST /worlds/nonexistent/delete
Then the response is a 200 with an empty body
```

#### Scenario 25.12: A storage failure during a World delete renders the error fragment with a 500

```gherkin
Given a storage that rejects the World delete
When the client POST /worlds/{key}/delete
Then the response status is "500 Internal Server Error"
And the body carries the error fragment naming the storage failure
```
