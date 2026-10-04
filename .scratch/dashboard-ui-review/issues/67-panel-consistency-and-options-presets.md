# Panel consistency and Options presets

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Implements the decisions in
[Decide one save model for panel forms](16-decide-save-model.md),
[Decide the panel layout convention and supported viewports](19-decide-layout-convention.md)
and [Decide whether Options presets appear in Prompt Presets](18-decide-options-presets.md).
Theme 4 of the review plus the Options-preset wiring, done as one sweep because
both rewrite the Prompt Presets and Games templates and handlers, and both touch
the Settings/Worlds panels.

### Save model — option E, hybrid by control type

- **Instant**, with visible feedback: connection *Set as Narrator* / *Set as
  Quantifier* / *Delete*; Text Check mode and *check before sending*; World
  Narrator Mode / Perspective / Tense.
- **On Save:** connection Add and Edit (already so); World details — Name,
  Description, Global Rules, Default Room Image, `options_always_on`, Map JSON,
  Scenarios JSON.
- **Couple the Text Check pair.** Mode "Disabled" disables and clears "check
  before sending". Remove its Save button and the `#settings-status` "saved!"
  line (finding 4.3).
- **World posture group.** Keep the selects auto-saving, but move them into a
  labelled group ("Posture — saves automatically") with their own status,
  visually separate from the details form. State on the page that Cancel applies
  to the details form only, so the mix is visible rather than confusing
  (finding 4.4).
- **Fix the stale status (finding 05.F8).** Submitting the details form clears
  the posture group's green "Saved", so a failed Update World cannot sit under a
  green confirmation.
- **Delete `POST /settings`** and `save_settings_handler`, plus the tests that
  cover only that route.

### Layout — option A1

- One centered content column for every panel: `max-width` 960px, 24px padding.
  Replaces Settings/Games at 800px, Prompt Presets at 900px, Worlds with no cap.
- The tab body is the only scroll region. Remove the inner scroll boxes from
  Games, Prompt Presets, and Worlds; Settings already scrolls the whole tab.
- Worlds: give the form view the same card frame as the list view, or remove the
  card inset in both, so the text inset does not change between views.
- The tab bar scrolls horizontally below 1024px, so it cannot overflow.
- Declare the viewports in
  `docs/diataxis/reference/frontend/ui_design.md`: ≥1024×700 fully supported;
  768–1024 best-effort; phone out of scope.

### Options presets — option C

- Add an "Options Prompts" section to the Prompt Presets tab, beside System,
  Quantifier, and Impersonate: list, view, edit, activate. Activation sets the
  settings-level default (`active_options_prompt_preset_id`), because Options is
  mode-agnostic and has no bundle slot.
- Add a fourth select to the Games tab preset picker so a game chooses its own
  Options preset. The game already stores this id and the resolution already
  prefers it over the settings default.
- Extend the Prompt Presets handler and the Games picker form/handler to include
  Options. Today nothing calls `list_presets(PresetType::Options)`.
- The second seed (`data/prompt_presets/options/roadway_domains.json`) becomes
  reachable.

## Context

- Check the posture specs (`docs/specs/worlds.md` 25.5,
  `docs/specs/browser_worlds.md` 29.2, `docs/specs/games.md` 20.8). They stay
  true; a scenario changes only if it asserts the removed Text Check Save button
  or the `#settings-status` line.
- The Settings-panel internals also appear in
  [Settings panel: roles, buttons and text-check controls](15-settings-panel-prototype.md).
  Coordinate: 15 owns the Settings look; this ticket owns the Text Check save
  behaviour. If 15 resolves first, apply its design here.
- `git status` for overlapping edits before starting.

## Done when

- The save-model and layout rules are stated in the reference docs.
- The Options section and the per-game selector exist, and spec scenarios cover
  them.
- `python build.py` is green, the user reviews the diff, then commit through
  `/commit-and-push`.

## Answer

Resolved. Theme 4 plus the Options-preset wiring.

- **Save model (option E, hybrid by control type).** Instant-with-feedback: connection
  Set-as-Narrator/Quantifier/Delete, Text Check mode + check-before-sending, World
  Narrator Mode/Perspective/Tense. On Save: connection Add/Edit and World details. The
  Text Check pair is coupled (Disabled clears and disables the box) and lost its Save
  button and the `#settings-status` "saved!" line. The World posture selects stay
  auto-saving in a labelled group separate from the details form, whose Cancel scope is
  stated on the page; submitting details clears the group's stale "Saved". `POST
  /settings` and `save_settings_handler` were deleted with their tests.
- **Layout (option A1).** One centered 960px/24px column for every panel; the tab body is
  the only scroll region (inner scroll boxes removed from Games, Prompt Presets, Worlds);
  Worlds' form and list share one card frame; the tab bar scrolls horizontally below
  1024px. Viewports declared in `ui_design.md` (>=1024x700 supported, 768-1024
  best-effort, phone out of scope).
- **Options presets (option C).** An "Options Prompts" section in the Prompt Presets tab
  (list/view/edit/activate; activation sets the settings-level default) and a fourth
  Options select in the Games picker. The second seed is reachable.

Review follow-ups applied: `SelectOptionView`'s two constructors share one `(missing)`
tail (`every_preset`); `role_effect` uses the `AGENT_*` constants; `PresetType::Options`
is compared as the enum; the world posture selects are single-sourced in Rust with a
create/edit flag; the text-check invariant is single-sourced on
`TextCheckSettings::effective_enable_auto_check`; `set_preset_selection` takes a
`PresetSelection`; `ui_design.md`'s Panel Save Model defers behaviour to the specs. The
active-Options-default delete refusal was kept (it mirrors the mode-default guard) and
given tier-1 coverage (21.37/21.38).

Tests: tier 1 `settings.rs` (20.12/20.13), `games_config.rs` (20.11), `prompt_presets.rs`
(21.35–21.38); unit `catalogue_tests.rs`, `settings_tests.rs`.

`python build.py` is green (1566 integration, 62 browser). Uncommitted, pending review
and `/commit-and-push`.
