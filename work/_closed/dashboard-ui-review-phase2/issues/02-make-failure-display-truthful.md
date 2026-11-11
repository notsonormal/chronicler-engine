# Make every failure display tell the truth

Type: task (AFK)
Status: resolved
Blocked by: —

## Question

Four surfaces of the failure display give wrong or unreadable information. Fix each so it matches the states in `docs/diataxis/reference/frontend/dashboard.md` ("Failure and health states").

## Items

From the [re-review](../../dashboard-ui-review/re-review-2026-10-09.md):

- **R1 (P2)** The banner popover shows "last call succeeded" for a role with no recorded call. `banner_detail` (`src/adapters/driving/http/builders/headers.rs`) must show "No calls yet" when there is no attempt. Shot `tmp/t24/03-banner-details.png`.
- **R20 (P2)** A generation error in `#status-display` keeps the class `status ready`, so it is drawn in the "Ready" blue. Give the error state its own class and colour. The 240px clamp also cuts the one-sentence short line ("The language model could not …"); make the short line readable without Details, without letting the action area resize (the reason for the fixed width). Shot `tmp/t24/32-narrator-failure.png`.
- **R8 (P2)** Settings role health and the Connections warning marker are fetched once and go stale: the banner said "Quantifier failed" while both role rows read "No calls yet". Keep them current — for example the header poll could also swap the role-health cells out of band. Shots `13`, `15`.
- **R2 (P2) + R19 (P3)** LLM Messages failure rows: `.llm-message-failure-text` has no CSS, so the `<pre>` does not wrap and the card clips it; the "Failure" label is unstyled. Wrap the text like the prompt blocks and style the label. Shot `04`.

## Done when

- Each item is fixed. Tests follow `tests/STRATEGY.md` (R1 and R8 are visible in a fragment response; R20's class is visible to a stub-tier browser test).
- `python build.py` is green. Commit after user approval.

## Answer

All four items are fixed. Full gate green in `/workspace/wt-p2-02` (branch `wf/p2-02`):
`nextest: 1596 passed, 0 failed, 2 skipped (integration)` and `nextest: 70 passed, 0 failed (browser)`, with architecture 1/1 and guardrails 162/162 (log `logs/build_20261010_000652.log`).

### R1 — the banner's per-role detail told a lie about roles that never ran

`banner_detail` mapped "no error" to success. The per-role line now comes from `role_health_line` (`src/adapters/driving/http/builders/headers.rs`): a failure shows the raw text, a role with a recorded successful attempt shows "last call succeeded", and a role with no recorded attempt shows "No calls yet" in its own `error-role-nocalls` class (`assets/styles.css`, the same muted small style as its neighbours). The backend column still reads "no recorded call" when there is no attempt, as the shot shows.

Test: tier 1 (`tests/http/failure_display.rs`), `test_degraded_role_raises_header_banner_and_clears_on_recovery` (scenario 38.1, extended). It now seeds a successful Narrator attempt beside the failed Quantifier and asserts exactly one "last call succeeded" and two "No calls yet" in the fragment; the old assertion `contains("Mock mock") && contains("last call succeeded")` was the one that pinned the lie, so it was replaced. Scenario 38.1 gained the two matching `And` lines.

### R20 — the error state had no class and its sentence was clipped

- `assets/dashboard-action-area.js`: `syncStatusClass()` re-derives `#status-display`'s state class (`ready` / `thinking` / `error`) from the content it holds and runs from `applyStatusDisplay()`, so the server-rendered error disclosure can no longer leave the region in the Ready blue. It touches only the three state classes, so htmx's transient classes survive.
- `assets/styles.css`: `#status-display .error-disclosure` wraps (`white-space: normal`) and its message wraps inside the fixed 240px column, clamped to two lines (`line-height: 1.4; max-height: 2.8em`) — two lines fit the action area's 42px content box, so the sentence reads without Details and the action area keeps its 64px height. Measured with the longest clamped sentence (`PromptTooLong`, 52 chars): box 183×34px inside the column, input width and action-area height unchanged.

Test: tier 2 (`tests/browser/stub/failure_display.rs`), `test_generation_error_takes_the_error_class_and_reads_without_details` (new scenario 16.41 in — renumbered from 16.36 at integration; 04 took 16.36–16.38, 06 took 16.39–16.40 — `docs/specs/browser_dashboard.md`). It asserts the container carries the `error` state and not `ready`, that the short line's computed colour equals the palette's `--color-accent-red` (resolved in the browser, not hard-coded) and differs from Ready's, that the line is drawn inside the column unclipped, and that Ready restores its own class and colour. Both halves were verified to fail with the fix reverted (JS reverted → the state gate times out; CSS reverted → the unclipped assertion fails). `tests/test_utils/stub_server.rs` now carries the `GenerationFailureKind` in `StubStatus::Error`, so the stub renders the same clamped sentence the endpoint does and the test can drive the longest line; four call sites were updated.

### R8 — the Settings role health went stale

The header poll (5s) now ships out-of-band swaps for the two role-row health cells and the Connections sub-tab marker, alongside the banner. Shared renderers keep the panel's cell and the swap from drifting: `role_health_cell`, `role_health_cell_id`, `connections_degraded`, `connections_degraded_marker` and `settings_health_swaps` in `builders/headers.rs`; `SettingsTemplate` gained the `degraded_marker` field and renders `id="role-health-<agent>"` on each health slot plus the `id="subtab-connections-marker"` slot, replacing the inline `{% if roles_degraded %}` marker.

Tests: tier 1 (`tests/http/failure_display.rs`), `test_header_poll_keeps_the_settings_role_health_cells_current` (new scenario 38.6 in `docs/specs/failure_display.md`) — it reads `/fragment/settings` and asserts the three swap-target ids (and that the panel's own cells carry no `hx-swap-oob`), then walks `/fragment/header` through no-calls → Degraded + marker → Healthy + empty marker. Verified end to end in a real browser with a throwaway probe on the existing 16.22 stub test: after the header poll, `#role-health-quantifier` read "Degraded…", `#role-health-narrator` "No calls yet" and `#subtab-connections-marker` held the warning triangle. No permanent browser test was added for this because the ticket places R8's test in the fragment response; the re-review's shots 13/15 cover the panel visually.

### R2 + R19 — the LLM Messages failure row

`assets/styles.css` gained the `.llm-message-failure*` rules: the container stacks label and text, the label is the panel's uppercase red label style (like `.llm-message-agent`/`.llm-message-error`), and `.llm-message-failure-text` mirrors `.llm-message-prompt-pre` (pre-wrap, break-word, bordered, `max-height: 300px`, `overflow: auto`). Measured in a real browser against the card from shot 04: `scrollWidth` 1212 == `clientWidth` 1212 (it was 1264 > 1214), no text past the card's clipped edge. Pure CSS, so no test, per the map's notes.

R19's second half — "an empty gap sits below the last disclosure" — is *not* in the item's wording ("the 'Failure' label is unstyled"), but it is the other half of the same finding, it sits in the same block, and I measured it at 40px, so it is fixed too: `.llm-message-body > *:last-child`, `.llm-message-prompts .llm-message-prompt-details:last-child` and `.llm-message-raw details:last-child` lose their trailing margin (measured 16px after, i.e. the body's own padding). Say the word if you would rather have that reverted to a ticket-06 follow-up.

### Docs

- `docs/diataxis/reference/frontend/dashboard.md`: the error state's `--color-accent-red` and wrapping, the container's state class and how it is re-derived, role health being refreshed rather than fetched once (Tabs table and Polling Cadences included), and the swap-target ids in the Settings Tab section.
- `docs/specs/failure_display.md` (38.1 extended, 38.6 new), `docs/specs/browser_dashboard.md` (16.41 new).

### Follow-ups (non-blocking)

- The out-of-band swaps target elements that only exist while the Settings panel shows the Connections list. While it shows the shared connection form page, each header poll makes htmx report `htmx:oobErrorNoTarget` in the console. Nothing user-visible changes (the panel re-renders with fresh health when it returns to the list). Making it silent would need the panel's own role-health poll instead of the header's swap.
- R20's fix clamps the wrapped line to two lines, which is exactly the action area's height budget. A future longer clamped sentence (a new `GenerationFailureKind`) would clip on its third line; `tests/browser/stub/failure_display.rs`'s `read_status_state` assertion fails when that happens, so the failure is loud rather than silent.
- The stub's `StubStatus::Error` now takes a `GenerationFailureKind`; the four existing call sites pass `Other`, which keeps their behaviour identical.
