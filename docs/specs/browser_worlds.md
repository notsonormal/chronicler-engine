# Feature Spec: Browser Worlds

Endpoint: browser DOM.

## Scenarios

#### Scenario 29.2: Changing a world posture select reaches the server

```gherkin
Given the world edit form is open with #world-posture-status rendered
When the client changes the Tense select to "present"
Then the browser POSTs /worlds/test/posture with the posture group
And after a reload the re-opened edit form shows "present" selected
```

#### Scenario 29.3: Cancelling the world edit keeps focus in the panel

```gherkin
Given the world edit form is open
And the Cancel control has focus
When the client activates Cancel
Then the worlds panel shows the world list again
And focus is on a control inside the worlds panel
```
