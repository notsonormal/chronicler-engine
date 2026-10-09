# Feature Spec: Settings

Endpoints: 
 - `GET /fragment/settings`
 - `GET /fragment/connections/new`
 - `GET /fragment/connections/{id}/edit`
 - `POST /settings/text-check`
 - `POST /connections/add`
 - `POST /connections/test`
 - `POST /connections/{id}/test`
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
And the Connections panel holds one connection row per connection with its name, provider and model, role tags, Edit, Test and Delete
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
And the Connections sub-tab shows a degraded marker whose tooltip names the engine-wide scope
When a later call for that role succeeds
And the client GET /fragment/settings
Then that role row reports Healthy and the degraded marker is gone
```

#### Scenario 20.23: A failed Settings panel load reports the failure inside the panel

```gherkin
Given a fresh app state whose settings cannot be read
When the client GET /fragment/settings
Then the body carries the read failure text
And the body carries no Settings panel
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

#### Scenario 20.22: A failed text-check save keeps the card

```gherkin
Given a fresh app state whose settings cannot be saved
When the client POST /settings/text-check
Then the response is a 500
And the body carries the save failure text
And the body carries no Text Check card
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

#### Scenario 20.17: A failed connection add answers non-2xx and carries the failure

```gherkin
Given a fresh app state with the default connections
When the client POST /connections/add with conn_provider="bogus_provider", a name and a model
Then the response is a 400
And the body carries the unknown-backend failure text naming "bogus_provider"
And a following GET /fragment/settings still lists the Connections panel and its connection rows
```

### GET /fragment/connections/{id}/edit — edit form

#### Scenario 20.24: The connection Edit form reports an unknown connection inside the panel

```gherkin
Given a fresh app state
When the client GET /fragment/connections/does-not-exist/edit
Then the response is a 200
And the body says the connection was not found
And the body carries no connection form
```

### POST /connections/{id}/edit — update

#### Scenario 20.11: Editing a Connection keeps its own name

```gherkin
Given a fresh app state with a Connection named "Alpha"
When the client edits that Connection with the same name "Alpha" and a changed model
Then the response is 200
And the body contains a Connection row named "Alpha" and the changed model
```

#### Scenario 20.18: A failed connection edit answers non-2xx and carries the failure

```gherkin
Given a fresh app state with a Connection named "Alpha"
When the client edits that Connection with conn_provider="bogus_provider"
Then the response is a 400
And the body carries the unknown-backend failure text
And a following GET /fragment/settings still renders the Connection named "Alpha"
```

### POST /connections/{id}/test — connection test

The test sends one short fixed prompt to a connection and reports the result
inline. It runs only on a click and writes no LLM Messages row, so it never
moves role health or the failure banner.

#### Scenario 20.19: A passing connection test reports the reply time and records nothing

```gherkin
Given a saved mock connection
When the client tests that connection
Then the response is 200
And the body reports a successful reply naming the connection's model and the reply time
And the LLM Messages list is still empty
And no role's health has a backend or a failure
```

#### Scenario 20.20: A failed connection test renders the error disclosure

```gherkin
Given a saved connection whose provider fails the test call
When the client tests that connection
Then the response is 200
And the body carries a short failure message with a Details disclosure holding the raw failure text
And the LLM Messages list is still empty
```

### POST /connections/test — test the form's values

#### Scenario 20.21: The Add/Edit form tests the typed values before Save

```gherkin
Given a saved mock connection
When the client tests the Add/Edit form with a different mock model typed in
Then the response is 200
And the body reports a successful reply naming the typed model
And the saved connections are unchanged
```
