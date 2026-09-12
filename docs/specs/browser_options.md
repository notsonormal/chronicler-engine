# Feature Spec: Browser Options

Endpoint: browser DOM.

## Scenarios

#### Scenario 26.1: Clicking Use posts the option as the player's input and narrates

```gherkin
Given the dock shows a generated option
When the player clicks the option's Use button
Then the command input is filled with the option text and submitted
And the story log gains one Input entry with the option text followed by a Narration entry
```

#### Scenario 26.2: The current set survives a page reload

```gherkin
Given the dock shows a generated option set
When the player reloads the page
Then the dock repopulates with the same option set
```

#### Scenario 26.3: Clicking Edit fills the input without submitting

```gherkin
Given the dock shows a generated option
When the player clicks the option's Edit button
Then the command input is filled with the option text and focused
And no new story-log entry is created
```
