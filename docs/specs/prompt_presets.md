# Feature Spec: Prompt Presets

Endpoints:
- `GET /fragment/prompt-presets`
- `GET /fragment/prompt-presets/{id}`
- `GET /fragment/prompt-presets/{id}/edit`
- `GET /fragment/prompt-presets/{id}/view`
- `POST /prompt-presets`
- `POST /prompt-presets/{id}`
- `POST /prompt-presets/{id}/activate`
- `POST /prompt-presets/{id}/delete`
- `POST /prompt-presets/{id}/duplicate`

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
And the body contains an "Add System Prompt Preset" toggle
And the body contains an "Add Quantifier Prompt Preset" toggle
And the system add-form contains inputs named name, role, instructions, writing_style, output_format
And the system add-form contains a hidden input named preset_type with value "system"
And the quantifier add-form contains inputs named name, role, instructions, output_format
And the quantifier add-form contains a hidden input named preset_type with value "quantifier"
And the body contains an "Impersonate Prompts" heading
And the body contains an "Add Impersonate Prompt Preset" toggle
And the impersonate add-form contains inputs named name, role, instructions, writing_style, output_format
And the impersonate add-form contains a hidden input named preset_type with value "impersonate"
```

#### Scenario 21.28: Each Add form is a closed disclosure

```gherkin
Given a fresh app state
When the client GET /fragment/prompt-presets
Then the response is 200
And each category's Add form is collapsed until the user opens it
```

### GET /fragment/prompt-presets/{id} — single card

#### Scenario 21.2: Single card returns the preset

```gherkin
Given a fresh app state with a seeded non-default system preset named "My System"
When the client GET /fragment/prompt-presets/{id} for that preset's id
Then the response is 200
And the response body contains "<div class=\"preset-card"
And the body contains the preset name "My System"
And the body contains a "Set Active (Novel)" button (the preset is not active for the Novel bundle)
And the body contains a "Set Active (Interactive Fiction)" button (the preset is not active for the Interactive Fiction bundle)
And the body contains an "Edit" button
And the body contains a "Delete" button
And the body contains a "Duplicate" button
```

#### Scenario 21.3: Single card for a nonexistent preset returns an error fragment

```gherkin
Given a fresh app state
When the client GET /fragment/prompt-presets/does-not-exist
Then the response is 200
And the response body is `<div class="error-message">Preset not found</div>`
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

#### Scenario 21.5: Edit form for a nonexistent preset returns an error fragment

```gherkin
Given a fresh app state
When the client GET /fragment/prompt-presets/does-not-exist/edit
Then the response is 200
And the response body is `<div class="error-message">Preset not found</div>`
```

#### Scenario 21.6: Edit form for a default preset returns an error fragment

```gherkin
Given a fresh app state with a seeded default system preset
When the client GET /fragment/prompt-presets/{default_id}/edit
Then the response is 200
And the response body is `<div class="error-message">Cannot edit default presets</div>`
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

#### Scenario 21.8: View form for a nonexistent preset returns an error fragment

```gherkin
Given a fresh app state
When the client GET /fragment/prompt-presets/does-not-exist/view
Then the response is 200
And the response body is `<div class="error-message">Preset not found</div>`
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

#### Scenario 21.11: Create with an invalid preset_type returns an error fragment

```gherkin
Given a fresh app state
When the client POST /prompt-presets with name="Bad Type" and instructions="Test." and preset_type="invalid"
Then the response is 200
And the response body is `<div class="error-message">Invalid preset type</div>`
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
And the response body contains `<div class="error-message">Save failed:` (the error is surfaced in the fragment, not as a HTTP error status)
```

The same failure shape applies to `POST /prompt-presets/{id}` (update),
`POST /{id}/delete`, and `POST /{id}/activate` — 200 with a
`<div class="error-message">{Update|Delete|Save} failed: …</div>` fragment. Not
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
And the body preserves the stored allowed modes (the form omitted them)
```

#### Scenario 21.15: Update a nonexistent preset returns an error fragment

```gherkin
Given a fresh app state
When the client POST /prompt-presets/does-not-exist with name="Updated" and instructions="Updated." and preset_type="system"
Then the response is 200
And the response body is `<div class="error-message">Preset not found</div>`
```

#### Scenario 21.16: Update ignores the form's preset_type and keeps the stored type

```gherkin
Given a fresh app state with a seeded non-default system preset
When the client POST /prompt-presets/{id} with name="Updated" and instructions="Updated." and preset_type="quantifier"
Then the response is 200
And the response body contains "preset-card" and no error fragment
And the stored preset still has preset_type "system" and name "Updated"
```

#### Scenario 21.17: Update a default preset returns an error fragment

```gherkin
Given a fresh app state with a seeded default system preset
When the client POST /prompt-presets/{default_id} with name="Changed" and instructions="Changed." and preset_type="system"
Then the response is 200
And the response body is `<div class="error-message">Cannot edit default presets</div>`
```

### POST /prompt-presets/{id}/delete — delete

#### Scenario 21.18: Delete a preset → empty body

```gherkin
Given a fresh app state with a seeded non-default system preset
When the client POST /prompt-presets/{id}/delete
Then the response is 200
And the response body is empty
```

#### Scenario 21.19: Delete a nonexistent preset returns an error fragment

```gherkin
Given a fresh app state
When the client POST /prompt-presets/does-not-exist/delete
Then the response is 200
And the response body is `<div class="error-message">Preset not found</div>`
```

#### Scenario 21.20: Delete a default preset returns an error fragment

```gherkin
Given a fresh app state with a seeded default system preset
When the client POST /prompt-presets/{default_id}/delete
Then the response is 200
And the response body is `<div class="error-message">Cannot delete default presets</div>`
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

#### Scenario 21.22: Duplicate a nonexistent preset returns an error fragment

```gherkin
Given a fresh app state
When the client POST /prompt-presets/does-not-exist/duplicate
Then the response is 200
And the response body is `<div class="error-message">Preset not found</div>`
```

### POST /prompt-presets/{id}/activate — activate

#### Scenario 21.23: Activate a system preset → panel re-renders with an Active badge

```gherkin
Given a fresh app state with a seeded non-default system preset that is not the active system preset
When the client POST /prompt-presets/{id}/activate?mode=novel
Then the response is 200
And the response body contains "<div class=\"prompt-presets-panel\">"
And the body contains an "Active · Novel" badge in the system preset's card-badges
And that preset's card does not contain a "Set Active (Novel)" button (it is now active for the Novel bundle)
```

Activating a quantifier preset follows the same shape, writing to the
quantifier slot instead of the system slot. Not enumerated as a
separate scenario; this one covers the shape.

Activation is per narrator mode: the `mode` query parameter selects the
bundle the preset becomes active in (absent or invalid falls back to
`novel`). Each card renders one activation button per allowed mode,
gated by the preset's allowed_modes and hidden for the bundle the
preset already leads.

#### Scenario 21.25: Activate an Interactive Fiction-only preset for the IF bundle

```gherkin
Given a fresh app state with a seeded system preset whose allowed_modes is ["interactive_fiction"]
When the client POST /prompt-presets/{id}/activate?mode=interactive_fiction
Then the response is 200
And the response body contains an "Active · Interactive Fiction" badge in that preset's card-badges
And the Interactive Fiction bundle's system slot holds that preset's id
And activating the same preset without the mode parameter returns `<div class="error-message">Preset not allowed for novel mode</div>`
```

#### Scenario 21.26: Panel gates activation buttons by allowed_modes

```gherkin
Given a fresh app state with a seeded system preset whose allowed_modes is ["interactive_fiction"]
When the client GET /fragment/prompt-presets
Then that preset's card contains a "Set Active (Interactive Fiction)" button
And that preset's card does not contain a "Set Active (Novel)" button
```

#### Scenario 21.24: Activate a nonexistent preset returns an error fragment

```gherkin
Given a fresh app state
When the client POST /prompt-presets/does-not-exist/activate
Then the response is 200
And the response body is `<div class="error-message">Preset not found</div>`
```

#### Scenario 21.27: Duplicate → edit-form flags → save toggles per-mode activation

```gherkin
Given a fresh app state with a system preset whose allowed_modes are ["novel", "interactive_fiction"]
When the client POST /prompt-presets/{id}/duplicate (the copy carries the source's flags)
And the client GET /fragment/prompt-presets/{copy_id}/edit
Then the edit form renders the allowed_mode_novel and allowed_mode_if checkboxes checked
When the client POST /prompt-presets/{copy_id} with allowed_mode_novel and allowed_mode_if both set
Then the response is 200
And the returned card contains a "Set Active (Interactive Fiction)" button
And the returned card contains a "Set Active (Novel)" button
And storage reports the copy's allowed_modes as ["novel", "interactive_fiction"]
```

### Duplicate names

#### Scenario 21.29: Creating a preset whose name already exists in its category is refused

```gherkin
Given a fresh app state with a System preset named "Alpha"
When the client POST /prompt-presets with name "Alpha" and preset_type "system"
Then the response is a 400
And the body names "Alpha" and says a system preset with that name already exists
```

#### Scenario 21.30: Creating a preset differing only in case and surrounding space is refused

```gherkin
Given a fresh app state with a System preset named "Alpha"
When the client POST /prompt-presets with name "  alpha  " and preset_type "system"
Then the response is a 400
And the body says a system preset with that name already exists
```

#### Scenario 21.31: The same name in a different category is allowed

```gherkin
Given a fresh app state with a System preset named "Alpha"
When the client POST /prompt-presets with name "Alpha" and preset_type "quantifier"
Then the response is 200
And the body contains a Quantifier preset named "Alpha"
```

#### Scenario 21.32: Editing a preset keeps its own name

```gherkin
Given a fresh app state with a System preset named "Alpha"
When the client POST /prompt-presets/{id} for that preset with the same name "Alpha" and changed instructions
Then the response is 200
And the body contains a preset card named "Alpha" and no error
```

#### Scenario 21.33: Duplicating the same preset twice yields two distinct copies

```gherkin
Given a fresh app state with a System preset named "Original"
When the client POST /prompt-presets/{id}/duplicate for that preset
Then the response is 200
And the body contains the copy name "Original (Copy)"
When the client POST /prompt-presets/{id}/duplicate for the same preset again
Then the response is 200
And the body contains the copy name "Original (Copy 2)"
And the body still contains the first copy name "Original (Copy)"
```

#### Scenario 21.34: Renaming a preset onto a same-category sibling's name is refused

```gherkin
Given a fresh app state with a System preset named "Alpha"
And a System preset named "Beta"
When the client POST /prompt-presets/{beta_id} with name "Alpha"
Then the response is a 400
And the body names "Alpha" and says a system preset with that name already exists
And the preset named "Beta" is unchanged
```
