# Announce dynamic state changes to assistive technology

Type: task (AFK)
Status: resolved
Blocked by: 08

## Question

No element in the dashboard carries `role` or `aria-live` (the single `aria-hidden` in the markup is the NPC name label). The status display (Ready → Thinking... → Generating narration... → Quantifying scene... → Ready), generation errors, the toast, new narration, the options dock and text-check results are all silent for a screen reader: a real keyboard turn produced the whole phase sequence unannounced, and the toast enters the accessibility tree as bare `StaticText` with no `role="alert"`. Add live regions where they belong, choosing politeness per region.

## Context

- Finding K4 (P2) of [Review keyboard use and screen-reader output](07-review-keyboard-screen-reader.md). Evidence `tmp/ui-review/A7-toast-no-live-region.png`, `tmp/a11y-*.txt`, attribute scan showing `aria-live` null everywhere (local only).
- `#story-log` is replaced wholesale every 2s, so a naive `aria-live` on it re-announces the whole log on every poll. Scope the announcement to the changed entry, or wait for [Decide how the story-log poll keeps DOM state](10-decide-story-log-poll-swap.md).
- Blocked by [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md) so the health and status model settles before regions are wired to it. Settled: the failure banner ([51](51-add-failure-banner.md)) is `role="status"` while degraded and `role="alert"` while unreachable; the status display's clamped error and the anchored popovers are the other regions to wire. The toast this ticket lists is retired by [54](54-retire-toast-and-route-callers.md) — drop it from the list.
- Structural semantics (tablist, listbox, landmarks) are [Expose the dashboard to assistive technology](46-expose-dashboard-to-assistive-technology.md)'s; this ticket is only about what gets announced.

## Done when

- Each dynamic region announces what matters and nothing more, verified against the accessibility tree; a whole-log re-announcement on each poll is explicitly ruled out.
- A test covers the announcement of a status transition, and of a toast if feasible.
- Test placement follows `tests/STRATEGY.md`, and the answer names the tier. Until [Decide what a tier-1 test may observe](31-decide-tier-1-observations.md) resolves, write new spec Givens and Thens in `CONTEXT.md` terms rather than field names.
- `python build.py` is green, the user reviews the diff, and the work is committed through `/commit-and-push`.

## Answer

Resolved. Dynamic surfaces are announced without re-announcing unchanged content.

- Dedicated hidden live regions, written only on change, rather than `aria-live` on the
  polled containers (which `innerHTML`-swap every cycle): `#status-announcer`
  (`role="status"`, polite) for phase changes; `#status-error-announcer` (`role="alert"`,
  assertive) for the generation error's short line; `#narration-announcer`
  (`role="status"`, polite) for a new story-log entry; `#options-announcer`
  (`role="status"`, polite) for a changed option set. `#text-check-result` owns
  `role="status"` on its stable container, and the template's nested `role="status"` was
  removed so the result is not double-announced.
- **Story-log no-re-announcement: scoped to the changed entry.** `#story-log` is not a
  live region (consistent with 65); `#narration-announcer` is written only for entry ids
  not seen before, seeded on first render so page load announces nothing. An unchanged 2s
  morph poll adds no id, so it writes nothing.
- The failure banner's `role="status"`/`role="alert"` (63) and `#restore-notice` were
  already wired. The toast is 54's and was dropped from this ticket's list.

Verified with the closest a11y evidence the harness exposes: Playwright role selectors
(`get_by_role(role) ∩ #id`) confirm the computed roles, and MutationObserver counts prove
exactly one announcement per change and zero on unchanged polls across a cycle.

Tests: tier 2 `tests/browser/stub/dashboard.rs` (16.23, 16.24; 16.11 extended). No tier-1
test — these are DOM/AT mutations `curl` cannot observe (STRATEGY placement rule step 2).
Review follow-up: the fixed sleeps were replaced with `wait_for_htmx_requests`, so the
absence assertions wait on the real poll cycle.

`python build.py` is green (1566 integration, 62 browser). Uncommitted, pending review
and `/commit-and-push`.
