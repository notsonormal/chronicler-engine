# Feature Spec: Browser Dashboard

Endpoint: browser DOM.

## Scenarios

#### Scenario 16.5: Command form stays static after submission

```gherkin
Given the command form #command-form is rendered
When the client submits the form (htmx POST to /action/check)
And the story log updates with new entries
Then the same #command-form element is still in the document (form is a static shell, not re-rendered)
```

#### Scenario 16.6: Status display updates during generation

```gherkin
Given the page is loaded and idle
When the client submits the command "wait"
Then #status-display text is not "Ready" within 500ms
And #status-display text contains one of "Thinking", "Narrating", "Generating", or "Quantifying"
```

#### Scenario 16.7: Action failure renders an error toast

```gherkin
Given the dashboard is loaded and the action form is visible
When the client submits the command "Internal server error" and the server fails the action with a 500
Then #error-notification gains the .visible class
And #error-notification displays the server's rendered error text with HTML tags stripped ("Error: Failed to process action: Internal server error"), not the submitted command
```

#### Scenario 16.8: A newer error is not hidden by an older error's timer

```gherkin
Given the dashboard is loaded and #error-notification is hidden
When the client submits the command "First failure" and the server fails it
And 2.5 seconds later submits the command "Second failure" and the server fails it
Then #error-notification is still visible 6 seconds after the first failure
And #error-notification displays the second failure's rendered server text ("Error: Failed to process action: Second failure")
```

#### Scenario 16.9: Send button still locks and unlocks after an action-area swap

```gherkin
Given the client submitted a command the text-check preview intercepted, so #action-area was replaced by the preview
When the client confirms the preview, so a fresh #action-area (form and status display) is swapped in
And the client submits the command form
Then #submit-btn is disabled with a "Stop" label while the status shows the pending state
When the status poll reports idle and the status display returns to Ready
Then #submit-btn is enabled again with a "Send" label
```

#### Scenario 16.10: Status errors still reach the observer after an action-area swap

```gherkin
Given the client submitted a command the text-check preview intercepted, so #action-area was replaced by the preview
When the client confirms the preview, so a fresh #status-display is swapped in
And the /status/generating poll returns an error span
Then #error-notification becomes visible and displays the error text
And after the status poll returns Ready, the same error span returned again re-shows the notification (the lastStatusError dedupe was reset)
```

#### Scenario 16.11: Text-check result does not replace the command form

```gherkin
Given the dashboard is loaded with the command form and status display
When the client triggers a text check on a log entry
Then the text-check result renders in its own element
And #command-form and #status-display are the same nodes as before
```
