# Feature Spec: Prompt Presets

Endpoints:
- `GET /fragment/prompt-presets` — renders the preset panel.
- `GET /fragment/prompt-presets/{id}` — renders a single preset card.
- `GET /fragment/prompt-presets/{id}/edit` — renders the edit form.
- `GET /fragment/prompt-presets/{id}/view` — renders the read-only view form.
- `POST /prompt-presets` — create a preset (system or quantifier).
- `POST /prompt-presets/{id}` — update a preset.
- `POST /prompt-presets/{id}/activate` — activate a preset.
- `POST /prompt-presets/{id}/delete` — delete a preset.
- `POST /prompt-presets/{id}/duplicate` — duplicate a preset.

Behavioural authority for the prompt-preset endpoints — what a client
observes through HTTP. The mutating endpoints return HTML fragments (a
re-rendered panel, a single card, or an empty body) and signal failure
with 200 and a `<span class='error'>…</span>` body span; malformed
form bodies return 422. Presets showing a `Default` badge cannot be
edited or deleted. The `preset_type` form field must be `system` or
`quantifier` (anything else yields `Invalid preset type`). Each "When"
is an HTTP request; each "Then" is an HTTP-observable outcome asserted
against the response body.

Scenario IDs are `21.x` and stay stable across edits. The pilot dedups
by ID across `docs/specs/*.md`, so IDs must stay unique.

## Scenarios

### Panel

#### Scenario 21.1: Panel renders the full surface

```gherkin
Given a fresh app state seeded with at least one default system preset and one default quantifier preset
When the client GET /fragment/prompt-presets
Then the response is 200
And the response body contains "<div class=\"prompt-presets-panel\">"
And the body contains an "System Prompts" heading
And the body contains an "Quantifier Prompts" heading
And the body contains one preset-card per seeded system preset (with name and "Default" badge)
And the body contains one preset-card per seeded quantifier preset (with name and "Default" badge)
And the body contains an "Add System Prompt Preset" heading
And the body contains an "Add Quantifier Prompt Preset" heading
And the system add-form contains inputs named name, role, instructions, writing_style, output_format
And the system add-form contains a hidden input named preset_type with value "system"
And the quantifier add-form contains inputs named name, role, instructions, output_format
And the quantifier add-form contains a hidden input named preset_type with value "quantifier"
```

### GET /fragment/prompt-presets/{id} — single card

#### Scenario 21.2: Single card returns the preset

```gherkin
Given a fresh app state with a seeded non-default system preset named "My System"
When the client GET /fragment/prompt-presets/{id} for that preset's id
Then the response is 200
And the response body contains "<div class=\"preset-card"
And the body contains the preset name "My System"
And the body contains a "Set Active" button (the preset is not active)
And the body contains an "Edit" button
And the body contains a "Delete" button
And the body contains a "Duplicate" button
```

#### Scenario 21.3: Single card for a nonexistent preset returns an error span

```gherkin
Given a fresh app state
When the client GET /fragment/prompt-presets/does-not-exist
Then the response is 200
And the response body is "<span class='error'>Preset not found</span>"
```

### GET /fragment/prompt-presets/{id}/edit — edit form

#### Scenario 21.4: Edit form returns a populated form for a non-default preset

```gherkin
Given a fresh app state with a seeded non-default system preset named "Editable"
When the client GET /fragment/prompt-presets/{id}/edit for that preset's id
Then the response is 200
And the response body contains "<div class=\"preset-card edit-form\">"
And the body contains a form posting to /prompt-presets/{id}
And the body contains a hidden input named preset_type with value "system"
And the body contains a name input whose value is "Editable"
And the body contains textarea inputs named role, instructions, writing_style, output_format
And the body contains a "Save" button
And the body contains a "Cancel" button
```

#### Scenario 21.5: Edit form for a nonexistent preset returns an error span

```gherkin
Given a fresh app state
When the client GET /fragment/prompt-presets/does-not-exist/edit
Then the response is 200
And the response body is "<span class='error'>Preset not found</span>"
```

#### Scenario 21.6: Edit form for a default preset returns an error span

```gherkin
Given a fresh app state with a seeded default system preset
When the client GET /fragment/prompt-presets/{default_id}/edit
Then the response is 200
And the response body is "<span class='error'>Cannot edit default presets</span>"
```

### GET /fragment/prompt-presets/{id}/view — view form

#### Scenario 21.7: View form returns a read-only form

```gherkin
Given a fresh app state with a seeded default system preset named "Viewer"
When the client GET /fragment/prompt-presets/{id}/view for that preset's id
Then the response is 200
And the response body contains "<div class=\"preset-card view-form\">"
And the body contains the preset name "Viewer"
And the body contains read-only fields for Role, Instructions, Writing Style, Output Format
And the body contains a "Close" button
```

#### Scenario 21.8: View form for a nonexistent preset returns an error span

```gherkin
Given a fresh app state
When the client GET /fragment/prompt-presets/does-not-exist/view
Then the response is 200
And the response body is "<span class='error'>Preset not found</span>"
```

### POST /prompt-presets — create

#### Scenario 21.9: Create a system preset → panel re-renders with the new preset

```gherkin
Given a fresh app state
When the client POST /prompt-presets with name="My System Prompt" and instructions="You are a test narrator." and preset_type="system"
Then the response is 200
And the response body contains "<div class=\"prompt-presets-panel\">"
And the body contains the preset name "My System Prompt"
And the body contains the preview text "You are a test narrator."
And the body contains a "Set Active" button for the new preset (it is not active)
And the body contains an "Edit" button for the new preset (it is not default)
```

#### Scenario 21.10: Create a quantifier preset → panel re-renders with the new preset

```gherkin
Given a fresh app state
When the client POST /prompt-presets with name="My Quantifier Prompt" and instructions="Quantify this scene." and preset_type="quantifier"
Then the response is 200
And the response body contains "<div class=\"prompt-presets-panel\">"
And the body contains the preset name "My Quantifier Prompt"
And the body contains the preview text "Quantify this scene."
```

#### Scenario 21.11: Create with an invalid preset_type returns an error span

```gherkin
Given a fresh app state
When the client POST /prompt-presets with name="Bad Type" and instructions="Test." and preset_type="invalid"
Then the response is 200
And the response body is "<span class='error'>Invalid preset type</span>"
```

#### Scenario 21.12: Create with a missing required field returns 422

```gherkin
Given a fresh app state
When the client POST /prompt-presets with a form body that omits preset_type
Then the response is 422 Unprocessable Entity (axum Form rejection)
```

#### Scenario 21.13: Create reports a save failure in the response body

```gherkin
Given an app state whose preset storage fails on save
When the client POST /prompt-presets with valid name and preset_type="system" fields
Then the response is 200
And the response body contains "<span class='error'>Save failed:" (the error is surfaced in the fragment, not as a HTTP error status)
```

The same failure shape applies to `POST /prompt-presets/{id}` (update),
`POST /{id}/delete`, and `POST /{id}/activate` — 200 with a
`<span class='error'>{Update|Delete|Save} failed: …</span>` span. Not
enumerated as separate scenarios; this one covers the shape.

### POST /prompt-presets/{id} — update

#### Scenario 21.14: Update a preset → card re-renders with the new name

```gherkin
Given a fresh app state with a seeded non-default system preset named "Before"
When the client POST /prompt-presets/{id} with name="After" and instructions="Updated text." and preset_type="system"
Then the response is 200
And the response body contains "<div class=\"preset-card"
And the body contains the new name "After"
And the body does not contain the old name "Before"
```

#### Scenario 21.15: Update a nonexistent preset returns an error span

```gherkin
Given a fresh app state
When the client POST /prompt-presets/does-not-exist with name="Updated" and instructions="Updated." and preset_type="system"
Then the response is 200
And the response body is "<span class='error'>Preset not found</span>"
```

#### Scenario 21.16: Update with an invalid preset_type returns an error span

```gherkin
Given a fresh app state with a seeded non-default system preset
When the client POST /prompt-presets/{id} with name="Updated" and instructions="Updated." and preset_type="invalid"
Then the response is 200
And the response body is "<span class='error'>Invalid preset type</span>"
```

#### Scenario 21.17: Update a default preset returns an error span

```gherkin
Given a fresh app state with a seeded default system preset
When the client POST /prompt-presets/{default_id} with name="Changed" and instructions="Changed." and preset_type="system"
Then the response is 200
And the response body is "<span class='error'>Cannot edit default presets</span>"
```

### POST /prompt-presets/{id}/delete — delete

#### Scenario 21.18: Delete a preset → empty body

```gherkin
Given a fresh app state with a seeded non-default system preset
When the client POST /prompt-presets/{id}/delete
Then the response is 200
And the response body is empty
```

#### Scenario 21.19: Delete a nonexistent preset returns an error span

```gherkin
Given a fresh app state
When the client POST /prompt-presets/does-not-exist/delete
Then the response is 200
And the response body is "<span class='error'>Preset not found</span>"
```

#### Scenario 21.20: Delete a default preset returns an error span

```gherkin
Given a fresh app state with a seeded default system preset
When the client POST /prompt-presets/{default_id}/delete
Then the response is 200
And the response body is "<span class='error'>Cannot delete default presets</span>"
```

### POST /prompt-presets/{id}/duplicate — duplicate

#### Scenario 21.21: Duplicate a preset → panel re-renders with the copy

```gherkin
Given a fresh app state with a seeded non-default system preset named "Original"
When the client POST /prompt-presets/{id}/duplicate
Then the response is 200
And the response body contains "<div class=\"prompt-presets-panel\">"
And the body contains the copy name "Original (Copy)"
```

#### Scenario 21.22: Duplicate a nonexistent preset returns an error span

```gherkin
Given a fresh app state
When the client POST /prompt-presets/does-not-exist/duplicate
Then the response is 200
And the response body is "<span class='error'>Preset not found</span>"
```

### POST /prompt-presets/{id}/activate — activate

#### Scenario 21.23: Activate a system preset → panel re-renders with an Active badge

```gherkin
Given a fresh app state with a seeded non-default system preset that is not the active system preset
When the client POST /prompt-presets/{id}/activate
Then the response is 200
And the response body contains "<div class=\"prompt-presets-panel\">"
And the body contains an "Active" badge in the system preset's card-badges
And the body does not contain a "Set Active" button for that preset (it is now active)
```

Activating a quantifier preset follows the same shape, writing to the
quantifier slot instead of the system slot. Not enumerated as a
separate scenario; this one covers the shape.

#### Scenario 21.24: Activate a nonexistent preset returns an error span

```gherkin
Given a fresh app state
When the client POST /prompt-presets/does-not-exist/activate
Then the response is 200
And the response body is "<span class='error'>Preset not found</span>"
```
