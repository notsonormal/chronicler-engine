# Feature Spec: Browser LLM Messages

Endpoint: browser DOM.

## Scenarios

#### Scenario 35.1: An LLM Messages row expands and collapses from the keyboard

```gherkin
Given the LLM Messages panel shows an LLM attempt
When the client tabs to the attempt's header and presses Enter
Then the attempt's body is shown
And the header reports its expanded state to assistive technology
When the client presses Space on the same header
Then the attempt's body is hidden again
And the header reports its collapsed state to assistive technology
```

#### Scenario 35.2: An expanded row stays expanded when the panel refreshes

```gherkin
Given the client expanded an LLM Messages row
When the panel is refreshed while the row is expanded
Then the row is still expanded
And the header still reports its expanded state
```

#### Scenario 35.3: A focused row keeps focus when the panel refreshes

```gherkin
Given the LLM Messages panel shows an LLM attempt
And the attempt's header has focus
When the panel is refreshed
Then the attempt's header has focus again
```
