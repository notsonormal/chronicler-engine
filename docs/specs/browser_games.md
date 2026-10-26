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
And the row that took its place holds keyboard focus on its first control, the Rename control
When the client deletes the last saved game and confirms
Then no saved-game row remains
And keyboard focus is inside the Games panel, or on the panel itself when no control is left in it
```

#### Scenario 27.4: An open rename dialog keeps its typed text and focus through a load of each kind the Games panel lists

```gherkin
Given the Games tab is open and the rename form of a saved Game is open in the edit dialog, with a typed name and focus in its name field
When the number of `games` moves, and the page asks
And the number of `worlds` moves, and the page asks
And the number of `presets` moves, and the page asks
Then the Games panel loads once for each move, and it reports data-live-state "current"
And the dialog is open, the rename form is the same element, and its name field is the same element, with the typed name and focus
```

#### Scenario 27.5: Saving the rename dialog closes it, and the Games panel shows the name

```gherkin
Given the Games tab is open and the rename form of a saved Game is open in the edit dialog, with a typed name
When the client presses Save and the engine answers 200
Then the dialog is closed
And the Games panel loads exactly once, and it reports data-live-state "current"
And the Game's row shows the new name
```

#### Scenario 27.6: The New Game dialog's World and Persona selects keep the player's choice through a load

```gherkin
Given the Games tab is open
And the New Game form is open in the edit dialog
And the client chose a World and a Persona other than the first ones in the New Game form
When the number of `games` moves, and the page asks
And the number of `worlds` moves, and the page asks
And the number of `presets` moves, and the page asks
Then the Games panel loads once for each move, and it reports data-live-state "current"
And the dialog is open, and the World select and the Persona select are the same elements, and show the chosen options
```

#### Scenario 27.7: Cancelling the New Game dialog discards the choice

```gherkin
Given the Games tab is open
And the New Game form is open in the edit dialog
And the client chose a World and a Persona other than the first ones in the New Game form
When the client cancels the dialog
Then the dialog is closed, and its body is empty
When the client opens the New Game form again
Then the World select and the Persona select show their first options
```
