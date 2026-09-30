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
