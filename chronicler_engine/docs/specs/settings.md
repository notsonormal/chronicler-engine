# Feature Spec: Settings

Endpoints: `GET /fragment/settings`, `POST /settings`

Behavioural authority for the settings endpoints — what a client
observes through HTTP. `POST /settings` takes two form fields
(`narration_connection_id`, `quantifier_connection_id`) and returns
`Settings saved!` on success; malformed form bodies return 422. Each
"When" is an HTTP request; each "Then" is an HTTP-observable outcome
asserted against the response body.

Scenario IDs are `20.x` and stay stable across edits. The pilot dedups
by ID across `docs/specs/*.md`, so IDs must stay unique.

## Scenarios

### Settings panel

#### Scenario 20.1: Settings panel renders the full surface

```gherkin
Given a fresh app state with the default connections
When the client GET /fragment/settings
Then the response is 200
And the response body contains "<div class=\"settings-panel\">"
And the body contains a "Connections" heading
And the body contains one connection-card per connection (name, provider, model)
And the body contains an "Add LlmProviderConfig" heading
And the body contains a conn_name input
And the body contains a conn_provider select (with OpenRouter, DeepSeek, Ollama options)
And the body contains a conn_model input
And the body contains a conn_api_key input
And the body contains a conn_base_url input
And the body contains a single_user_message checkbox labelled "Single User Message"
And the body contains a "Text Check" heading
And the body contains a check_mode select
And the body contains an enable_auto_check checkbox
```

### POST /settings — success paths

#### Scenario 20.2: POST /settings switches the narrator connection

```gherkin
Given a fresh app state where the narrator connection is the first connection
When the client POST /settings with narration_connection_id set to a different existing connection
And quantifier_connection_id set to the current quantifier connection
Then the response is 200
And the response body is "Settings saved!"
```

#### Scenario 20.3: POST /settings switches the quantifier connection

```gherkin
Given a fresh app state where the quantifier connection is the first connection
When the client POST /settings with quantifier_connection_id set to a different existing connection
And narration_connection_id set to the current narrator connection
Then the response is 200
And the response body is "Settings saved!"
```

#### Scenario 20.4: POST /settings switches both connections

```gherkin
Given a fresh app state where the narrator and quantifier connections are both the first connection
When the client POST /settings with narration_connection_id and quantifier_connection_id each set to a different existing connection
Then the response is 200
And the response body is "Settings saved!"
```

### POST /settings — current behaviour

#### Scenario 20.5: POST /settings accepts a connection id that is not in the connections list

```gherkin
Given a fresh app state
When the client POST /settings with narration_connection_id set to a string that is not any connection's id
And quantifier_connection_id set to the current quantifier connection
Then the response is 200 (the handler does not validate the id)
And the response body is "Settings saved!"
```

### POST /settings — error paths

#### Scenario 20.6: POST /settings with a missing required field returns 422

```gherkin
Given a fresh app state
When the client POST /settings with a form body that omits quantifier_connection_id
Then the response is 422 Unprocessable Entity (axum Form rejection)
```

#### Scenario 20.7: POST /settings reports a save failure in the response body

```gherkin
Given an app state whose settings storage fails on save
When the client POST /settings with valid narration_connection_id and quantifier_connection_id fields
Then the response is 200
And the response body contains "<span class='error'>Save failed:" (the error is surfaced in the fragment, not as a HTTP error status)
```
