# Decide what a tier-1 test may observe

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A tier-1 (HTTP E2E) test acts through HTTP. What may it assert on: only the response, the response plus stored state, or the response plus stored state read through the application's read seams? And in what words may a spec Given or Then describe it?

## Context

- `tests/STRATEGY.md:12` says tier 1 checks "the server response a `curl` would see". Lines 22-23 say every spec scenario maps to an HTTP E2E test. Lines 28-34 send internal state (`last_trigger`, phase transitions) to the unit tier. The doc contradicts itself, and each spec file resolves it its own way. [known]
- A precedent already exists: [Spec restructure: HTTP-observable, endpoint-named](../../test-strategy-execution/issues/11-spec-restructure-http-observable-only.md) settled "specs cover HTTP-observable behaviour only". It never defined "observable", and the specs drifted past it. This ticket sharpens that rule. It does not reopen endpoint naming or the tier split.
- What the code does today [known, counted]:
  - 83 calls to `state.message_service.load_messages()` across 7 files in `tests/http/`, for example `tests/http/actions.rs:32`.
  - About 40 raw `GameState` field reads, mostly `final_state.narrative.input_buffer.status`, in `actions.rs` (13), `swipe_new.rs` (17), and `retrigger.rs` (9). List: [raw_gamestate_reads.txt](../assets/test-audit/raw_gamestate_reads.txt).
  - 65 of 146 scenarios name internal state in a Given or Then, for example `docs/specs/retrigger.md` 13.1, "Given … `narrative.last_trigger` is set". The 58 hard leaks come from a grep. The 7 soft ones are a lower bound. Detail: [specs.md](../assets/test-audit/specs.md).
  - `scripts/validate_feature_spec.py` passes (146/146). It checks tags and directories only and never reads spec prose or test bodies.
- `tests/http/settings.rs` 20.2–20.4 show the opposite failure. Each asserts only `body == "Settings saved!"`, so none proves the switch took effect.
- Options raised in the audit session:

| Option | A tier-1 test observes | Specs name | Cost |
|---|---|---|---|
| A. Strict | The response, plus follow-up GETs | Only what a response shows | Weaker tests. `POST /action` returns "Thinking...", so outcomes must be scraped from fragment HTML. Rejected in session as less useful. |
| B. Codify practice | The response, plus any state | Anything, including field names | No rework. Tier 1 becomes unit tests with a router in front, and ~40 tests break on internal reshapes. |
| C. Domain outcome | The response, plus stored state read through the application's read seams | `CONTEXT.md` terms only | Rewrites ~40 raw reads (status reads can use `GET /status/generating`). Some may need a new read seam. "Read seam" needs a definition. |

- The session leaned to C. The proposed leading word is **domain outcome**: a Given or Then describes a domain outcome in `CONTEXT.md` terms, or something a client sees in a response.
- This map's tickets place tests by `tests/STRATEGY.md`. Most of them add tier-2 browser tests, which this decision barely touches.

## Done when

- One option is chosen, with the user's reasons recorded under `## Answer`.
- If C: "read seam" has a one-sentence definition that a reviewer can apply to a single line of test code.
- The answer states what happens to the 65 leaking scenarios and the ~40 raw reads: migrate now, migrate on touch, or leave. The map's *Not yet specified* entry graduates into tickets, or is ruled out of scope.
