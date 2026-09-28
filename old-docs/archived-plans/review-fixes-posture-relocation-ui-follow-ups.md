# Review fixes: posture relocation UI follow-ups

## Summary

Apply the four obvious fixes plus the three accepted judgement calls (1-A, 3-B, 5-C) from the two-axis review of the uncommitted ticket-07 work, and record the deferred decisions (2-B, 4, 6-B, plus the marker-test deletion) in the issue tracker. No route changes; one template gains an error state; one handler's validation style aligns to house style; one vacuous test is deleted in favour of a real parse test in ticket 12.

## Key Changes

1. **Type-aware preset card.** `preset_card_html` stops taking `novel_active_id`/`if_active_id` strings and takes the two `ModePresetBundle` references instead, computing `preset.preset_type.bundle_slot(&bundle) == preset.id` internally so callers cannot pass the wrong slot. Only two callers exist (`prompt_presets.rs:79`, `:265`).
2. **World posture validation.** `update_world_posture_handler` `FromStr`-parses each `Some` field (mirroring `update_game_posture_handler`), renders the error span into `#world-posture-status` on any invalid value, and mutates nothing. `parse_or_default` leaves the endpoint. Preserve semantics unchanged (decision 2-B).
3. **Picker failure surfaces honestly.** `posture_controls_html` propagates the three `list_presets` results; on any failure the template renders an error span in the presets section and skips the preset selects; posture controls still render and auto-save.
4. **Trivial cleanups.** Import order in `games/handlers/games.rs`; import `NarratorMode` in `games/templates/games.rs`; delete the vacuous marker test (`prompt_presets_tests.rs:664`) — its grammar note already lives on `PresetForm`'s doc comment.
5. **Issue-tracker records.** Ticket 07 Answer gains the `switch_mode` retarget edge; ticket 07 scope line 15 gains "(editor-level; see Answer)"; ticket 12 scope gains three lines: worlds posture/update contract documentation (decision 2-B), `TestOverride` failure-injection tests for the three new game handlers (decision 6-B), and a real `serde_urlencoded` parse test for `PresetForm` checkbox grammar.

## Implementation

### Phase 1: Code fixes

- [ ] #### Task 1.1: Make `preset_card_html` type-aware (3 SP)
  - Change signature to take `&ModePresetBundle` × 2; compute active per mode via `bundle_slot` internally; update both callers.
  - Handler-level tests (review finding 1): seed a quantifier preset, activate it into the Novel bundle's quantifier slot via `activate_preset_handler`, call `preset_card_handler` — assert `Active · Novel` badge and no `Set Active (Novel)` button; same shape for an impersonate preset in the IF bundle.
- [ ] #### Task 1.2: Error-span validation in `update_world_posture_handler` (3 SP)
  - `FromStr`-parse each `Some` field of `WorldPostureForm`; any parse error → error span into `#world-posture-status`, zero mutation; drop `parse_or_default` here.
  - Test: fresh world, post an invalid value → asserts error span and unchanged stored posture. Also assert an omitted field keeps the stored value (pins decision 2-B from the posture endpoint's side).
- [ ] #### Task 1.3: Preset-load failure span in the game posture fragment (3 SP)
  - `posture_controls_html`: collect results, pass error text into `GamePostureTemplate` on failure; template renders the span and no preset selects; posture controls render unconditionally.
  - Test via `TestOverride::internal` on `"list_presets"` (seam confirmed at `driven/storage/presets.rs:11`): assert span renders and posture selects still render.
- [ ] #### Task 1.4: Trivial cleanups (1 SP)
  - Import order; `NarratorMode` import; delete `test_save_preset_invalid_mode_is_rejected_by_form_deserialization` and its marker comment.

### Phase 2: Issue-tracker records

- [ ] #### Task 2.1: Ticket 07 records (1 SP)
  - Append the `switch_mode` retarget/`allows()` edge (reachable after flag-narrowing an active non-default preset; discarded per review decision 5-C) to the Answer's "Discovered edge (known, not fixed)" section.
  - Add "(editor-level; see Answer)" to "at least one required" in scope item 2.
- [ ] #### Task 2.2: Ticket 12 scope lines (1 SP)
  - Worlds posture/update contract docs: post nothing preserves all three; partial post resets absent posture fields; invalid values return an error span.
  - `TestOverride` failure-injection tests for `update_game_posture_handler`, `update_game_presets_handler`, `switch_game_mode_handler` (Cross-cutting A).
  - `serde_urlencoded` parse test for `PresetForm` checkbox grammar (omitted flags → both false; `allowed_mode_novel=true` → novel only) plus the dev-dependency line.

## Test Plan

- Touched unit suites: prompt-presets builders/handlers, worlds handlers, games handlers/templates. Targeted: `python build.py nextest preset`, `python build.py nextest worlds`, `python build.py nextest games`.
- Regression: 21.x scenarios and the 23 browser tests stay green; system-slot card path and 21.14 preserve contract are pinned by existing tests.
- No new spec scenarios; route count stays 56.

## Per Task/Sub Task Validation Steps

- Task 1.1: `python build.py nextest preset_card` — new quantifier/impersonate handler tests pass; no `&str` active-id params remain on `preset_card_html`.
- Task 1.2: `python build.py nextest worlds` — invalid-input and omitted-field tests pass; browser posture scenarios untouched.
- Task 1.3: `python build.py nextest games` — TestOverride failure-branch test passes; panel tests pass.
- Task 1.4: `python build.py nextest prompt_presets` — suite green after deletion (count drops by 1).
- Full gate: `python build.py`.

## Assumptions

- Only two `preset_card_html` call sites exist (verified by grep).
- Browser tests post only valid posture values, so stricter validation in Task 1.2 needs no spec or browser change.
- Decision 2-B's contract lives in ticket 12's scope only; no `docs/specs/` edit now.
- `.pi/extensions/pi-permission-system/config.json` and `.agents/skills/retro/SKILL.md` excluded; split at commit time.
- Commit shape deferred to commit time (Finding 2, option D).
- The marker-test deletion is safe: the checkbox grammar note already lives in `PresetForm`'s doc comment.
