# Stop the hand-copied stub fixtures from drifting

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Five stub fixtures are still hand copies of an Askama template: `tests/test_utils/stub_fixtures/action_area.html`, `games.html`, `prompt_presets.html`, `visual_sidebar.html`, `worlds.html`. [Remove the story-log ✓ and `POST /check-text`](../dashboard-ui-review/issues/70-remove-story-log-check.md) changed `ActionAreaTemplate`, then had to hand-edit `action_area.html` to match, and reported it as extra scope.

[Render the story-log and LLM Messages stub fixtures from the real templates](../dashboard-ui-review/issues/_resolved/50-render-stub-fixtures-from-templates.md) rendered two fixtures from their real templates and promised "the other six fixtures follow in a later ticket". That ticket was never filed. [Split Settings into Connections and Text Check sub-tabs](../dashboard-ui-review/issues/68-split-settings-sub-tabs.md) removed the sixth (`settings.html`).

How should the remaining five be kept in step?

- **Finish ticket 50's pattern.** Render each fixture from its real template; delete the hand copy.
- **Add a drift check.** A test renders the template and compares it to the fixture.
- **Drop the fixtures.** Serve the real templates through the stub server.

## Context

- `tests/test_utils/stub_fixtures/` — the five files above.
- Ticket 50's answer: "The other six fixtures follow in a later ticket."
- Ticket 70's report: "the ticket named only the stub server, but `tests/test_utils/stub_fixtures/action_area.html` is a hand-copied `ActionAreaTemplate` render, so I removed the same element there to keep the canned shape from drifting".
- `tests/STRATEGY.md` calls a hand-copied fixture "the stub tier's accepted tax".
