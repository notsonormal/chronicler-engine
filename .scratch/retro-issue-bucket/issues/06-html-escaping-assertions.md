# Decide how the test suite asserts HTML escaping

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Askama's HTML escaper emits numeric entities: `<` becomes `&#60;`. The hand-written escaper behind `render_error` emits `&lt;`. The repo asserts both forms, one per escaper, with no written rule. [Split Settings into Connections and Text Check sub-tabs](../dashboard-ui-review/issues/68-split-settings-sub-tabs.md) asserted `&lt;script&gt;` for an Askama template, failed twice, then read the rendered output and switched to `&#60;script&#62;`.

How should an escaping test state its claim?

- **Assert the raw markup is absent.** `!output.contains("<script>")` works for either escaper.
- **Add a shared helper.** `assert_escaped(output, raw)` owns the entity knowledge in one place.
- **Add a guardrail.** Flag an entity-specific assertion such as `contains("&lt;script&gt;")`.

## Context

- `src/adapters/driving/http/settings/templates/settings_tests.rs:181` — `&#60;script&#62;`.
- `src/adapters/driving/http/utils/response_tests.rs:94` — `&lt;script&gt;`.
- `src/adapters/driving/http/templates_tests.rs:578` — `&lt;script&gt;`.
- `src/adapters/driving/http/error_tests.rs:69` — `&lt;script&gt;`.
- The t68 session failed at entry idx 270 (`assertion failed: html.contains("&lt;script&gt;")`) and fixed it at idx 280.
