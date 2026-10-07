# Decide how a settings test slices a rendered panel

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The settings template tests slice one row out of a rendered panel with a class-prefix string. The anchor `<div class="connection-row` also matches `<div class="connection-row-actions">`, so the slice ended before the row's actions and the escaping assertion failed for the wrong reason. The agent renamed three CSS classes (`connection-row-info`, `connection-row-roles`, `connection-row-actions`) to make the helper work.

How should a test isolate one rendered row?

- **Anchor precisely.** Match the row's own closing delimiter, not a prefix.
- **Parse the fragment.** Use an HTML parser in the test helper.
- **Restructure.** Give each row a stable `id` the helper targets.

## Context

- `src/adapters/driving/http/settings/templates/settings_tests.rs` — the `card_html_slice` helper.
- The t68 session diagnosed it at entry idx 271 and renamed the classes by idx 285.
- The rename touched `assets/styles.css` and `src/adapters/driving/http/settings/templates/settings.rs`.
