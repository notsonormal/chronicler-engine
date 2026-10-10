# Feature Spec: Browser Games

Endpoint: browser DOM.

## Scenarios

#### Scenario 27.1: Opening the Games tab renders the posture fragment, and a change reaches the server

```gherkin
Given the dashboard is loaded with an active game in Novel mode (Third person, Past)
When the client opens the Games tab
Then #game-posture-controls is rendered with the Narrator Mode, Perspective, and Tense selects
And the System, Quantifier, Impersonate, and Options preset selects are rendered
When the client changes the Tense select to "present"
Then the browser POSTs /games/{id}/posture with the posture group
And after a reload the re-opened Games tab shows "present" selected
```

#### Scenario 27.2: Changing a posture select keeps focus on that select

```gherkin
Given the client opened the Games tab with #game-posture-controls rendered
And the Tense select has focus
When the client changes the Tense select to "present"
Then the posture fragment is replaced
And the Tense select has focus again
```

#### Scenario 27.3: Deleting a saved game keeps keyboard focus in the Games panel

```gherkin
Given the Games tab shows more than one saved game
And a saved game's Delete control has keyboard focus
When the client confirms the delete
Then that game's row is gone from the saved-games list
And the row that took its place holds keyboard focus on its Switch control
When the client deletes the last saved game and confirms
Then no saved-game row remains
And keyboard focus is inside the Games panel
```
