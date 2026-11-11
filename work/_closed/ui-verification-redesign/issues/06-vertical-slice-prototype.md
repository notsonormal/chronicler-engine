# Prototype the vertical slice: worlds posture flow across all three tiers

Type: prototype
Status: resolved
Blocked by: 04

## Question

**HITL.** Prove the ticket-04 design (see its Answer) on one surface before
multiplying it, per the user's prototype-first decision. The worlds surface is
chosen because it holds the ticket-03 flake signature (audit test 26,
`test_world_posture_change_autosaves_status`). Everything in this ticket is
spike-shaped: it is allowed to be rough, and the verdict is the deliverable,
not the polish.

Build four things:

1. **The settle-gate primitive** in `tests/test_utils/`: an `add_init_script`
   that installs a counter on `htmx:afterSettle`, a baseline armed immediately
   before the interaction, and a Rust poll over `evaluate_value` that waits for
   the counter to advance past the baseline. playwright-rs 0.9.0 has no
   `wait_for_function` — reuse the `wait_for_condition_async` shape. Then wrap
   it inside the interaction helpers (`select_option`, `click`-style paths) so
   a test author cannot opt out. Caveat from the research asset: the five
   `every Ns` pollers also settle, so a document-level counter cannot name
   *which* swap settled — the record names whether the baseline-before-action
   pattern was sufficient here, or whether `detail.elt` target-scoping was
   needed.
2. **HTTP contract coverage** for `POST /worlds/:key/posture` (route
   `router.rs:124`, handler `worlds/handlers/worlds.rs:240`, returns the
   `Saved` span): post with the `HX-Request` header, assert status + fragment.
   This contract has no HTTP test today.
3. **The tier-2 stub**, minimal: a small axum stub in `tests/test_utils/`
   serving the real `assets/index.html` shell plus canned responses for the
   five pollers (`/story-log`, `/status/generating`, options dock, visual
   sidebar, llm messages). Port 1–2 class-1 tests onto it (suggested:
   `test_slash_menu_opens_on_slash` and `test_error_toast_on_action_failure`),
   one shared browser process, fresh page per test. Measure the per-test wall
   time honestly and record it.
4. **The stress loop.** Convert test 26 per the design — the `Saved` contract
   moves to (2); the browser keeps one wiring check (change → server state
   changed) behind the settle-gate — then run that flow ~50 times *without retry
   masking* (e.g. direct test-binary invocation; `cargo nextest run` today applies
   `retries = 1`, so do not let it) and count failures with the legacy signature
   (zero POSTs in the engine log; `Saved` never appears; CDN-style timeouts).
   Acceptance for the slice: zero legacy signatures, new-signature failures
   investigated and characterised.

The **verdict is HITL**. Present the slice results — per-test wall times in
each tier, the stress counts, and whether the settle-gate held on the machine —
and the user rules go/no-go. **Go** unblocks 07–11. **No-go** reopens ticket
04's readiness decision with the prototype evidence, not with argument. If any
part of the slice reads as faking the thing under test (e.g. a canned fragment
that no longer exercises real behaviour), say so in the record — that is the
tier-2 drift tax being measured for the first time.

Record the slice results and the verdict under `## Answer`; keep spike code
either as a small, clean harness module or noted as throwaway with its path —
not a half-finished rewrite of the suite.

## Answer

Prototype built and measured, 2026-09-17. **The verdict is HITL and is still
open** — the results are recorded here and the go/no-go is put to the user.

### 1. The settle-gate primitive — built, and its mechanism corrected by measurement

`tests/test_utils/settle_gate.rs`, plus the `add_init_script` call moved into
`goto_with_connection_check` so every browser page carries the gate and no test
can forget it.

**The ticket's framing of the race was incomplete, and the slice corrected it.**
A gate that waits for the counter to advance past a baseline *is not enough*: the
dashboard runs five self-polling regions (`every 2s` / `every 4s` / `every 5s`),
every poll swap settles, and a document-level counter is therefore satisfied by
poller noise. Measured: the counter advanced on a bare `DIV` poller swap while
the posture POST never left the browser. The wait must name the element, so the
gate records `detail.elt` (id, tag, classes) and only a matching settle counts.
`SettleOutcome::expect_settled` fails loudly with the settles it *did* see, which
is what made this diagnosable.

Target-scoping alone then still failed 6 of 50 runs. The second half of the fix
is a **discipline** the design did not name: await the settle of the swap that
*registered* the element before interacting with it. htmx attaches an element's
`hx-trigger` listeners at the end of the settle task for the swap that produced
it, so interacting on visibility alone is the lost interaction.
`open_world_edit` now awaits the `.worlds-panel` settle before the posture select
is touched.

**A wrong hypothesis, tested and refuted.** The first fix was to zero htmx's
`defaultSettleDelay` (20 ms), on the theory that the `change` dispatches inside
that window. That also went 50/50 green — so `defaultSettleDelay` was **not** the
cause. The delay knob was removed, stock htmx 1.9.10 behaviour is restored, and
the 50/50 result was reproduced with no config override. The record keeps this
because the first attribution was wrong and the experiment is what settled it.

Consequence for ticket 04's design: the gate is not four lines of `add_init_script`;
it is a module with a target-scoping rule *and* an "await the registering swap"
discipline that must live inside the interaction helpers. Ticket 09 must carry
both, not just the counter.

### 2. HTTP contract coverage — `POST /worlds/:key/posture`

Four tests in `tests/http/worlds.rs`, tagged SCENARIO 25.5 (a new scenario added
to `docs/specs/worlds.md`): the `Saved` span + persistence, an invalid value
rendering an error span and mutating nothing, an unknown key returning 400, and a
storage failure returning 500. Each runs in 11–15 ms.

### 3. The tier-2 stub — built, two tests ported, wall times measured

`tests/test_utils/tier2_stub.rs` serves the real `assets/index.html` (via
`include_str!`) plus canned fragments captured from a live engine
(`tests/test_utils/stub_fixtures/`), the static assets nested at `/assets` exactly
as the engine routes them, and one dynamic endpoint, `POST /action/check`, whose
outcome the test names up front. One shared browser process, fresh page per test.

Two tests ported, each a *mirror* of its tier-3 twin so the comparison is
like-for-like (`tests/browser/tier2.rs`):

| Test | Tier 2 (stub) | Tier 3 (engine) | Ratio |
|---|---|---|---|
| slash menu opens on `/` | 0.80 s | 2.73 s | 3.4x |
| error toast on action failure | 0.79 s | 2.94 s | 3.7x |

(median of 3 direct binary runs; in the full-binary nextest run the same two
report 0.58 s and 0.61 s.)

The whole 28-test browser binary is 141.7 s wall, so the two tier-2 tests are
~1% of it — the speed win is real per test but the tier-3 *count* is what sets
the binary's cost.

**Drift tax, measured honestly.** For these two tests the tax is low, because
neither actually needs a server: the slash menu is pure client-side JS, and the
error-toast test dispatches a synthetic `htmx:beforeSwap` rather than provoking
a real 500. So they prove the stub *hosts* the shell faithfully; they do not prove
it hosts a test that reads canned server content. The real drift tax lands on the
fragment-reading class-1 tests in ticket 07, and is unmeasured until then. One
genuine finding: the stub must mount assets at `/assets` (`nest_service`), not as
a fallback — the shell requests `/assets/htmx.min.js`, and a fallback double-nests
the path and serves no JS at all, which silently kills every htmx behaviour.

### 4. The stress loop — 50 runs, zero legacy signatures

`scripts/stress_posture.sh` invokes the browser test **binary directly**, so
nextest's `retries = 1` cannot mask a failure. Test 26 was converted as the design
specified: the `Saved` contract moved to the HTTP tier (SCENARIO 25.5), and the
browser keeps one wiring check — change → reload → the re-rendered select shows
the persisted tense.

| Run | Configuration | Pass | Legacy signatures |
|---|---|---|---|
| 1 | target-scoped gate, no registering-swap await | 44/50 | 6 |
| 2 | + await registering swap, `defaultSettleDelay = 0` | 50/50 | 0 |
| 3 | + await registering swap, **stock htmx delay** | 50/50 | 0 |

The 6 failures in run 1 are all the legacy signature — the gate timed out on
`#world-posture-status` and the log shows only poller swaps. Runs 2 and 3 are
identical because the delay was never the cause. Run 3 is the shipped
configuration. Every failure in run 1 was investigated and characterised, as the
ticket required; none were unexplained.

### Verification actually run

- `python build.py` — **green, exit 0**: 1592 integration passed, 28 browser
  passed, 0 failed, clippy + all guardrails + spec-coverage clean.
- `python scripts/validate_feature_spec.py` — 141 declared, 141 covered, 0 gaps,
  0 orphans, 0 untagged, 0 surface mismatches.
- The vendored `playwright-rs` 0.9.0 crate is unmodified (an earlier attempt to
  patch `add_init_script` was reverted before it took effect; confirmed by
  reading the crate source).

### Assets and spike paths

The spike is a small, clean harness module, not throwaway:

- `tests/test_utils/settle_gate.rs` — the gate (ship in ticket 09)
- `tests/test_utils/tier2_stub.rs` + `tests/test_utils/stub_fixtures/` — the stub
  (ship in ticket 07)
- `tests/browser/tier2.rs` — the two ported tests (grow in ticket 07)
- `scripts/stress_posture.sh`, `scripts/measure_tiers.sh` — the measuring tools
  (ticket 12 reuses the stress loop)

### Open question for the verdict

One thing the slice could not settle, and it is the ticket's own caveat named in
the map's fog: the "await the registering swap" discipline is a *convention* a
test author can forget, unlike target-scoping which the helper enforces. Ticket
09 must decide whether the helpers can enforce it too (e.g. a page-level
"last settled swap" watermark that `select_option_and_settle` checks against), or
whether it stays a documented rule. That decision is the fog this ticket's
verdict graduates.

**Go** unblocks 07–11; **no-go** reopens ticket 04's readiness decision with this
evidence.

## HITL verdict

**Go** — ruled by the user in session, 2026-09-17, on the measured evidence
recorded above. The verdict ratified two attached decisions:

1. **Ticket 07 gets a drift-tax exit check.** The slice did not measure the
   drift tax for tests that *read* server content (both ported tests are
   server-independent). Ticket 07 must characterise it on the first
   fragment-reading port before multiplying it, per the ticket's own
   fake-detection rule.
2. **Enforcement lives in the helpers (decision 2, option 1).** The
   registering-swap discipline is structural, not documented: every helper
   that triggers a swap awaits that swap's settle before returning. The bug
   lived in a helper (`open_world_edit`'s visibility wait); the fix belongs
   there. Ticket 09's keeper-set wiring owns this. The generic gate-level
   watermark option was rejected as more machinery for the same guarantee.

A standing correction follows from the slice: [ticket 03's mechanism]
(03-root-cause-the-hx-post-no-fire.md) (the `change` lost inside the 20 ms
`defaultSettleDelay` window) was refuted experimentally — see the correction
appended to that ticket's Answer and the slice's mechanism section above. The
operative race is interacting before the *registering* swap settles; the delay
window is neither necessary nor sufficient.

Tickets 07–11 are unblocked.

