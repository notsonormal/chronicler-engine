# Feature Spec: Browser LLM Messages

Endpoint: browser DOM.

An LLM Messages row is a header. Its Open control loads the full record into the
edit dialog (`#edit-dialog-body`). The panel is a part of the screen (`llm-messages`), and the dialog
lies outside it, so a load of the panel never changes an open record.

## Scenarios

#### Scenario 35.1: Open shows the record of an LLM attempt in the dialog from the keyboard

```gherkin
Given the LLM Messages panel shows an LLM attempt
Then the row holds no prompt text and no response text
When the client tabs to the attempt's Open control and presses Enter
Then the edit dialog is open
And it shows the system prompt, the user prompt and the response of the attempt
And its root carries the title that names the dialog
```

#### Scenario 35.2: Close returns focus to the Open control

```gherkin
Given the edit dialog shows the record of an LLM attempt, opened with the Open control
When the client activates Close
Then the dialog is closed and its body is empty
And the focus is on the attempt's Open control
```

#### Scenario 35.3: A focused Open control keeps focus when the panel loads

```gherkin
Given the LLM Messages panel shows an LLM attempt
And the attempt's Open control has focus
When the number of `llm` moves, and the panel loads
Then the attempt's Open control is the same element, and it has focus
```

#### Scenario 35.4: A new LLM message appears after any number of opens and closes

```gherkin
Given the LLM Messages panel shows an LLM attempt
When the client opens its record and closes it, twice
And a new LLM message is recorded
Then the panel shows the new LLM message without a reload of the page
```

#### Scenario 35.5: An open record stays as it is when the panel loads

```gherkin
Given the edit dialog shows the record of an LLM attempt
When a new LLM message is recorded, and the panel loads
Then the panel lists the new LLM message
And the dialog is still open with the same record
```

#### Scenario 35.6: A record that the engine no longer holds reports in the notice

```gherkin
Given the LLM Messages panel shows an LLM attempt whose record the engine has removed
When the client activates the attempt's Open control
Then no dialog opens
And #notice shows "The form was not opened: LLM message not found."
And the focus stays on the Open control
```
