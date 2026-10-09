# Make every failure display tell the truth

Type: task (AFK)
Status: open
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
