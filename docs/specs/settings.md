# Feature Spec: Settings

Endpoints: 
 - `GET /fragment/settings`
 - `POST /settings`
 - `POST /settings/text-check`
 - `POST /connections/add`
 - `POST /connections/{id}/edit`
 - `POST /connections/:id/set-narrator`
 - `GET /debug/backend`

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
And the body contains an "Add Connection" heading
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
And a following GET /fragment/settings marks the switched narrator connection as Narrator
```

#### Scenario 20.3: POST /settings switches the quantifier connection

```gherkin
Given a fresh app state where the quantifier connection is the first connection
When the client POST /settings with quantifier_connection_id set to a different existing connection
And narration_connection_id set to the current narrator connection
Then the response is 200
And the response body is "Settings saved!"
And a following GET /fragment/settings marks the switched quantifier connection as Quantifier
```

#### Scenario 20.4: POST /settings switches both connections

```gherkin
Given a fresh app state where the narrator and quantifier connections are both the first connection
When the client POST /settings with narration_connection_id and quantifier_connection_id each set to a different existing connection
Then the response is 200
And the response body is "Settings saved!"
And a following GET /fragment/settings marks the switched narrator connection as Narrator and the switched quantifier connection as Quantifier
```

### POST /settings — error paths

#### Scenario 20.5: POST /settings rejects a connection id that is not in the connections list

```gherkin
Given a fresh app state
When the client POST /settings with narration_connection_id set to a string that is not any connection's id
And quantifier_connection_id set to the current quantifier connection
Then the response is 200
And the response body contains `<div class="error-message">Save failed:` (a dangling id is a configuration fault, not a HTTP error status)
And the saved narration connection id is unchanged
```

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
And the response body contains `<div class="error-message">Save failed:` (the error is surfaced in the fragment, not as a HTTP error status)
```

### Backend resolution

#### Scenario 20.8: Switching the narrator takes effect on the next request

```gherkin
Given an app state whose narrator connection is the mock connection with model mock-model-a
And a second mock connection with model mock-model-b
When the client POST /connections/mock-b/set-narrator
Then the response is 200
And a following GET /debug/backend reports mock-model-b (settings resolve per request, so the switch needs no restart)
```

### POST /connections/add — create

#### Scenario 20.9: Adding a Connection whose name already exists is refused

```gherkin
Given a fresh app state with a Connection named "Duplicate Probe"
When the client adds a second Connection named "Duplicate Probe"
Then the response is a 400
And the body names "Duplicate Probe" and says a connection with that name already exists
And the Connection list still holds exactly one Connection named "Duplicate Probe"
```

#### Scenario 20.10: Adding a Connection differing only in case and surrounding space is refused

```gherkin
Given a fresh app state with a Connection named "Alpha"
When the client adds a Connection named "  alpha  "
Then the response is a 400
And the body says a connection with that name already exists
```

### POST /connections/{id}/edit — update

#### Scenario 20.11: Editing a Connection keeps its own name

```gherkin
Given a fresh app state with a Connection named "Alpha"
When the client edits that Connection with the same name "Alpha" and a changed model
Then the response is 200
And the body contains a Connection card named "Alpha" and the changed model
```
