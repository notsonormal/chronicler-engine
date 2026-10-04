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
