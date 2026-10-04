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

#### Scenario 16.9: Primary button locks and unlocks with the status after confirming a preview

```gherkin
Given the client submitted a command the text-check preview intercepted, so the preview is open
When the client confirms the preview, so a turn starts
Then the primary button is disabled and labelled as a generating indicator
And the command form is the same node as before
When the status poll reports idle and the status display returns to Ready
Then the primary button is enabled again with a "Send" label
```

#### Scenario 16.10: Status errors still reach the toast after confirming a preview

```gherkin
Given the client submitted a command the text-check preview intercepted, so the preview is open
When the client confirms the preview
And the /status/generating poll returns an error fragment
Then #error-notification becomes visible and displays the error text
And after the status poll returns Ready, the same error fragment returned again re-shows the notification (the dedupe resets once the status is no longer an error)
```

#### Scenario 16.11: A log-entry check renders a read-only result and keeps the command form

```gherkin
Given the dashboard is loaded with the command form and status display
When the client checks a log entry that has issues
Then the result renders in its own element
And the result names the entry it checked and offers a dismiss control
And the result offers no way to send a turn
And #command-form and #status-display are the same nodes as before
When the client dismisses the result
Then the result element is empty
```

#### Scenario 16.12: Confirming a preview leaves the command form usable and Ready

```gherkin
Given the dashboard is loaded with text check enabled
When the client submits a misspelled command and the preview opens
And the client confirms the preview
Then the turn runs and the status returns to Ready
And the command form is still in the document
And the command input accepts and submits a new command with no reload
```

#### Scenario 16.13: The send preview opens beside the status display

```gherkin
Given a turn is generating and the status display shows its phase
When the client submits a command the preview intercepts
Then the preview is open
And the status display is still in the document and still shows the phase
```

#### Scenario 16.14: A clean log-entry check result can be dismissed

```gherkin
Given the dashboard is loaded with the command form and status display
When the client checks a log entry with no issues
Then the result names the entry it checked and reports no issues
And the result offers a dismiss control
When the client dismisses the result
Then the result element is empty
```

#### Scenario 16.15: Focus moves into the preview and back to the command input

```gherkin
Given the dashboard is loaded with the command form
When the client submits a command the preview intercepts
Then the preview opens with focus in the correction textarea
When the client cancels the preview
Then the preview closes and focus returns to the command input
```

#### Scenario 16.16: The tab bar exposes a tablist and its panels

```gherkin
Given the dashboard is loaded
Then the tab bar exposes a tablist
And each tab exposes a tab role and names the panel it controls
And the Game tab reports itself selected
When the client activates the Settings tab
Then the Settings tab reports itself selected and the Game tab reports itself unselected
And the Settings panel is the visible one
```

#### Scenario 16.17: The page exposes a main landmark and a labelled command input

```gherkin
Given the dashboard is loaded
Then a skip link targets the main landmark
And a label names the command input "Command"
```
