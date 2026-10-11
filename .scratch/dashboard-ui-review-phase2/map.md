# Map: dashboard UI review, phase 2

Labels: wayfinder:map

## Destination

Every finding in the [2026-10-09 re-review](../dashboard-ui-review/re-review-2026-10-09.md) is fixed, or ruled out of scope with a reason. Every finding in the [2026-10-11 re-review](re-review-2026-10-11.md) is fixed, or ruled out of scope with a reason. [Re-review the dashboard after the phase-2 fixes](issues/14-re-review-after-phase-2-fixes.md) finds no new P1/P2, and `python build.py` passes.

This map continues the [dashboard UI review](../dashboard-ui-review/map.md). That map closed with [Final re-review of the dashboard](../dashboard-ui-review/issues/24-final-re-review.md), which found one new P1 and eight new P2s.

## Notes

- **This map carries execution,** like phase 1. It overrides wayfinder's "plan, don't do": `task` tickets make the change, not only decide it. Decision tickets (`grilling`, `prototype`) graduate their implementation work into new tickets.
- **Domain:** the dashboard frontend: `assets/` (`index.html`, `styles.css`, the `dashboard-*.js` modules) and the Askama templates and handlers under `src/adapters/driving/http/`. Backend code changes only where a finding needs it. [Decide when a reasoning-only reply counts as an answer](issues/01-decide-reasoning-only-replies.md) is the expected exception: it touches the LLM transport.
- **Source of findings:** the [2026-10-09 re-review](../dashboard-ui-review/re-review-2026-10-09.md). Tickets cite findings by R-number (for example "R6") and screenshots by number (`tmp/t24/NN-*.png`, local only).
- **Tests:** the rules of phase 1 hold. Place every test by the placement rule in `tests/STRATEGY.md` and name the tier in the ticket answer. Specs describe the system, not changes. Pure CSS and copy fixes need no test, but update the existing assertions that a fix breaks. A tier-1 test asserts domain outcomes only (the "Domain outcome" section of `tests/STRATEGY.md`).
- **End of each execution ticket:** `python build.py` is green, then the user reviews the diff. After approval, commit through `/commit-and-push`. One ticket = one session = one commit.
- **Skills:**
  - grilling and prototype tickets: `/grilling`, `/domain-modeling`, `/prototype`
  - review tickets and visual checks: `/chronicler-ui-investigator`
- **Reference docs:** `docs/diataxis/reference/frontend/ui_design.md` (design tokens), `docs/diataxis/reference/frontend/dashboard.md` (including "Failure and health states"), `docs/diataxis/explanation/dashboard_design.md`.
- **Real LLM calls are fine.** The user prefers the real connections to a stub for review and manual checks; a few calls cost less than a cent.
- **User data:** review tickets run on a probe server with its own database (`python build.py run -- --world redmist_estate --port <port>` writes `target/debug/chronicler_<port>.db`). Restore any changed setting before the session ends.
- **Final re-review:** every ticket added to this map must also be added to the `Blocked by:` line of [Re-review the dashboard after the phase-2 fixes](issues/14-re-review-after-phase-2-fixes.md).
- **Small follow-ups:** issues from code reviews or implementers that don't block a merge go under `## Items` in [Follow up on small issues found in the phase-2 re-review](issues/13-follow-up-phase-2-re-review-p3s.md), with their source ticket, while it is open.
- Other agents may work in the repo at the same time. Do not touch unrelated changes.

## Decisions so far

<!-- the index — one line per closed ticket -->

- [Decide when a reasoning-only reply counts as an answer](issues/01-decide-reasoning-only-replies.md) — never; empty `content` is always a failure for every role, a spent token budget (`"length"`) gets its own failure kind, and failed rows keep the raw body. Built by [Fail a reply that has no content](issues/08-fail-reply-without-content.md).
- [Make every failure display tell the truth](issues/02-make-failure-display-truthful.md) — "No calls yet" for a role that never ran; `#status-display` re-derives its state class (`ready`/`thinking`/`wait`/`error`) and wraps the error line; the header poll swaps Settings role health out of band; LLM Messages failure rows wrap and their label is styled.
- [Keep the story log on the newest entry](issues/03-story-log-follows-new-entries.md) — a swap that adds an entry scrolls the log to the bottom only when it was already at the bottom; an open edit is left alone.
- [Keep the command text until a send goes through](issues/04-keep-command-text-until-sent.md) — the input clears only on a consumed command (2xx, no preview, not refused); a confirmed preview clears it; the action area grows for its inline error, which clears on recovery.
- [Restore focus after Settings, Games and story-log delete swaps](issues/05-restore-focus-settings-games.md) — Settings and Games swaps land focus inside the panel; a story-log delete focuses the new last entry or the command input.
- [Follow up on small issues found in the final re-review](issues/06-follow-up-re-review-p3s.md) — nine P3s fixed (R4, R11, R13–R15, R17, R18, R24, R25) plus review follow-ups; R5, R12 and R23 dropped with reasons.
- [Fail a reply that has no content](issues/08-fail-reply-without-content.md) — the transport reads `content` only; `finish_reason: "length"` with no content is the new `TokenBudgetSpent` kind, with token counts in the details.
- [Re-review the dashboard after phase 2](issues/07-re-review-after-phase-2.md) — every phase-2 fix holds in the running app, and Retrigger and event delete leave no stuck status; new P1 S1 (event Retry fails and rolls the game back) and P2s S2–S4 became tickets 09–12, P3s S5–S8 ticket 13; the gate moved to [Re-review the dashboard after the phase-2 fixes](issues/14-re-review-after-phase-2-fixes.md). [Report](re-review-2026-10-11.md).
- [Make event Retry restore the trigger context and never roll the game back](issues/09-fix-event-retry-snapshot.md) — event Retry starts from the event Swipe's own snapshot; a redo that fails before it saves a Swipe restores the state from before the click; `seed_event_flow` follows the real save order.
- [Report a failed trigger continuation without a story-log Message](issues/10-report-failed-trigger-without-message.md) — no `System` Message on failure (status line and banner report it); Retry anchor and target both use the last Input or Narration; the NPC-detection and options notices stay.
- [Show the whole token-budget failure, remedy included](issues/11-show-the-token-budget-remedy.md) — the sentence is now "The model used its budget on reasoning. Try another model."; a stub test fits every `GenerationFailureKind` sentence at 1280 and 1024.
- [Restore focus after preset and world deletes](issues/12-restore-focus-after-card-deletes.md) — the removed-row rule covers games, presets and worlds (next, previous, panel), claimed at `htmx:beforeSwap`.
- [Follow up on small issues found in the phase-2 re-review](issues/13-follow-up-phase-2-re-review-p3s.md) — 11 of 16 items fixed (S5, S7, S8, F11-1, F12-1–F12-4, F09-1 and more); S6, F10-1 and F10-2 moved to the fog below; F09-2 kept; F09-3 dropped as rare.

## Not yet specified

- **The story log renders the oldest 50 entries** (`app_state.rs`) while the controls hang off `loop.last`. Reported by ticket 13; games with more than 50 Messages are affected (inferred).

- **Structured editing of Map and Scenarios JSON.** Carried from phase 1 (finding 4.8, pushed here by [Decide the panel layout convention and supported viewports](../dashboard-ui-review/issues/_resolved/19-decide-layout-convention.md)). Raw JSON sits in plain textareas. R15 (the placeholder JSON looks like a value) touches the same form. A separate question; it can wait.
- **Cross-panel staleness (R12).** [Follow up on small issues found in the final re-review](issues/06-follow-up-re-review-p3s.md) dropped a whole-panel refresh, but an out-of-band swap of the echoing counts (as ticket 02 does for role health) is possible. Open.
- **Truncated replies that do have content.** A `finish_reason: "length"` reply with non-empty `content` still counts as a full answer, so narration can stop mid-sentence. [Decide when a reasoning-only reply counts as an answer](issues/01-decide-reasoning-only-replies.md) kept this out of its rule. Failing it outright could make the quantifier and options roles (small `max_tokens`) fail often.
- **A Max Tokens control, or a larger default.** No dashboard control sets `max_tokens`, so every call uses 2048, and the default Narrator spent that budget on 2 of 11 calls in the [2026-10-11 re-review](re-review-2026-10-11.md). [Show the whole token-budget failure, remedy included](issues/11-show-the-token-budget-remedy.md) fixes only the wording; its sentence no longer names Max Tokens. Open.
- **A failed or refused preview confirm loses the corrected text.** The preview closes and the input keeps only the original text. Noted in ticket 06's answer.

## Out of scope

Carried from [Final re-review of the dashboard](../dashboard-ui-review/issues/24-final-re-review.md), which names an owner for each:

- **Two inert route-boundary guardrails.** `check_handler_return_type` and `check_server_layer_boundaries` can never fire, because the gate strips the `src/` prefix. Owner: the guardrail suite.
- **Panel padding at 48px instead of 24px.** Every panel nests a same-class fragment root inside a same-class shell. Pre-existing on `main`. R16 (the inset jump after Duplicate and World Edit) is its visible symptom.
- **Panel wipe on a failed preset add or activate.** Owner: `.scratch/architecture-deepening/issues/15-single-fragment-error-policy.md`.
- **`isPollRequest` reports a server-answered 500 as "unreachable".** Scenario 16.18 pins the banner on purpose; only the heuristic and the wording are open.
- **A "thinking" block in the story log.** Showing reasoning text in its own collapsible block, as SillyTavern does, is a new feature, not a fix for R6. Ruled out by [Decide when a reasoning-only reply counts as an answer](issues/01-decide-reasoning-only-replies.md).
- **Changing the Quantifier's fallback behaviour.** A backend design question; the banner already reports the fallback.
- **The Quantifier misses an NPC that a trigger introduces.** In the [2026-10-11 re-review](re-review-2026-10-11.md), the post-event Quantifier detected only Carla while Gabriella spoke, so she got no portrait. Detection quality is a backend question, like the fallback item above.
- **Trailing `System` notices (F10-1, F10-2).** The notices are an error channel inside the Message history: LLMs read them and they hide the Retrigger and Swipe controls. The user wants no notice to reach an LLM. Owner: [Replace `System` Messages with an error channel outside the Message history](../turn-state-model/issues/02-replace-system-messages.md) on the turn state model map.
