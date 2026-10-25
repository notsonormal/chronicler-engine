# Feature Spec: Browser Worlds

Endpoint: browser DOM.

## Scenarios

#### Scenario 29.2: Changing a world posture select reaches the server

```gherkin
Given the world edit form is open with #world-posture-status rendered
When the client changes the Tense select to "present"
Then the browser POSTs /worlds/test/posture with the posture group
And #world-posture-status shows the text "Saved"
And after a reload the re-opened edit form shows "present" selected
```

#### Scenario 29.3: Cancelling the world edit keeps focus in the panel

```gherkin
Given the world edit form is open
And the Cancel control has focus
When the client activates Cancel
Then the editor slot is empty and the worlds panel shows the world list again
And Cancel sent no request
And focus is on a control inside the worlds panel
```

#### Scenario 29.4: Deleting a world keeps focus in the worlds panel

```gherkin
Given the Worlds tab shows two worlds
And the first world's Delete control has focus
When the client confirms the delete
Then the panel loads and that world's row is removed
And the surviving world's Edit control has focus
When the client confirms the delete of the last world
Then no world row remains
And focus is on a control inside the worlds panel
```

#### Scenario 29.5: A refused world delete reports in the row's own slot and keeps focus

```gherkin
Given the Worlds tab shows two worlds
And the first world's Delete control has focus
When the client confirms a delete the engine refuses because the world still has games
Then that row's own inline slot holds the refusal message
And both rows stay in place, as the same elements
And the Delete control keeps focus
```
