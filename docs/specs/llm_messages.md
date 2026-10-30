# Feature Spec: LLM Messages

Endpoints:
- `POST /action`
- `GET /fragment/llm-messages`
- `GET /fragment/llm-messages/:id`

`GET /fragment/llm-messages` answers a header for each of the 50 newest attempts:
the Agent, the backend and model, the time, and for a failed attempt the error mark
with a short line of its failure text. A header holds no prompt, no response and no
Raw JSON. The Open control of a header loads the record of the attempt into the edit
dialog (`#edit-dialog-body`) through `GET /fragment/llm-messages/:id`. The record is
read-only. Its root (`data-dialog-form`) carries the title of the dialog.

## Scenarios

### Recording every attempt

#### Scenario 33.1: A failed narration attempt is recorded with its failure text and shown in the LLM Messages panel

```gherkin
Given an active Game whose narrator backend fails every narration request
When the client POST /action with command="look"
And the Game is no longer busy
Then GET /screen?parts=llm-messages shows one entry for the narrator that names the backend/model the attempt was made against
And that entry carries the backend's failure text
And the record of that entry carries the failure text and no Raw JSON section
```

#### Scenario 33.2: A later successful attempt does not overwrite an earlier failed attempt

```gherkin
Given an active Game whose narrator backend fails the first narration request and succeeds the second
When the client POST /action with command="look"
And the Game is no longer busy
And the client POST /action with command="listen"
And the Game is no longer busy
Then GET /screen?parts=llm-messages shows two entries for the narrator
And the first entry carries the backend's failure text
And the record of the second entry carries the second narration
```

#### Scenario 33.3: The LLM Messages list sends a header for each attempt

```gherkin
Given an active Game with a recorded LLM attempt that holds every field
And a recorded LLM attempt that failed
When the client GET /screen?parts=llm-messages
Then the response status is "200 OK"
And the body does not hold an element with the class "llm-messages-panel"
And each attempt's card has the id "llm-msg-{id}"
And each card's Open control loads `/fragment/llm-messages/{id}` into `#edit-dialog-body`
And each header names the Agent and the backend/model
And the failed attempt's header carries the error mark
And the body holds no system prompt, no user prompt, no response, no Raw JSON and no `details` element
```

#### Scenario 33.4: The record route answers the full record as a read-only dialog

```gherkin
Given an active Game with a recorded LLM attempt that holds every field
And a recorded LLM attempt that failed
When the client GET /fragment/llm-messages/{id} of the first attempt
Then the response status is "200 OK"
And the body is a dialog fragment: its root carries `data-dialog-form`, and it holds the title with the id "edit-dialog-title"
And the body carries the system prompt, the user prompt, the response, the Raw Request JSON and the Raw Response JSON
And the body names the backend/model
And the body holds no form control
And the body holds a Close control with `data-close-dialog`
When the client GET /fragment/llm-messages/{id} of the failed attempt
Then the body carries the failure text and no response
```

#### Scenario 33.5: The record route answers a 404 for an id that holds no record

```gherkin
Given an active Game with a recorded LLM attempt
When the client GET /fragment/llm-messages/{id} with an id that no attempt holds
Then the response status is "404 Not Found"
And the body is an error fragment that says "LLM message not found"
```
