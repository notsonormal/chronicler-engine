# Feature Spec: Browser Settings

Endpoint: browser DOM.

#### Scenario 40.1: The Settings sub-tabs switch and keep their place across a top-level tab change

```gherkin
Given the dashboard is loaded
When the client opens Settings and activates the Text Check sub-tab
Then the Text Check sub-tab reports itself selected (aria-selected)
And the Text Check panel is the visible sub-tab panel
And the Connections panel is hidden
When the client switches to another top-level tab and back to Settings
Then the Text Check sub-tab is still selected and its panel is still the visible one
```

#### Scenario 40.2: Add and Edit open the connection form in the dialog

```gherkin
Given the dashboard is loaded on the Connections sub-tab
When the client activates the Add Connection control
Then the edit dialog is open with the connection form, titled "Add Connection"
When the client activates Cancel
Then the edit dialog is closed and its body is empty
And Cancel sent no request
When the client activates a connection's Edit control
Then the edit dialog is open with the same form, titled with the connection's name
When the client activates Cancel
Then the edit dialog is closed
```

#### Scenario 40.3: Opening and closing the connection dialog keeps keyboard focus where the player works

```gherkin
Given the dashboard is loaded on the Connections sub-tab
And a connection's Edit control has keyboard focus
When the client activates it
Then the connection form is shown in the dialog
And #conn_name has keyboard focus
When the client activates Cancel
Then the Edit control that opened the dialog has keyboard focus
When the client changes the Quantifier role's connection
Then #role-select-quantifier has keyboard focus
```

#### Scenario 40.4: A text check change that is saved shows the visible "Saved" line

```gherkin
Given the dashboard is loaded on the Text Check sub-tab
And #text-check-status shows no text
When the client changes the Check Mode select to "spell"
And the engine answers 200
Then the browser POSTs /settings/text-check
And #text-check-status shows the text "Saved"
```

#### Scenario 40.5: The Test result of a saved connection opens in the dialog and survives a Settings load

```gherkin
Given the dashboard is loaded on the Connections sub-tab
When the client activates a connection's Test control
Then the edit dialog is open with the result of the test
When the number of `settings` moves, and the Settings panel loads
Then the edit dialog still shows the result
When the client presses Escape
Then the edit dialog is closed
And the Test control has keyboard focus
```
