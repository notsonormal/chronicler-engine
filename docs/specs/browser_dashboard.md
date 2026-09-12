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
