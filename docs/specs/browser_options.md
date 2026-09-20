# Feature Spec: Browser Options

Endpoint: browser DOM.

## Scenarios

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

#### Scenario 26.4: Clicking Use submits the rendered option text

```gherkin
Given the dock shows a generated option
When the player clicks the option's Use button
Then the browser POSTs /action/check with the option text read from the rendered button
And the story log gains an input entry carrying that same text
And a narration follows it
```