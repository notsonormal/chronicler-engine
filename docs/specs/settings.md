# Feature Spec: Settings

Endpoints: 
 - `GET /fragment/settings`
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

### POST /settings/text-check — auto-save

Text Check is instant: the mode and the check-before-sending box apply on
change, with no Save button. Disabled mode clears and disables the check box,
so the two controls cannot contradict each other.

#### Scenario 20.12: The text-check auto-save stores the mode and the check-before-sending box

```gherkin
Given a fresh app state
When the client POST /settings/text-check with check_mode="spell" and the check-before-sending box checked
Then the response is a 200 carrying the re-rendered Text Check card with its own save feedback
And a following GET /fragment/settings renders "spell" selected and the check box checked
```

#### Scenario 20.13: Disabling the text-check mode clears and disables the check-before-sending box

```gherkin
Given a fresh app state with text check mode "spell" and check-before-sending enabled
When the client POST /settings/text-check with check_mode="disabled" and the check-before-sending box checked
Then the response is a 200
And the stored check-before-sending is disabled
And a following GET /fragment/settings renders "disabled" selected and the check box cleared and disabled
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
