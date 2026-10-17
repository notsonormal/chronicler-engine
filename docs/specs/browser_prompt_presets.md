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

#### Scenario 28.3: Deleting a preset card keeps focus on a card

```gherkin
Given the Prompt Presets tab shows two non-default presets in one section
And the first card's Delete control has focus
When the client confirms the delete
Then that card is removed
And the next card's Edit control has focus
When the client confirms the delete of the last non-default card
Then that card is removed
And a control in the card before it has focus
```

#### Scenario 28.4: Opening a preset editor moves focus straight into the form

```gherkin
Given a non-default system preset exists beside another card
When the client clicks the preset card's Edit button
Then the card renders an edit form
And the form's Name field is the only control the swap moved focus to
```

#### Scenario 28.5: A refused preset editor draws its message in the card's place and focuses it

```gherkin
Given the Prompt Presets tab shows its preset cards
And a non-default card's Edit control has focus
When the client clicks Edit and the engine refuses to open that card's editor
Then the refusal message replaces that card
And the other cards stay in place
And the refusal message has focus
```
