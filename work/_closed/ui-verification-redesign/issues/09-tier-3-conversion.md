# Tier-3 conversion: settle-gate wrappers and the keeper set

Type: task
Status: resolved
Blocked by: 06

## Question

Put every remaining full-stack browser test behind the settle-gate primitive
built in ticket 06, and finalise the keeper set per the ratified design
(ticket 04 Answer: readiness option A — enforcement inside the harness helpers,
`hx-preserve` as fallback for a surface that resists wrapping).

In order:

1. **Wrapper converts the suite, not vice versa.** Route every swap→interact
   call in `tests/test_utils/browser.rs` through the settle-gate:
   `select_option` and the `click`-then-swapped-interact paths wait for the
   `htmx:afterSettle` counter to advance past the baseline armed before the
   action. `wait_until_visible` stops being a readiness primitive — it may
   remain as a rendering assertion in tier 2, but it no longer licenses an
   interaction in tier 3. The ticket-06 slice recorded `detail.elt`
   target-scoping as **required** (poller noise satisfied the plain counter —
   measured, not hypothetical): wire that in now.

   **Registering-swap rule (ratified with ticket 06's verdict, decision 2
   option 1):** target-scoping alone failed 6/50 in the slice. Every helper
   that triggers a swap must await *that swap's* settle before returning —
   enforcement is structural, in the helper, not a documented convention.
   `open_world_edit`'s await of the `.worlds-panel` settle is the reference
   shape (ticket-03's 20 ms `defaultSettleDelay` mechanism was refuted; see
   the correction on that ticket's Answer).
2. **Keeper set converts.** Move the surviving tier-3 tests onto the wrapped
   helpers, unchanged assertions: tests 14/15 (kept per rule-07's client-wiring
   rationale), `test_options_dock_survives_reload` (17), test 8 from
   `slash_menu.rs` *if ticket 07 left it here*, and the wiring smoke guards
   (see 3).

   **Disposition required for tests 21 and 24.** Ticket 08b added stronger HTTP
   coverage for both (games.md 20.8 for the posture-fragment render,
   worlds.md 25.6 for the world edit-form render), so their assertions are now
   curl-observable — but their *click-hop wiring* (worlds tab → Edit htmx swap;
   games tab switch) is not, and this ticket owns the swap→interact helpers
   that would cover it. Decide each explicitly and record the call: either fold
   its wiring into the item-3 guard for that surface, or delete it with the
   guard as its replacement. Do not leave either undeclared.
3. **Wiring smoke guards** — convert from ticket 08's demotions (one per
   surface, assert the server actually changed, not the fragment): posture
   change in `worlds.rs` (the ticket-03 flake home — ticket 06's slice already
   ran its ~50-run stress loop; **this ticket owns the guard's final form**,
   and ticket 12 re-runs the loop against the final suite), posture change in
   `games.rs`, one preset-chain guard, one options-Use guard (assert the guard
   submits the option text *read from the rendered button*, not a hardcoded
   seed string — otherwise the dock-render → submit linkage stays uncovered).
   All behind the wrappers. These replace — not supplement — the browser tests
   ticket 08 deleted.
4. **Story-log gate becomes explicit.** The shared `#story-log .log-entry`
   precondition in `with_test_page` (`browser.rs:87`) is replaced by an
   explicit `wait_for_story_log()` called only by the tier-3 tests that need it
   — per ticket 04's ratified deletion item; a failure is then attributable to
   the test that needed the log, not to N unrelated tests.
5. Fallback path: if one surface genuinely resists wrapping (evidence, not
   hunch — record what failed), `hx-preserve` on that surface's template is
   the ratified structural fix. Name the surface and the evidence in
   `## Answer`.

Do not widen tier 3 while converting. If a keeper's assertions prove
curl-observable mid-work, move it in this ticket's record rather than leaving
it.

Record under `## Answer`: converted-helper surface (which helpers now gate,
and how test authors are prevented from opting out), the final keeper list with
paths, any per-surface fallback applied and why, and the tier-3 binary's wall
time before/after.

## Answer

Resolved 2026-09-19. Tier 3 now holds 7 tests, all behind the settle-gate
wrappers; the two browser copies whose HTTP twins carried their assertions are
deleted; four wiring smoke guards exist; the gate cannot be bypassed without
failing the build. Full gate green, validator 141/141.

### 1. Converted-helper surface

`tests/test_utils/settle_gate.rs` gained one function:

- `await_panel_ready(page, selector) -> SettleOutcome` — a **baseline-free**
  wait over the whole recorded settle-target list. It exists because
  `await_settle_target` filters to entries past a baseline, and every dashboard
  panel is `hx-trigger="load"`: its swap lands at page load, *before* any test
  arms a baseline, so a baseline-relative wait can never match it and times
  out. This closed a latent gap in the shipped `open_world_edit`, whose
  `wait_for_element_children` proved content was present but not that the swap
  that registered the Edit button had settled — the exact ticket-03 shape.
  Documented caveat: the recorded list caps at 64 entries, so this is safe for
  a load settle read at test start, not as a general historical search.

`tests/test_utils/browser.rs` is now the sole home of swap-triggering helpers:

| Helper | Triggers | Awaits |
|---|---|---|
| `open_worlds_tab` | Worlds tab click | `.worlds-panel` (absolute, load swap) |
| `open_world_edit` | Worlds tab + "Test Realm" Edit click | `.worlds-panel` (load, then `click_and_settle`) |
| `open_games_tab` | Games tab click | `.games-panel` (absolute, load swap) |
| `open_prompt_presets_tab` | Prompt Presets tab click | `.prompt-presets-panel` (absolute, load swap) |
| `open_preset_editor` | a card's Edit click | `.preset-card` (`click_and_settle`) |
| `seed_system_preset` | engine's own `POST /prompt-presets` | n/a (HTTP seed) |

**How opt-out is prevented (the ratified structural enforcement).** A new
guardrail, `check_browser_interactions_use_settle_gate`
(`tests/infrastructure/guardrails/structure.rs`), fails the build on any
literal `.click(` or `.select_option(` in a `tests/browser/*.rs` file that
contains `with_test_page`. Scope is self-maintaining: `tier2.rs`
(`with_stub_page`) and `invariants.rs` (`SharedBrowser`) drive a stub with no
htmx swap lifecycle, so they are exempt without a names list. The scan is
text-based because locator chains wrap across lines
(`page.locator(..)\n.await\n.first()\n.click(None)`), which a positional regex
cannot match reliably. Moving an interaction into a `page.evaluate` string is
not an escape hatch — that literal is flagged too. Verified to fire on an
injected raw click (`browser/options.rs:117`), then reverted;
`grep` confirms zero raw interactions remain in the five tier-3 files.

### 2. Final keeper list

Seven tier-3 tests, all through `with_test_page`:

| Test | File | Role |
|---|---|---|
| `test_form_stays_static_after_submission` | `tests/browser/dashboard.rs` | 14 — static-shell wiring |
| `test_status_updates_during_generation` | `tests/browser/dashboard.rs` | 15 — status wiring |
| `test_games_posture_change_reaches_server` | `tests/browser/games.rs` | **guard** (21's replacement) |
| `test_options_dock_survives_reload` | `tests/browser/options.rs` | 17 — reload persistence |
| `test_options_use_click_submits_rendered_option_text` | `tests/browser/options.rs` | **guard** (options Use) |
| `test_preset_duplicate_edit_save_click_chain` | `tests/browser/prompt_presets.rs` | **guard** (preset chain) |
| `test_world_posture_change_autosaves_server_state` | `tests/browser/worlds.rs` | **guard** (24's replacement) |

Tier 3 did not widen: 19 → 20 binary tests is −1 games, −1 worlds, +1 games
guard, +1 worlds guard, +1 preset guard, +1 options-Use guard.

### 3. Tests 21 and 24 dispositions

**Both deleted, with their wiring folded into the guards.**

- **Test 21** (`test_games_panel_renders_posture_fragment`, 27.1) → deleted. Its
  content assertions are carried (and exceeded) by `tests/http/games_fragment.rs`
  SCENARIO 20.8. Its one unique fact — the Games-tab click loading the posture
  fragment — is now the first half of `test_games_posture_change_reaches_server`,
  which calls `open_games_tab` before touching any select.
- **Test 24** (`test_world_edit_form_renders_posture_selects`, 29.1) → deleted.
  Content carried by `tests/http/worlds.rs` SCENARIO 25.6. Its unique fact — the
  Worlds-tab click plus Edit htmx swap — is the Worlds guard's `open_world_edit`
  call, which now fully awaits that swap (see the `await_panel_ready` gap above).

Neither was left undeclared: 27.1 and 29.1 were retired and their files reworded
to the guards' actual assertions.

### 4. Per-surface fallback

**None applied.** No surface resisted wrapping, so `hx-preserve` was not used.
Evidence: all seven tests pass 50/50 in the stress loop and green in repeated
full-gate runs with the string-typed wait paths in place.

### 5. Story-log precondition split

`with_test_page` no longer calls `wait_for_story_log`. Consumers after the
split: **no tier-3 test calls it at all** — every tier-3 test either does not
read the log or asserts it explicitly (`#story-log .log-entry` /
`.log-entry.input` count waits, classified as rendering assertions per item 1's
rule, not readiness primitives). `SharedBrowser::open_page` (tier-2) keeps its
call: the stub serves canned entries synchronously, so it is genuine readiness
there, and it is the one place the failure attribution argument does not apply.
The shared precondition therefore had no tier-3 consumer.

### 6. Mechanism corrections found in implementation

Recorded because each was a real bug in the plan's assumed mechanics:

1. **`#action-area` is not the POST's swap target.** `action_check_handler`
   returns `HX-Retarget: #status-display` (`builders/headers.rs:21`); the
   action-area swap is a later, unsolicited `action-area-refresh` fetch, so
   awaiting `#action-area` from a click would race. The options guard awaits
   `#status-display`. (Successor to ticket 03's correction.)
2. **`send_action` already *is* the action-submit path** and needed no new
   helper: it sets the field, submits the static shell, and latches the ack via
   `wait_for_status_generating`. Test 14 was converted to it; the plan's
   `submit_command_and_settle` was dropped as duplication.
3. **`.click`/`.option-btn` are strict-mode multi-match.** `.option-btn`
   resolves to 3 buttons, and the worlds panel has two Edit buttons (two seeded
   worlds). The options guard scopes to `>> nth=0`; `open_world_edit` scopes to
   the "Test Realm" world item by name rather than `.first()` (order-independent).
4. **The preset card's title changes on Edit** (`Edit <name>`), so a
   title-scoped selector stops matching after the Edit swap. The guard scopes
   the form fields to `.preset-card.edit-form` (single instance) and the Save
   swap to `.preset-card` (the form's own `hx-target`).
5. **Playwright-only pseudo-classes do not work in native `querySelector`.**
   `:has()`/`:text-is()` selectors must be resolved by a locator method
   (`inner_html()`), not interpolated into a `page.evaluate` string.
6. **Ticket 07 left no tier-3 test to receive.** `grep -rn "_tier2" tests/`
   returns nothing and no slash-menu file exists in `tests/browser/` — test 8
   and its `_tier2` twins are already tier-2.

### 7. Verification

| Check | Result |
|---|---|
| Full gate (`python build.py --target-dir target/agent9`) | **green, exit 0** |
| clippy | 0 warnings |
| architecture tests | 1 passed |
| guardrail tests | 135 passed |
| integration tests | 1604 passed, 0 failed, 2 skipped |
| browser tests | **20 passed, 0 failed** |
| `validate_feature_spec.py` | **141 declared, 141 covered, 0 gap(s), 0 orphan(s), 0 untagged, 0 surface mismatch(es)**, quarantine 86/86 |
| Guardrail fires | injected raw `.click(` flagged at `browser/options.rs:117`; reverted |
| `cargo fmt --check` | clean |

**Wall time.** Browser binary: **52.45 s** under nextest (was 490 s — ticket
07's serialization artifact, unchanged by this ticket and still ticket 12's
step 4). Per-test direct medians not re-measured here; tier-3 is 7 engine-boot
tests against tier-2's 12 stub tests at ~0.6 s each.

**Stress loop — final guard form.** 50 runs of
`worlds::test_world_posture_change_autosaves_server_state`, direct binary
invocation, no retry masking: **50 pass / 0 fail / 0 lost interactions.** The
new guard form holds on the machine.

**Note for ticket 12:** `scripts/stress_posture.sh` hardcodes
`target/debug/deps/browser-*`, so it cannot see an isolated `--target-dir`
build; the loop above was run directly against the isolated binary. Ticket 12
should either run it against the default `target/` or parameterize the script.
The test name and module path are unchanged, so the script's `--exact` filter
still matches.

### 8. Spec/test bookkeeping

| Spec | Change |
|---|---|
| `browser_games.md` | 27.1 reworded to the guard (tab open + change reaches server); 20.8 cross-referenced |
| `browser_worlds.md` | 29.1 retired; 29.2 reworded to the guard (reload shows persisted tense); 25.5/25.6 cross-referenced |
| `browser_options.md` | new 26.4 for the options-Use guard; 24.13 cross-referenced |
| `browser_prompt_presets.md` | re-created (deleted by ticket 08) with 28.1 for the preset-chain guard |
| `guardrails.md` | regenerated for the new rule and shifted line numbers |

### 9. Known overlap and residual risk

- **Preset guard overlaps `tests/http/prompt_presets.rs` SCENARIO 21.27** —
  cross-tier overlap per STRATEGY.md's overlap rule, not same-tier duplication:
  HTTP has the request chain, the browser has the click handling. Recorded in
  the test's doc comment.
- **The preset guard must not be stress-looped.**
  `generate_preset_id()` (`handler_helpers.rs`) is millisecond-based, so a
  create and a duplicate in the same millisecond collide (ticket 08's finding).
  One seeded create and one Duplicate click keeps the window small but nonzero.
  The underlying bug remains unfixed — it still warrants its own ticket.

### 10. Review follow-up (post-Answer)

A two-axis code review (`code-review` skill) raised findings on both axes. All
were addressed; two turned out to be wrong, and one uncovered a real bug the
review had not spotted. The permission-config change was left alone on the
user's instruction (and is not this ticket's — its mtime predates the session).

**Fixed.**

1. **Task-referencing comments (Standards, hard).** Seven new comments violated
   `CODING_STANDARDS.md` ("Never reference the task in code comments"). Stripped
   from `games.rs`, `worlds.rs`, `options.rs`, `prompt_presets.rs` and the
   guardrail's doc block; each now explains the mechanism instead. The
   references inside `settle_gate.rs` are pre-existing at `HEAD` and were left
   untouched.
2. **Guardrail understated its own coverage (Spec, wrong).** It flagged only
   `.click(`/`.select_option(`, so `dispatchEvent(` — which can synthesize the
   `change` a posture select listens for — passed. Added `dispatchEvent(` and
   `requestSubmit(` to the scan, two unit tests, and verified the rule fires on
   an injected `dispatchEvent` (`browser/options.rs:123`) before reverting the
   probe.
3. **Preset guard could not separate server persist from client re-render
   (Spec, weak spot).** Added a `page.reload()` plus re-open before the final
   assertion.
4. **Vacuous preset assertion (found during the fix, missed by the review).**
   The seed posted no mode flags, so the handler fell back to
   `default_allowed_modes()` (Novel + IF) — both activation buttons rendered
   *before* the edit, so the post-save assertion held regardless. The seed now
   posts `allowed_mode_novel` only, a pre-edit assertion checks the copy carries
   Novel alone, and only IF is ticked — so the save must *add* a mode for the
   test to pass.
5. **Unused `seed_system_preset` return (Standards, Speculative Generality).**
   The id was produced only to be discarded. Now returns `()`, with the
   card-rendered check kept as a plain assertion.
6. **`wait_until_visible` licensed a tier-3 interaction (Spec item 1, partial).**
   `open_world_edit` waited for the Edit button's visibility before clicking.
   Removed: `click_and_settle` arms the gate and Playwright waits for
   actionability, so visibility guaranteed nothing here. Post-removal stress
   loop: 50/50.
7. **Duplicated target-list read (Standards, judgement call).** Commented the
   single final read in `await_panel_ready` rather than restructuring.

**Rejected, with reasons.**

- **Spec sediment.** The review claimed the specs carried "deleted test 21/24"
  prose. No such line exists (`grep` over `docs/specs/*.md`); the quote was
  fabricated. No change.
- **Middle Man on `open_games_tab`/`open_worlds_tab`/`open_prompt_presets_tab`.**
  Each names a surface and pins its own gate target; `open_tab` is a real seam,
  not a pass-through. Removing them would put a target string at every call
  site. No change.
- **Duplicated guard shape (games/worlds).** The shared shape is the assertion
  the spec demands ("server actually changed"); extracting it would hide the
  reload-then-reread step that makes each guard meaningful. No change.

**Re-verification.** Full gate green (exit 0): clippy clean, 137 guardrails
(+2), 1606 integration, 20 browser, validator 141/141. Stress loop on the
changed `open_world_edit`: 50/50, zero lost interactions.
