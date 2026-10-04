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

#### Scenario 30.3: Edit textarea persists across polling cycles

```gherkin
Given edit mode is active (#edit-textarea visible)
When 3 seconds elapse (client-side polling cycles run)
Then #edit-textarea remains in the DOM (polling does not destroy edit state)
```

#### Scenario 30.4: A failed save restores the entry and resumes polling

```gherkin
Given edit mode is active (#edit-textarea visible) on a .log-entry
And the textarea text has been modified
When the client clicks .save-btn and the save request fails
Then #error-notification is visible
And #edit-textarea is removed from the DOM
And .log-entry .text inner text is restored to the original
And the entry's pre-edit action controls are available again
And #story-log resumes polling
```

#### Scenario 30.5: A failed retry clears the pending status and re-enables Send

```gherkin
Given the dashboard is idle and #submit-btn is enabled
When the client clicks the retry swipe button and the retry request fails
Then #error-notification is visible
And #status-display no longer shows "Thinking..."
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
When the client presses Escape in #edit-textarea
Then #edit-textarea is removed from the DOM
And .log-entry .text inner text is restored to the original
And the entry's pre-edit action controls are available again
And the entry's edit button has focus
And the edit button still has focus after the resumed polling cycle
```

#### Scenario 30.8: Ctrl+Enter saves the edit

```gherkin
Given edit mode is active (#edit-textarea visible)
And the textarea text has been modified
When the client presses Ctrl+Enter (or Cmd+Enter) in #edit-textarea
Then the client sends the save request for the entry
```

#### Scenario 30.9: Editing locks the entry's other controls

```gherkin
Given a story log with at least one .log-entry rendered
When the client clicks .edit-btn
Then the entry's swipe controls are disabled
When the client cancels the edit
Then the entry's pre-edit action controls are restored immediately
```

#### Scenario 30.10: A failed retrigger reports the failure and restores the ready state

```gherkin
Given the dashboard is idle with Send enabled
And a retrigger control is available on the last narration
When the client clicks the retrigger control and the retrigger request fails
Then #error-notification is visible
And the client sent the request to the retrigger endpoint
And #status-display no longer shows "Thinking..."
And #submit-btn is enabled
```

#### Scenario 30.11: A text selection in the log survives the poll

```gherkin
Given a story log with at least one .log-entry rendered
And no edit is active
And part of an entry's text is selected
When at least two story-log polling cycles elapse
Then the same text is still selected
And the entries are the same nodes that held the selection
```

#### Scenario 30.12: Focus inside a log entry survives the poll

```gherkin
Given a story log with at least one .log-entry rendered
And no edit is active
And a control inside an entry has keyboard focus
When at least two story-log polling cycles elapse
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

#### Scenario 30.14: A poll that drops the oldest entry keeps the rest

```gherkin
Given a story log showing an oldest entry the next poll no longer returns
When the polling cycle renders the shorter log
Then the dropped entry is removed from the DOM
And the entries that remain are the same nodes as before the poll
```

#### Scenario 30.15: Editing locks the other entries' edit controls

```gherkin
Given a story log with at least two .log-entry rendered
When the client clicks .edit-btn on one entry
Then every other entry's .edit-btn is disabled
When the client cancels the edit
Then every other entry's .edit-btn is enabled again
```

#### Scenario 30.16: A failed save releases the edit lock

```gherkin
Given edit mode is active on one entry
And every other entry's .edit-btn is disabled
When the client saves the edit and the save request fails
Then every other entry's .edit-btn is enabled again
```

#### Scenario 30.17: A successful save holds the edit lock until the poll re-renders

```gherkin
Given edit mode is active on one entry
And every other entry's .edit-btn is disabled
When the client saves the edit and the save request succeeds
Then the other entries' .edit-btn stays disabled while the entry still shows its editor
And the resumed poll re-renders the log and enables every .edit-btn
```
