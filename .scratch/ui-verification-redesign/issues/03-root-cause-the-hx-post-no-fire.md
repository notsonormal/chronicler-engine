# Root-cause the hx-post change-event no-fire: what actually stops the POST?

Type: research
Status: resolved
Blocked by:

## Question

`tests/browser/worlds.rs::test_world_posture_change_autosaves_status` fails
because the `change`-triggered `hx-post` never produces a network request.
The probe evidence (see
[flake-investigation-2026-09-13.md](../assets/flake-investigation-2026-09-13.md))
showed that at the failure moment: htmx is loaded (v1.9.10), the select has its
`hx-post` attribute, the select is in the document, and there is exactly one
`.worlds-panel` — so the DOM looks right, yet the event never fires a request.

Work out the *actual* mechanism, not a hypothesis. Use the vendored htmx source
(read `assets/htmx.min.js` or fetch the unminified 1.9.10 source) and, if
needed, a decisive experiment to distinguish among:

1. **Swap-process race** — Playwright's `select_option` dispatches `change`
   before htmx has attached its listener to the swapped-in element.
2. **Synthetic-event mismatch** — Playwright's event isn't trusted/bubbling as
   htmx's listener expects (htmx uses delegated listeners at a boundary in
   some configs).
3. **hx-sync / debounce / disabled-elt re-entrancy** dropping the request.
4. **The element is re-swapped/detached between readiness and change** so the
   change fires on an orphan node.

State the reproducing condition precisely and the htmx internals that explain
it. **Stopping rule:** cap experiments at 3 — a targeted dispatch-order test, a
trusted/synthetic event test, and a detach/re-swap test (or equivalent). If
after those you cannot reproduce the no-fire deterministically, conclude
"racy by construction" and record it; a confirmed nondeterminism is a valid
resolution, and it means ticket 04 (Decide the UI verification design) must
choose a design where the race cannot exist, not one that times it right.

Note this is a mechanism ticket — designing the *fix* belongs in ticket 04,
but say plainly whether the correct fix is a harness-side wait, an app-side
readiness signal, or an htmx config change.

Deliverable: linked markdown asset with the confirmed mechanism + evidence.
Experiments must run in `tmp/` with an isolated
`CARGO_TARGET_DIR` (e.g. `target/ui_redesign`); make no committed code changes.

## Answer

**Racy by construction, confirmed from evidence already on the map — the three
capped experiments were deliberately skipped.** This resolution applies the
ticket's own stopping rule early. By the time this ticket reached the
frontier, [ticket 02's resolution](02-research-how-to-verify-htmx-apps.md)
had confirmed the listener-attachment timing at source level in the vendored
1.9.10 source, and the flake investigation's failure signature plus the
flake's intermittency exclude every deterministic candidate. A deterministic
reproduction would re-demonstrate a race whose class is already
source-confirmed, whose design consequence is already fixed, and whose
implementation the redesign is scheduled to replace — post-mortem diagnosis
of code about to be deleted.

### Mechanism (candidate 1: swap-process race)

1. `GET /worlds/{key}/edit` swaps the form in. htmx 1.9.10 inserts the nodes
   synchronously — they are visible immediately — but defers `processNode`
   (which attaches the `hx-trigger="change"` listener) to the settle task,
   delayed by `defaultSettleDelay = 20` ms.
   - `defaultSettleDelay: 20` read in vendored `assets/htmx.min.js`; listener
     attachment inside the settle task confirmed in ticket 02's resolution.
   - The selects carry bare `hx-trigger="change"` in edit mode:
     `src/adapters/driving/http/worlds/templates/worlds.rs:84,90,96`.
2. The harness's only readiness primitive is visibility polling
   (`wait_until_visible`, `tests/test_utils/wait.rs`). It returns the moment
   the select renders — statistically inside the 20 ms window.
3. `select_option` dispatches `change` on the live node (Playwright re-queries
   by locator; the probe confirmed the node in-document, exactly one
   `.worlds-panel`). A live node whose listener is not yet attached swallows
   the event silently; nothing observable in the DOM differs.
4. Every failure shows **zero** posture POSTs in the teed engine log — the
   listener never fired. Not a request dropped in flight.

Candidate scoring:

| Candidate | Verdict | Why |
| --- | --- | --- |
| 1. Swap-process race | **Confirmed** | Fits zero-POST signature and 3-of-8 intermittency; listener timing source-confirmed |
| 2. Synthetic-event mismatch | Excluded | Deterministic — would fail every run, not 3 of 8 |
| 3. hx-sync / re-entrancy drop | Excluded | Zero POSTs at failure — nothing in flight to drop; no `hx-sync` attribute exists on the form or selects (`templates/worlds.rs:71,84-96`) |
| 4. Orphan re-swap | Excluded (inferred) | No second swap source between visibility and action; probe: one panel, node in-doc |

### Fix class (stated plainly, as the ticket required)

The mechanism is a **harness-side readiness gap**: acting on visibility
instead of htmx-readiness. The fix families are already on ticket 02's
shortlist for [ticket 04](04-decide-ui-verification-design.md): settle-gate
the swap→interact step (harness-side); an app-side structural change
(`hx-preserve`/stable ids — the only by-construction fix); or demote the
posture autosave contract to the HTTP tier (removes the browser race
entirely). An htmx config change (`defaultSettleDelay: 0`, as htmx's own
suite does) narrows the window but does not close it. Per ticket 04's
question 2, the chosen design must make the race structurally impossible,
not time it right.

### Loose ends, recorded not chased

- The exact sub-flavor of the race window is unpinned — not design-relevant,
  since every surviving flavor dies under the same readiness protocol.
- Passing runs show **2** posture POSTs for a single `select_option` —
  unexplained, and non-load-bearing here (the discriminator is zero-vs-nonzero
  POSTs). Worth pinning if the posture contract is demoted to the HTTP tier,
  where request counts become directly assertable.

No fog graduates from this answer; every remaining map item graduates from
ticket 04's design decision. No new tickets surfaced.
