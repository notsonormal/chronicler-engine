# Settings panel: roles, buttons and text-check controls

Type: prototype (HITL)
Status: resolved
Blocked by: 08

## Question

How should the Settings panel look so that:

- the active Narrator and Quantifier connections are visible at the top,
- "Set as Narrator/Quantifier" buttons do not dominate the Edit/Delete controls, and
- Text Check controls cannot contradict each other ("Disabled" with "check before sending" ticked)?

Also: should the ✓ buttons in the story log show when text check is disabled?

## Context

- Findings 4.1, 4.2, 4.3. Screenshots 08, 10, 11.
- Blocked by [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md), because connection health may show on this panel.
- The "Add LlmProviderConfig" copy is in [Copy sweep](20-copy-sweep.md), not here.
- Added from [Keep the command form when a text-check result shows](03-keep-command-form-on-text-check.md): `checkCurrentInput()` in `assets/index.html` has no caller in `assets/`, `src/` or `tests/`, so the "✓ on the player input" button does not exist in the shipped UI. Decide it together with finding 4.3.
- Added from [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md): this panel is now the **only** place to check role health when nothing is wrong. The header keeps no always-on health element, so add a per-role health summary (Healthy / Degraded, from the newest LLM attempt per role) and a connection test here. The ambient signal is the failure banner ([51](51-add-failure-banner.md)), which does not replace this.

## Done when

- A rough prototype is linked from the ticket, and the user has reacted to it.
- The decision is in the ticket answer. Implementation tickets are created.

## Prototype

- `tmp/ui-review/t15/settings-prototype.html` (`?variant=A|B|C|D|E`, `?health=ok|bad`; D and E split the page after the user found one page overloaded); screenshots `tmp/ui-review/t15/{A,B,C,D,E}.png`. Local only.
- The served copy under `assets/prototype-t15/` was deleted when this ticket resolved.

## Answer

Resolved. Settings splits into sub-tabs, the role buttons become two selects, a connection test is added, and the story-log ✓ goes. The user judged one long page overloaded, so variants A–C (one page, three orders) lost to D (sub-tabs). Text Check's contradiction (finding 4.3) was already fixed by [Panel consistency and Options presets](67-panel-consistency-and-options-presets.md).

**Layout (Q6 → D, Q13 → B).** Settings has two client-side sub-tabs: **Connections** and **Text Check**. The Settings panel loads once and is not reloaded on a tab change, so the sub-tab stays where the user left it until a page reload (Q12 → A). No URL or storage state.

**Connections sub-tab.**
- **Role rows at the top (Q2 → C).** Narrator and Quantifier only. Each row has a connection `<select>` that applies at once (ticket 16's save model) and the role's health: Healthy, Degraded (short message plus the shared Details popover), or No calls yet. Health comes from `GameViewQuery::role_health`. The "Set as Narrator/Quantifier" buttons are removed (fixes 4.1 by removal). Options and Trigger get no row: they keep using the Narrator connection (and the hidden `options`-id rule in `src/bootstrap/wiring.rs`); their failures still show in the banner. An orange dot on the Connections sub-tab shows while Narrator or Quantifier is Degraded.
- **Connection list.** One line per connection: name, provider and model, role tags, then Edit · Test · Delete.
- **Add and Edit (Q8 → A, Q11 → A).** Both open one shared form as their own page inside the Connections sub-tab, with a "‹ Connections" back link. The inline card-swap edit form and the always-open Add form go.
- **Delete (Q10 → A).** Refused while Narrator or Quantifier uses the connection, with a message that names the role and points to the role rows. The silent move to `connections[0]` in `delete_connection_handler` goes. The "last connection" refusal stays.

**Connection test (Q3 → A, Q4 → C).** Test sends one short fixed prompt to a connection and shows the result inline: success with the reply time, or the error through the shared short-message-plus-popover shape. It writes **no** `llm_messages` row, so it never changes role health or the banner (ticket 08 defines health as the role's newest real attempt; a short prompt can pass where a real call fails). It runs only on a click. It is on each list row (saved values) and on the Add/Edit form (values typed in the form, before Save).

**Banner (Q9 → B).** No change. The banner gets no link to Settings.

**Story-log ✓ (Q5 → A).** Removed in every mode, not only while Disabled. It checked text already sent to the LLM; the check before sending covers the case that matters. `POST /check-text`, its handler, the `#text-check-result` slot, `checkText`/`checkLogText`/`checkCurrentInput`/`clearTextCheckResult`, the spec `docs/specs/text_check.md` scenario 34.1 and their tests go. Text Check then runs only before sending.

**Ruled out:** Options and Trigger rows, and an `options_connection_id` setting (Q2 option B), were offered and declined. Four always-on role tiles (variant C) and per-card health (variant B) lost to the role rows.

**Graduated:**
- [Split Settings into Connections and Text Check sub-tabs](68-split-settings-sub-tabs.md)
- [Add a connection test](69-add-connection-test.md) (blocked by 68)
- [Remove the story-log ✓ and `POST /check-text`](70-remove-story-log-check.md)
