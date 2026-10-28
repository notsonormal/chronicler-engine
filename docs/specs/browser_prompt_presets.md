# Feature Spec: Browser Prompt Presets

Endpoint: browser DOM.

## Scenarios

#### Scenario 28.1: Duplicating, editing and saving a preset through its buttons

```gherkin
Given a non-default system preset exists that allows only Novel
When the client opens the Prompt Presets tab
And clicks the preset card's Duplicate button
Then the panel loads with a copy named "<preset> (Copy)"
And the copy offers only "Set Active (Novel)"
When the client clicks the copy's Edit button
Then the edit dialog opens with the edit form and the mode checkboxes
When the client ticks the Interactive Fiction checkbox and submits Save
Then the dialog closes and the panel loads the saved card
When the page is reloaded
Then the re-opened panel's copy card offers both "Set Active (Novel)" and "Set Active (Interactive Fiction)"
```

#### Scenario 28.2: Opening a preset editor moves focus into the form

```gherkin
Given a non-default system preset exists
When the client opens the Prompt Presets tab
And the client clicks the preset card's Edit button
Then the edit dialog opens with the edit form
And the form's Name field has focus
```

#### Scenario 28.3: Deleting a preset card keeps focus on a card

```gherkin
Given the Prompt Presets tab shows two non-default presets in one section
And the first card's Delete control has focus
When the client confirms the delete
Then the panel loads and that card is removed
And a control in the next card has focus
When the client confirms the delete of the last non-default card
Then that card is removed
And a control in the card before it has focus
```

#### Scenario 28.4: Opening a preset editor moves focus straight into the form

```gherkin
Given a non-default system preset exists beside another card
When the client clicks the preset card's Edit button
Then the edit dialog opens with the edit form
And the form's Name field is the only control the opening moved focus to
```

#### Scenario 28.5: A refused preset editor reports in the notice and keeps focus

```gherkin
Given the Prompt Presets tab shows its preset cards
And a non-default card's Edit control has focus
When the client clicks Edit and the engine refuses to open that card's editor
Then no dialog opens
And #screen-error shows "The form was not opened: <the engine's message>."
And every card stays in place, as the same element
And the Edit control keeps focus
```

#### Scenario 28.6: The View of a default preset opens in the dialog, and Close returns focus to the control

```gherkin
Given the Prompt Presets tab shows a default preset card with a View control that has focus
When the client clicks View
Then the edit dialog opens with a read-only form of the preset
When the client closes the dialog
Then focus is on the View control
```
