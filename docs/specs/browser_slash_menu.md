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
And #slash-menu exposes a listbox with one option per suggestion
And the command input reports the menu expanded and points at the first option
And the first option reports itself selected
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
And the command input points at the second option
And the second option reports itself selected
When the client presses ArrowUp
Then the first .slash-suggestion regains the .active class
And the command input points at the first option
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

