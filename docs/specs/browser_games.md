# Feature Spec: Browser Games

Endpoint: browser DOM.

## Scenarios

#### Scenario 27.1: The games panel renders the posture fragment for the active game

```gherkin
Given the dashboard is loaded with an active game in Novel mode (Third person, Past)
When the client opens the Games tab
Then #game-posture-controls is rendered with narrator_mode, narrative_perspective, and narrative_tense selects
And the system_preset_id, quantifier_preset_id, and impersonate_preset_id selects are rendered
And the selects show the game's current posture and preset ids
```

#### Scenario 27.2: Changing perspective or tense auto-saves and re-renders the fragment

```gherkin
Given #game-posture-controls is rendered for the active game
When the client changes the narrative_tense select to "present"
Then the browser POSTs /games/{game_id}/posture with the posture row
And #game-posture-controls is re-rendered with narrative_tense "present" selected
```

#### Scenario 27.3: Switching narrator mode retargets presets and nudges perspective

```gherkin
Given #game-posture-controls is rendered with the game in Novel mode and perspective "third"
When the client changes the narrator_mode select to "interactive_fiction"
Then #game-posture-controls is re-rendered with narrative_perspective "second" selected
And the system_preset_id select's selected option is the Interactive Fiction bundle's system preset ("system_if_default")
```
