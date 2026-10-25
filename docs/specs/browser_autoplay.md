# Feature Spec: Browser Autoplay

Endpoint: browser DOM.

The Autoplay control sits beside the command form. It holds a budget field `#autoplay-cycles` (default 10, from 1 to 100) and a start button `#autoplay-btn`. The start button posts the budget to `POST /autoplay`.

While an Autoplay run holds the Game, the Game is view-only. The status display shows the progress of the run, and the action area offers `#stop-btn` in place of the command form and the Autoplay control. The story-changing controls of the story log and the config controls of the Games panel are disabled. Reading, scrolling and switching to another Game stay open.

## Scenarios

#### Scenario 42.1: The Autoplay control starts a run with the default budget

```gherkin
Given the dashboard shows a fresh Game and the status display shows "Ready"
Then #autoplay-cycles holds 10
When the client presses #autoplay-btn
Then the engine starts a run with a budget of 10
And #status-display shows "Autoplay: cycle 1/10 Generating narration..." at once, while the first Job waits for the model
```

#### Scenario 42.2: The status display shows the progress of the run and the action area offers only Stop

```gherkin
Given the status display loads the label "Autoplay: cycle 2/10 Generating narration..."
Then #status-display shows "Autoplay: cycle 2/10 Generating narration..."
And #stop-btn is visible
And the command input, the Send button and #autoplay-btn are not visible

When the status display loads the label "Autoplay: cycle 2/10"
Then #status-display shows "Autoplay: cycle 2/10"
And #stop-btn is visible

When the status display loads the label "Ready"
Then #status-display shows "Ready"
And #stop-btn is not visible
And the command input, the Send button and #autoplay-btn are visible and enabled
```

#### Scenario 42.3: While a run holds the Game, the controls that change the story are disabled, and Stop enables them again

```gherkin
Given a Game whose story log holds an Input and a Narration after the opening Narration
And a run that holds the Game, and its first Job waits for the model
Then the Edit, Delete and Retry controls of the story log are disabled
When the client opens the Games tab
Then the Narrator Mode, Perspective, Tense and preset selects of the Games panel are disabled

When the client goes back to the Game tab and presses #stop-btn
Then the status display shows "Ready"
And the command input and the Send button are visible and enabled
And the Edit, Delete and Retry controls of the story log are enabled
When the client opens the Games tab
Then the Narrator Mode, Perspective, Tense and preset selects of the Games panel are enabled
```

#### Scenario 42.4: The Games list marks a Game that a run holds, without a reload

```gherkin
Given the Games panel has loaded, and its list shows no `.run-badge`
When the client goes back to the Game tab and presses #autoplay-btn, and the first Job of the run waits for the model
And the client opens the Games tab, with no reload of the page
Then the item of the active Game carries a `.run-badge`
When the client presses #stop-btn and opens the Games tab again
Then no item carries a `.run-badge`
```

#### Scenario 42.5: A story write that the busy Game refuses keeps the client's work and shows the busy message

```gherkin
Given a Job or a run holds the Game, so each story write answers "Still thinking..."

When the client opens the editor of a story-log entry, types a new text and presses Save
Then the story log's error slot shows a line that names the busy Game
And the editor is still open and holds the typed text
And the story log stays held while the editor is open

When the client presses the Previous swipe control
Then the story log's error slot shows a line that names the busy Game
And the story log still shows the Swipe it showed

When the client presses the Delete control and confirms
Then the story log's error slot shows a line that names the busy Game
And the story log still shows every entry it showed
```

#### Scenario 42.6: A start that the busy Game refuses reports in the Autoplay form's slot

```gherkin
Given a Job or a run holds the Game, so POST /autoplay answers "Still thinking..."
When the client presses #autoplay-btn
Then the Autoplay form's own inline error slot says that the Game is busy
And its Details disclosure holds the engine's answer
And the command input and #autoplay-btn are enabled
And #action-status is empty
```

#### Scenario 42.7: A retry that the busy Game refuses reports in the story log's error slot

```gherkin
Given a Job or a run holds the Game, so POST /swipe/new answers "Still thinking..."
When the client presses the Retry control
Then the story log's error slot shows a line that names the busy Game
And #action-status is empty
And the command input and #submit-btn are enabled
```

