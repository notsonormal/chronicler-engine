# Add a failure banner for degraded roles and an unreachable server

Type: task (AFK)
Status: open
Blocked by: 08

## Question

The header's `Connected` is a static string in `HeaderTemplate`, fetched once with `hx-trigger="load"`, so the header reads as a live health signal while never changing (finding 1.3). On the review turn the Quantifier failed twice and the engine used fallback NPC IDs, while the header said "Connected" and the status said "Ready" (finding 1.1). Replace it with a banner that appears only when there is something to say.

## Context

Decided in [Decide how the dashboard shows each kind of failure](08-decide-failure-display.md). The model is quiet by default.

- `Connected` is deleted. The header keeps the game title and the game name only.
- The banner sits directly under the header bar and above the tab bar, in the visual zone the toast occupies today. While healthy it is absent and costs 0px; when up it costs about 30px.
- **Degraded** means the newest `llm_messages` row for that role carries `error_message`, and it clears on the role's next success. Two states, Healthy and Degraded — no third state for "the engine worked around it". That distinction belongs to the generation, and the status display owns the generation.
- Source: the newest row per agent. The roles are `narrator`, `quantifier`, `options`, `trigger`. Ticket 41 makes every LLM attempt, success or failure, persist a row naming the agent and backend/model, so no new persisted state is needed.
- **Unreachable** is client-owned: a body-level `htmx:sendError` / `htmx:responseError` listener sets it and any successful response clears it. A server cannot render its own unreachability. Role health is the opposite — a Quantifier fallback still returns 200 — so it is server-rendered in `/fragment/header`, which gains a poll.
- **No close control** and no timer. The banner is present exactly while the condition holds. Ticket 52 owns the failed-request listener that shares this state; do not write a second one.
- The banner names the degraded role and what the engine did instead where it can ("using fallback NPC IDs"), and carries a **Details** control opening a popover with the per-role list, each role's last error and its backend/model. The popover is the anchored pattern from ticket 53 — one pattern, two call sites. Never render the raw error inline.
- On-demand health lives in Settings (ticket 15), which gains the per-role summary and a connection test. Do not add an ambient header affordance; that was considered and rejected.
- Semantics for ticket 47: `role="status"` while degraded, `role="alert"` while unreachable.

## Done when

- `Connected` is gone. The banner appears on a degraded role and on an unreachable server, and disappears when the condition clears.
- A test covers a role degrading, then recovering, and the unreachable state. Tier by `tests/STRATEGY.md` and name it in the answer. Until [Decide what a tier-1 test may observe](31-decide-tier-1-observations.md) resolves, write new spec Givens and Thens in `CONTEXT.md` terms rather than field names.
- The spec scenario for the banner exists (a new scenario in `docs/specs/browser_dashboard.md`, or the right spec if one already covers the header). Add or change a scenario only if the spec is incomplete or wrong.
- `python build.py` is green, the user reviews the diff, then commit through `/commit-and-push`.
