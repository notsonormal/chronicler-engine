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
