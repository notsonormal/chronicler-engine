# Feature Spec: Prompt Presets

Endpoints:
- `GET /fragment/prompt-presets`
- `GET /fragment/prompt-presets/new?type={type}`
- `GET /fragment/prompt-presets/{id}/edit`
- `GET /fragment/prompt-presets/{id}/view`
- `POST /prompt-presets`
- `POST /prompt-presets/{id}`
- `POST /prompt-presets/{id}/activate`
- `POST /prompt-presets/{id}/delete`
- `POST /prompt-presets/{id}/duplicate`

The Prompt Presets panel is a part of the screen (`prompt-presets`). Its fragment carries no root element of its own, and each preset
card carries a stable id (`preset-card-{id}`), so a load keeps the cards it does
not change. The panel holds no form that waits for a Save. The Add control of each
type, and the Edit and View controls of a card, load a form into the edit dialog
(`#edit-dialog-body`). The root of each such form (`data-dialog-form`) carries the
title of the dialog (`edit-dialog-title`). A form fragment that cannot open answers a
non-2xx error fragment, which the page shows in the notice place.

A create, an update, a delete, a duplicate and an activate answer nothing on
success: an empty 200. An invalid value answers a 400 error fragment, and a failure a
500 error fragment. The page shows the report of an action in the notice place, or in the
form that holds the typed input. The change reaches the screen when the panel loads.

## Scenarios

### Panel

#### Scenario 21.1: Panel renders the full surface

```gherkin
Given a fresh app state seeded with at least one default system preset and one default quantifier preset
When the client GET /fragment/prompt-presets
Then the response is 200
And the response body repeats no `prompt-presets-panel` root of its own
And the body contains an "System Prompts" heading
And the body contains an "Quantifier Prompts" heading
And the body contains one preset-card per seeded system preset (with a stable id, name and "Default" badge)
And the body contains one preset-card per seeded quantifier preset (with name and "Default" badge)
And the body contains an "Add System Prompt Preset" control
And the body contains an "Add Quantifier Prompt Preset" control
And the body contains an "Impersonate Prompts" heading
And the body contains an "Add Impersonate Prompt Preset" control
And the body holds no form and no textarea
```

#### Scenario 21.28: Each Add control opens the dialog

```gherkin
Given a fresh app state
When the client GET /fragment/prompt-presets
Then the response is 200
And each category has an Add control that loads `/fragment/prompt-presets/new?type={type}` into `#edit-dialog-body`
And the panel holds no disclosure
```

#### Scenario 21.42: The Add form is a dialog fragment for its type

```gherkin
Given a fresh app state
When the client GET /fragment/prompt-presets/new?type={type} for each of system, quantifier, impersonate and options
Then the response is 200
And the body is a dialog form (`data-dialog-form`) with the title `edit-dialog-title`
And the form posts to /prompt-presets and carries a hidden input named preset_type with that type
And the form has inputs named name, role, instructions and output_format
And the form has an input named writing_style for every type except quantifier
And the form has an error slot for that type and a Cancel control that closes the dialog
```

#### Scenario 21.43: The Add form of an unknown type is refused

```gherkin
Given a fresh app state
When the client GET /fragment/prompt-presets/new?type=bogus
Then the response is a 400
And the response body is `<div class="error-message">Invalid preset type</div>`
```

### GET /fragment/prompt-presets/{id}/edit — edit form

#### Scenario 21.4: Edit form returns a populated form for a non-default preset

```gherkin
Given a fresh app state with a seeded non-default system preset named "Editable"
When the client GET /fragment/prompt-presets/{id}/edit for that preset's id
Then the response is 200
And the response body contains a `preset-form edit-form` root that is a dialog form (`data-dialog-form`), with the title `edit-dialog-title` "Edit {name}"
And the body contains a form posting to /prompt-presets/{id}
And the body contains a hidden input named preset_type with value "system"
And the body contains a name input whose value is "Editable"
And the body contains textarea inputs named role, instructions, writing_style, output_format
And the body contains a "Save" button
And the body contains a "Cancel" button that closes the dialog
```

#### Scenario 21.5: Edit form for a nonexistent preset answers 404

```gherkin
Given a fresh app state
When the client GET /fragment/prompt-presets/does-not-exist/edit
Then the response is a 404
And the response body is `<div class="error-message">Preset not found</div>`
```

#### Scenario 21.6: Edit form for a default preset is refused

```gherkin
Given a fresh app state with a seeded default system preset
When the client GET /fragment/prompt-presets/{default_id}/edit
Then the response is a 400
And the response body is `<div class="error-message">Cannot edit default presets</div>`
```

### GET /fragment/prompt-presets/{id}/view — view form

#### Scenario 21.7: View form returns a read-only form

```gherkin
Given a fresh app state with a seeded default system preset named "Viewer"
When the client GET /fragment/prompt-presets/{id}/view for that preset's id
Then the response is 200
And the response body contains a `preset-form view-form` root that is a dialog form (`data-dialog-form`), with the title `edit-dialog-title` "View {name}"
And the body contains the preset name "Viewer"
And the body contains read-only fields for Role, Instructions, Writing Style, Output Format
And the body contains a "Close" button that closes the dialog
```

#### Scenario 21.8: View form for a nonexistent preset answers 404

```gherkin
Given a fresh app state
When the client GET /fragment/prompt-presets/does-not-exist/view
Then the response is a 404
And the response body is `<div class="error-message">Preset not found</div>`
```

### POST /prompt-presets — create

#### Scenario 21.9: Create a system preset → the panel shows the new preset

```gherkin
Given a fresh app state
When the client POST /prompt-presets with name="My System Prompt" and instructions="You are a test narrator." and preset_type="system"
Then the response is a 200 with an empty body
And a following GET /fragment/prompt-presets contains the preset name "My System Prompt"
And the body contains the preview text "You are a test narrator."
And the body contains a "Set Active" button for the new preset (it is not active)
And the body contains an "Edit" button for the new preset (it is not default)
```

#### Scenario 21.10: Create a quantifier preset → the panel shows the new preset

```gherkin
Given a fresh app state
When the client POST /prompt-presets with name="My Quantifier Prompt" and instructions="Quantify this scene." and preset_type="quantifier"
Then the response is a 200 with an empty body
And a following GET /fragment/prompt-presets contains the preset name "My Quantifier Prompt"
And the body contains the preview text "Quantify this scene."
```

#### Scenario 21.11: Create with an invalid preset_type answers non-2xx with the failure

```gherkin
Given a fresh app state
When the client POST /prompt-presets with name="Bad Type" and instructions="Test." and preset_type="invalid"
Then the response is a 400
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
Then the response is a 500
And the response body is one `<div class="error-message">` that holds the text of the failure (a non-2xx so htmx leaves the panel in place)
```

The same failure shape applies to `POST /prompt-presets/{id}` (update) and
`POST /{id}/delete` — a non-2xx carrying a
`<div class="error-message">…</div>` fragment that holds the text of the failure. Not
enumerated as separate scenarios; this one covers the shape.

### POST /prompt-presets/{id} — update

#### Scenario 21.14: Update a preset → the card shows the new name

```gherkin
Given a fresh app state with a seeded non-default system preset named "Before"
When the client POST /prompt-presets/{id} with name="After" and instructions="Updated text." and preset_type="system"
Then the response is a 200 with an empty body
And a following GET /fragment/prompt-presets lists the card of that preset with the new name "After"
And it does not contain the old name "Before"
And it preserves the stored allowed modes (the form omitted them)
```

#### Scenario 21.15: Update a nonexistent preset returns an error fragment

```gherkin
Given a fresh app state
When the client POST /prompt-presets/does-not-exist with name="Updated" and instructions="Updated." and preset_type="system"
Then the response is a 400
And the response body is `<div class="error-message">The preset "Updated" was not saved: Preset not found.</div>`, with the name typed in the form
```

#### Scenario 21.16: Update ignores the form's preset_type and keeps the stored type

```gherkin
Given a fresh app state with a seeded non-default system preset
When the client POST /prompt-presets/{id} with name="Updated" and instructions="Updated." and preset_type="quantifier"
Then the response is a 200 with an empty body
And the stored preset still has preset_type "system" and name "Updated"
```

#### Scenario 21.17: Update a default preset returns an error fragment

```gherkin
Given a fresh app state with a seeded default system preset
When the client POST /prompt-presets/{default_id} with name="Changed" and instructions="Changed." and preset_type="system"
Then the response is a 400
And the response body is `<div class="error-message">The preset "Changed" was not saved: Cannot edit default presets.</div>`, with the name typed in the form
```

#### Scenario 21.39: A failed preset edit answers non-2xx and carries the failure

```gherkin
Given an app state whose preset storage fails on save
And a seeded non-default system preset named "Original"
When the client POST /prompt-presets/{id} with name="Changed"
Then the response is a 500
And the body carries the update failure text
And the first line of the body is `The preset "Changed" was not saved`, with the name typed in the form
And the body does not name "Original"
And a following GET /fragment/prompt-presets still renders the preset named "Original"
```

### POST /prompt-presets/{id}/delete — delete

#### Scenario 21.41: A failed preset delete answers non-2xx and carries the failure

```gherkin
Given an app state whose preset storage fails on delete
And a seeded non-default system preset
When the client POST /prompt-presets/{id}/delete
Then the response is a 500
And the body carries the delete failure text
And a following GET /fragment/prompt-presets still renders the preset
```

#### Scenario 21.18: Delete a preset → empty body, and the panel no longer lists it

```gherkin
Given a fresh app state with a seeded non-default system preset
When the client POST /prompt-presets/{id}/delete
Then the response is 200
And the response body is empty
And a following GET /fragment/prompt-presets does not list the preset
```

#### Scenario 21.19: Delete a nonexistent preset returns an error fragment

```gherkin
Given a fresh app state
When the client POST /prompt-presets/does-not-exist/delete
Then the response is a 400
And the response body is `<div class="error-message">Preset not found</div>`
```

#### Scenario 21.20: Delete a default preset returns an error fragment

```gherkin
Given a fresh app state with a seeded default system preset
When the client POST /prompt-presets/{default_id}/delete
Then the response is a 400
And the response body is `<div class="error-message">Cannot delete default presets</div>`
```

#### Scenario 21.37: Delete a preset referenced as a mode default is refused

```gherkin
Given a fresh app state with a non-default system preset referenced as the Interactive Fiction bundle's system default
When the client POST /prompt-presets/{id}/delete
Then the response is a 400
And the response body contains "Preset is a mode default; change the default before deleting"
And the preset is not deleted
```

#### Scenario 21.38: Delete the active settings-level Options preset is refused

```gherkin
Given a fresh app state with a non-default Options preset set as the settings-level Options default
When the client POST /prompt-presets/{id}/delete
Then the response is a 400
And the response body contains "Preset is the default Options preset; change the default before deleting"
And the preset is not deleted
```

#### Scenario 21.40: Delete a preset that a Game uses is refused with the count of Games

```gherkin
Given a fresh app state with a non-default system preset
And one Game whose system preset is that preset
When the client POST /prompt-presets/{id}/delete
Then the response is a 400
And the response body is `<div class="error-message">Cannot delete preset used by 1 game</div>`
When a second Game takes that preset as its system preset
And the client POST /prompt-presets/{id}/delete
Then the response is a 400
And the response body is `<div class="error-message">Cannot delete preset used by 2 games</div>`
And the preset is not deleted
```

### POST /prompt-presets/{id}/duplicate — duplicate

#### Scenario 21.21: Duplicate a preset → the panel shows the copy

```gherkin
Given a fresh app state with a seeded non-default system preset named "Original"
When the client POST /prompt-presets/{id}/duplicate
Then the response is a 200 with an empty body
And a following GET /fragment/prompt-presets contains the copy name "Original (Copy)"
```

#### Scenario 21.22: Duplicate a nonexistent preset returns an error fragment

```gherkin
Given a fresh app state
When the client POST /prompt-presets/does-not-exist/duplicate
Then the response is a 400
And the response body is `<div class="error-message">Preset not found</div>`
```

### POST /prompt-presets/{id}/activate — activate

#### Scenario 21.23: Activate a system preset → the panel shows an Active badge

```gherkin
Given a fresh app state with a seeded non-default system preset that is not the active system preset
When the client POST /prompt-presets/{id}/activate?mode=novel
Then the response is a 200 with an empty body
And a following GET /fragment/prompt-presets contains an "Active · Novel" badge in the system preset's card-badges
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
Then the response is a 200 with an empty body
And a following GET /fragment/prompt-presets contains an "Active · Interactive Fiction" badge in that preset's card-badges
And the Interactive Fiction bundle's system slot holds that preset's id
And activating the same preset without the mode parameter answers a 400 with `<div class="error-message">Preset not allowed for novel mode</div>`
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
Then the response is a 400
And the response body is `<div class="error-message">Preset not found</div>`
```

#### Scenario 21.27: Duplicate → edit-form flags → save toggles per-mode activation

```gherkin
Given a fresh app state with a system preset whose allowed_modes are ["novel", "interactive_fiction"]
When the client POST /prompt-presets/{id}/duplicate (the copy carries the source's flags)
And the client GET /fragment/prompt-presets/{copy_id}/edit
Then the edit form renders the allowed_mode_novel and allowed_mode_if checkboxes checked
When the client POST /prompt-presets/{copy_id} with allowed_mode_novel and allowed_mode_if both set
Then the response is a 200 with an empty body
And a following GET /fragment/prompt-presets has the copy's card with a "Set Active (Interactive Fiction)" button
And it contains a "Set Active (Novel)" button
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
Then the response is a 200 with an empty body
And a following GET /fragment/prompt-presets contains a Quantifier preset named "Alpha"
```

#### Scenario 21.32: Editing a preset keeps its own name

```gherkin
Given a fresh app state with a System preset named "Alpha"
When the client POST /prompt-presets/{id} for that preset with the same name "Alpha" and changed instructions
Then the response is a 200 with an empty body
And a following GET /fragment/prompt-presets lists a preset card named "Alpha" and no error
```

#### Scenario 21.33: Duplicating the same preset twice yields two distinct copies

```gherkin
Given a fresh app state with a System preset named "Original"
When the client POST /prompt-presets/{id}/duplicate for that preset
Then the response is a 200 with an empty body
And a following GET /fragment/prompt-presets contains the copy name "Original (Copy)"
When the client POST /prompt-presets/{id}/duplicate for the same preset again
Then the response is a 200 with an empty body
And a following GET /fragment/prompt-presets contains the copy name "Original (Copy 2)"
And it still contains the first copy name "Original (Copy)"
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

### Options prompts — mode-agnostic

Options prompts have no `ModePresetBundle` slot, so the panel's Options
section is mode-agnostic: each card carries one Set Active button and
activating it writes the settings-level default
(`active_options_prompt_preset_id`). A game-level selection still takes
precedence at resolution time.

#### Scenario 21.35: The panel renders the Options Prompts section

```gherkin
Given a fresh app state with a seeded Options preset named "Custom Options"
When the client GET /fragment/prompt-presets
Then the response is 200
And the body contains an "Options Prompts" heading
And the body contains an "Add Options Prompt Preset" toggle whose form posts preset_type "options"
And the body contains one card per seeded Options preset
And that card offers a single "Set Active" button and no mode-specific activation buttons
```

#### Scenario 21.36: Activating an Options preset sets the settings-level default

```gherkin
Given a fresh app state with a seeded Options preset that is not the settings-level Options default
When the client POST /prompt-presets/{id}/activate
Then the response is a 200 with an empty body
And a following GET /fragment/prompt-presets renders the Options card with the "Active" badge and hides its "Set Active" button
And the stored settings-level Options default is that preset's id
```
