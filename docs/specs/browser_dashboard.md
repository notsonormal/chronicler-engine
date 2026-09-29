# Feature Spec: Browser Dashboard

Endpoint: browser DOM.

## Scenarios

#### Scenario 16.5: Command form stays static after submission

```gherkin
Given the command form #command-form is rendered
When the client submits the form (htmx POST to /action)
And the story log updates with new entries
Then #command-form id is unchanged (form is a static shell, not re-rendered)
```

#### Scenario 16.6: Status display updates during generation

```gherkin
Given the page is loaded and idle
When the client sends an action (send_action("wait"))
Then #status-display text is not "Ready" within 500ms
And #status-display text contains one of "Thinking", "Narrating", "Generating", or "Quantifying"
```

#### Scenario 16.7: Action failure renders an error toast

```gherkin
Given the dashboard is loaded and the action form is visible
When the client dispatches a synthetic htmx:beforeSwap event with isError=true and a serverResponse of "<p>Internal server error</p>"
Then #error-notification gains the .visible class
And #error-notification displays the response body with HTML tags stripped ("Internal server error")
```

#### Scenario 16.8: A newer error is not hidden by an older error's timer

```gherkin
Given the dashboard is loaded and #error-notification is hidden
When the client dispatches a synthetic htmx:beforeSwap error event with a serverResponse of "<p>First failure</p>"
And 2.5 seconds later dispatches a synthetic htmx:beforeSwap error event with a serverResponse of "<p>Second failure</p>"
Then #error-notification is still visible 6 seconds after the first event
And #error-notification displays "Second failure"
```

#### Scenario 16.9: Send button still locks and unlocks after an action-area swap

```gherkin
Given #action-area was swapped through the shipped save/restore path, so the form and status display are fresh nodes
When the client submits the command form
Then #submit-btn is disabled with a "Stop" label while the status shows the pending state
When the status poll reports idle and the status display returns to Ready
Then #submit-btn is enabled again with a "Send" label
```

#### Scenario 16.10: Status errors still reach the observer after an action-area swap

```gherkin
Given #action-area was swapped through the shipped save/restore path, so the status display is a fresh node
When an error span of the shape /status/generating returns is swapped into the live #status-display
Then #error-notification becomes visible and displays the error text
And after the status returns to Ready, the same error span swapped in again re-shows the notification (the lastStatusError dedupe was reset by the swap-aware observer)
```
