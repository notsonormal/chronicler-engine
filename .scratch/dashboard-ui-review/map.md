# Map: dashboard UI review

Labels: wayfinder:map

## Destination

The dashboard has no open finding from the [UI review](review-2026-09-29.md). Each finding is fixed, or ruled out of scope with a reason. This includes new findings from the areas the first review skipped. The error and health display is redesigned (Theme 1). The [final re-review](issues/24-final-re-review.md) finds no new P1/P2, and `python build.py` passes.

The test rules the map's tickets rely on hold: `tests/STRATEGY.md` states one checkable rule for what a tier-1 test may observe, and the test standards docs match the code. This was added after the [test audit](assets/test-audit/), because this map's tickets write specs and tests by those rules.

## Notes

- **This map carries execution.** It overrides wayfinder's "plan, don't do": `task` tickets make the change, not only decide it. Decision tickets (`grilling`, `prototype`) graduate their implementation work into new tickets.
- **Domain:** the dashboard frontend. That means `assets/index.html`, `assets/styles.css`, and the Askama templates and handlers under `src/adapters/driving/http/`. Backend code changes only where a finding needs it.
- **Source of findings:** [review-2026-09-29.md](review-2026-09-29.md). Tickets cite findings by number (for example "finding 2.3") and screenshots by number (`tmp/ui-review/NN-*.png`, local only).
- **Tests:**
  - Place every test by the placement rule in `tests/STRATEGY.md`. The ticket answer names the tier.
  - Specs describe the system, not changes. Add or change a spec scenario only when the spec is incomplete or wrong.
  - Pure CSS and copy fixes need no test. Do update existing assertions that the fix breaks.
  - Write new spec Givens and Thens in `CONTEXT.md` terms, or as something a client sees in a response. A tier-1 test asserts **domain outcomes**: the HTTP response, or stored state read through a read seam (an HTTP GET or an application read service/port). It never reads a `GameState` field or a storage row. The rule is the "Domain outcome" section of `tests/STRATEGY.md`.
  - The test-rule tickets (31–34) can run apart from the dashboard tickets. They share `tests/` and `docs/specs/`, so check `git status` for overlapping edits first.
- **End of each execution ticket:** `python build.py` is green, then the user reviews the diff. After approval, commit through `/commit-and-push`. One ticket = one session = one commit.
- **Skills:**
  - grilling and prototype tickets: `/grilling`, `/domain-modeling`, `/prototype`
  - review tickets and visual checks: `/chronicler-ui-investigator`
- **Reference docs:** `docs/diataxis/reference/frontend/ui_design.md` (design tokens), `docs/diataxis/reference/frontend/dashboard.md`, `docs/diataxis/explanation/dashboard_design.md`.
- **User data:** review tickets run against the user's real `data/` and settings. Use throwaway games, worlds, and presets, and restore any changed setting before the session ends.
- **Final re-review:** every ticket added to this map must also be added to the `Blocked by:` line of [Final re-review of the dashboard](issues/24-final-re-review.md).
- **Small review follow-ups:** issues from code reviews or implementers that don't block a merge go under `## Items` in [Follow up on small issues found during review, round 2](issues/37-follow-up-small-review-issues-2.md), with their source ticket. Bigger ones get their own ticket.
- Other agents may work in the repo at the same time. Do not touch unrelated changes.

## Decisions so far

<!-- the index — one line per closed ticket -->

- [05](issues/05-review-create-save-delete.md) — review of create/save/delete flows; report at `tmp/ui-review/t05-report.md`; findings F1–F8 graduate to 25–29 and extend 08/16.

01 [Stop the story-log fragment nesting a second #story-log](issues/01-fix-nested-story-log.md): resolved — the narrative-log fragment no longer ships the `#story-log` wrapper (shell keeps sole ownership); tier 1 `test_story_log_fragment_declares_no_log_container` (scenario 8.5) + aligned stub fixture; commit 60d1ef65.

02 [Rebind action-area handles after swaps](issues/02-rebind-action-area-handles.md): resolved — use-time `#submit-btn` lookup + body-level MutationObserver delegation; tier 2 stub tests 16.9/16.10 (`tests/browser/stub/dashboard.rs`); commit 262a4857.

29 [Stop world creation from silently overwriting an existing key](issues/29-world-key-silent-overwrite.md): resolved — user create refuses an existing key (`Storage::create_world` → `WorldAlreadyExists` → 400 toast); bootstrap `seed_world` still replaces, InMemory now matches; tier 1 scenario 25.7 + storage pair tests; graduated [35](issues/35-sqlite-reseed-keeps-world-id.md).

03 [Keep the command form when a text-check result shows](issues/03-keep-command-form-on-text-check.md): resolved — ✓ results render into their own `#text-check-result`, never replacing the command form; tier 2 scenario 16.11; 16.9/16.10 rewritten to drive submit → preview → confirm.

30 [Stop the story-log delete test flaking under parallel load](issues/30-fix-flaky-story-log-delete-test.md): resolved — `wait_idle` saw persisted Idle before the generation slot was released, so follow-up actions were dropped as `ConcurrentGeneration`; the helper now also waits for the slot (all 87 call sites).

33 [Make the test standards docs match the code](issues/33-fix-test-standards-drift.md): resolved — ~30 phantom names and false claims fixed across the three test standards docs; `integration_test_standards.md` pruned 352→250 lines to pointers plus the rationale the code cannot state; guardrail message names `TestAppBuilder`.

27 [Recover from a failed message save instead of freezing the story log](issues/27-recover-failed-message-save.md): resolved — failed save/swipe/retrigger show the toast, revert the entry or status, and always resume polling; tier 2 scenarios 30.4/30.5; restoring entry actions on revert handed to 11.

35 [Keep the world id stable when SQLite re-seeds a world](issues/35-sqlite-reseed-keeps-world-id.md): resolved — SQLite re-seed now upserts in place (`ON CONFLICT(key) DO UPDATE`), keeping the world id, map and characters; `create_world` refusal now enforced by the `worlds.key UNIQUE` constraint.

17 [Collapse the Prompt Presets add forms](issues/17-collapse-preset-add-forms.md): resolved — each Add form is a closed `<details>` disclosure (no JS); tier 1 scenario 21.28; failed-add error display left to 25.

11 [Fix edit mode: size, focus, keys and locked controls](issues/11-fix-edit-mode.md): resolved — auto-growing textarea capped at 50vh, autofocus, Escape/Cmd+Enter, entry controls locked while editing and restored by every revert; tier 2 scenarios 30.6–30.9.

22 [Show Character names in the visual sidebar](issues/22-show-npc-names.md): resolved — each portrait shows its Character name in the reinstated `.image-label` (escaped, ellipsised); tier 1 scenarios 32.1/32.2 in a new `visual_sidebar.md` spec.

20 [Copy sweep](issues/20-copy-sweep.md): resolved — type names, plurals, jargon and unlabelled selects fixed per findings 5.1, 5.2, 5.4, 5.5; tier 1 scenario 25.8 plus extended 20.8.

28 [Decide whether duplicate connection and preset names are allowed](issues/28-decide-duplicate-names.md): resolved — rejected on add (trimmed, case-insensitive; connections global, presets per category), edits keep their own name, Duplicate picks a free "(Copy N)"; tier 1 scenarios 20.9–20.11, 21.29–21.34.

36 [Follow up on small issues found during review](issues/36-follow-up-small-review-issues.md): resolved — 21 of 28 items fixed or closed, 7 dropped with reasons; leftovers now in [round 2](issues/37-follow-up-small-review-issues-2.md), and the edit-another-entry freeze is [its own ticket](issues/38-edit-another-entry-freezes-poll.md).

34 [Fix the weak dashboard and settings tests](issues/34-fix-weak-dashboard-and-settings-tests.md): resolved — 16.5, 16.7/16.8, 16.10 (B2) and 20.2–20.4 now fail on the regression they name, each proven by a temporary break; `StubActionOutcome::Idle` deleted.

37 [Follow up on small issues found during review, round 2](issues/37-follow-up-small-review-issues-2.md): resolved — 17 of 20 items fixed, the 16.9 acknowledgement flake explained and fixed, 3 deferred or dropped with reasons; judgement leftovers listed in its answer.

09 [Decide whether failed LLM calls appear in LLM Messages](issues/09-decide-failed-llm-calls.md): resolved — every LLM attempt is recorded, success or failure; the failure marker is the existing nullable `error_message`; the error text is stored and shown verbatim; no attempt number, no degraded-outcome row, no duration or tokens; graduated [Record failed LLM attempts and show them in LLM Messages](issues/41-record-failed-llm-attempts.md).

04 [Review the text-check-enabled flow](issues/04-review-text-check-enabled.md): resolved — 8 findings, one P1: a confirm through the send preview leaves the command input permanently disabled while the status reads "Ready". Graduated [Rework the action area so a text check cannot strand it](issues/42-rework-the-action-area.md); the ✓-while-disabled half of C5 joins 15, the tofu ✍ joins 13.

06 [Review swipes, the options dock and the Thinking states](issues/06-review-swipes-options-thinking.md): resolved — 6 findings, one P1: switching a swipe restores the snapshot written mid-narration, so `/status/generating` answers `narrating` for good and the dashboard sticks on "Generating narration..." with no in-page way out. Graduated [Reset the generation status and the options dock when a swipe switch restores a snapshot](issues/43-reset-status-and-dock-on-swipe-switch.md); the Stop-label finding went into [Rework the action area so a text check cannot strand it](issues/42-rework-the-action-area.md), and the ▶ glyph instance joins 13. Thinking/generating states are now captured with a slow-stub connection (`tmp/ui-review/t06-llm-stub.py`).

07 [Review keyboard use and screen-reader output](issues/07-review-keyboard-screen-reader.md): resolved — 11 findings, two P1: the 2s story-log poll ejects keyboard focus to `<body>`, and LLM Messages rows are mouse-only (`div onclick`, zero focusable elements). Graduated [Make LLM Messages rows keyboard-operable](issues/44-keyboard-operable-llm-messages.md), [Keep DOM state and focus through in-place swaps](issues/65-keep-dom-state-through-swaps.md), [Expose the dashboard to assistive technology](issues/46-expose-dashboard-to-assistive-technology.md) and [Announce dynamic state changes to assistive technology](issues/47-announce-state-changes-to-at.md) (blocked by 08); the story-log focus findings K1/K7 joined [Decide how the story-log poll keeps DOM state](issues/10-decide-story-log-poll-swap.md).

10 [Decide how the story-log poll keeps DOM state](issues/10-decide-story-log-poll-swap.md): resolved — morph the poll swap (vendored idiomorph) instead of a server no-swap or a selection pause. Keeps selection, hover, focus and scroll on idle polls; closes the switchSwipe hole for free; no client-side hash to maintain. K7 joins as `tabindex="0"` on `#story-log`. Tier 2 tests. Graduated [Keep DOM state and focus through in-place swaps](issues/65-keep-dom-state-through-swaps.md).

42 [Rework the action area so a text check cannot strand it](issues/42-rework-the-action-area.md): resolved — the action area is a static shell driven by one client state machine (`checking → generating → preview → idle`); confirm retargets `#status-display` instead of swapping a disabled input back in; `/check-text` renders a read-only result; the preview lives in its own `#action-preview`; focus follows the preview; the primary button is a disabled "Generating…" indicator, not a fake "Stop"; no cancel route built. 16.9–16.15, tier-3 proof 16.12.

41 [Record failed LLM attempts and show them in LLM Messages](issues/41-record-failed-llm-attempts.md): resolved — every LLM attempt persists one row (success or failure); the failure text is stored verbatim in `error_message` and rendered in the panel; the row names its agent and backend/model; new `docs/specs/llm_messages.md` 33.1/33.2 with tier-1 HTTP tests.

39 [Enforce preset name uniqueness in storage](issues/39-storage-level-preset-name-uniqueness.md): resolved — `Storage::save_preset` checks and writes under one backend lock (no TOCTOU); the service delegates to it; no `UNIQUE` constraint, so legacy duplicate-holding DBs still open; predicate moved to `domain/model/utils` to satisfy `arch-lint.toml`; driven-adapter InMemory/SQLite pair + legacy-duplicates test.

40 [Drive the retrigger control in a browser test](issues/40-drive-retrigger-in-a-browser-test.md): resolved — worth a tier-2 scenario; 30.10 drives the shipped `submitRetrigger` against the fixture's real control and the stub's counted `/retrigger` → 500 route, proving the URL and the shared recovery. (The earlier planted-control/fetch-intercept version was replaced during the review follow-up.)

44 [Make LLM Messages rows keyboard-operable](issues/44-keyboard-operable-llm-messages.md): resolved — each row header is a real `<button aria-expanded>` with `aria-controls` the body, so Enter/Space use native activation and the state is exposed to AT; the restore hook moved to `hx-on::after-settle` after a swap race left the class on a replaced node, and a stable header id keeps focus through the 4s poll; tier-2 35.1–35.3 in a new `browser_llm_messages.md`.

46 [Expose the dashboard to assistive technology](issues/46-expose-dashboard-to-assistive-technology.md): resolved — skip link plus `<main>` landmark; tablist/tab/tabpanel with `aria-selected`/`aria-controls`; the slash menu is a listbox/option pattern with the input a combobox tracking `aria-activedescendant` (the `.active` highlight is mirrored, not replaced); a real label names the command input; a forced-colors `:focus-visible` ring restores what `outline: none` removed; tier-2 16.16/16.17 and extended 31.1/31.3.

08 [Decide how the dashboard shows each kind of failure](issues/08-decide-failure-display.md): resolved — quiet by default. The hardcoded "Connected" is deleted; a banner under the header appears only while a role's newest LLM attempt has failed or the server is unreachable (the latter client-owned), with per-role detail behind a popover and on-demand health in Settings. Four always-on per-role chips were mocked, measured at ~310px, and rejected. The toast retires in sequence after the banner, the inline slots and the clamped status display exist; a failure never swaps into the region it describes; error text is a short message plus an anchored popover. Graduated [Redesign the error and health display](issues/63-redesign-error-health-display.md) and [54](issues/54-retire-toast-and-route-callers.md).

13 [Decide the icon-button approach](issues/13-decide-icon-buttons.md): resolved — one `<symbol>` sprite in the shell with `<use>` references tinted by `currentColor`, paths from a permissive 24×24 stroke set; every glyph control replaced, not only the ones that rendered as boxes; `▶` stays one control whose glyph and `aria-label` change with the swipe state. Graduated [Visual identity pass](issues/66-visual-identity-pass.md).

14 [Decide whether to keep the neon palette](issues/14-decide-palette.md): resolved — the neon is inherited from the SillyTavern lineage, not an identity; replaced by a full token value pass (accents, bubble backgrounds, borders, hardcoded gradients) chosen in a prototype. Token names stay hue-based. Graduated [56](issues/56-prototype-dark-palettes.md) and [Visual identity pass](issues/66-visual-identity-pass.md); 12 folded into that ticket so the values and the contrast fix ship as one commit.

12 [Fix dialogue colour and small-text contrast](issues/12-fix-dialogue-colour-contrast.md): absorbed into [Visual identity pass](issues/66-visual-identity-pass.md) — the same colour rules in `assets/styles.css`, so one commit.

16 [Decide one save model for panel forms](issues/16-decide-save-model.md): resolved — option E, hybrid by control type: commands and single-choice controls apply at once, text fields and JSON commit on Save. Text Check's pair is coupled and loses its Save; the World posture selects stay instant but move into a labelled auto-saving group so Cancel's scope is visible; the stale "Saved" is scoped out of the details form; the unused `POST /settings` route is deleted. Graduated [Panel consistency and Options presets](issues/67-panel-consistency-and-options-presets.md).

19 [Decide the panel layout convention and supported viewports](issues/19-decide-layout-convention.md): resolved — option A1: one centered 960px column with 24px padding for every panel, the tab body as the only scroll region (removes the floating inner scrollbar), a shared card frame so the Worlds inset stops changing, a horizontally scrolling tab bar below 1024px, and desktop-first viewports (≥1024×700 supported, phone out of scope). Raw Map/Scenarios JSON editing pushed to the fog. Graduated [Panel consistency and Options presets](issues/67-panel-consistency-and-options-presets.md), shared with the save-model decision.

21 [Decide how games are named and renamed](issues/21-decide-game-names.md): resolved — option A: a Game gains a stored display name (default a readable world/date/ordinal form), the raw generated key stays as the stable name, the header and Games tab show the display name, and rename is added; `CONTEXT.md` gains the term. Graduated [59](issues/59-game-display-names.md).

18 [Decide whether Options presets appear in Prompt Presets](issues/18-decide-options-presets.md): resolved — option C: an "Options Prompts" section in the Prompt Presets tab (activating sets the settings-level default) plus a per-game Options select in the Games picker; the second seed becomes reachable. Graduated [Panel consistency and Options presets](issues/67-panel-consistency-and-options-presets.md).

31 [Decide what a tier-1 test may observe](issues/31-decide-tier-1-observations.md): resolved — option C (**domain outcome**). A tier-1 test observes the response plus stored state through a read seam: an HTTP GET or a method on an application read service/port; never a `GameState` field or a storage row. Storage is not a read surface (`GameViewQuery::list_latest_llm_messages` already exists for the recorder). The rule judges helper bodies. Spec Givens/Thens name `CONTEXT.md` terms or client-seen values. The 45 raw field reads and ~11 storage observation reads migrate now in [61](issues/61-migrate-tier-1-test-reads.md); the 58 hard-leak scenarios reword on touch; the 7 soft ones already comply.

38 [Stop editing a second entry from freezing the story-log poll](issues/38-edit-another-entry-freezes-poll.md): resolved — option C: while an edit is open, every other entry's Edit button is disabled; the lock outlives a successful save until the resumed poll re-renders the log. `pausePolling` now saves the trigger only when `originalTrigger` is null, and `showEditForm` no-ops while an edit is open. Graduated [Keep DOM state and focus through in-place swaps](issues/65-keep-dom-state-through-swaps.md).

[Reset the generation status and the options dock when a swipe switch restores a snapshot](issues/43-reset-status-and-dock-on-swipe-switch.md): resolved — a restored Swipe is a settled branch point: `MessageService::switch_swipe` normalises the restored snapshot to Idle, the default phase, and no offered options, and the restore is surfaced in a transient `#restore-notice`. Persisting the post-turn snapshot was rejected because a Swipe's snapshot *is* its retry anchor for guided / impersonate / user-regeneration redos. Finding 6.6 is a defect, not intended: `useOption` now refuses while the status is generating. Tier 1 `swipe_switch.rs` 36.1/36.2; tier 2 37.1 and 26.5.

[Render the story-log and LLM Messages stub fixtures from the real templates](issues/50-render-stub-fixtures-from-templates.md): resolved — `/fragment/story-log` and `/fragment/llm-messages` render through `NarrativeLogTemplate` and `LlmMessagesTemplate` with the canned data in Rust, and both hand-copied fixtures are deleted. A temporary mutation proved a template hook change reaches the served fragment with no fixture edit. The focus-ring invariant's Tab bound widened (a search limit) because truthful rendering adds the last entry's delete control. The other six fixtures follow in a later ticket.

[Name and rename Games](issues/59-game-display-names.md): resolved — option A: a Game gains a stored display name (`World — DD Mon YYYY (N)` by default) beside the stable generated name the uniqueness scan keeps reading; migration v25 backfills legacy rows; `POST /games/:id/rename` is the first post-insert write to a Game's name; the header and the Games tab show the display name; `CONTEXT.md` gains the term. A rename bumps `updated_at`, so the renamed game moves to the top of the Saved Games list — deliberate.

[Migrate tier-1 tests to observe through legal read seams](issues/61-migrate-tier-1-test-reads.md): resolved — every raw `GameState` read and storage observation read in `tests/http/` now goes through `GET /status/generating`, `GameViewQuery`, `MessageService`, `GameCatalogue`, `SettingsService` or `PromptPresetService`, with no production seam added. The scenario message's swipe is asserted through `load_messages`, and the initial snapshot by a new application-tier test on `save_message_and_snapshot` (mutation-proven), so both dropped assertions stay covered.

63 [Redesign the error and health display](issues/63-redesign-error-health-display.md): resolved — a banner for degraded roles and an unreachable server, failed requests keep their region, and the generation error clamps to one line with the raw text in an anchored popover; `role_health` reads each role's true newest attempt; tier 1 `failure_display.rs` 38.1–38.4, tier 2 16.18–16.27.

65 [Keep DOM state and focus through in-place swaps](issues/65-keep-dom-state-through-swaps.md): resolved — the story-log poll morphs (vendored idiomorph, per-entry ids), an open edit locks every other Edit control, the dead `htmx:refresh`/`action-area-refresh` calls are gone, and the focus-restore manager was verified already shipped by 46; tier 2 30.11–30.17.

67 [Panel consistency and Options presets](issues/67-panel-consistency-and-options-presets.md): resolved — hybrid save model, coupled Text Check pair, single-column layout with declared viewports, and an Options Prompts section plus a per-game Options select; `POST /settings` deleted; tier 1 20.11–20.13, 21.35–21.38.

47 [Announce dynamic state changes to assistive technology](issues/47-announce-state-changes-to-at.md): resolved — hidden live regions written only on change (polite phases/options/narration, assertive generation error), with the story-log announcement scoped to the changed entry; tier 2 16.23/16.24.

15 [Settings panel: roles, buttons and text-check controls](issues/15-settings-panel-prototype.md): resolved — Settings splits into Connections and Text Check sub-tabs; Narrator and Quantifier get a connection select plus health at the top of Connections (the Set-as buttons and any Options/Trigger rows are out); Add/Edit open one form page; Delete is refused while a role uses the connection; a connection test writes no `llm_messages` row; the story-log ✓ and `POST /check-text` are removed. Graduated [68](issues/68-split-settings-sub-tabs.md), [69](issues/69-add-connection-test.md) and [70](issues/70-remove-story-log-check.md).

68 [Split Settings into Connections and Text Check sub-tabs](issues/68-split-settings-sub-tabs.md): resolved — Settings gains client-side Connections/Text Check sub-tabs reusing ticket 46's tablist pattern; Narrator and Quantifier role rows with a live select and `role_health` (plus a degraded sub-tab dot); one shared Add/Edit form page with a back link; delete refused while a role uses the connection (the `connections[0]` reassignment is gone, the last-connection refusal stays); new `/connections/set-{narrator,quantifier}` routes replace the per-id ones; tier 1 20.1, 20.8, 20.14–20.16 and tier 2 40.1/40.2 (new `browser_settings.md`); commit 6916556a.

70 [Remove the story-log ✓ and `POST /check-text`](issues/70-remove-story-log-check.md): resolved — the `.check-btn`, the JS check helpers and `#text-check-result` slot, `TextCheckResultTemplate`, the handler and route, the stub route and the dead CSS are gone in every mode; the pre-send check and `#text-check-card` stay; `text_check.md` and browser 16.11/16.14 deleted, 30.12 retargeted; the combined quarantine pin was 76 at this commit (a later cleanup deleted the caller-less `/fragment/action-area` test and lowered it to 75); commit 6916556a.

64 [Keep failed forms and cards in place](issues/64-keep-failed-requests-in-place.md): resolved — a failed add/edit and a refused delete answer non-2xx, so htmx leaves the panel or card in place and the shared short-message + Details disclosure renders into that surface's inline slot; tier 1 for every path, tier 2 for the connection form and the preset card paths (16.29–16.32); commit 6077371b.

69 [Add a connection test](issues/69-add-connection-test.md): resolved — a Test control on each Connections row and on the Add/Edit form sends one fixed prompt through a new `ConnectionTestService` that writes no `llm_messages` row, so role health cannot move; success shows the reply time, failure the shared disclosure; tier 1 20.19–20.21 plus unit tests; commit 6077371b.

56 [Prototype three calmer dark palettes](issues/56-prototype-dark-palettes.md): resolved — the user picked D, a neutral grey base with plain near-white prose and amber dialogue (colour only, not bold), with blue `#8ab4f8` in place of green for OK/active state. Calmer green vs red was near-identical under a red/green colour-blind simulation, and the user has that condition. Every measured text pair is at least 5:1. `--color-accent-green` gets renamed. Full values were handed to [Visual identity pass](issues/66-visual-identity-pass.md).

66 [Visual identity pass](issues/66-visual-identity-pass.md): resolved — one 13-symbol Lucide sprite in the shell (`<use>` tinted by `currentColor`, every icon-only button named by an `aria-label` equal to its `title`, the forward swipe button kept as one control whose icon and name follow the swipe state); palette D applied with `--color-accent-green` renamed `--color-accent-ok` (blue), hardcoded gradients, glows and `rgba()` tints replaced by `--color-button-*`, `--color-tint-*` and one `--shadow-focus-ring`, quoted dialogue moved off error red to `--color-accent-orange`, narration prose to `--color-text-primary`, and the Connections degraded dot replaced by a `#i-triangle-alert` icon (WCAG 1.4.1); `--color-error-gradient-start` darkened to `#bd5252` so white toast text clears 4.5:1; tier 1 `story_log.rs` 8.6 and `dashboard.rs` 39.2 plus extended `settings.rs`; `ui_design.md` tables rewritten.

32 [Rewrite the test strategy around one checkable tier-1 rule](issues/32-rewrite-test-strategy.md): resolved — `tests/STRATEGY.md` now uses one term, **domain outcome**, and has a line-level test for it. The tier table has one row per `tests/` directory. Every rule ends with a `Check:` line (`spec-coverage`, `guardrails` or review-only). "Complete specs" and "same-tier overlap" now have an end point. Two old false claims are fixed (the htmx-settle guardrail scope and the poller count). `tests/AGENTS.md` points at the table. Commit 03405acc.

25 [Stop server errors from wiping whole panels](issues/_resolved/25-stop-errors-wiping-panels.md): resolved — absorbed into [64](issues/64-keep-failed-requests-in-place.md); the failed-add panel cases (Settings connection add, preset add, preset edit) land there so one session owns the "a failure must not wipe its region" work.

26 [Stop failed edits and refused deletes from destroying the entity card](issues/_resolved/26-stop-errors-destroying-cards.md): resolved — absorbed into [64](issues/64-keep-failed-requests-in-place.md) with 25, so both surfaces of findings 05.F1/05.F2 share one implementation and one error fragment.

45 [Restore keyboard focus after in-place htmx swaps](issues/_resolved/45-restore-focus-after-inline-swaps.md): resolved — absorbed into [65](issues/65-keep-dom-state-through-swaps.md), so every swap site that loses focus or selection is fixed with one mechanism in one pass.

48 [Morph the story-log poll swap](issues/_resolved/48-morph-story-log-poll-swap.md): resolved — absorbed into [65](issues/65-keep-dom-state-through-swaps.md), beside the edit-button lock that shares `pausePolling` and the same spec and test files.

49 [Delete the dead `htmx:refresh` calls and fix the docs](issues/_resolved/49-delete-dead-htmx-refresh.md): resolved — absorbed into [65](issues/65-keep-dom-state-through-swaps.md); the call sites are in the same file, and the morph makes deletion the right call.

51 [Add a failure banner for degraded roles and an unreachable server](issues/_resolved/51-add-failure-banner.md): resolved — absorbed into [63](issues/63-redesign-error-health-display.md), which owns the anchored popover, the `beforeSwap` listener and the header poll.

52 [Keep a failed poll from replacing its region](issues/_resolved/52-failed-request-keeps-its-region.md): resolved — absorbed into [63](issues/63-redesign-error-health-display.md), which now owns the shared short-message + popover fragment the form and card tickets consume.

53 [Clamp the status-display error and move the raw text into a popover](issues/_resolved/53-clamp-status-error-with-popover.md): resolved — absorbed into [63](issues/63-redesign-error-health-display.md), which reuses that ticket's anchored popover instead of building its own.

55 [Replace the glyph controls with an SVG icon sprite](issues/_resolved/55-replace-glyphs-with-svg-sprite.md): resolved — absorbed into [66](issues/66-visual-identity-pass.md), so the shell and stylesheet are rewritten once alongside the palette.

57 [Apply the chosen palette and fix the colour contrast](issues/_resolved/57-apply-chosen-palette.md): resolved — absorbed into [66](issues/66-visual-identity-pass.md) with 12, so the palette values and the contrast fixes ship as one commit.

58 [Make the dashboard panels consistent: save model and layout](issues/_resolved/58-panel-consistency.md): resolved — absorbed into [67](issues/67-panel-consistency-and-options-presets.md), which rewrites the Prompt Presets and Games templates and handlers.

60 [Expose Options presets in the UI](issues/_resolved/60-options-presets-ui.md): resolved — absorbed into [67](issues/67-panel-consistency-and-options-presets.md), in the same sweep over the Prompt Presets and Games templates.

62 [Lock every other entry's Edit button while an edit is open](issues/_resolved/62-lock-other-edit-buttons.md): resolved — absorbed into [65](issues/65-keep-dom-state-through-swaps.md), beside the morph and focus work in the same file, spec and test file.

23 [Align the frontend docs with the dashboard](issues/23-align-frontend-docs.md): resolved — the four docs now describe the shipped action area (four client states, `#action-preview`, "Send Original" / "Send with edits", Cancel closing the preview) and `dashboard.md` defines Healthy / Degraded / Unreachable; the reset-button location already followed the code (Games tab Active Game row).

54 [Retire the toast and route its callers](issues/54-retire-toast-and-route-callers.md): resolved — `#error-notification`, `showError`, its 5s timer and the `.error-notification` styles are gone. Per call site: the status-text path now shows only the status display's own clamped disclosure (the `lastStatusError` dedupe went with it) and the `htmx:beforeSwap` listener is deleted; `submitGenerationRequest` reports into the status display; `submitEdit`, `switchSwipe` and `deleteMessage` report into a new `#story-log-error` slot outside the polled log. `inlineErrorSlotFor` now adds a slot when the server rendered none (`.posture-override` and `.posture-group` joined the container list), or the games, worlds and posture action failures would have gone silent. Scenarios 16.7/16.8 removed (16.19 covers the inline slot; 16.8 was the toast's own timer), 16.10 rewritten, 30.4/30.5/30.10 moved onto their surfaces, new 30.18/30.19 for a failed delete and a failed swipe switch; tier 2, mutation-proven. `dashboard.md` and `ui_design.md` no longer describe the toast, and `--color-error-gradient-start` is deleted.

## Not yet specified

- **Other snapshot-restore paths.** [Reset the generation status and the options dock when a swipe switch restores a snapshot](issues/43-reset-status-and-dock-on-swipe-switch.md) landed the swipe case only, where the snapshot was written mid-generation; its normalisation does not touch the other restore paths. Retrigger and history revert restore snapshots by different paths and may show the same stuck status. Unverified; [Drive the retrigger control in a browser test](issues/40-drive-retrigger-in-a-browser-test.md) drove the failure path only and did not surface it.
- **Structured editing of Map and Scenarios JSON.** [Decide the panel layout convention and supported viewports](issues/19-decide-layout-convention.md) pushed finding 4.8 (raw JSON in plain textareas) here. It is a separate question and can wait for a later pass.
- **Browser suite cost.** Each stub-tier test launches its own Chromium, and Theme 1/Theme 2 fixes will add several. Check the cost at the final re-review. Decide then whether it needs action.
- **A validator check for spec prose.** `scripts/validate_feature_spec.py` never reads Givens or Thens. One cheap check might flag dotted identifiers in backticks, such as `narrative.last_trigger`. Its false-hit rate is unknown. [Rewrite the test strategy around one checkable tier-1 rule](issues/32-rewrite-test-strategy.md) left the spec-vocabulary rule review-only. Two related checks are also unscheduled: a guardrail for `GameState` field reads in `tests/http/`, and a guardrail for a `tests/` directory with no row in the tier table. Decide whether any of the three is worth building. (A third rule, `check_test_layer_boundaries`, guarded a `tests/components/` tier that does not exist and could never fire — the runner strips the `tests/` prefix before matching — so it was deleted with its two detection tests rather than re-pointed at `tests/http/`, which would make the first decision above.)

## Out of scope

- **Changing the Quantifier's fallback behaviour.** The engine falling back to fallback NPC IDs is now reported by the failure banner ([63](issues/63-redesign-error-health-display.md)); whether the fallback itself should behave differently is a backend design question and this map covers the dashboard frontend.
