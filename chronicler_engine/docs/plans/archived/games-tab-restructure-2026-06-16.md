# Games Tab Restructure & Remove Modal Dependency

## Context

The "Save / Load" tab crams Active Game, Saved Games, New Game (a `<details>` dropdown), and Reset Current Game into one panel with poor visual hierarchy. The New Game form is jammed beside Reset and lacks space for future fields. Meanwhile, the Worlds tab uses a `#world-modal` overlay that fights HTMX's natural inline-swap flow and isn't the right pattern.

**End state:** Both the Games and Worlds panels use inline HTMX swaps exclusively — no modals. The Games panel has clear vertical hierarchy (Active → New Game → Saved Games), Reset is a small action on the Active card, and New Game renders as a real form with room for future expansion. The tab label changes from "Save / Load" to "Games".

---

## Approach

### Step 1 — Remove world-modal from index.html and rewire Worlds to inline swap

The Worlds panel currently uses `openWorldModal()` JS + `#world-modal` overlay. Replace with pure HTMX inline swap matching the pattern already used by the form submission (`hx-target=".worlds-panel" hx-swap="outerHTML"`).

**`chronicler_engine/assets/index.html`:**
- Delete the `<!-- Create/Edit World Modal -->` div (lines 122–133) — the entire `<div id="world-modal" …>…</div>` block.
- Delete `window.openWorldModal` and `window.closeWorldModal` function definitions (lines 573–581).
- Delete the modal overlay-click listener (lines 583–589).

**`chronicler_engine/src/server/worlds_fragment/template.rs` — `WorldsPanelTemplate`:**
- Replace `<button class="btn-new-world" onclick="openWorldModal()">Create New World</button>` with:
  ```html
  <button class="btn-new-world" hx-get="/fragment/worlds/new" hx-target=".worlds-panel" hx-swap="outerHTML">Create New World</button>
  ```
- Replace Edit button `<button onclick="openWorldModal('{{ world.key }}')">Edit</button>` with:
  ```html
  <button hx-get="/worlds/{{ world.key }}/edit" hx-target=".worlds-panel" hx-swap="outerHTML">Edit</button>
  ```
- The `WorldFormTemplate` already has `hx-target=".worlds-panel" hx-swap="outerHTML"` on its form, so submitting create/update will swap the panel back to the list view. No change needed there.
- Add a **Cancel** button to `WorldFormTemplate` (currently the form has only a submit button; without the modal close button, there's no way back to the list):
  ```html
  <button type="button" hx-get="/fragment/worlds" hx-target=".worlds-panel" hx-swap="outerHTML">Cancel</button>
  ```
  Place it beside the submit button inside the form.

**`chronicler_engine/assets/styles.css`**: Remove `.modal-overlay`, `.modal`, `.modal-header`, `.modal-close`, `.modal-body`, `.modal-actions` rules (lines 1867–1936). These are now unused.

### Step 2 — Restructure Games template: vertical hierarchy, inline New Game form, Reset on Active card

Replace the current `GamesPanelTemplate` HTML with a reorganized layout:

**New structure of the template:**
```
<div class="save-load-panel">
  <!-- Active Game → with Reset as small icon button -->
  <div class="save-load-section">
    <h2>Active Game</h2>
    {active game card — same content + Reset button on the card}
  </div>

  <!-- New Game → inline form, always visible -->
  <div class="save-load-section new-game-section">
    <h2>New Game</h2>
    <form hx-post="/games" hx-swap="none">
      <select name="world_key" required>…worlds…</select>
      <button type="submit" class="btn-primary">Start New Game</button>
    </form>
  </div>

  <!-- Saved Games → list below -->
  <div class="save-load-section">
    <h2>Saved Games</h2>
    {same game items as before}
  </div>
</div>
```

**Concrete edits to `chronicler_engine/src/server/games_fragment/template.rs`:**

Replace the entire `source = r#"…"#` string of `GamesPanelTemplate` with:

```html
<div class="save-load-panel">
    <div class="save-load-section">
        <h2>Active Game</h2>
        {% match active_game %}
        {% when Some(game) %}
        <div class="game-item active">
            <div class="active-game-info">
                <span class="game-name">{{ game.name }}</span>
                <span class="world-badge">{{ game.world_name }}</span>
                <span class="game-badge">Current</span>
            </div>
            <button class="btn-reset-small" hx-post="/reset" hx-confirm="Reset the current game? All progress will be lost." hx-swap="none" title="Reset game">&#x21bb;</button>
        </div>
        {% when None %}
        <div class="game-item"><span class="game-name">No active game</span></div>
        {% endmatch %}
    </div>

    <div class="save-load-section new-game-section">
        <h2>New Game</h2>
        {% if worlds.is_empty() %}
        <div class="games-empty">No worlds available. Create a world first.</div>
        {% else %}
        <form class="new-game-form" hx-post="/games" hx-swap="none">
            <div class="form-row">
                <select name="world_key" required>
                    {% for world in worlds %}
                    <option value="{{ world.key }}" title="{{ world.description }}">{{ world.name }}</option>
                    {% endfor %}
                </select>
                <button type="submit" class="btn-primary">Start New Game</button>
            </div>
        </form>
        {% endif %}
    </div>

    <div class="save-load-section">
        <h2>Saved Games</h2>
        <div class="games-list">
            {% if saved_games.is_empty() %}
            <div class="games-empty">No saved games.</div>
            {% else %}
            {% for game in saved_games %}
            <div class="game-item" data-id="{{ game.id }}">
                <span class="game-name">{{ game.name }}</span>
                <span class="world-badge">{{ game.world_name }}</span>
                <div class="game-actions">
                    <button class="btn-switch" hx-post="/games/{{ game.id }}/switch" hx-swap="none">Switch</button>
                    <button class="btn-delete" hx-post="/games/{{ game.id }}/delete" hx-target="closest .game-item" hx-swap="outerHTML" hx-confirm="Delete this game? This cannot be undone.">Delete</button>
                </div>
            </div>
            {% endfor %}
            {% endif %}
        </div>
    </div>
</div>
```

Key changes from current:
- New Game is **not** inside `<details>` — it's a full section with the form always visible. Uses `class="new-game-section"` and `class="new-game-form"`.
- Reset button moves from the bottom `.save-load-actions` into the Active Game card as `btn-reset-small` (↺ character, title attribute for tooltip).
- The old `.save-load-actions` div with `btn-new-game` and `btn-reset` is removed entirely.
- Active game card gains a `<div class="active-game-info">` wrapper so the Reset button sits to the right.
- The form uses a `.form-row` div so select + button sit side-by-side (flex-row).

The `GamesPanelTemplate` struct fields (`active_game`, `saved_games`, `worlds`) stay identical — no handler changes needed.

### Step 3 — Update CSS for restructured Games panel

**`chronicler_engine/assets/styles.css`** — make these edits in the save-load section (lines ~1569–1763):

**Remove** (no longer used):
- `.save-load-panel .world-picker` and its children (`summary`, `select`, `.btn-primary`) — lines 1654–1678
- `.save-load-panel .btn-new-game` and `:hover` — lines 1742–1752
- `.save-load-panel .btn-reset` and `:hover` — lines 1754–1763
- `.save-load-actions` — lines 1686–1692

**Add** (new classes):

```css
/* Active Game Card — info + reset layout */
.active-game-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    flex: 1;
    min-width: 0;
}

.active-game-info .game-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.btn-reset-small {
    background: none;
    border: 1px solid var(--color-accent-red);
    color: var(--color-accent-red);
    border-radius: 4px;
    padding: 4px 8px;
    cursor: pointer;
    font-size: 16px;
    line-height: 1;
    flex-shrink: 0;
    transition: background var(--transition-fast), box-shadow var(--transition-fast);
}

.btn-reset-small:hover {
    background: rgba(255, 68, 68, 0.15);
    box-shadow: 0 0 6px rgba(255, 68, 68, 0.25);
}

/* New Game Section */
.new-game-form {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-sm);
}

.new-game-form .form-row {
    display: flex;
    gap: var(--spacing-sm);
    align-items: stretch;
}

.new-game-form select {
    flex: 1;
    padding: 8px 12px;
    background: var(--color-bg-primary, #1a1a2e);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border-primary, #333);
    border-radius: 4px;
    font-size: var(--font-size-base);
    font-family: inherit;
}

.new-game-form select:focus {
    outline: none;
    border-color: var(--color-accent-green);
    box-shadow: 0 0 8px rgba(0, 255, 0, 0.2);
}

.new-game-form .btn-primary {
    background: linear-gradient(180deg, #2a5a2a 0%, #1a4a1a 100%);
    border: 1px solid var(--color-accent-green);
    color: var(--color-accent-green);
    padding: 8px 20px;
    font-weight: 600;
    white-space: nowrap;
    cursor: pointer;
    border-radius: 4px;
    font-size: var(--font-size-base);
    font-family: inherit;
    transition: background var(--transition-fast), box-shadow var(--transition-fast);
}

.new-game-form .btn-primary:hover {
    background: linear-gradient(180deg, #3a6a3a 0%, #2a5a2a 100%);
    box-shadow: 0 0 10px rgba(0, 255, 0, 0.25);
}
```

Note: `.game-item.active` already has `display: flex; align-items: center; justify-content: space-between;` and the border/color/box-shadow styles. The new `.active-game-info` wrapper handles the left side; the Reset button naturally floats to the right via the existing `justify-content: space-between`. No override needed for `.game-item.active`.

### Step 4 — Rename tab from "Save / Load" to "Games"

**`chronicler_engine/assets/index.html`:**
- Change tab button text: `<button class="tab" data-tab="save-load">Save / Load</button>` → `<button class="tab" data-tab="save-load">Games</button>`
- The `data-tab` attribute and `id="save-load-tab"` stay as-is to avoid renaming the tab content div and all references. Only the visible label changes.

### Step 5 — Clean up orphaned modal references

This is a consolidation step ensuring nothing from the modal remains:

- In `index.html`: verify the three deletions from Step 1 (modal div, JS functions, overlay listener) are complete.
- In `styles.css`: verify the modal CSS rules (`.modal-overlay`, `.modal`, `.modal-header`, `.modal-close`, `.modal-body`, `.modal-actions`, `.modal-actions button`) are removed — lines ~1867–1936.
- In `index.html`: the `htmx:beforeSwap` error handler added in the prior bugfix session (the block that calls `showError()` on `isError`) must remain — it is unrelated to the modal and provides global HTMX error visibility.

---

## Critical files & anchors

| File | Anchor | Why |
|---|---|---|
| `chronicler_engine/src/server/games_fragment/template.rs:14-68` | `GamesPanelTemplate` source string | The HTML being restructured — Step 2 |
| `chronicler_engine/src/server/worlds_fragment/template.rs:28-44` | `WorldsPanelTemplate` source string | Buttons being changed from modal JS to inline hx-get — Step 1 |
| `chronicler_engine/src/server/worlds_fragment/template.rs:55-90` | `WorldFormTemplate` source string | Needs Cancel button added — Step 1 |
| `chronicler_engine/assets/index.html:20-27` | Tab bar buttons | "Save / Load" → "Games" label — Step 4 |
| `chronicler_engine/assets/index.html:122-133` | `#world-modal` div | Deleted entirely — Step 1 |
| `chronicler_engine/assets/styles.css:1569-1763` | `.save-load-panel` through `.btn-reset:hover` | CSS being added/removed/replaced — Step 3 |
| `chronicler_engine/assets/styles.css:1867-1940` | Modal styles + `.worlds-panel .add-world-btn` | Modal CSS to remove — Step 1 |

## Verification

1. **Build**: `cd chronicler_engine && cargo check` — must pass.
2. **Manual UI — Games tab**: Start server (`cargo run -- --world redmist_estate --port 3000`), open `http://127.0.0.1:3000`, click "Games" tab. Confirm:
   - Active Game card shows game name, world badge, "Current" badge, and a small ↺ reset button.
   - New Game section shows world dropdown + "Start New Game" button side-by-side.
   - Saved Games section lists existing games with Switch/Delete.
   - No "Reset Current Game" button at the bottom.
3. **Manual UI — Worlds tab**: Click "Worlds" tab. Confirm:
   - "Create New World" button, when clicked, replaces the worlds list with the create form inline (no modal overlay).
   - "Edit" button on a world replaces the list with the edit form inline.
   - "Cancel" button on form returns to the worlds list.
   - No `#world-modal` overlay exists in DOM.
4. **Functional — New Game**: Select a world from dropdown, click "Start New Game". Game creates and the page refreshes (matching existing `ok_refresh()` from `create_game_handler`).
5. **Functional — Reset**: Click ↺ on Active Game card, confirm dialog appears, accept → game resets.
6. **Functional — World Create**: Click "Create New World" → form appears → fill in key + name → submit → page refreshes, world appears in list.

## Assumptions & contingencies

- **No persona dropdown in this plan.** The user explicitly said to exclude it. The `.form-row` structure supports adding a second select or more fields later — just add more elements inside `.form-row` or add additional `.form-row` divs.
- **`.save-load-panel` class name stays.** Renaming it to `.games-panel` would require touching the CSS and the HTML simultaneously, with no functional benefit. The tab label changing to "Games" is sufficient for discoverability.
- **World form Cancel uses `hx-get="/fragment/worlds"`.** The response from the existing `list_worlds_fragment` handler is already wrapped in `<div class="worlds-panel">`, so targeting `.worlds-panel` with `outerHTML` swap replaces it correctly. Cancel button: `<button type="button" hx-get="/fragment/worlds" hx-target=".worlds-panel" hx-swap="outerHTML">Cancel</button>`.
