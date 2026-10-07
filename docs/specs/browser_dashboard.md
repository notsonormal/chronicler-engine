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

#### Scenario 16.18: A failed poll leaves its region unchanged and marks the banner

```gherkin
Given the dashboard is loaded and the story log shows its entries
When a story-log poll fails
Then the banner is visible and reports the engine as unreachable
And the story log still shows the same entries
When a later story-log poll succeeds
Then the banner disappears
```

#### Scenario 16.19: A failed action renders its message in the form's inline slot

```gherkin
Given the dashboard is loaded and the action form is visible
When the client submits a command and the server fails the action
Then the form shows an inline error naming the action as failed
And the inline error's short line carries no raw server text
And the raw server text is reachable in that error's disclosure
And the banner is not raised
```

#### Scenario 16.20: A dead engine renders the same inline error for a pending action

```gherkin
Given the dashboard is loaded and the engine has stopped
When the client submits a command
Then the form shows an inline error saying the engine is unreachable
And the banner is visible and reports the engine as unreachable
```

#### Scenario 16.21: A generation error clamps to one line with a popover

```gherkin
Given the dashboard is loaded and idle
And the action area and command input are measured
When the status display reports a generation error
Then the status display shows one short line and the raw text is not visible
And the action area's height and the command input's width are unchanged
And the error's Details control opens its disclosure, which shows the raw text
When the client presses Escape
Then the disclosure is closed
When the status returns to Ready
Then the error is gone
```

#### Scenario 16.22: A degraded role raises a status banner with its Details disclosure

```gherkin
Given a role is degraded
When the header poll carries the degraded-role banner
Then the banner is visible and reports itself as status
And its Details control opens the anchored disclosure
And that disclosure names each role with its backend and model
```

#### Scenario 16.23: Status-display changes are announced to assistive technology

```gherkin
Given the dashboard is loaded and idle
When the status display shows a generation phase
Then a polite region announces that phase
And the phase is not announced again while it is unchanged
When the generation fails
Then an assertive region announces the error's short line but not its raw text
When the generation finishes
Then the polite region announces Ready
```

#### Scenario 16.24: A new narration and a changed option set are announced once

```gherkin
Given the dashboard is loaded showing the story log and a set of options
When a new narration arrives and the option set changes
Then the new narration is announced once and not the whole log
And the new options are announced once
And neither is announced again while its content is unchanged
```

#### Scenario 16.25: A failed action on a reachable server never reports the engine unreachable

```gherkin
Given the dashboard is loaded and a panel control's save cannot be stored
When the client changes that control and the reachable engine fails the save
Then the failure reports on its own surface
And the panel keeps the content it had
And the banner is not raised
```

#### Scenario 16.26: The status display's open Details disclosure survives a poll

```gherkin
Given the status display shows a generation error
And the error's Details disclosure is open with focus in its Details control
When the status poll re-renders the display
Then the disclosure is still open
And focus is still in its Details control
When the client presses Escape
Then the disclosure is closed
```

#### Scenario 16.27: The banner's open Details disclosure survives a poll

```gherkin
Given the banner reports a degraded role
And the disclosure's Details control has been activated, so it is open with focus
When the header poll refreshes the banner
Then the disclosure is still open
And focus is still in its Details control
And the banner is still up
```

#### Scenario 16.28: The status poll leaves generating and re-enables Send

```gherkin
Given the dashboard is loaded and the status poll reports a generation phase
Then #status-display shows a generating state
And #submit-btn is disabled
When the status poll reports idle
Then #status-display returns to Ready
And #submit-btn is enabled
And pressing Enter in #command-form input[name="command"] submits the form
```
