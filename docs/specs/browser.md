# Feature Spec: Browser

Endpoint: browser DOM.

## Scenarios

#### Scenario 16.1: Clicking the edit button activates edit mode

```gherkin
Given a story log with at least one .log-entry rendered
When the client clicks .edit-btn
Then #edit-textarea appears in the DOM
```

#### Scenario 16.2: Cancelling edit restores the original text

```gherkin
Given edit mode is active (#edit-textarea visible)
And the textarea text has been modified
When the client clicks .cancel-btn
Then #edit-textarea is removed from the DOM
And .log-entry .text inner text is restored to the original
```

#### Scenario 16.3: Edit textarea persists across polling cycles

```gherkin
Given edit mode is active (#edit-textarea visible)
When 3 seconds elapse (client-side polling cycles run)
Then #edit-textarea remains in the DOM (polling does not destroy edit state)
```

#### Scenario 16.4: Clicking delete removes the message

```gherkin
Given a story log with at least 2 .log-entry elements
And window.confirm is overridden to return true
When the client clicks .delete-btn
Then the .log-entry count decreases
```

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

Note: the test dispatches the `htmx:beforeSwap` event directly because
this playwright-rs version's `route.fulfill` is broken for status/body,
and the real server has no path that returns 500 from `/action` without
production-code changes. The body-level listener that calls `showError`
on `isError` is the app code under test; htmx's 500→`isError=true`
mapping is htmx's contract, not ours.

#### Scenario 17.1: Typing slash opens the command suggestion menu

```gherkin
Given the dashboard is loaded and the command input #command-form input[name="command"] is rendered
When the client types "/" into the command input
Then a #slash-menu element appears in the DOM
And #slash-menu contains three .slash-suggestion elements
And the suggestions are /narrator, /impersonate, and /guide
```

#### Scenario 17.2: Typing a prefix filters the suggestions

```gherkin
Given the command suggestion menu is open (#slash-menu visible)
When the client extends the input to "/g"
Then #slash-menu contains exactly one .slash-suggestion element
And that suggestion is /guide
```

#### Scenario 17.3: Arrow keys move the active suggestion

```gherkin
Given the command suggestion menu is open with multiple .slash-suggestion elements
And the first .slash-suggestion has the .active class
When the client presses ArrowDown
Then the second .slash-suggestion gains the .active class
And the first .slash-suggestion loses the .active class
When the client presses ArrowUp
Then the first .slash-suggestion regains the .active class
```

#### Scenario 17.4: Enter populates the input with the highlighted command

```gherkin
Given the command suggestion menu is open with a .slash-suggestion highlighted (.active)
When the client presses Enter
Then the command input value becomes the highlighted command followed by a space
And #slash-menu is removed from the DOM
And the form is not submitted (no generation starts)
```

#### Scenario 17.5: Escape closes the suggestion menu

```gherkin
Given the command suggestion menu is open (#slash-menu visible)
When the client presses Escape
Then #slash-menu is removed from the DOM
And the command input value is unchanged
```

#### Scenario 17.6: Clicking a suggestion populates the input

```gherkin
Given the command suggestion menu is open (#slash-menu visible)
When the client clicks a .slash-suggestion element
Then the command input value becomes that suggestion's command followed by a space
And #slash-menu is removed from the DOM
```

#### Scenario 17.7: The menu reopens after the action-area is re-rendered

```gherkin
Given the command suggestion menu is open (#slash-menu visible)
When the #action-area innerHTML is replaced with a fresh command form (mimicking the htmx swap that /action/check performs)
Then #slash-menu is removed from the DOM
And a new #command-form input[name="command"] is rendered
And typing "/" into the new input opens #slash-menu again
```

#### Scenario 17.8: Submitting `/impersonate` produces a Dialogue log entry

```gherkin
Given the dashboard is loaded and the command input is rendered
When the client submits "/impersonate hello"
And the generation completes
Then a new .log-entry.dialogue appears in #story-log
And no .log-entry.input whose text is "/impersonate hello" appears
```

#### Scenario 17.9: Submitting `/guide` does not persist an Input log entry

```gherkin
Given the dashboard is loaded and the command input is rendered
When the client submits "/guide look around"
And the generation completes
Then at least one new .log-entry.narration appears in #story-log
And no .log-entry.input whose text is "/guide look around" appears
```

#### Scenario 17.10: Submitting `/narrator` persists a Narrator log entry

```gherkin
Given the dashboard is loaded and the command input is rendered
When the client submits "/narrator the room is dark"
And the generation completes
Then a new .log-entry.narrator appears in #story-log
And at least one new .log-entry.narration appears in #story-log
```

Note: the test replaces `#action-area` innerHTML directly rather than
submitting the form, because a real submit starts a generation and the
action-area view model disables the input while generating
(`is_disabled = status.is_generating()`), which would block the final
"type `/`" step. The app code under test is the document-level event
delegation that re-binds to the recreated input; the htmx swap is htmx's
mechanism, not ours. This mirrors scenario 16.7's synthetic-event
approach for the same reason.
