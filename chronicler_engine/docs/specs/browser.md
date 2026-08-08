# Feature Spec: Browser

Endpoint: browser DOM. Behavioural authority for the 7 browser-only
interactions. Each When is a browser action; each Then is a
DOM-observable outcome. Rendering invariants (CSS computed styles,
layout measurements) live as test code in `tests/browser/invariants.rs`,
not as spec prose — the test code is the definition.

Scenario IDs are `16.x` and stay stable across edits. The pilot dedups
by ID across `docs/specs/*.md`, so IDs must stay unique.

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
