# Feature Spec: Browser Prompt Presets

Endpoint: browser DOM.

## Scenarios

#### Scenario 28.1: The editor exposes allowed-modes checkboxes and saving updates per-mode activation

```gherkin
Given the prompt presets panel is loaded with only default presets
When the client duplicates a preset, clicks Edit on the copy, checks the interactive_fiction checkbox, and saves the form
Then the edit form renders the Allowed Modes checkbox group reflecting the stored flags before saving
And the saved card re-renders with a "Set Active (IF)" button
```
