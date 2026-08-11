# Design the browser tier's target state

Type: grilling (HITL)
Status: resolved
Blocked by: 03, 05

## Question

The browser tier audit (planning map ticket 14) answered a subtraction
question: "which tests only re-verify pipeline logic and should move
down?" It classified 17 tests as move-down and 13 as keep. But it didn't
ask the forward-looking question: **what should the browser tier look
like after consolidation?**

This ticket grills out the target state of the browser tier — not just
what moves out, but what stays, what's missing, and how it's organized.

### Updates from ticket 11 (spec restructure)

Ticket 11 dissolved `action_pipeline.md` + `flow.md` into endpoint-named
specs and `flow_sequence.rs` into three endpoint-named test files. This
affects ticket 07's move-down list and reclassification work:

- **S1.3 dropped (room change not HTTP-observable).** Ticket 08's audit
  says browser `trigger.rs` tests map to "S1.1, S1.4, I.5, S1.3". S1.3 is
  gone — the quantifier-movement room change is internal state
  (`pending_location` is set by `handle_movement` in `phase_engine_commit`,
  AFTER the main narration is pushed). Any browser test that mapped to
  S1.3 needs reclassification: stay at browser (DOM-only assertion),
  move to unit tier (internal state), or delete (redundant).
- **Spec file renames.** Browser tests mapping to S6/S7/S8 now target
  `tests/http/actions.rs` (S6.x), `tests/http/reset.rs` (S7.x),
  `tests/http/story_log.rs` (S8.x) — not the deleted `flow_sequence.rs`.
  Spec targets are `docs/specs/actions.md`, `reset.md`, `story_log.md`.
- **STRATEGY.md tag placement** narrowed to `tests/http/` only. Any
  browser move-down that would land tags outside `tests/http/` is invalid.

### Questions to resolve

1. **Are the 13 kept tests well-organized?** The audit classified them
   as browser-only (CSS computed styles, layout/rendering, JS
   interaction) but didn't assess whether they're grouped sensibly or
   whether some overlap. Should they be reorganized?

2. **What browser tests are missing?** The audit was backward-looking
   (what exists). Are there presentation-layer behaviours only a browser
   can verify that have no test? (e.g., responsive layout, accessibility,
   htmx swap visual transitions, error-state rendering.)

3. **Does the browser tier need a spec?** The tier rules (07) say browser
   is presentation-only with a qualitative rule (must assert something
   only a browser can see). Should that rule be codified as a spec
   (Given/When/Then for visual/layout/interaction scenarios), or is the
   qualitative rule sufficient?

4. **How do browser tests relate to spec scenarios?** The 17 move-down
   tests mapped to spec scenarios (S1.1, S1.4, etc.). The kept tests
   don't map to any spec. Is that the right model — browser tests are
   orthogonal to component specs — or should browser tests also tag
   against a presentation spec?

5. **Coordination with HTTP E2E.** The 17 move-down tests go to HTTP
   E2E (tickets 03, 05). The grilling should confirm the move-down list
   is still accurate after the E2E work lands, and identify any
   additional tests that should move (or new browser tests that should
   be created once the E2E scenarios exist to compare against).

### What this ticket does NOT do

- Does not move tests — that's the implementation ticket that graduates
  from this resolution.
- Does not write a browser spec — that graduates from this resolution if
  the grilling decides one is needed.

### Context

- [Browser tier audit (planning ticket 14)](../../test-strategy/issues/14-browser-tier-audit.md) —
  the 17/13 classification.
- [Browser tier audit asset](../../test-strategy/assets/browser-tier-audit.md) —
  full per-test classification.
- Tier rules in `tests/STRATEGY.md` — browser is
  presentation-only, qualitative rule, no numeric cap.

## Answer

Grilling resolved the target state of the browser tier. The audit's
subtraction question ("what moves down?") stays valid — all 17
move-down tests belong at HTTP E2E. This grilling answered the
forward-looking question: **what does the browser tier look like after
consolidation?**

### Browser tier = two files, split by assertion shape

The 13 kept tests partition into two groups by what they assert, not by
domain. The split is load-bearing because it determines whether a spec
applies.

**`tests/browser/behaviour.rs` — 6 tests, specced.**
Asserts click→DOM change, htmx swap persistence, polling-pause, status-
update wiring — client-side JS behaviour with a Given/When/Then shape.
Tagged against `docs/specs/browser.md` the same way HTTP tests tag
against `actions.md` / `swipe_new.md`.

- `test_edit_mode_activates_on_click`
- `test_edit_cancel_restores_original`
- `test_polling_pauses_during_edit`
- `test_delete_removes_message`
- `test_form_stays_static_after_submission`
- `test_status_updates_during_generation` (live transition during real
  generation — static status strings already HTTP-covered via
  `/status/generating`; browser test's value is the client-side polling
  → DOM-update wiring)

**`tests/browser/invariants.rs` — 7 tests, not specced.**
Asserts rendering invariants (CSS computed styles, layout measurements).
No Given/When/Then fits — `overflowY is auto` is a style contract, not a
behaviour. The **test code is the definition**; no separate doc. Named
exemption in STRATEGY.md.

- `test_story_log_scrollable` (overflowY)
- `test_no_horizontal_overflow` (scrollWidth/clientWidth)
- `test_log_entry_text_wraps_within_bubble` (<pre><code> overflow)
- `test_element_positioning` (getBoundingClientRect vertical order)
- `test_npc_portraits_horizontal_layout` (flex-wrap, overflow-x)
- `test_npc_portraits_fixed_width` (portrait img width 50–120px)
- `test_edit_textarea_matches_original_height` (textarea height bounds)

### Spec: `docs/specs/browser.md` (behaviour-only)

One file, 6 Given/When/Then scenarios. No rendering-invariant section —
those live as test code in `invariants.rs`, not as spec prose. A split
file layout (`editing.md` / `polling.md`) was rejected: 6 scenarios
don't earn multiple files, and browser tests aren't endpoint-named so
the endpoint-named split rule (ticket 11) doesn't apply.

### STRATEGY.md + guardrail amendments

- SCENARIO tag placement rule widened: tags allowed in `tests/http/` **and**
  `tests/browser/behaviour.rs`.
- Guardrail (in `tests/infrastructure/guardrails/`) allowlists
  `tests/browser/behaviour.rs`.
- `tests/browser/invariants.rs` carries a named exemption: no tags, no
  spec link, test code is the definition. (Same shape as unit branch
  tests — STRATEGY.md's "every branch needs a unit test" rule doesn't
  produce a per-branch doc; the test is the definition.)

The browser placement test ("must assert something only a browser can
see") stays as the tier-membership rule. It was not used as the
definition of the invariant group — that was a conflation caught during
grilling. Placement and definition are separate: placement decides tier
membership, the test code defines the assertion.

### Move-down list: still accurate, 9 gaps to fill in ticket 08

Subagent re-verified all 17 destinations against current `tests/http/`
after tickets 03/05/11 landed. **All 17 still belong at HTTP.** But the
audit's implied framing ("move = already covered downstream") was wrong
for 9 of 17 — ticket 08 must **write 9 new HTTP tests**, not just delete
browser tests.

- **5 already covered** (#1, 2, 5, 16, 17) — delete browser test, no
  replacement.
- **2 partial** (#11 page_loads, #14 action_area_elements) — tighten
  existing HTTP tests if the full assertion set is wanted.
- **7 fragment assertions** (#7 edit-btn, #8 delete-btn, #9 retry-btn
  absent, #10 delete-btn count ≤2, #12 game-title in .game-title, #13
  connection-status id, #15 input no required attribute) — add to
  `tests/http/fragment.rs` / `index_handler.rs` as plain substring/count
  assertions. **No spec** — template-rendering invariants, not
  behaviour.
- **2 I.5 gaps** (#3 no-trigger NPC, #4 no-refire on repeat) — I.5 was
  dropped by ticket 11. **Restore as new scenarios in `actions.md`**
  (negative no-trigger case + no-refire invariant). Ticket 08 writes
  both scenarios + HTTP tests.
- **1 S1.3 conflict** (#6 `freeaction_with_movement_no_triggers`) —
  audit mapped to S1.3, ticket 11 deleted S1.3 as not-HTTP-observable.
  Substance ("movement action → Idle") is subsumed by Invariant I.1,
  which every S1.x HTTP test already asserts. **Delete browser test, no
  replacement.** S1.3 stays dissolved into I.1.

### No HTTP dual coverage for behaviour tests

All 6 behaviour tests stay browser-only. The two borderlines
(`test_status_updates_during_generation`, `test_form_stays_static_after_submission`)
have static aspects already covered at HTTP (`/status/generating`
strings; POST /action response fragment) — but the browser tests' value
is the client-side wiring (polling JS, htmx swap persistence), not the
HTTP response. Adding HTTP dual coverage would duplicate without adding
authority.

### File deletions

After moves: `trigger.rs`, `interaction.rs`, `editing.rs`, `structure.rs`
all deleted. Browser tier = `behaviour.rs` + `invariants.rs` + `mod.rs`.

### Graduates

- **New grilling ticket (child of map):** "What browser tests are
  missing?" — forward-looking presentation-gap investigation
  (responsive layout, accessibility, htmx swap transitions, error-state
  rendering, connection-status transitions, empty-state rendering).
  Backward-looking audit didn't ask this; deferred to its own grilling
  rather than resolved here. Blocked by ticket 07.
- **Ticket 08 scope sharpened** (see updated body): delete 17 browser
  tests + write 9 new HTTP tests + reorganize 13 keep-tests into 2
  files + write `docs/specs/browser.md` (6 scenarios) + amend
  STRATEGY.md + amend guardrail. Likely ≥8 story points — candidate
  for sub-task split when worked.

### Rejected alternatives (for the record)

- **No browser spec at all** — rejected: specs define tests regardless
  of tier; behaviour tests earn scenarios.
- **Spec all 13 keep-tests** — rejected: Given/When/Then doesn't fit
  `overflowY is auto`; forcing it is ceremony without authority.
- **Rendering-invariant spec (separate or colocated)** — rejected: the
  test code is more precise than any prose bullet; a doc paraphrasing
  the test adds nothing. Same shape as unit branch tests.
- **Split browser spec into multiple files** — rejected: 6 scenarios
  don't earn multiple files; browser tests aren't endpoint-named.
- **Restore S1.3** — rejected: substance subsumed by I.1; no regression
  history justifying a dedicated scenario.
- **Drop I.5 gaps as unit-only** — rejected: both are real invariants
  the browser tier was silently enforcing; dropping loses coverage.
- **Add missing browser tests now** — rejected: out of scope for this
  map (destination = component tier dissolved, not exhaustive browser
  coverage). Graduates as a separate grilling ticket.
