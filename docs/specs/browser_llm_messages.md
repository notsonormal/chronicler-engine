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

#### Scenario 35.2: An expanded row stays expanded when the panel loads

```gherkin
Given the client expanded an LLM Messages row
When the number of `llm` moves, and the panel loads while the row is expanded
Then the row is the same element, and it is still expanded
And the header still reports its expanded state
And the row shows its body
```

#### Scenario 35.3: A focused row keeps focus when the panel loads

```gherkin
Given the LLM Messages panel shows an LLM attempt
And the attempt's header has focus
When the number of `llm` moves, and the panel loads
Then the attempt's header is the same element, and it has focus
```

#### Scenario 35.4: A new LLM message appears after any number of expands and collapses

```gherkin
Given the LLM Messages panel shows an LLM attempt
When the client expands the attempt, collapses it, expands it again and collapses it again
And a new LLM message is recorded
Then the panel shows the new LLM message without a reload of the page
And the attempt the client collapsed is collapsed
```

#### Scenario 35.5: A prompt disclosure that the client changed stays as the client left it when the panel loads

```gherkin
Given the LLM Messages panel shows an LLM attempt with an expanded body
And the client closed the Response disclosure, which the engine renders open, and opened the System disclosure
When the number of `llm` moves, and the panel loads
Then the Response disclosure is closed and the System disclosure is open
And both are the same elements
```
