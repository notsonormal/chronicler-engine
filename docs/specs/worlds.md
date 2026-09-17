# Feature Spec: Worlds

Endpoints:
- `POST /worlds/:key`
- `POST /worlds/:key/posture`

## Scenarios

### Generic world update posture contract

The generic update form always carries the full world surface. When a posture field is absent from the post, the stored value survives; when present, it replaces the stored value per field.

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
Then the response is a 200 carrying an error span
And the stored world is mutated not at all
When the client POST /worlds/{key}/posture for an unknown key
Then the response is a 400
When the storage rejects the world update
Then the response is a 500
```
