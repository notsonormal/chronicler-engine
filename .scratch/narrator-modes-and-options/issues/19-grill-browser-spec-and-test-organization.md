# Grill: browser-tier spec tags + test-file organization

Type: grilling
Status: pending
Blocked by: (none) — follow-up to ticket 12 (resolved)

## Question

Where do browser-tier scenario tests live — one monolithic `tests/browser/behaviour.rs` carrying tags that point at several feature specs, or per-feature browser test files — and where is that convention written down so it stops being re-litigated per ticket?

Raised during ticket 12 close-out (2026-09-12): options.md scenarios 24.3/24.8/24.9 are browser-tier tests tagged `[docs/specs/options.md]` inside `tests/browser/behaviour.rs`.

### Ground established in-session

- **Two live spec-placement precedents, neither pinned by a standards doc.** Dominant: feature contracts live in feature specs even when observed in the browser — `games.md` 20.1–20.3 and `prompt_presets.md` 21.27 are tagged in `behaviour.rs`, and ticket 07 records it as a user decision ("Browser tests ship WITH their spec scenarios"). Counter-precedent: `browser.md` 18.1–18.2 (world posture form) is a feature interaction living in browser.md. browser.md's own scenarios are otherwise shell mechanics (16.x edit/status/toast, 17.1–17.7 slash-menu mechanics, 17.8/17.9 slash submissions) under the header "Endpoint: browser DOM". A grep of `CODING_STANDARDS.md` + `docs/diataxis/reference/coding_standards/` found no rule either way.
- **The first challenge (spec placement) was resolved in favor of feature-scoped specs**: scenario text is tier-agnostic ("When the client clicks Use"); the covering test picks the tier; splitting scenarios by tier couples the spec to an implementation choice. The user conceded this ("Hmm, in that case") and pivoted to TEST placement.
- **The live question is test-file organization**: `tests/browser/` is already multi-file (`mod.rs` root, `behaviour.rs` interaction, `invariants.rs` CSS invariants), and the HTTP tier uses per-feature files (`tests/http/options.rs`). User leaning: a per-feature `tests/browser/options.rs` mirroring the HTTP convention, instead of appending to the behaviour.rs monolith (which now carries tags from at least 4 spec files: browser, games, prompt_presets, options).
- **The move is mechanical (verified):** every helper the three dock tests use is shared via `tests/test_utils/` and re-exported by the binary root's `pub use test_utils::*` — `with_test_page` (browser.rs:72), `send_action` (browser.rs:95), `dismiss_text_check_if_present` (browser.rs:128), `wait_for_element_children` (wait.rs:41), `wait_for_status_ready` (wait.rs:113), `wait_for_status_generating` (wait.rs:145), plus `CONFIG_PATH`/`TEST_WORLD`/`TEST_PERSONA`. The three options tests are the TAIL of behaviour.rs (~lines 1019–1233).
- **Stale doc found:** `tests/browser/mod.rs`'s `//!` header says behaviour is "tagged against `docs/specs/browser.md`" — already false (games/prompt_presets/options tags coexist). Needs a rewrite whichever way this lands.
- **Validator facts:** `validate_feature_spec.py` keys coverage by (spec path, scenario id); cross-file scenario-number collisions are intentional; either spec layout validates. Ticket 18 plans to wire the validator into the gate — a convention note could ride along there.

### Work already done (do not redo)

- The 24.3+24.8 tag stack on one compound test was split into three tests, one tag each: `test_options_use_click_submits_option` (24.3), `test_options_dock_survives_reload` (24.8), `test_options_dock_edit_fills_without_submitting` (24.9). All green: `behaviour::test_options` → 3 passed; full gate 1613 passed, 0 failed (logs/build_20260912_183014.log).
- The FILE move to `tests/browser/options.rs` was agreed in principle but NOT executed — the three tests still sit at behaviour.rs's tail.

### Sub-questions to grill

1. **Spec-placement rule (confirm, don't reopen).** (a) feature-scoped specs, tier chosen by the covering test (dominant precedent, ticket-07 user decision) vs (b) strict browser.md ownership of every DOM-expressed scenario (would also relocate games 20.x + presets 21.27). If (a) holds, say why browser.md 18.x is not a violation to fix.
2. **Test-file organization.** Does `tests/browser/options.rs` start a decomposition (future `games.rs`, `prompt_presets.rs`) or is it a one-off? Naming: `options.rs` (HTTP symmetry) vs `options_dock.rs`. Where do shared cross-feature interaction tests live if behaviour.rs keeps only shell mechanics?
3. **Move scope.** Just the three options tests now (tail cut, zero helper surgery), or a full behaviour.rs decomposition ticket?
4. **Where the convention is written down.** Candidates: `tests/browser/mod.rs` //! header (stale either way), browser.md header note, the coding-standards doc, `tests/AGENTS.md` — and whether a structure guardrail or validate_feature_spec should enforce it.
5. **Bookkeeping on landing.** Ticket 12's `## Answer` and the map's 12-line describe the behaviour.rs placement — amend both; regenerate `tests/AGENTS.md`; re-run the gate.
6. **invariants.rs boundary.** It is untagged by design (named exemption, "test code is the definition") — confirm the invariant/behaviour boundary survives a per-feature split.

## Answer

(to fill after grilling)
