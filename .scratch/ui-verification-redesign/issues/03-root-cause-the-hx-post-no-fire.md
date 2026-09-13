# Root-cause the hx-post change-event no-fire: what actually stops the POST?

Type: research
Status:
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
