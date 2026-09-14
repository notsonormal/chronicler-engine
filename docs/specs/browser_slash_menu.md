# Feature Spec: Browser Slash Menu

Endpoint: browser DOM.

## Scenarios

#### Scenario 31.1: Typing slash opens the command suggestion menu

```gherkin
Given the dashboard is loaded and the command input #command-form input[name="command"] is rendered
When the client types "/" into the command input
Then a #slash-menu element appears in the DOM
And #slash-menu contains three .slash-suggestion elements
And the suggestions are /impersonate, /guide, and /options
```

#### Scenario 31.2: Typing a prefix filters the suggestions

```gherkin
Given the command suggestion menu is open (#slash-menu visible)
When the client extends the input to "/g"
Then #slash-menu contains exactly one .slash-suggestion element
And that suggestion is /guide
```

#### Scenario 31.3: Arrow keys move the active suggestion

```gherkin
Given the command suggestion menu is open with multiple .slash-suggestion elements
And the first .slash-suggestion has the .active class
When the client presses ArrowDown
Then the second .slash-suggestion gains the .active class
And the first .slash-suggestion loses the .active class
When the client presses ArrowUp
Then the first .slash-suggestion regains the .active class
```

#### Scenario 31.4: Enter populates the input with the highlighted command

```gherkin
Given the command suggestion menu is open with a .slash-suggestion highlighted (.active)
When the client presses Enter
Then the command input value becomes the highlighted command followed by a space
And #slash-menu is removed from the DOM
And the form is not submitted (no generation starts)
```

#### Scenario 31.5: Escape closes the suggestion menu

```gherkin
Given the command suggestion menu is open (#slash-menu visible)
When the client presses Escape
Then #slash-menu is removed from the DOM
And the command input value is unchanged
```

#### Scenario 31.6: Clicking a suggestion populates the input

```gherkin
Given the command suggestion menu is open (#slash-menu visible)
When the client clicks a .slash-suggestion element
Then the command input value becomes that suggestion's command followed by a space
And #slash-menu is removed from the DOM
```

#### Scenario 31.7: The menu reopens after the action-area is re-rendered

```gherkin
Given the command suggestion menu is open (#slash-menu visible)
When the #action-area innerHTML is replaced with a fresh command form (mimicking the htmx swap that /action/check performs)
Then #slash-menu is removed from the DOM
And a new #command-form input[name="command"] is rendered
And typing "/" into the new input opens #slash-menu again
```

#### Scenario 31.8: Submitting `/impersonate` produces an Input log entry

```gherkin
Given the dashboard is loaded and the command input is rendered
When the client submits "/impersonate hello"
And the generation completes
Then a new .log-entry.input appears in #story-log
And no .log-entry.input whose text is "/impersonate hello" appears
```

#### Scenario 31.9: Submitting `/guide` does not persist an Input log entry

```gherkin
Given the dashboard is loaded and the command input is rendered
When the client submits "/guide look around"
And the generation completes
Then at least one new .log-entry.narration appears in #story-log
And no .log-entry.input whose text is "/guide look around" appears
```
