# Feature Spec: Browser Worlds

Endpoint: browser DOM.

## Scenarios

#### Scenario 29.1: The world edit form renders the posture selects

```gherkin
Given the dashboard is loaded with the seeded world "test"
When the client opens the Worlds tab and clicks Edit on the world card
Then the world form renders narrator_mode, narrative_perspective, and narrative_tense selects
And #world-posture-status is rendered
```

#### Scenario 29.2: Changing a world posture select auto-saves with a status report

```gherkin
Given the world edit form is open with #world-posture-status rendered
When the client changes the narrative_tense select to "present"
Then the browser POSTs /worlds/test/posture with the posture group
And #world-posture-status contains "Saved"
```
