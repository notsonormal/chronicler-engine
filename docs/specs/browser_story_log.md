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

#### Scenario 30.4: Clicking delete removes the message

```gherkin
Given a story log with at least 2 .log-entry elements
And window.confirm is overridden to return true
When the client clicks .delete-btn
Then the .log-entry count decreases
```
