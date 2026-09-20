# Feature Spec: Browser Worlds

Endpoint: browser DOM.

## Scenarios

#### Scenario 29.2: Changing a world posture select reaches the server

```gherkin
Given the world edit form is open with #world-posture-status rendered
When the client changes the narrative_tense select to "present"
Then the browser POSTs /worlds/test/posture with the posture group
And after a reload the re-opened edit form shows "present" selected
```
