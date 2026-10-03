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
