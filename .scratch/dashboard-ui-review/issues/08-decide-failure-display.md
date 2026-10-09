# Decide how the dashboard shows each kind of failure

Type: grilling (HITL)
Status: resolved
Blocked by: 04, 05

## Question

For each kind of failure, where does the dashboard show it, how loudly, and how does it clear?

- a background poll that fails
- the server being unreachable
- a degraded Agent (for example, the Quantifier falling back to fallback NPC IDs)
- a generation error
- a user action that fails

And what happens to the top toast (`#error-notification`) and to the header's hardcoded "Connected"?

## Context

- Findings 1.1 and 1.3–1.7. Screenshots 02, 03, 04.
- Earlier proposal (not decided): route by kind.
  - Background problems become a steady state in the header.
  - Generation errors show only in the status display, clamped to one line.
  - Action errors show inline next to the action.
  - The toast is retired, or kept only for rare one-off events.
- A dead server fires `htmx:sendError`, not `htmx:beforeSwap`, so today it shows nothing.
- Read the answers of the two review tickets (04, 05) first. They list the failure paths.

Findings from ticket 05 (review create/save/delete flows) that belong to this decision:

- **Network failures on htmx deletes are silent.** With the engine stopped, the confirm dialog closes and nothing happens for connection (screenshot 65), world (66) and game (67) deletes: no toast, item stays. The dead-server gap is already noted above; these are the concrete surfaces. In contrast `deleteMessage()` reports "Failed to delete message" (68) because it checks `response.ok` and catches — use that as the model for the "user action that fails" row.
- **Raw server text reaches the user.** Observed in the toast or inline: `Failed to deserialize form body: missing field 'preset_type'` (prior screenshot 33), `Failed to deserialize form body: missing field 'name'` (35), `Error: Configuration error: Unknown LLM backend 'bogus_provider'` (54, 56), `Invalid map JSON: key must be a string at line 1 column 3` (39), `Invalid scenarios JSON: EOF while parsing a value at line 1 column 0` (42). Decide what reaches the user and what goes to the log.
- Destructive error swaps that this decision gates: tickets [25](25-stop-errors-wiping-panels.md) and [26](26-stop-errors-destroying-cards.md).
- `CONTEXT.md` has no term for health or a degraded state. Use `/domain-modeling` to settle terms and add them to the glossary.

## Done when

- The decision is in the ticket answer.
- The Theme 1 fog in the map graduates into implementation or prototype tickets.

## Answer

Grilled with the user over four rounds. The model is **quiet by default**: the dashboard shows nothing while everything works, and a failure appears on the surface that owns it.

**Revised during the session.** The first answer to Q4 was always-present per-role header chips. Mocking that against three alternatives (`tmp/show-me-header-health.html`; crops in `tmp/show-me-shots/A-option1.png`–`D-option4-table-status-480.png`, local only) the user rejected it: four chips spend ~310px on every turn saying "nothing is wrong", and the degraded one is easy to miss. The header carries **no** health element. The revised model is Option 3 of the mock, the banner.

### 1. The failure map

| Kind | Surface | Loudness |
|---|---|---|
| Background poll fails | The banner; the polled region keeps its last good content | Quiet |
| Server unreachable | The banner (`Unreachable`); a failing action reports on its own surface | Quiet + local |
| Degraded role | The banner, naming the role | Quiet |
| Generation error | The status display: one clamped line plus a raw-text popover | Medium |
| User action fails | An inline slot inside the failing form or card (tickets 25, 26) | Loud and local |

### 2. The banner (Q4 revised, Q6=A, Q7=A, Q10=A, Q11=A)

- `Connected` is deleted. The header holds the game title and game name only. Finding 1.3 is fixed by removing the false signal rather than by animating it.
- The banner sits directly under the header bar and above the tab bar, in the visual zone the toast occupies today.
- It renders only when there is something to say: a role whose newest LLM attempt failed, or an unreachable server. While healthy it does not exist, so it costs 0px. When up it costs about 30px.
- **Degraded** means the role's newest `llm_messages` row carries `error_message`, and it clears on that role's next success. Two states per role, Healthy and Degraded. There is no third state for "the engine worked around it": whether a fallback carried the turn is a property of the generation, not the role, and the status display owns the generation.
- **Unreachable** is client-owned. A body-level `htmx:sendError` / `htmx:responseError` listener sets it and any successful response clears it. A server cannot render its own unreachability. Role health is the opposite: it is server-rendered, because a Quantifier fallback still returns 200.
- **No close control** (Q10=A). The banner is present exactly while the condition holds. A deliberate degradation, such as a local Ollama switched off, is the case that argues for a dismiss button; the user chose to keep the strip honest instead, and to fix intentional configurations in Settings.
- The banner names what the engine did instead where it can — "using fallback NPC IDs" — and carries a **Details** control that opens a popover with the per-role list, each role's last error and its backend/model.
- **On-demand health is Settings only** (Q11=A, ticket 15): a per-role summary and a connection test. Nothing ambient in the header. LLM Messages already carries the per-attempt record from ticket 41, so Settings does not grow a second log.
- Semantics for ticket 47: `role="status"` while degraded, `role="alert"` while unreachable.

### 3. The wire rule (Q5=A)

A failure never swaps into the region it describes.

- A failed poll changes nothing in the DOM: the response is non-2xx with `HX-Reswap: none`, and the region keeps its last good content. Today `render_fragment` returns HTTP 200 with an error div, which is why a failed load replaces the region it was meant to fill.
- A failed user action renders into a dedicated error slot inside its form or card, never over the panel (tickets 25, 26).
- Dead-server failures have no response body at all, so the client listener synthesises the same fragment into the failing form's slot and sets `Unreachable` on the banner.

### 4. Error text (Q2=B, Q12=the anchored popover)

Short user-facing message, with the raw server text behind a disclosure. The disclosure is an **anchored popover**, not an inline expansion: the inline version takes the action area from 64px to about 100px (visible in `tmp/show-me-shots/D-option4-table-status-480.png`), which is the same screen-budget problem this decision exists to fix. One popover pattern serves both the banner's per-role detail and the status display's raw text. Escape and outside-click dismissal are part of it, and ticket 47 must announce it.

The raw text still goes to the log. What reaches the user is clamped and mapped; what reaches the log is verbatim.

### 5. Clearing (Q3=A)

No timers anywhere. Health clears when the condition recovers; an action error clears on the next successful action or an explicit dismissal; a generation error clears on retry, on the reset route, or on dismissal. The banner has no dismissal of its own.

### 6. The toast (Q1=A)

`#error-notification` is retired, sequenced: only once the banner (51), the inline slots (25, 26) and the clamped status display (53) exist. Its eight `showError(...)` call sites in `assets/index.html` move to their routed surfaces — `submitEdit`, `submitGenerationRequest` (new swipe, retrigger), `switchSwipe`, `deleteMessage`, the `htmx:beforeSwap` fallback, and the status poll.

### 7. Terms (Q9=C)

No `CONTEXT.md` entries. The state names (Healthy, Degraded, Unreachable) live in this answer and in `docs/diataxis/reference/frontend/dashboard.md`, which ticket 23 aligns. This overrides the Context line above that asked for glossary terms.

### Graduated

[51](51-add-failure-banner.md), [52](52-failed-request-keeps-its-region.md), [53](53-clamp-status-error-with-popover.md), [54](54-retire-toast-and-route-callers.md). Tickets 15, 23, 25, 26 and 47 lose their 08 blocker.

### Judgement calls

- The banner is rendered by the header fragment, since role health is server-side; the client only adds the `Unreachable` state to it.
- "Role" means the four recorded agent names: `narrator`, `quantifier`, `options`, `trigger`. The banner names only the degraded ones; the full list appears in the popover and in Settings.
- A generation error and a degraded Narrator can both be true at once. They are two surfaces by design: the status display says this turn failed, the banner says the role is unhealthy.
- The mock and its crops are scratch evidence under `tmp/` and are not committed.
