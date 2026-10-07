# Feature Spec: Settings

Endpoints: 
 - `GET /fragment/settings`
 - `GET /fragment/connections/new`
 - `GET /fragment/connections/{id}/edit`
 - `POST /settings/text-check`
 - `POST /connections/add`
 - `POST /connections/{id}/edit`
 - `POST /connections/{id}/delete`
 - `POST /connections/set-narrator`
 - `POST /connections/set-quantifier`
 - `GET /debug/backend`

## Scenarios

### Settings panel

The panel splits into two client-side sub-tabs: **Connections** (default) and
**Text Check**. The panel loads once, so the selected sub-tab stays until a
page reload.

#### Scenario 20.1: Settings panel renders the Connections and Text Check sub-tabs

```gherkin
Given a fresh app state with the default connections
When the client GET /fragment/settings
Then the response is 200
And the body contains a settings panel with a Connections sub-tab and a Text Check sub-tab
And the Connections sub-tab is the selected one
And the Connections panel holds a role row for Narrator and a role row for Quantifier, each with a connection select
And the Connections panel holds one connection row per connection with its name, provider and model, role tags, Edit and Delete
And the Connections panel offers an Add Connection control
And the Text Check panel holds the check_mode select and the enable_auto_check checkbox
And the body contains no "Set as Narrator" or "Set as Quantifier" control
And the body contains no always-open Add Connection form
```

#### Scenario 20.16: The role rows report each role's health

```gherkin
Given a fresh app state with no recorded LLM calls
When the client GET /fragment/settings
Then each role row reports "No calls yet"
When a role's newest recorded call failed
And the client GET /fragment/settings
Then that role row reports it as Degraded with a Details disclosure carrying the raw failure text
And the Connections sub-tab shows a degraded marker
When a later call for that role succeeds
And the client GET /fragment/settings
Then that role row reports Healthy and the degraded marker is gone
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

### Role rows

#### Scenario 20.8: Setting the Narrator from its role row takes effect on the next request

```gherkin
Given an app state whose Narrator role uses the mock connection with model mock-model-a
And a second mock connection with model mock-model-b
When the client sets the Narrator role to the second connection
Then the response is 200 and its Narrator role row renders the second connection selected
And a following GET /debug/backend reports mock-model-b (settings resolve per request, so the switch needs no restart)
```

#### Scenario 20.14: Setting the Quantifier from its role row takes effect at once

```gherkin
Given an app state whose Quantifier role uses the mock connection with model mock-model-a
And a second mock connection with model mock-model-b
When the client sets the Quantifier role to the second connection
Then the response is 200 and its Quantifier role row renders the second connection selected
And the Quantifier role now uses the second connection
```

### POST /connections/{id}/delete — delete

#### Scenario 20.15: Deleting a connection a role uses is refused

```gherkin
Given an app state whose Narrator role uses a connection and whose Quantifier role uses another
And a third connection that no role uses
When the client deletes the Narrator's connection
Then the response is 200
And the body names Narrator and points at the role rows above
And the connection list still holds the Narrator's connection
When the client deletes the Quantifier's connection
Then the body names Quantifier
When the client deletes the connection no role uses
Then the connection list no longer holds it
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
And the body contains a Connection row named "Alpha" and the changed model
```
