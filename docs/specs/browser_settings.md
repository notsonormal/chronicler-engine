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

#### Scenario 40.2: Add and Edit open the shared connection form page in the editor slot

```gherkin
Given the dashboard is loaded on the Connections sub-tab
When the client activates the Add Connection control
Then the connection form page is shown in the editor slot, with a back link to Connections
And the Connections list is hidden while the editor is open
When the client activates the back link
Then the editor slot is empty and the Connections list is shown again
And the back link sent no request
When the client activates a connection's Edit control
Then the same connection form page is shown
When the client activates the back link
Then the Connections list is shown again
```

#### Scenario 40.3: Opening and closing the connection editor keeps keyboard focus in the panel

```gherkin
Given the dashboard is loaded on the Connections sub-tab
And a connection's Edit control has keyboard focus
When the client activates it
Then the connection form page is shown
And #conn_name has keyboard focus
When the client activates the back link
Then the Connections list is shown again
And the Edit control that opened the editor has keyboard focus
When the client changes the Quantifier role's connection
Then #role-select-quantifier has keyboard focus
```
