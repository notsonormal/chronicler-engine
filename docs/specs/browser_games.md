# Feature Spec: Browser Games

Endpoint: browser DOM.

## Scenarios

#### Scenario 27.1: Opening the Games tab renders the posture fragment, and a change reaches the server

```gherkin
Given the dashboard is loaded with an active game in Novel mode (Third person, Past)
When the client opens the Games tab
Then #game-posture-controls is rendered with narrator_mode, narrative_perspective, and narrative_tense selects
And the system_preset_id, quantifier_preset_id, and impersonate_preset_id selects are rendered
When the client changes the narrative_tense select to "present"
Then the browser POSTs /games/{id}/posture with the posture group
And after a reload the re-opened Games tab shows "present" selected
```
