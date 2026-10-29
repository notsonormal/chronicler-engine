# Feature Spec: Browser Story Log

Endpoint: browser DOM.

## Scenarios

#### Scenario 30.1: Clicking the edit button activates edit mode

```gherkin
Given a story log with at least one .log-entry rendered
When the client clicks .edit-btn
Then #edit-textarea appears in the DOM
```

#### Scenario 30.2: Cancelling edit restores the original text

```gherkin
Given edit mode is active (#edit-textarea visible)
And the textarea text has been modified
When the client clicks .cancel-btn
Then #edit-textarea is removed from the DOM
And .log-entry .text inner text is restored to the original
```

#### Scenario 30.3: An entry in edit survives loads of the story log, and the rest of the log loads

```gherkin
Given edit mode is active (#edit-textarea visible) on a .log-entry, with a typed text
When the story number moves, and a new narration entry exists
Then the story log loads
And the new entry shows in the log
And #edit-textarea remains in the DOM with the typed text
And the entry in edit carries data-editing, and #story-log carries data-editing-entry
```

#### Scenario 30.4: A failed save restores the entry and removes the edit marks

```gherkin
Given edit mode is active (#edit-textarea visible) on a .log-entry
And the textarea text has been modified
When the client clicks .save-btn and the save request fails
Then #notice shows the first line of the server's answer, "The edit was not saved."
And its Details disclosure holds the raw server text
And #edit-textarea is removed from the DOM
And .log-entry .text inner text is restored to the original
And the entry's pre-edit action controls are available again
And no .log-entry carries data-editing, and #story-log does not carry data-editing-entry
```

#### Scenario 30.5: A failed retry reports in the notice and re-enables Send

```gherkin
Given the dashboard is idle and #submit-btn is enabled
When the client clicks the retry swipe button and the retry request fails
Then #notice shows "The Retry was not started."
And the raw error text is not visible until the client opens the Details disclosure
And the status display does not show "Thinking..."
And #submit-btn is enabled
```

#### Scenario 30.6: The edit textarea fits the entry and takes focus

```gherkin
Given a story log with at least one .log-entry rendered
When the client clicks .edit-btn
Then #edit-textarea takes focus
And #edit-textarea grows to fit the entry text up to the 50vh cap, scrolling internally beyond it
```

#### Scenario 30.7: Escape cancels edit mode

```gherkin
Given edit mode is active (#edit-textarea visible)
And the textarea text has been modified
And the story number moved during the edit
When the client presses Escape in #edit-textarea
Then #edit-textarea is removed from the DOM
And .log-entry .text inner text is restored to the original
And the entry's pre-edit action controls are available again
And the entry's edit button has focus
And the edit button still has focus after the load that the end of the edit allows
```

#### Scenario 30.8: Ctrl+Enter saves the edit

```gherkin
Given edit mode is active (#edit-textarea visible)
And the textarea text has been modified
When the client presses Ctrl+Enter (or Cmd+Enter) in #edit-textarea
Then the client sends the save request for the entry, with the text of #edit-textarea
When a save has succeeded and the load that shows it has not run yet
And the client presses Ctrl+Enter in the old #edit-textarea
Then the client sends no second save request
```

#### Scenario 30.9: Editing locks the entry's swipe controls

```gherkin
Given a story log with at least one .log-entry rendered
When the client clicks .edit-btn
Then the entry's swipe controls show as locked, and a click on one of them does nothing
When the client cancels the edit
Then the entry's pre-edit action controls are restored immediately
And its swipe controls work again
```

#### Scenario 30.10: A failed retrigger reports the failure and re-enables Send

```gherkin
Given the dashboard is idle with Send enabled
And a retrigger control is available on the last narration
When the client clicks the retrigger control and the retrigger request fails
Then #notice shows "The retrigger was not started."
And the raw error text is not visible until the client opens the Details disclosure
And the client sent the request to the retrigger endpoint
And the status display does not show "Thinking..."
And #submit-btn is enabled
```

#### Scenario 30.11: A text selection in the log survives a load

```gherkin
Given a story log with at least one .log-entry rendered
And no edit is active
And part of an entry's text is selected
When the story number moves twice and the story log loads after each move
Then the same text is still selected
And the entries are the same nodes that held the selection
```

#### Scenario 30.12: Focus inside a log entry survives a load

```gherkin
Given a story log with at least one .log-entry rendered
And no edit is active
And a control inside an entry has keyboard focus
When the story number moves twice and the story log loads after each move
Then the same control still has keyboard focus
And the entry is the same node the control belonged to
```

#### Scenario 30.13: The story log is keyboard-scrollable

```gherkin
Given a story log whose content overflows its container
When the client focuses #story-log
Then #story-log has an accessible name
And pressing ArrowDown scrolls the log's content
```

#### Scenario 30.14: A load that drops the oldest entry keeps the rest

```gherkin
Given a story log showing an oldest entry the next load no longer returns
When the load renders the shorter log
Then the dropped entry is removed from the DOM
And the entries that remain are the same nodes as before the load
```

#### Scenario 30.15: Editing locks the other entries' edit controls

```gherkin
Given a story log with at least two .log-entry rendered
When the client clicks .edit-btn on one entry
Then #story-log carries data-editing-entry
And every other entry's .edit-btn shows as locked, and a click on one of them opens no second editor
When the client cancels the edit
Then #story-log does not carry data-editing-entry
And every other entry's .edit-btn works again
```

#### Scenario 30.16: A failed save releases the edit lock

```gherkin
Given edit mode is active on one entry
And #story-log carries data-editing-entry
When the client saves the edit and the save request fails
Then #story-log does not carry data-editing-entry
And every other entry's .edit-btn works again
```

#### Scenario 30.17: A successful save holds the edit lock until the load re-renders

```gherkin
Given edit mode is active on one entry
And #story-log carries data-editing-entry
When the client saves the edit and the save request succeeds
Then #story-log keeps data-editing-entry while the entry still shows its editor
And the swipe controls stay locked while the entry still shows its editor
And the load that follows the save re-renders the log and removes data-editing-entry
```

#### Scenario 30.18: A failed delete reports in the notice

```gherkin
Given a story log with more than one .log-entry rendered
When the client deletes the last entry and the server fails the delete
Then #notice shows "The Message was not deleted."
And its Details disclosure holds the raw server text
And the story log still shows the entries it showed before
```

#### Scenario 30.19: A failed swipe switch reports in the notice

```gherkin
Given a story log whose last entry offers a previous-swipe control
When the client activates that control and the server fails the switch
Then #notice shows "The Swipe was not switched."
And its Details disclosure holds the raw server text
And the story log still shows the entry it showed before
```

#### Scenario 30.20: A load that appends an entry follows the log's bottom

```gherkin
Given a story log whose content overflows its container
And #story-log is scrolled to its bottom
When a load appends an entry
Then #story-log is scrolled to its bottom
And the appended entry is in view
```

#### Scenario 30.21: A load that appends an entry leaves a scrolled-up log alone

```gherkin
Given a story log whose content overflows its container
And #story-log is scrolled away from its bottom
When a load appends an entry
Then #story-log keeps the scroll position it had
And the bottom of the appended entry stays below the fold
```

#### Scenario 30.22: A successful delete keeps keyboard focus in the log

```gherkin
Given a story log with more than one .log-entry rendered
And the last entry's Delete control has keyboard focus
When the client confirms the delete and the delete succeeds
Then the deleted entry is gone from the log
And the entry that is now last holds keyboard focus on its Edit control
```

#### Scenario 30.23: The first load swap leaves the log at its bottom

```gherkin
Given a story log whose content overflows its container
When the log's first load swap renders it
Then #story-log is scrolled to its bottom
And the newest entry is in view
```

#### Scenario 30.24: The Save of an edit looks disabled while a Job or an Autoplay run holds the Game

```gherkin
Given edit mode is active on one entry
When the status display shows a Job in progress (data-status "thinking")
Then the Save control shows as disabled
And its hover text says "Wait for the narration to finish"
When the status display shows an Autoplay run (data-status "autoplay")
Then the Save control shows as disabled
When the status display shows Ready (data-status "idle")
Then the Save control shows as enabled
When the status display shows a failure (data-status "failed")
Then the Save control shows as enabled
```

#### Scenario 30.25: An invalid answer to a Save reports in the slot of the entry and keeps the draft

```gherkin
Given edit mode is active on one entry, with a typed text
And the engine answers the save with 400
When the client clicks the Save of the entry in edit
Then the slot of the entry in edit shows "The edit was not saved: " and the engine's message
And #notice shows no report
And #edit-textarea holds the typed text
And the entry carries data-editing, and #story-log carries data-editing-entry
When the client clicks the Cancel of the entry in edit
Then the slot of the entry holds no report
```

#### Scenario 30.26: A delete of a Message that is no longer the last one reports in the notice

```gherkin
Given a story log with more than one .log-entry rendered
And a Narration was added after that render
When the client deletes the last entry of the render and the engine refuses the delete as stale
Then #notice shows "The last Message changed, so the Message was not deleted."
And its Details disclosure holds the engine's line
And the next load shows the added Narration and every entry of before
```
