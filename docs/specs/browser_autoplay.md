# Feature Spec: Browser Autoplay

Endpoint: browser DOM.

The Autoplay control sits beside the command form. It holds a budget field `#autoplay-cycles` (default 10, from 1 to 100) and a start button `#autoplay-btn`. The start button posts the budget to `POST /autoplay`.

While a run holds the Game, the Game is view-only. The status display shows the progress of the run, and the action area offers `#stop-btn` in place of the command form and the Autoplay control. The story-changing controls of the story log and the config controls of the Games panel are disabled. Reading, scrolling and switching to another Game stay open.

## Scenarios

#### Scenario 42.1: The Autoplay control starts a run with the default budget

```gherkin
Given the dashboard shows a fresh Game and the status display shows "Ready"
Then #autoplay-cycles holds 10
When the client presses #autoplay-btn
Then the engine starts a run with a budget of 10
And #status-display shows "Autoplay: cycle 1/10 Generating narration..." while the first Job waits for the model
```

#### Scenario 42.2: The status display shows the progress of the run and the action area offers only Stop

```gherkin
Given the status poll answers "autoplay 2/10 narrating"
Then #status-display shows "Autoplay: cycle 2/10 Generating narration..."
And #stop-btn is visible
And the command input, the Send button and #autoplay-btn are not visible

When the status poll answers "autoplay 2/10"
Then #status-display shows "Autoplay: cycle 2/10"
And #stop-btn is visible

When the status poll answers "idle"
Then #status-display shows "Ready"
And #stop-btn is not visible
And the command input, the Send button and #autoplay-btn are visible and enabled
```

#### Scenario 42.3: While a run holds the Game, the controls that change the story are disabled

```gherkin
Given a Game whose story log holds an Input and a Narration after the opening Narration
And a run that holds the Game
Then each Edit, Delete and Retry control of the story log is disabled
And the Narrator Mode, Perspective, Tense and preset selects of the Games panel are disabled
```

#### Scenario 42.4: Stop ends the run and gives the Game back

```gherkin
Given a run holds the Game, and its first Job waits for the model
When the client presses #stop-btn
Then the status display shows "Ready"
And the command input and the Send button are visible and enabled
And each Edit, Delete and Retry control of the story log is enabled
And the Narrator Mode select of the Games panel is enabled
```

#### Scenario 42.5: The Games list marks a Game that a run holds, without a reload

```gherkin
Given the Games panel has loaded, and its list shows no `.run-badge`
When the client goes back to the Game tab and presses #autoplay-btn, and the first Job of the run waits for the model
And the client opens the Games tab, with no reload of the page
Then the item of the active Game carries a `.run-badge`
When the client presses #stop-btn and opens the Games tab again
Then no item carries a `.run-badge`
```

#### Scenario 42.6: A save that the busy Game refuses keeps the editor open

```gherkin
Given the client opened the editor of a story-log entry and typed a new text
And a Job or a run holds the Game, so the save answers "Still thinking..."
When the client presses Save
Then the story log's error slot shows a line that names the busy Game
And the editor is still open and holds the typed text
And the story-log poll stays paused
```
