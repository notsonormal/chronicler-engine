# Plan: Spell & Grammar Check Integration

## Objective
Integrate spell-checking and grammar-checking (harper-core) into the Chronicler Engine.

1. **Automatic pre-flight check**: Before player input reaches the LLM, the engine checks it and — if issues are found — shows a preview UI where the player can choose the corrected or original text.
2. **Manual "Check Text" button**: A reusable UI button that can run the checker on any text (player input or existing LLM narration). This is on-demand, not automatic.

## Tech Stack
- **harper-core** (`harper-core` crate, v2.x): Pure-Rust grammar + spell linter from Automattic. Handles both spell suggestions and grammar suggestions in the preview. Actively maintained by Automattic, Apache-2.0 licensed.
- Uses harper-core's built-in FST dictionary (~8MB stripped, ~130MB full) and configurable `LintGroup` to enable/disable spell and grammar rules independently.

## Assumptions
1. The check is **global** (one setting for the entire engine), not per-connection.
2. harper-core operates on in-memory text only — no async I/O during checking.
3. Fantasy names, place names, and game-specific terms will be added to a personal/ignore dictionary over time; initial MVP uses stock English dictionaries.
4. LLM output is never auto-corrected.

## Detailed UI Design

This section describes exactly how the spell/grammar check UI integrates with the existing Chronicler Engine dashboard.

### Existing UI Context
The dashboard uses a dark theme (`#0a0a0a` background) with green/cyan/orange accents. The Game tab has three zones:
1. **Story log** (left, 80%) — chat bubbles for narration, dialogue, system, and player input
2. **Visual sidebar** (right, 20%) — location image + NPC portraits
3. **Action area** (bottom) — input field, Send button, action hints, status

Log entries already have inline action buttons: an edit button (✎) and a retry button (↻). The action area uses `#command-form` with an `<input>` and a green Send button.

---

### 1. Automatic Pre-Flight Check (Player Input → LLM)

When the player hits **Send**, instead of immediately POSTing to `/action`, the form POSTs to `/action/check`.

**If no issues are found:** The server silently forwards to the existing action handler. The player experiences zero change.

**If issues are found:** The server returns a preview fragment that replaces the action area via HTMX (`hx-swap="outerHTML"`).

#### Preview Fragment Layout

The preview appears **in place of the action area**, using the same `#command-form` container so the player never loses context.

```html
<div class="action-area" id="action-area">
  <div class="text-check-preview">
    <div class="preview-header">
      <span class="preview-icon">&#x270D;</span>
      <span>Did you mean?</span>
    </div>
    <div class="preview-compare">
      <div class="preview-original">
        <label>Original</label>
        <span>go to the casle</span>
      </div>
      <div class="preview-arrow">&#x2192;</div>
      <div class="preview-corrected">
        <label>Corrected</label>
        <span>go to the castle</span>
      </div>
    </div>
    <div class="preview-issues">
      <span class="issue-tag spell">Spelling: "casle" &rarr; "castle"</span>
      <span class="issue-tag grammar">Grammar: missing article</span>
    </div>
    <div class="preview-actions">
      <form hx-post="/action" hx-target="#status-display" hx-swap="innerHTML"
            hx-on::after-request="document.getElementById('action-area').innerHTML = originalFormHTML;">
        <input type="hidden" name="command" value="go to the castle" />
        <button type="submit" class="btn-corrected">Send Corrected</button>
      </form>
      <form hx-post="/action" hx-target="#status-display" hx-swap="innerHTML"
            hx-on::after-request="document.getElementById('action-area').innerHTML = originalFormHTML;">
        <input type="hidden" name="command" value="go to the casle" />
        <button type="submit" class="btn-original">Send Original</button>
      </form>
      <button class="btn-cancel" onclick="restoreActionArea()">Cancel</button>
    </div>
  </div>
</div>
```

**CSS additions** (matching existing design tokens):
- `.text-check-preview` — uses `--color-bg-header` background, `--color-border` border, 8px radius
- `.preview-header` — `--color-accent-cyan` icon + text, small bold header
- `.preview-compare` — flex row with `.preview-original` (muted, strikethrough-ish) and `.preview-corrected` (green accent)
- `.preview-arrow` — centered arrow, muted color
- `.issue-tag.spell` — orange tint (`--color-accent-orange`)
- `.issue-tag.grammar` — pink tint (`--color-accent-pink`)
- `.btn-corrected` — green gradient (same as Send button)
- `.btn-original` — neutral gradient (same as settings buttons)
- `.btn-cancel` — transparent, `--color-text-muted`, hover red

After the player clicks **Send Corrected** or **Send Original**, the action area restores to its normal form (`#command-form`) via the `hx-on::after-request` handler.

---

### 2. Manual "Check Text" Button

A small **"Check"** button is added to the action area, next to the Send button.

#### Action Area Modification

```html
<form id="command-form" hx-post="/action" ...>
  <input type="text" name="command" placeholder="Enter command..." ... />
  <button type="button" class="btn-check" onclick="checkCurrentInput()"
          title="Check spelling & grammar">&#x2713;</button>
  <button type="submit" id="submit-btn"><span class="btn-icon">&#9654;</span> Send</button>
</form>
```

**`.btn-check`** styling:
- Same height as Send button (`--input-height: 40px`)
- Background: transparent with `--color-accent-cyan` border
- Color: `--color-accent-cyan`
- Hover: subtle cyan glow (`box-shadow: 0 0 12px rgba(0, 255, 255, 0.25)`)
- Disabled when input is empty

When clicked, it POSTs the current input value to `/check-text` and returns the **same preview fragment** described above, but with different button labels:
- **Use Corrected** → replaces the input field's value with the corrected text, then restores the action area
- **Keep Original** → simply restores the action area
- **Cancel** → restores the action area

The JS `checkCurrentInput()` function reads `document.querySelector('#command-form input[name="command"]').value` and POSTs it via `fetch()` or HTMX.

---

### 3. Settings UI

In the **Settings tab**, below the Connections list, a new **Text Check** card is added.

```html
<div class="connection-card">
  <div class="card-header">
    <span class="card-title">Text Check</span>
  </div>
  <div class="card-details">
    Spell and grammar checking for player input.
  </div>
  <form hx-post="/settings/text-check" hx-target="closest .connection-card" hx-swap="outerHTML">
    <div class="form-group">
      <label for="check_mode">Check Mode</label>
      <select name="check_mode" id="check_mode">
        <option value="disabled" selected>Disabled</option>
        <option value="spell">Spell Check Only</option>
        <option value="grammar">Grammar Check Only</option>
        <option value="spell_grammar">Spell + Grammar</option>
      </select>
    </div>
    <div class="form-group">
      <label>
        <input type="checkbox" name="enable_auto_check" checked />
        Check before sending to LLM
      </label>
    </div>
    <div class="form-actions">
      <button type="submit" class="primary">Save</button>
    </div>
  </form>
</div>
```

This reuses the existing `.connection-card`, `.form-group`, and `.form-actions` classes from the settings CSS, so it visually matches the connection cards above it.

---

### Summary of UI Changes

| File | Change |
|------|--------|
| `assets/index.html` | Add `btn-check` to `#command-form`; add `checkCurrentInput()` and `restoreActionArea()` JS helpers |
| `assets/styles.css` | Add `.text-check-preview`, `.preview-*`, `.issue-tag`, `.btn-check` classes |
| `server/templates.rs` | Add `TextCheckPreviewTemplate` struct + Askama template |
| `server/settings_fragment.rs` | Add text-check settings card |
| `server/fragments.rs` | Add `/action/check` and `/check-text` handlers; modify action form target |

## Integration Points

| Direction | Location | Action |
|-----------|----------|--------|
| **To LLM** (primary) | New `server::action_check_handler` | Runs check on `command`. If issues found, returns preview fragment. If clean or Disabled, forwards to existing action flow. |
| **Manual** (on-demand) | New `server::check_text_handler` | Accepts any text via POST, returns preview fragment. Used by the "Check Text" button in the UI. |
| **UI** | `server::templates`, `server::fragments` | New preview template + manual check button in action area. |

## Configuration

Add to `model::settings::AppSettings`:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum TextCheckMode {
    #[default]
    Disabled,
    Spell,        // harper-core spell lint only
    Grammar,      // harper-core grammar lint only
    SpellGrammar, // harper-core: both spell and grammar lints
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextCheckSettings {
    pub mode: TextCheckMode,
    pub enable_auto_check: bool, // default true
}
```

Stored in `settings.json` alongside existing `AppSettings`. Settings UI gets a dropdown for mode + checkbox to enable/disable the automatic pre-flight check.

## Project Structure

```
src/
  narrative/
    text_check/
      mod.rs              # Facade: check_input(), check_output()
      harper_backend.rs   # Spell + grammar lint + suggest + diff
      types.rs            # CheckIssue, SuggestedCorrection, CheckResult
  server/
    fragments.rs          # New action_check_handler + check_text_handler
    templates.rs          # New TextCheckPreviewTemplate
    settings_fragment.rs  # Add text_check controls
```

## Code Style

Standard Chronicler conventions: Rust 2024 edition, `Result` over panic, `EngineError` propagation, doc anchors to `docs/system/text_check.md`.

Example API:

```rust
// [DOC: docs/system/text_check.md]
pub struct CheckResult {
    pub original: String,
    pub corrected: String,
    pub issues: Vec<CheckIssue>,
}

pub struct CheckIssue {
    pub span: Range<usize>,
    pub message: String,
    pub suggestion: Option<String>,
}

pub fn check_player_input(text: &str, mode: TextCheckMode) -> Result<Option<CheckResult>, EngineError> {
    // Returns None if mode is Disabled or no issues found
}
```

## Testing Strategy

- Unit tests in `text_check/`:
  - Known misspellings produce suggestions
  - Grammar issues produce suggestions
  - XML tags / commands (e.g. `<PlayerInput>`, `[Look]`) are untouched
  - Fantasy names can be added to ignore list
  - harper-core lint configuration respects `TextCheckMode`
- Integration tests:
  - Preview endpoint returns fragment when issues exist
  - Preview endpoint forwards when mode is Disabled
  - Settings JSON round-trip
- `python build.py` fast suite.

## Boundaries
- **Always**: Preserve prompt structure (XML tags, markdown). Never break `Action` parsing.
- **Ask first**: Adding new dictionary files to repo, changing default check mode.
- **Never**: Auto-correct LLM output silently. Block the user with a mandatory preview (original must always be sendable). Commit dictionary secrets.

## Success Criteria
- [ ] `AppSettings` gains `text_check: TextCheckSettings` and serializes correctly.
- [ ] `TextCheckMode::Spell` previews spelling corrections before sending.
- [ ] `TextCheckMode::Grammar` previews grammar corrections before sending.
- [ ] `TextCheckMode::SpellGrammar` previews both.
- [ ] Player can always choose "Send Original" to bypass corrections.
- [ ] A "Check Text" button can run the checker on-demand against any text (input or log entry).
- [ ] All modes can be disabled (`Disabled`).
- [ ] `python build.py` passes (fmt + clippy + tests + arch-lint).

## Decisions
- **Dictionary**: Use harper-core's stripped dictionary (~8MB) to keep binary size reasonable.
- **Ignored words**: Persisted globally in `TextCheckSettings.ignored_words: Vec<String>`.
- **No LLM output checking**: Removed from scope. Only player input is checked (automatically or on-demand).

## Implementation Phases

### Phase 1 — Core text_check module
Add `narrative::text_check` with harper-core backend, `CheckResult`/`CheckIssue` types, and the `check_player_input()` facade. No UI changes yet.

### Phase 2 — Preview endpoint + templates
Add `/action/check` handler, `TextCheckPreviewTemplate`, and modify the action form to post to the new endpoint. Wire up "Send Corrected" / "Send Original" buttons.

### Phase 3 — Settings UI + manual check button
Add `TextCheckSettings` to `AppSettings`, expose dropdown in settings fragment. Add a reusable "Check Text" button to the action area and/or log entries that posts to `/check-text`.

## Chosen Approach

**Harper-Only (Option B)**
Uses only `harper-core` for both spell and grammar checking. Spell issues and grammar issues both come from harper, surfaced in the same preview UI.

**Rationale:**
- Single dependency keeps compile times and binary size lower.
- Harper-core is actively maintained by Automattic and covers both concerns.
- The preview UI makes grammar checking actionable, removing the need for a separate specialized spell-checker.
- Simpler code and maintenance burden.
