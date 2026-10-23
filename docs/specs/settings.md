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

#### Scenario 20.25: The context window field sets the window the next narration uses

```gherkin
Given a Game with a long story log, whose Narrator role uses a mock Connection with an empty context window field
And the player submits an Action, and its narration settles
When the client edits that Connection with the context window field set to a larger value
And the player submits a second Action, and its narration settles
Then the recorded prompt of the second narration holds more of the story log than the first
```

#### Scenario 20.26: An empty context window field uses the provider default

```gherkin
Given a Game with a long story log, whose Narrator role uses a mock Connection whose context window field holds a large value
And the player submits an Action, and its narration settles
When the client edits that Connection with an empty context window field
And the player submits a second Action, and its narration settles
Then the recorded prompt of the second narration holds less of the story log than the first
```

#### Scenario 20.27: An edit that changes the provider keeps an explicit context window

```gherkin
Given a Connection whose provider has a small default context window and whose context window field holds a larger explicit value
When the client edits that Connection to a provider with a larger default window and leaves the field as it is
Then the response is 200
And the stored Connection still holds the explicit value
And a following GET /fragment/connections/{id}/edit renders the explicit value in the context window field
```

#### Scenario 20.28: A context window that is not a positive whole number is refused

```gherkin
Given a fresh app state with a Connection named "Alpha"
When the client edits that Connection with a context window of text, of zero or of a negative number
Then the response is 400
And the body says the context window must be a positive whole number, or empty for the provider default
And the stored Connection still holds no explicit context window
And a following GET /fragment/settings still renders the Connection named "Alpha"
When the client adds a Connection with a context window of text
Then the response is 400
And the connection list still holds the connections it held before
```

#### Scenario 20.29: The reasoning effort field stores the chosen effort

```gherkin
Given a fresh app state with a Connection that has no reasoning effort
When the client edits that Connection with the reasoning effort field set to "high"
Then the response is 200
And the stored Connection holds the reasoning effort "high"
And a following GET /fragment/connections/{id}/edit renders "high" as the selected reasoning effort option
```

#### Scenario 20.30: An empty reasoning effort field stores no effort

```gherkin
Given a fresh app state with a Connection whose reasoning effort is "high"
When the client edits that Connection with an empty reasoning effort field
Then the response is 200
And the stored Connection holds no reasoning effort
And a following GET /fragment/connections/{id}/edit renders "Provider default" as the selected reasoning effort option
```

#### Scenario 20.31: An unknown reasoning effort is refused

```gherkin
Given a fresh app state with a Connection whose reasoning effort is "high"
When the client edits that Connection with the reasoning effort field set to "turbo"
Then the response is 400
And the body says the reasoning effort must be one of the listed values, or empty for the provider default
And the stored Connection still holds the reasoning effort "high"
When the client adds a Connection with the reasoning effort field set to "turbo"
Then the response is 400
And the body says the reasoning effort must be one of the listed values, or empty for the provider default
And the connection list still holds the connections it held before
```

#### Scenario 20.32: Adding a connection stores the chosen reasoning effort

```gherkin
Given a fresh app state
When the client adds a Connection with the reasoning effort field set to "low"
Then the response is 200
And the stored new Connection holds the reasoning effort "low"
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

#### Scenario 20.33: The form test hands the typed reasoning effort to the provider

```gherkin
Given a saved mock connection
When the client tests the Add/Edit form with the reasoning effort field set to "low"
Then the response is 200
And the provider that runs the test is built from a connection that holds the reasoning effort "low"
And the saved connections are unchanged
```
