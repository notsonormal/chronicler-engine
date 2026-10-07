# Split Settings into Connections and Text Check sub-tabs

Type: task (AFK)
Status: claimed
Blocked by: —

## Question

Implement the layout decided in [Settings panel: roles, buttons and text-check controls](15-settings-panel-prototype.md) (see its Answer). Findings 4.1 and 4.2.

- **Sub-tabs.** The Settings panel gets two client-side sub-tabs: **Connections** (default) and **Text Check**. No URL or storage state: the panel loads once, so the selected sub-tab stays until a page reload. Use the tablist/tab/tabpanel pattern that ticket 46 shipped for the top-level tabs.
- **Role rows** at the top of Connections: Narrator and Quantifier only. Each has a connection `<select>` that applies at once, and the role's health from `GameViewQuery::role_health`: Healthy, Degraded (short message plus the shared `error_disclosure` popover), or No calls yet. An orange dot on the Connections sub-tab shows while either role is Degraded.
- **Remove** the "Set as Narrator" / "Set as Quantifier" buttons. Decide whether the select posts to the existing `set-narrator` / `set-quantifier` routes or to one new route; delete any route left without a caller.
- **Connection list.** One line per connection: name, provider and model, role tags, Edit and Delete. (Test comes in [Add a connection test](69-add-connection-test.md); leave room for it between Edit and Delete.)
- **Add and Edit** open one shared form as their own page inside the Connections sub-tab, with a "‹ Connections" back link. Save and Cancel return to the list. The inline card-swap edit form and the always-open Add form are removed.
- **Delete** is refused while Narrator or Quantifier uses the connection. The message names the role and points to the role rows. Remove the silent reassignment to `connections[0]` in `delete_connection_handler`. Keep the "last connection" refusal.
- **Text Check sub-tab.** The existing coupled `#text-check-card`, unchanged.

## Context

- Prototype: `tmp/ui-review/t15/settings-prototype.html?variant=D` (local only). The Roles page there became the role rows on Connections (Q13 → B), and its Options/Trigger rows are out (Q2 → C).
- Code: `src/adapters/driving/http/settings/` (templates and handlers), `src/adapters/driving/http/builders/connections.rs` (card and edit-form HTML builders), `assets/index.html`, `assets/styles.css`.
- Layout rules from ticket 19 hold: one 960px column, the tab body is the only scroll region.
- The stub fixture `tests/test_utils/stub_fixtures/settings.html` is a hand copy. Render it from the real template, as [Render the story-log and LLM Messages stub fixtures from the real templates](50-render-stub-fixtures-from-templates.md) did, or update it.

## Tests

- Tier 1 (`tests/http/settings.rs`, spec `docs/specs/settings.md`): the role select applies at once; delete is refused while a role uses the connection; the role rows show health. Rewrite scenario 20.1 for the new surface.
- Tier 2 (stub browser): sub-tab switching keeps its place across a top-level tab change; Edit and Add open the form page and the back link returns to the list.

## Done when

- `python build.py` is green, the user has reviewed the diff, and it is committed through `/commit-and-push`.
