# Feature Spec: Browser Prompt Presets

Endpoint: browser DOM.

## Scenarios

#### Scenario 28.1: Duplicating, editing and saving a preset through its buttons

```gherkin
Given a non-default system preset exists that allows only Novel
When the client opens the Prompt Presets tab
And clicks the preset card's Duplicate button
Then the panel re-renders with a copy named "<preset> (Copy)"
And the copy offers only "Set Active (Novel)"
When the client clicks the copy's Edit button
Then that card renders an edit form with the mode checkboxes
When the client ticks the Interactive Fiction checkbox and submits Save
And the page is reloaded
Then the re-opened panel's copy card offers both "Set Active (Novel)" and "Set Active (Interactive Fiction)"
```

#### Scenario 28.2: Opening a preset editor moves focus into the form

```gherkin
Given a non-default system preset exists
When the client opens the Prompt Presets tab
And the client clicks the preset card's Edit button
Then the card renders an edit form
And the form's Name field has focus
```
