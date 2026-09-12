# Feature Spec: Narrator Mode

Endpoints:
- `POST /games`
- `POST /games/:id/posture`
- `POST /games/:id/mode`
- `POST /action`

## Scenarios

### World-to-game posture inheritance

#### Scenario 23.1: Creating a game in an Interactive Fiction world inherits the IF posture and prompt bundle

```gherkin
Given a seeded world in Interactive Fiction mode with perspective "second" and tense "present"
When the client POST /games with that world and a valid persona
And the client POST /action with command="open the door"
And the pipeline returns to idle
Then the narrator's recorded system prompt carries the Interactive Fiction system preset (the mode bundle's system preset)
And the prompt's perspective macro is resolved to the inherited "second" and the tense macro to the inherited "present"
And the story log contains one Input entry with text "open the door" and at least one Narration entry
```

### Mode switching

#### Scenario 23.2: Switching a Novel game to Interactive Fiction retargets the system preset

```gherkin
Given an active game in Novel mode
And a completed narration whose recorded system prompt carries the Novel system preset
When the client POST /games/{game_id}/mode with narrator_mode="interactive_fiction"
And the client POST /action with command="look"
And the pipeline returns to idle
Then the new narration's recorded system prompt carries the Interactive Fiction system preset
And it no longer carries the Novel system preset
```

#### Scenario 23.3: The mode switch re-renders the posture fragment with the IF bundle and nudges the perspective

```gherkin
Given an active game in Novel mode with perspective "third"
When the client POST /games/{game_id}/mode with narrator_mode="interactive_fiction"
Then the response re-renders the posture controls with narrative_perspective "second" selected (the nudge)
And the system_preset_id select shows the Interactive Fiction bundle's system preset selected
And the next narration's recorded user prompt resolves the perspective macro to "second" (posture macros render into the post-history splice of the user prompt, not the system prompt)
```

#### Scenario 23.4: A deliberately-set perspective survives a mode switch

```gherkin
Given an active game in Novel mode whose perspective was set to "second" through the posture auto-save
When the client POST /games/{game_id}/mode with narrator_mode="interactive_fiction"
Then the re-rendered posture controls still show narrative_perspective "second" selected
And the next narration's recorded user prompt still resolves the perspective macro to "second"
```

### Steering availability

#### Scenario 23.5: Impersonate steering is available in an Interactive Fiction game

```gherkin
Given an active game in Interactive Fiction mode
When the client POST /action with command="/impersonate hello"
And the pipeline returns to idle
Then the story log contains exactly one Input entry with the impersonated flag set and steering instruction "hello"
And no Input entry carries the raw "/impersonate" command text
```
