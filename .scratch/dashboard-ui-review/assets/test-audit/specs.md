# Spec Scenario Audit — docs/specs/*.md vs tests/STRATEGY.md

Read-only audit. 146 scenarios declared across 18 spec files (matches
`scripts/validate_feature_spec.py` output: `146 declared, 146 covered, 0
gap(s), 0 orphan(s), 0 untagged, 0 surface mismatch(es), quarantine 85/85`
— run 2026-09-29, read-only, no build/test invoked).

## Surface counts

| Surface | Count |
|---|---|
| HTTP (status/body/header, clean) | 59 |
| DOM (browser_*.md, clean) | 22 |
| INTERNAL — hard leak (names `message_service.*`, `narrative.*`, `movement.*`, `GenerationStatus::`, `storage.*`, `game_catalogue.*` directly in the spec prose) | 58 |
| INTERNAL — soft leak ("the stored X is Y" phrasing, no internal identifier in the spec prose, but the covering test verifies it via a direct `AppState`/storage read rather than a follow-up HTTP request) | 7 |
| **Total** | **146** |

65 of 146 scenarios (~45%) are INTERNAL by some measure. The 58 hard-leak
scenarios live in non-browser spec files (`actions.md`, `swipe_new.md`,
`retrigger.md`, `reset.md`, `story_log.md`, `options.md` 24.13, `games.md`
20.7, `prompt_presets.md` 21.27); the 7 soft-leak scenarios are `worlds.md`
25.1-25.5 and `prompt_presets.md` 21.16, 21.25 — spec prose that reads as a
plain domain-state assertion ("the stored world's tense is X") but whose only
verification path in the covering test is `state.world_catalogue.get_world()`
/ `state.prompt_preset_service.get_preset()` / `storage.get_settings()`
called directly, not a second HTTP GET reading the value back. All 65 are
covered by tier-1 (`tests/http/`) tests — by construction, since the
surface-consistency rule forbids non-`browser_*` specs from being tagged
outside `tests/http/`. So the "tier mismatch" this audit finds is not
wrong-directory placement (the validator confirms 0 surface mismatches) —
it's scenario **content** and covering-test **assertion mechanism** that
contradict STRATEGY.md's own carve-out list and its stated tier-1 purpose.
See Finding 1.

**Methodology note**: hard leaks were found by grepping spec prose for
internal identifiers (`message_service`, `narrative.`, `GenerationStatus::`,
etc.) — mechanical and complete. Soft leaks were found by manually reading
each spec file and cross-checking ambiguous "the stored/reports X" phrasing
against the covering test's actual assertion code — this pass covered every
spec file at least once, but wasn't re-run exhaustively line-by-line a second
time, so the 59-scenario "HTTP clean" bucket is a **lower bound on leaks, not
a guarantee of zero** for any single row; treat borderline "OK" rows (any
Then using "reports", "still has", "shows", or similar passive-state phrasing)
as worth a second look before relying on them.

## Scenario table

| Spec file | # | Title | Surface | Covering test(s) | Tier | Placement note |
|---|---|---|---|---|---|---|
| actions.md | 1.1 | Successful action produces exactly one narration and returns to Idle | INTERNAL | tests/http/actions.rs:20 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 1.2 | User input is persisted in history before the narration entry | INTERNAL | tests/http/actions.rs:56 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 1.4 | Action whose quantifier detects an NPC fires the trigger | INTERNAL | tests/http/actions.rs:94 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 1.5 | Empty input produces a continuation narration without adding an Input message | INTERNAL | tests/http/actions.rs:295 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 1.6 | Action trigger continuation re-runs the quantifier and detects newly-present NPCs | INTERNAL | tests/http/actions.rs:329 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 1.7 | NPC without triggers produces narration with no event header | INTERNAL | tests/http/actions.rs:170 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 1.8 | Repeat action against a one-shot trigger does not refire | INTERNAL | tests/http/actions.rs:223 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 2.1 | Action in a non-existent room sets GenerationStatus to Error | INTERNAL | tests/http/actions.rs:390 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 2.2 | LLM transport failure sets GenerationStatus to Error | INTERNAL | tests/http/actions.rs:426 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 2.3 | Empty LLM response sets Error without persisting an empty narration | INTERNAL | tests/http/actions.rs:454 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 2.4 | Trigger narration failure preserves main narration, logs the failure, and sets Error | INTERNAL | tests/http/actions.rs:487 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 3.3 | Delayed LLM response does not deadlock the pipeline | INTERNAL | tests/http/actions.rs:565 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 6.1 | Three actions in sequence produce three Input and three Narration entries | INTERNAL | tests/http/actions.rs:587 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 6.2 | Execute → retry → execute produces two inputs and at least two narrations | INTERNAL | tests/http/actions.rs:623 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 6.3 | Async action sequence then retry completes with two inputs persisted | INTERNAL | tests/http/actions.rs:656 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 1.9 | `/impersonate` dispatches as impersonate and produces an Input message | INTERNAL | tests/http/actions.rs:679 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 1.10 | `/guide` dispatches as guided generation and does not persist an Input message | INTERNAL | tests/http/actions.rs:726 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| actions.md | 1.12 | Recognized engine commands bypass the player-input text check | INTERNAL | tests/http/actions.rs:767 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| browser_dashboard.md | 16.5 | Command form stays static after submission | DOM | tests/browser/dashboard.rs:9 | 3 (full-stack browser) | OK (tier 3 (full-stack browser)) |
| browser_dashboard.md | 16.6 | Status display updates during generation | DOM | tests/browser/dashboard.rs:42 | 3 (full-stack browser) | OK (tier 3 (full-stack browser)) |
| browser_dashboard.md | 16.7 | Action failure renders an error toast | DOM | tests/browser/stub/dashboard.rs:118 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_dashboard.md | 16.8 | A newer error is not hidden by an older error's timer | DOM | tests/browser/stub/dashboard.rs:139 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_dashboard.md | 16.9 | Send button still locks and unlocks after an action-area swap | DOM | tests/browser/stub/dashboard.rs:163 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_dashboard.md | 16.10 | Status errors still reach the observer after an action-area swap | DOM | tests/browser/stub/dashboard.rs:200 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_games.md | 27.1 | Opening the Games tab renders the posture fragment, and a change reaches the server | DOM | tests/browser/games.rs:8 | 3 (full-stack browser) | OK (tier 3 (full-stack browser)) |
| browser_options.md | 26.2 | The current set survives a page reload | DOM | tests/browser/options.rs:5 | 3 (full-stack browser) | OK (tier 3 (full-stack browser)) |
| browser_options.md | 26.3 | Clicking Edit fills the input without submitting | DOM | tests/browser/stub/options.rs:9 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_options.md | 26.4 | Clicking Use submits the rendered option text | DOM | tests/browser/options.rs:46 | 3 (full-stack browser) | OK (tier 3 (full-stack browser)) |
| browser_prompt_presets.md | 28.1 | Duplicating, editing and saving a preset through its buttons | DOM | tests/browser/prompt_presets.rs:15 | 3 (full-stack browser) | OK (tier 3 (full-stack browser)) |
| browser_slash_menu.md | 31.1 | Typing slash opens the command suggestion menu | DOM | tests/browser/stub/slash_menu.rs:20 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_slash_menu.md | 31.2 | Typing a prefix filters the suggestions | DOM | tests/browser/stub/slash_menu.rs:47 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_slash_menu.md | 31.3 | Arrow keys move the active suggestion | DOM | tests/browser/stub/slash_menu.rs:75 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_slash_menu.md | 31.4 | Enter populates the input with the highlighted command | DOM | tests/browser/stub/slash_menu.rs:127 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_slash_menu.md | 31.5 | Escape closes the suggestion menu | DOM | tests/browser/stub/slash_menu.rs:151 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_slash_menu.md | 31.6 | Clicking a suggestion populates the input | DOM | tests/browser/stub/slash_menu.rs:171 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_slash_menu.md | 31.7 | The menu reopens after the action-area is re-rendered | DOM | tests/browser/stub/slash_menu.rs:207 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_story_log.md | 30.1 | Clicking the edit button activates edit mode | DOM | tests/browser/stub/story_log.rs:12 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_story_log.md | 30.2 | Cancelling edit restores the original text | DOM | tests/browser/stub/story_log.rs:37 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_story_log.md | 30.3 | Edit textarea persists across polling cycles | DOM | tests/browser/stub/story_log.rs:81 | 2 (stub browser) | OK (tier 2 (stub browser)) |
| browser_worlds.md | 29.2 | Changing a world posture select reaches the server | DOM | tests/browser/worlds.rs:8 | 3 (full-stack browser) | OK (tier 3 (full-stack browser)) |
| games.md | 17.1 | Creating a game with valid world and persona returns success and refreshes | HTTP | tests/http/games_create.rs:13 | 1 (HTTP) | OK (tier 1) |
| games.md | 17.2 | Creating a game with an unknown world key returns 400 | HTTP | tests/http/games_create.rs:94 | 1 (HTTP) | OK (tier 1) |
| games.md | 17.3 | Creating a game with an unknown persona key returns 400 | HTTP | tests/http/games_create.rs:126 | 1 (HTTP) | OK (tier 1) |
| games.md | 18.1 | Switching to an existing game returns success and refreshes | HTTP | tests/http/games_switch.rs:13 | 1 (HTTP) | OK (tier 1) |
| games.md | 18.2 | Switching to an unknown game id returns 400 | HTTP | tests/http/games_switch.rs:48 | 1 (HTTP) | OK (tier 1) |
| games.md | 19.1 | Deleting a non-active game returns success | HTTP | tests/http/games_delete.rs:13 | 1 (HTTP) | OK (tier 1) |
| games.md | 19.2 | Deleting the active game returns 400 | HTTP | tests/http/games_delete.rs:43 | 1 (HTTP) | OK (tier 1) |
| games.md | 19.3 | Deleting an unknown game id returns success (idempotent) | HTTP | tests/http/games_delete.rs:76 | 1 (HTTP) | OK (tier 1) |
| games.md | 20.4 | A posture auto-save that fails storage surfaces a 500 error span | HTTP | tests/http/games_config.rs:20 | 1 (HTTP) | OK (tier 1) |
| games.md | 20.5 | A presets auto-save that fails storage surfaces a 500 error span | HTTP | tests/http/games_config.rs:44 | 1 (HTTP) | OK (tier 1) |
| games.md | 20.6 | A mode switch that fails storage surfaces a 500 error span | HTTP | tests/http/games_config.rs:68 | 1 (HTTP) | OK (tier 1) |
| games.md | 20.7 | A perspective or tense auto-save re-renders the fragment with the new value selected | INTERNAL | tests/http/games_config.rs:96 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| games.md | 20.8 | The games fragment renders the posture controls for the active game | HTTP | tests/http/games_fragment.rs:10 | 1 (HTTP) | OK (tier 1) |
| narrator_mode.md | 23.1 | Creating a game in an Interactive Fiction world inherits the IF posture and prompt bundle | INTERNAL | tests/http/narrator_mode.rs:48 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| narrator_mode.md | 23.2 | Switching a Novel game to Interactive Fiction retargets the system preset | INTERNAL | tests/http/narrator_mode.rs:102 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| narrator_mode.md | 23.3 | The mode switch re-renders the posture fragment with the IF bundle and nudges the perspective | INTERNAL | tests/http/narrator_mode.rs:144 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| narrator_mode.md | 23.4 | A deliberately-set perspective survives a mode switch | INTERNAL | tests/http/narrator_mode.rs:184 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| narrator_mode.md | 23.5 | Impersonate steering is available in an Interactive Fiction game | HTTP | tests/http/narrator_mode.rs:227 | 1 (HTTP) | OK (tier 1) |
| options.md | 24.1 | The /options command renders the generated set in the dock | HTTP | tests/http/options.rs:85 | 1 (HTTP) | OK (tier 1) |
| options.md | 24.2 | /options with no scene history surfaces an error and an empty dock | HTTP | tests/http/options.rs:109 | 1 (HTTP) | OK (tier 1) |
| options.md | 24.4 | Regenerating replaces the set and adds no history entries | HTTP | tests/http/options.rs:132 | 1 (HTTP) | OK (tier 1) |
| options.md | 24.5 | A failed regeneration keeps the previous set | HTTP | tests/http/options.rs:166 | 1 (HTTP) | OK (tier 1) |
| options.md | 24.13 | Using an offered option submits it as the player's input and narrates | INTERNAL | tests/http/options.rs:343 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| options.md | 24.6 | The always-on toggle generates options after a narration turn | HTTP | tests/http/options.rs:210 | 1 (HTTP) | OK (tier 1) |
| options.md | 24.7 | The always-on toggle never fires after an impersonate turn | HTTP | tests/http/options.rs:245 | 1 (HTTP) | OK (tier 1) |
| options.md | 24.12 | A narration turn with the toggle off generates no options | HTTP | tests/http/options.rs:269 | 1 (HTTP) | OK (tier 1) |
| options.md | 24.10 | A numbered-list response renders parsed options | HTTP | tests/http/options.rs:292 | 1 (HTTP) | OK (tier 1) |
| options.md | 24.11 | /options works in an Interactive Fiction game | HTTP | tests/http/options.rs:314 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.1 | Panel renders the full surface | HTTP | tests/http/prompt_presets.rs:96 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.2 | Single card returns the preset | HTTP | tests/http/prompt_presets.rs:127 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.3 | Single card for a nonexistent preset returns an error span | HTTP | tests/http/prompt_presets.rs:161 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.4 | Edit form returns a populated form for a non-default preset | HTTP | tests/http/prompt_presets.rs:176 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.5 | Edit form for a nonexistent preset returns an error span | HTTP | tests/http/prompt_presets.rs:213 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.6 | Edit form for a default preset returns an error span | HTTP | tests/http/prompt_presets.rs:228 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.7 | View form returns a read-only form | HTTP | tests/http/prompt_presets.rs:246 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.8 | View form for a nonexistent preset returns an error span | HTTP | tests/http/prompt_presets.rs:267 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.9 | Create a system preset → panel re-renders with the new preset | HTTP | tests/http/prompt_presets.rs:282 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.10 | Create a quantifier preset → panel re-renders with the new preset | INTERNAL | tests/http/prompt_presets.rs:304 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| prompt_presets.md | 21.11 | Create with an invalid preset_type returns an error span | HTTP | tests/http/prompt_presets.rs:324 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.12 | Create with a missing required field returns 422 | HTTP | tests/http/prompt_presets.rs:342 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.13 | Create reports a save failure in the response body | HTTP | tests/http/prompt_presets.rs:358 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.14 | Update a preset → card re-renders with the new name | HTTP | tests/http/prompt_presets.rs:381 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.15 | Update a nonexistent preset returns an error span | HTTP | tests/http/prompt_presets.rs:412 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.16 | Update ignores the form's preset_type and keeps the stored type | INTERNAL (soft: 'the stored X is Y' verified via direct app-state/storage read, not a follow-up GET) | tests/http/prompt_presets.rs:430 | 1 (HTTP) | STRATEGY.md tier-1 purpose ('server response a curl would see') violated by covering test's direct state read — see Finding 1 addendum |
| prompt_presets.md | 21.17 | Update a default preset returns an error span | HTTP | tests/http/prompt_presets.rs:469 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.18 | Delete a preset → empty body | HTTP | tests/http/prompt_presets.rs:490 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.19 | Delete a nonexistent preset returns an error span | HTTP | tests/http/prompt_presets.rs:518 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.20 | Delete a default preset returns an error span | HTTP | tests/http/prompt_presets.rs:533 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.21 | Duplicate a preset → panel re-renders with the copy | HTTP | tests/http/prompt_presets.rs:551 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.22 | Duplicate a nonexistent preset returns an error span | HTTP | tests/http/prompt_presets.rs:580 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.23 | Activate a system preset → panel re-renders with an Active badge | HTTP | tests/http/prompt_presets.rs:597 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.25 | Activate an Interactive Fiction-only preset for the IF bundle | INTERNAL (soft: 'the stored X is Y' verified via direct app-state/storage read, not a follow-up GET) | tests/http/prompt_presets.rs:690 | 1 (HTTP) | STRATEGY.md tier-1 purpose ('server response a curl would see') violated by covering test's direct state read — see Finding 1 addendum |
| prompt_presets.md | 21.26 | Panel gates activation buttons by allowed_modes | HTTP | tests/http/prompt_presets.rs:654; tests/http/prompt_presets.rs:754 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.24 | Activate a nonexistent preset returns an error span | HTTP | tests/http/prompt_presets.rs:637 | 1 (HTTP) | OK (tier 1) |
| prompt_presets.md | 21.27 | Duplicate → edit-form flags → save toggles per-mode activation | INTERNAL | tests/http/prompt_presets.rs:798 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| reset.md | 7.1 | Reset clears the previous game's story-log history | INTERNAL | tests/http/reset.rs:8 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| reset.md | 7.2 | Action after reset produces a fresh input | INTERNAL | tests/http/reset.rs:44 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| retrigger.md | 13.1 | Retrigger creates a new event narration message and does not roll back state | INTERNAL | tests/http/retrigger.rs:21 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| retrigger.md | 13.2 | Retrigger does not re-run the quantifier | INTERNAL | tests/http/retrigger.rs:67 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| retrigger.md | 14.1 | Retrigger with no trigger context returns 400 | INTERNAL | tests/http/retrigger.rs:110 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| retrigger.md | 14.2 | Retrigger with no messages returns 400 | HTTP | tests/http/retrigger.rs:129 | 1 (HTTP) | OK (tier 1) |
| retrigger.md | 14.3 | Retrigger when the last message is not a narration returns 400 | INTERNAL | tests/http/retrigger.rs:150 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| retrigger.md | 14.4 | Retrigger when the last message is an event continuation returns 400 | INTERNAL | tests/http/retrigger.rs:172 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| retrigger.md | 14.5 | Retrigger trigger narration failure sets Error | INTERNAL | tests/http/retrigger.rs:207 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| retrigger.md | 14.6 | Retrigger with no game context returns 400 | HTTP | tests/http/retrigger.rs:259 | 1 (HTTP) | OK (tier 1) |
| retrigger.md | 15.1 | Retrigger while a generation is already in flight returns "Still thinking..." | INTERNAL | tests/http/retrigger.rs:284 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| settings.md | 20.1 | Settings panel renders the full surface | HTTP | tests/http/settings.rs:58 | 1 (HTTP) | OK (tier 1) |
| settings.md | 20.2 | POST /settings switches the narrator connection | HTTP | tests/http/settings.rs:90 | 1 (HTTP) | OK (tier 1) |
| settings.md | 20.3 | POST /settings switches the quantifier connection | HTTP | tests/http/settings.rs:107 | 1 (HTTP) | OK (tier 1) |
| settings.md | 20.4 | POST /settings switches both connections | HTTP | tests/http/settings.rs:124 | 1 (HTTP) | OK (tier 1) |
| settings.md | 20.5 | POST /settings rejects a connection id that is not in the connections list | HTTP | tests/http/settings.rs:141 | 1 (HTTP) | OK (tier 1) |
| settings.md | 20.6 | POST /settings with a missing required field returns 422 | HTTP | tests/http/settings.rs:162 | 1 (HTTP) | OK (tier 1) |
| settings.md | 20.7 | POST /settings reports a save failure in the response body | HTTP | tests/http/settings.rs:174 | 1 (HTTP) | OK (tier 1) |
| settings.md | 20.8 | Switching the narrator takes effect on the next request | HTTP | tests/http/settings.rs:196 | 1 (HTTP) | OK (tier 1) |
| story_log.md | 8.1 | Delete-last between actions — deleted narration stays absent | INTERNAL | tests/http/story_log.rs:10 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| story_log.md | 8.2 | Delete mid-sequence removes the targeted narration | INTERNAL | tests/http/story_log.rs:55 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| story_log.md | 8.3 | Retry after delete of last input does not leave state generating | INTERNAL | tests/http/story_log.rs:95 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| story_log.md | 8.4 | Delete-last removes the rendered log entry | INTERNAL | tests/http/story_log.rs:121 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| story_log.md | 8.5 | Story-log fragment declares no log container | HTTP | tests/http/story_log.rs:150 | 1 (HTTP) | OK (tier 1) |
| swipe_new.md | 9.1 | Main retry replaces the narration with a new swipe | INTERNAL | tests/http/swipe_new.rs:25 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 9.2 | Main retry re-runs the quantifier and can move the player | INTERNAL | tests/http/swipe_new.rs:60 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 9.3 | Main retry preserves the input message | INTERNAL | tests/http/swipe_new.rs:101 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 9.4 | Main retry uses the edited input text | INTERNAL | tests/http/swipe_new.rs:122 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 9.5 | Main retry re-evaluates triggers | INTERNAL | tests/http/swipe_new.rs:170 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 9.6 | Main retry completes when the quantifier returns no movement | INTERNAL | tests/http/swipe_new.rs:241 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 10.1 | Event retry replaces the event narration with a new swipe | INTERNAL | tests/http/swipe_new.rs:276 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 10.2 | Event retry does not re-run the quantifier | INTERNAL | tests/http/swipe_new.rs:333 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 11.1 | Retry with no input to retry returns 400 | HTTP | tests/http/swipe_new.rs:379 | 1 (HTTP) | OK (tier 1) |
| swipe_new.md | 11.2 | Retry with no game context returns 400 | HTTP | tests/http/swipe_new.rs:398 | 1 (HTTP) | OK (tier 1) |
| swipe_new.md | 11.3 | Retry when the anchor message has no snapshot returns 500 | INTERNAL | tests/http/swipe_new.rs:423 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 11.4 | Retry when the anchor snapshot was deleted returns 500 | INTERNAL | tests/http/swipe_new.rs:453 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 11.5 | Retry LLM failure sets Error | INTERNAL | tests/http/swipe_new.rs:484 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 11.6 | Retry empty narration sets Error | INTERNAL | tests/http/swipe_new.rs:538 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 11.7 | Retry room not found sets Error | INTERNAL | tests/http/swipe_new.rs:594 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 11.8 | Event retry trigger narration failure sets Error and preserves main narration | INTERNAL | tests/http/swipe_new.rs:661 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 12.1 | Retry while a generation is already in flight returns "Still thinking..." | INTERNAL | tests/http/swipe_new.rs:713 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 22.1 | Re-impersonate retry generates a new Input swipe | INTERNAL | tests/http/swipe_new.rs:747 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 22.2 | User-regen retry generates a new Input swipe | INTERNAL | tests/http/swipe_new.rs:805 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 22.3 | Re-impersonate retry preserves the stored inputs | INTERNAL | tests/http/swipe_new.rs:865 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| swipe_new.md | 22.4 | Re-impersonate redo uses the impersonate preset's current content | INTERNAL | tests/http/swipe_new.rs:906 | 1 (HTTP) | STRATEGY.md carve-out (Internal state/Cancellation/Mid-flight/Call-seq) says unit tier; scenario lives at HTTP tier instead |
| worlds.md | 25.1 | A post omitting all posture fields preserves the stored posture | INTERNAL (soft: 'the stored X is Y' verified via direct app-state/storage read, not a follow-up GET) | tests/http/worlds.rs:56 | 1 (HTTP) | STRATEGY.md tier-1 purpose ('server response a curl would see') violated by covering test's direct state read — see Finding 1 addendum |
| worlds.md | 25.2 | A partial posture post merges per field | INTERNAL (soft: 'the stored X is Y' verified via direct app-state/storage read, not a follow-up GET) | tests/http/worlds.rs:76 | 1 (HTTP) | STRATEGY.md tier-1 purpose ('server response a curl would see') violated by covering test's direct state read — see Finding 1 addendum |
| worlds.md | 25.3 | An unknown posture value falls back to the domain default | INTERNAL (soft: 'the stored X is Y' verified via direct app-state/storage read, not a follow-up GET) | tests/http/worlds.rs:95 | 1 (HTTP) | STRATEGY.md tier-1 purpose ('server response a curl would see') violated by covering test's direct state read — see Finding 1 addendum |
| worlds.md | 25.4 | The options toggle obeys the checkbox grammar | INTERNAL (soft: 'the stored X is Y' verified via direct app-state/storage read, not a follow-up GET) | tests/http/worlds.rs:221 | 1 (HTTP) | STRATEGY.md tier-1 purpose ('server response a curl would see') violated by covering test's direct state read — see Finding 1 addendum |
| worlds.md | 25.5 | The posture endpoint patches and reports per outcome | INTERNAL (soft: 'the stored X is Y' verified via direct app-state/storage read, not a follow-up GET) | tests/http/worlds.rs:119; tests/http/worlds.rs:144; tests/http/worlds.rs:172; tests/http/worlds.rs:193 | 1 (HTTP) | STRATEGY.md tier-1 purpose ('server response a curl would see') violated by covering test's direct state read — see Finding 1 addendum |
| worlds.md | 25.6 | The world edit form renders the posture selects with the stored values | HTTP | tests/http/worlds.rs:253 | 1 (HTTP) | OK (tier 1) |

## Findings

### Finding 1 — HTTP-tier specs written at the internal-service level, contradicting STRATEGY.md's own carve-out (58 of 146 scenarios)

STRATEGY.md states the tier-1 purpose is "the server response a `curl` would
see" (`tests/STRATEGY.md:12`) and that "Specs (`docs/specs/`) are the
behavioural authority" whose Thens should be observable through the real
driving adapter (`tests/STRATEGY.md:22-23`). It then carves out exactly the
opposite content into the unit tier:

> `tests/STRATEGY.md:28-34`
> Scenarios whose Givens or Thens touch seams that only exist in-process live
> at the unit or driven-adapter tier:
> - **Cancellation** — needs `CancellationToken` → unit
> - **Internal state** (e.g. `last_trigger`, phase transitions) — assert on
>   `GameState` fields → unit
> - **Mid-flight observation** — needs sync flags → unit
> - **Call sequencing** — direct call-count assertion → unit

`docs/specs/retrigger.md` uses `narrative.last_trigger` — the exact example
STRATEGY.md names — as a Given in five of its seven scenarios (13.1, 13.2,
14.1, 14.3, 14.4, 14.5), and as internal state in the Then of 13.1, 13.2, 14.5.
These are not edge-case scenarios; they are the spec's entire Given clause
vocabulary. Yet the scenarios live in `docs/specs/retrigger.md` (a spec, not
`src/`), get a `SCENARIO:` tag, and are covered by `tests/http/retrigger.rs`
(tier 1) — the tier STRATEGY.md says internal-state scenarios should *not*
occupy.

The same pattern repeats at scale:

- `docs/specs/actions.md`: 18 of 19 scenarios assert via
  `message_service.load_messages()` / `.load_or_fresh().narrative.input_buffer.status`
  / `GenerationStatus::Error(msg)` — e.g. `docs/specs/actions.md:17`,
  `docs/specs/actions.md:71-73` (`scene.npcs_in_area`,
  `npc_encounter_log.npcs["gabriella"].times_met`). Scenario 3.3
  (`docs/specs/actions.md:162`) is literally STRATEGY's "Mid-flight
  observation" carve-out ("within 2 s, ...`is_generating()` is false").
- `docs/specs/swipe_new.md`: 20 of 24 scenarios, same pattern
  (`docs/specs/swipe_new.md:18-21`, `:33`, `:65`, `:101`, etc.).
- `docs/specs/reset.md`, `docs/specs/story_log.md` (8.1-8.3 only; 8.4-8.5 are
  clean HTTP/fragment assertions): same pattern.
- `docs/specs/options.md` 24.13 (`docs/specs/options.md:59`), `games.md` 20.7
  (`game_catalogue.current_game()`, `docs/specs/games.md:127`),
  `prompt_presets.md` 21.27 ("storage reports the copy's allowed_modes",
  `docs/specs/prompt_presets.md:334`): isolated internal Thens inside
  otherwise-clean specs.

Confirmed at the test-code level, not just the spec-prose level: the covering
tests reach into the wired `AppState` directly rather than reading an HTTP
response. Example, `tests/http/actions.rs:32`:
```
let messages = state.message_service.load_messages().unwrap();
```
`tests/http/worlds.rs` defines a helper `updated_world()` that calls
`state.world_catalogue.get_world(...)` directly (`tests/http/worlds.rs:47-52`)
rather than issuing a second HTTP request to observe the stored value.
`tests/http/narrator_mode.rs:31-36` reads `storage.list_latest_llm_messages(50)`
directly for "the narrator's recorded system prompt" Thens (this one is a
**documented** exception — `tests/AGENTS.md` "Seam recipes" names exactly this
pattern: "Assert what the narrator was sent → storage-backed recorder ... +
`Storage::list_latest_llm_messages`" — so it's not a leak, it's a blessed seam
without a curl-visible substitute). The `message_service.*` and
`*_catalogue.*` direct-state reads have no equivalent blessing anywhere in
`tests/STRATEGY.md` or `tests/AGENTS.md`.

**Why this matters**: nothing catches this. `scripts/validate_feature_spec.py`
checks scenario-to-test mapping and tag-directory consistency, not what the
Given/Then prose or the test assertions actually observe. The gate is green
(146/146 covered, 0 mismatches) while roughly 40% of the scenario corpus
violates the placement rule STRATEGY.md itself states two paragraphs earlier.

### Finding 1 addendum — the same leak, softer phrasing (worlds.md, prompt_presets.md)

`docs/specs/worlds.md` scenarios 25.1-25.5 and `docs/specs/prompt_presets.md`
21.16 and 21.25 phrase their Thens as plain domain state ("the stored world's
tense is 'present'", `docs/specs/worlds.md:25`; "the stored preset still has
preset_type 'system'", `docs/specs/prompt_presets.md:208`; "the Interactive
Fiction bundle's system slot holds that preset's id",
`docs/specs/prompt_presets.md:300`) rather than naming a code identifier. On
the page, these read like ordinary HTTP-observable assertions. But no
follow-up GET request appears anywhere in these scenarios' Given/When/Then —
the only way to check a "stored" value without a second HTTP round-trip is to
read it out of the running process. Confirmed in the covering tests:
`tests/http/worlds.rs:47-52` defines `updated_world()` as
`state.world_catalogue.get_world("posture_world")` and every one of 25.1-25.5
calls it instead of issuing `GET /worlds/posture_world/edit` and parsing the
selected option (which 25.6 *does* do, correctly, and is clean HTTP).
`tests/http/prompt_presets.rs:456-461` (21.16) reads
`app_state.prompt_preset_service.get_preset(&preset_id)` directly;
`tests/http/prompt_presets.rs:713-732` (21.25) reads
`storage.get_settings()` directly. All three are the same class of leak as
Finding 1's hard cases, just without a matching internal-identifier grep hit
— which is itself evidence the grep-based part of this audit undercounts.

### Finding 2 — Contradiction inside STRATEGY.md itself

Two statements in the same document are in tension:

> `tests/STRATEGY.md:22-23` — "Specs (`docs/specs/`) are the behavioural
> authority. Every spec scenario maps to at least one HTTP E2E test that
> validates it end-to-end through the real driving adapter."

versus

> `tests/STRATEGY.md:28-34` — "Scenarios whose Givens or Thens touch seams
> that only exist in-process live at the unit or driven-adapter tier: ...
> Internal state (e.g. `last_trigger`, phase transitions) ... Mid-flight
> observation ... Call sequencing."

The first statement treats "spec scenario" and "HTTP E2E scenario" as
synonyms — every spec scenario gets an HTTP test. The second statement
describes content that, by definition, cannot be a spec scenario at all under
`docs/AGENTS.md`'s framing ("The specs describe the behaviour a client
observes through HTTP and browser interactions") — internal-state content
isn't client-observable, so it shouldn't be phrased as a spec Given/Then in
the first place, whether or not it later gets a unit test.

STRATEGY.md never resolves which statement wins when a behaviour is both
"a failure mode that must be spec-complete" (line 36-38, "Spec completeness is
mandatory... every failure mode, every edge case") and "internal-state-only".
In practice the repo resolved it by writing the internal-state content as spec
scenarios anyway and giving the HTTP test direct access to `AppState`
internals — a third option STRATEGY.md never describes or licenses.

A second, smaller tension: `tests/STRATEGY.md:12`'s tier-1 purpose column says
tier 1 asserts "the server response a `curl` would see," but the "Placement
rule" section frames rule 1 ("Could `curl` observe this?") as scoped only to
choosing *among the three UI tiers* (`tests/STRATEGY.md:64` — "This section is
the decision of record" for "which of the three UI tiers"). It's silent on
whether curl-observability is also the bar for what a tier-1 test's
*assertions* may touch, versus merely which port is faked. The repo's own
tests answer this silently, and inconsistently: `narrator_mode.rs` uses a
blessed non-curl seam (storage-backed recorder); `actions.rs`/`swipe_new.rs`
use an unblessed one (`message_service` direct access).

### Finding 3 — Numbering scheme

Prefixes are sequential feature IDs assigned in the order features were
speced, not semantically tied to endpoint groups. No file documents the
mapping; it's reconstructable only by reading all specs:

| Prefix | File |
|---|---|
| 1, 2, 3, 6 | actions.md |
| 7 | reset.md |
| 8 | story_log.md |
| 9, 10, 11, 12, 22 | swipe_new.md |
| 13, 14, 15 | retrigger.md |
| 16 | browser_dashboard.md |
| 17, 18, 19, 20 | games.md |
| 20 | settings.md |
| 21 | prompt_presets.md |
| 23 | narrator_mode.md |
| 24 | options.md |
| 25 | worlds.md |
| 26 | browser_options.md |
| 27 | browser_games.md |
| 28 | browser_prompt_presets.md |
| 29 | browser_worlds.md |
| 30 | browser_story_log.md |
| 31 | browser_slash_menu.md |

Note the collision: prefix **20** is shared by `games.md` (20.4-20.8, per-game
posture/mode/presets) and `settings.md` (20.1-20.8, the settings panel) — two
entirely unrelated features carry the same feature number. This is harmless
for tooling (`validate_feature_spec.py` keys coverage by
`(spec_path, scenario_id)` pairs, not bare IDs — confirmed in
`scripts/validate_feature_spec.py`'s "Coverage is keyed by (spec_path,
scenario_id)" comment), but it defeats the scheme's apparent purpose (a human
skimming "scenario 20.5" across two files can't tell which feature it names
without the file path attached). `swipe_new.md` and `actions.md` also
interleave prefixes across sections within one file (actions.md has 1, 2, 3,
6; swipe_new.md has 9, 10, 11, 12, then jumps to 22) — the numbering tracks
some external feature-planning order the specs directory itself doesn't
record.

### Finding 4 — `Endpoint:`/`Endpoints:` header accuracy

All 18 files carry the header (`docs/specs/*.md:3`). Spot-checked against
each file's actual scenario surface:

- Single-endpoint files (`actions.md`, `reset.md`, `retrigger.md`,
  `story_log.md`, `swipe_new.md`) — header matches; every scenario exercises
  that one endpoint.
- `games.md` lists three endpoints (`POST /games`, `/:id/switch`,
  `/:id/delete`) but scenario 20.8 exercises `GET /fragment/games`, and
  20.4-20.7 exercise `POST /games/:id/posture`, `/presets`, `/mode` — none of
  which appear in the header list. The header is stale relative to the file's
  "Per-game posture, mode, and presets" section, which was evidently added
  after the header was written.
- `narrator_mode.md` lists `POST /games`, `POST /games/:id/posture`,
  `POST /games/:id/mode`, `POST /action` — accurate; posture is exercised only
  indirectly (through creation/mode-switch inheritance) but the endpoint is
  real.
- `options.md` lists `POST /action`, `GET /fragment/options-dock` — accurate.
- `prompt_presets.md`, `settings.md`, `worlds.md` — accurate, all scenario
  endpoints appear in the header list.
- `browser_*.md` files all say `Endpoint: browser DOM.` — a placeholder, not a
  real endpoint list; consistent across all seven browser specs, not
  file-specific, so it can't be "inaccurate" the way `games.md`'s is, but it
  also carries no information the format elsewhere is used for.

### Finding 5 — What `validate_feature_spec.py` enforces vs what STRATEGY.md claims

The script's own docstring is honest and matches its code exactly (verified
by reading `scripts/validate_feature_spec.py` in full):

1. Every declared scenario (`#### Scenario N.N:` heading) has >= 1 covering
   `// [spec] SCENARIO: N.N` tag; every tag references a declared scenario
   (gap/orphan check, `main()`).
2. Surface consistency: `browser_*.md` specs tagged only from
   `tests/browser/`; non-`browser_*` specs never tagged from `tests/browser/`
   (`find_surface_violations`).
3. Every `#[test]`/`#[tokio::test]` under `tests/http/` and `tests/browser/`
   carries a tag, unless exempted by `TAG_EXEMPT_DIRS` (only
   `tests/http/requires_migration/`) or `TAG_EXEMPT_FILES` (only
   `tests/browser/stub/invariants.rs`) (`find_untagged_tests`).
4. `tests/http/requires_migration/` quarantine count may not exceed
   `REQUIRES_MIGRATION_TEST_COUNT = 85` (`count_quarantine_tests`,
   currently exactly at the pin: 85/85 per this run's output).

What it does **not** enforce, despite STRATEGY.md's prose implying a
behavioural contract:

- **Whether a scenario's Given/Then is actually curl/DOM-observable** — the
  script only checks that a heading exists and a tag references it; it never
  reads scenario body text. This is why Finding 1's 58 internal-surfaced
  scenarios pass cleanly.
- **Whether the covering test's assertions match the tier's "what's faked"
  definition** — the script never reads test bodies past the tag/attribute
  line; it has no way to detect `state.message_service.load_messages()`
  inside a `tests/http/` test.
- **Whether a scenario belongs at unit tier per the four STRATEGY.md
  carve-outs** (Cancellation/Internal state/Mid-flight/Call sequencing) — the
  script has no unit-tier awareness at all; it scans only `tests/http/` and
  `tests/browser/` (`TEST_DIRS`), and STRATEGY.md itself says unit-tier tests
  carry no tags, so there's structurally no way for this script to ever flag
  "this should have been a unit test."
- **Endpoint-header accuracy** (Finding 4) — not a scenario-coverage concern,
  never touched.

So the two claims in `tests/STRATEGY.md`'s "SCENARIO tags" section — "Every
declared spec scenario has at least one covering test" and "Surface
consistency..." — are exactly and only what the script checks. STRATEGY.md
does not itself claim the script enforces content-correctness (rule ordering,
mechanism leaks); that's implicitly left to review, per
`tests/STRATEGY.md`'s closing line under "networkidle": "No mechanism
enforces the ban; review catches it" — the same posture applies here, just
undeclared.

**Quarantine pin**: `REQUIRES_MIGRATION_TEST_COUNT = 85`
(`scripts/validate_feature_spec.py`, top-level constant). Current actual count
also 85 (this run's validator output: `quarantine 85/85`) — at the pin, not
under it, meaning any new untagged test anywhere in
`tests/http/requires_migration/` fails the gate immediately; no slack
remains.

## Top 5 findings (file:line)

1. `tests/STRATEGY.md:32` names `narrative.last_trigger` as the canonical
   unit-tier example, yet `docs/specs/retrigger.md:12` (`Given a game state
   where narrative.last_trigger is set`) uses that exact field as a spec
   Given, covered by `tests/http/retrigger.rs:21` (tier 1) — the spec
   contradicts STRATEGY's own worked example.
2. `tests/http/actions.rs:32` (`state.message_service.load_messages()`) and
   17 sibling assertions in the same file access `AppState` internals
   directly rather than an HTTP response — the tier-1 purpose statement at
   `tests/STRATEGY.md:12` ("the server response a `curl` would see") is
   violated by the majority of `actions.md`'s covering tests.
3. `tests/STRATEGY.md:22-23` ("every spec scenario maps to... HTTP E2E") vs
   `tests/STRATEGY.md:28-34` (internal-state scenarios belong at unit tier) —
   an unresolved contradiction the repo answers ad hoc, differently per file
   (blessed seam in `tests/http/narrator_mode.rs:31-36` vs unblessed
   `state.message_service`/`state.world_catalogue` direct access elsewhere).
4. `docs/specs/games.md:3-6` lists three endpoints but omits
   `GET /fragment/games` (scenario 20.8, `docs/specs/games.md:128`) and
   `POST /games/:id/posture|presets|mode` (scenarios 20.4-20.7,
   `docs/specs/games.md:86-126`) — the header predates the "Per-game posture,
   mode, and presets" section.
5. `scripts/validate_feature_spec.py`'s coverage check is purely structural
   (heading exists, tag references it) — it cannot and does not detect
   Finding 1 or Finding 2; the gate reports clean (`146 declared, 146
   covered, 0 gap(s), 0 orphan(s), 0 untagged, 0 surface mismatch(es)`) while
   ~40% of scenarios violate STRATEGY.md's stated tier-1 purpose.
