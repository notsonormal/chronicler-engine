# Feature Spec: Worlds

Endpoints:
- `POST /worlds`
- `POST /worlds/:key`
- `POST /worlds/:key/posture`

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

#### Scenario 25.9: A storage failure during world creation renders the error fragment with a 200

A storage failure is not a client refusal, so it keeps the panel's in-fragment rendering: the create handler answers 200 carrying the shared error fragment rather than a 500.

```gherkin
Given a storage that rejects world creation
When the client POST /worlds with a valid World form
Then the response status is "200 OK" (not a 500)
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
