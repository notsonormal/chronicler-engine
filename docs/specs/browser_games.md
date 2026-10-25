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
Then the Games panel loads
And the Tense select is the same element, shows "present" and has focus
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
And keyboard focus is inside the Games panel, or on the panel itself when no control is left in it
```

#### Scenario 27.4: An open rename form keeps its typed text and focus through a load of each kind the Games panel lists

```gherkin
Given the Games tab is open and the rename form of a saved Game is open, with a typed name and focus in its name field
When the number of `games` moves, and the page asks
And the number of `worlds` moves, and the page asks
And the number of `presets` moves, and the page asks
Then the Games panel does not load
And the Games panel reports data-live-state "held"
And the rename form is the same element, it is open, and its name field is the same element, with the typed name and focus
```

#### Scenario 27.5: Ending the rename edit loads the Games panel once

```gherkin
Given the Games tab is open and the rename form of a saved Game is open, with a typed name
And each kind that the Games panel lists moved while the form was open
When the client presses Save and the engine answers 200
Then the Games panel loads exactly once
And the Games panel reports data-live-state "current"
```

#### Scenario 27.6: The New Game form's World and Persona selects keep the player's choice through a load

```gherkin
Given the Games tab is open
And the client chose a World and a Persona other than the first ones in the New Game form
When the number of `games` moves, and the page asks
And the number of `worlds` moves, and the page asks
Then the Games panel does not load, and it reports data-live-state "held"
And the World select and the Persona select are the same elements, and show the chosen options
```
