# Map: build the Rust rebuild

Labels: wayfinder:map

## Destination

[Target design](../turn-state-model/assets/target-design.md) version 2 runs in the current Rust crate: the six preparation slices and the core step of [the build plan](../../docs/plans/rust-rebuild-build-plan.md) are done, the response to the reviews of the whole rebuild is built, and all of it is merged to `main`. (Redrawn by the owner on 2026-10-19 to take in the response. On 2026-10-20 the owner merged the rebuild to `main` first (`22a8f090`); the response is built on `wf/rebuild-response` and merges after it.)

## Notes

- **Do, not only plan.** This map overrides "plan, don't do" of `/wayfinder`: each ticket is a `task` that builds one slice of the plan (build plan §10; ticket 09 Q9 of [the Rust rebuild](../platform-choice/map.md)).
- The plan is the source of truth. A gap that needs a decision stops the slice and goes to the owner as a new ticket, numbered from 08.
- One implementer at a time; the slices run in order. Every commit passes the commit gate of plan §2.
- **Standing approval** (owner, 2026-10-17): the owner approves the commits and merges of the slices in advance. The lead commits after a green commit gate, merges a done slice to `main`, and starts the next slice. The lead stops only for a gap that needs a decision.
- **Seam-case form** (owner, 2026-10-17, option B): `// CASE <id>: <description>` on the tag above the test; no module-doc list. Amends ticket 26 R4. [The seam-case form: the tag carries the description](issues/08-seam-case-tag-only-form.md) is built and merged.
- **The response to the reviews** (owner, 2026-10-19): its design questions are grilling tickets (19 to 23); once they are answered, a build plan for the response gives the task tickets. The standing approval covers the commits of those tasks.
- Terms: `CONTEXT.md`. "Turn" and "Action Pipeline" are deprecated.

## Decisions so far

- [P0: the test rule and the guardrail machinery](issues/01-p0-test-rule-and-guardrail-machinery.md) — built and merged; the walker tables are empty until their rows land; `unit_test_standards.md` is deleted (owner), so read `tests/STRATEGY.md` where the plan names it; the `Narrator` joins the box lists in P4.
- [P1: `ActiveGame` and `CurrentGame`](issues/02-p1-active-game-and-current-game.md) — built and merged; `GameId` is a newtype (lead); rule 2's row allows only `catalogue.rs`, and `CurrentGame` reads through `GameCatalogue`; call-site rows take a `callees` list.
- [P2: delete the arrival opening](issues/03-p2-delete-the-arrival-opening.md) — built and merged; the boot-time `load_game_state` call stays until the core step; `with_max_tokens` waits for P4.
- [P3: `SwipeState` and the game rules](issues/04-p3-swipe-state-and-game-rules.md) — built and merged; `apply_game_rules` takes a `GameRuleRequest` and returns `Result<GameRulesOutcome>`; the wrappers keep the story writes and the old error-path order.
- [P4: the Agent edge on values](issues/05-p4-agent-edge-on-values.md) — built and merged in two commits; the preset and budget reads stay in the pipeline (rule 11); the Options agent struct is `OptionsGeneratorAgent`; 15 prompting helper tests stay legacy (owner item).
- [P5: the storage `Tx`](issues/06-p5-storage-tx.md) — built and merged in two commits; tier 1 runs on SQLite `:memory:`; 5 target tests are red and ignored until C5; the Job-task tracker moves to the Job runner in the core step.
- [The core step (C1 to C7)](issues/07-core-step.md) — built in seven commits and resolved before the merge to `main` (owner); merged into `wf/rebuild-review` for a review of the whole rebuild; migration v28 deletes saved Games; follow-ups are tickets 09 to 12.
- [Move or delete the 15 legacy prompting tests](issues/09-legacy-prompting-tests.md) — option D (owner): prompting is inside the Narrator box; the 15 tests are deleted and the new case `narrator-history-cap` pins the history cap.
- [Decide if the Connection form sets the context window and the answer budget](issues/10-connection-form-budget-fields.md) — option B (owner): the form sets the context window only; empty means the provider default, any positive integer is valid, a provider change keeps an explicit value.
- [Close or fix: `build.py test-pattern` cannot run ignored tests](issues/11-test-pattern-ignored-tests.md) — option A (owner): closed; no ignored test is left outside `tests/llm/`; the next plan that parks red tests adds the flag.
- [Decide if a stop cancels a running model call](issues/12-stop-cancels-the-model-call.md) — option C (owner): a call that runs ends on its own, and the id check refuses its answer; `game_flow.md` states the limit.
- [Decide how to respond to the review of the whole rebuild](issues/13-respond-to-the-whole-rebuild-review.md) — widened option B (owner): fix before the merge; the architecture candidates are problems to resolve now; the UI findings are fixed; graduated tickets 19 to 24.
- [Run the architecture review of the rebuilt code on Opus 5.5](issues/14-architecture-review-opus.md) — 8 candidates (3 Strong); top: deepen the Game rules, one call per generation.
- [Run the architecture review of the rebuilt code on GLM 5.3](issues/15-architecture-review-glm.md) — 5 candidates (1 Strong); top: one domain module for the room of a Swipe state.
- [Run the architecture review of the rebuilt code on Qwen 3.8](issues/16-architecture-review-qwen.md) — 5 candidates (1 Strong); top: one type family for the analysis hook.
- [Synthesize the three architecture reviews of the rebuilt code](issues/17-synthesize-the-architecture-reviews.md) — 6 merged and 5 single-model candidates, none rejected; they touch review findings 4, 5, 7, 12, 15, 17, 19, not 2, 3, 6; order: analysis type, Game rules, plan rules to the Tail, narrator.
- [Review the dashboard after the Rust rebuild](issues/18-review-the-ui-after-the-rebuild.md) — 1 P1, 1 P2, 3 P3. New in the rebuild: a bare `/impersonate` answers 500, and `/retrigger` accepts an Impersonate Input. Older: past 50 Messages, the story log hides the newest Messages and its controls act on a hidden Message.
- [Decide the shape of a generation: its kind, its analysis output and the Game rules](issues/19-shape-of-a-generation.md) — a data-carrying `Generation` enum (bare `/impersonate` allowed; ids and texts only); pure plan fn `JobPlan::for_request` in the domain; `QuantifierResult` of two facts, no merge; one `game_rules::apply` per generation (on the Swipe state loaded at `saved.at`; `PreAnalysisState` dropped by ticket 20); the trigger context holds the event record; hooks named by position; the room report moved to ticket 25.
- [Decide the shape of the Job: one run loop, its failure lines and the narrator's prompt rules](issues/20-shape-of-the-job.md) — each step loads by id, calls once, saves small (no input bundle; `PreAnalysisState` deleted from ticket 19); one read-line rule ("missing: <record>"); `EngineError::Database { context, source }`; the narrator owns the prompt rules of each kind; one loop with `JobFailure { EntryMoved, Line }`; the Options run follows a save failure.
- [Decide the storage seam: the in-memory adapter, the story projections and the read failures](issues/21-storage-seam.md) — the in-memory adapter is deleted (`new_in_memory()` opens SQLite `:memory:`); a typed `StorageCall` key for read failures; `StoryRow` is the one story entry.
- [R2: the story window](issues/30-r2-story-window.md) — built: `GameViewQuery::story_window` reads the newest 50 entries and the Retrigger fact in one read; `MAX_LOG_DISPLAY` is gone.
- [R3: the room is one fact](issues/31-r3-the-room-is-one-fact.md) — built: `SwipeState::room` is the one lookup; the story window derives the room marker by name; v29 drops the stored room report.
- [R4: the Quantifier result and the Game rules](issues/32-r4-quantifier-result-and-game-rules.md) — built: `QuantifierResult` of two facts; hooks are traits by position (`Hook` enum deleted); one `game_rules::apply` per generation; the trigger context holds the event record; v30 deletes saved Games.
- [R5: the kind carries its data, and the plan rules](issues/33-r5-kind-carries-its-data-and-plan-rules.md) — built: data-carrying `Generation`; `GenerationOrder::write` is the one record home; pure `JobPlan::for_request` in the domain; a bare `/impersonate` runs; `Tail::retrigger_record` is the one R-65 rule.
- [R6: the preset module](issues/34-r6-preset-module.md) — built: `PromptPresetService` holds the preset choice and the edit/delete rules; a Game-row read failure ends the Job; a preset delete is refused while a Game uses it.
- [Decide what the story log shows past 50 Messages](issues/23-story-log-past-50-messages.md) — the newest 50 Messages (option A); one story-window read `GameViewQuery::story_window` returns the entries and the `Tail` facts together (S3); new term **Story window**; scenarios 8.11 and 8.12 land with their tests in the build task.
- [Decide what test machinery the production build carries](issues/22-test-machinery-in-production.md) — no code change (owner): `testing` stays default; S4, findings 9 and 14 closed; `diagnostics` stays, so the Contradiction check runs in production (the mapless plan text says otherwise).
- [Rethink Messages and rooms: what a room on a Message is for, how it shows, and mapless Worlds](issues/25-messages-rooms-and-mapless-worlds.md) — one fact: the stored room report is deleted (D-19 replaced); the story log shows the room name when it differs from the name of the Message before, and on the oldest Message of the story window; compare by name; R-24, R-25 and R-25a follow from the model; mapless not followed up.
- [Fix the small dashboard findings](issues/24-fix-small-dashboard-findings.md) — built: `/debug/state` names a Step only while a Job runs (`Option<Step>`, no `Default`); the World delete refusal reads "1 game"; the delete route and `/debug/state` gain complete specs (25.10 to 25.12, 39.6, 39.7).
- [Triage the small items of the review, and choose the branch to merge](issues/26-triage-the-small-items-and-the-merge-branch.md) — candidate S2 goes into the plan (one preset module; a preset delete is refused while a Game uses it); M6 is closed; the plan takes findings 10, 11, 13, 16, 21 to 23, C-4, P-1 and parts of 26 to 29; `wf/rebuild-core` merges to `main` with `--no-ff`, then `wf/rebuild-review` is removed.
- [Write the build plan of the response to the reviews](issues/27-write-the-build-plan-of-the-response.md) — [the plan](../../docs/plans/rust-rebuild-response-build-plan.md): nine slices on `wf/rebuild-response` (storage seam, story window, rooms, Quantifier result and Game rules, kind and plan rules, presets, failure lines and loop, step rule and narrator, small fixes), then the `--no-ff` merge; tickets 29 to 38.
- [Confirm three fills of the build plan of the response](issues/28-confirm-the-fills-of-the-response-plan.md) — all option A (owner): `game_rules::apply` takes `kind: Kind` (amends ticket 20 Q13); a missing preset of any kind gives the `Record::Preset` line; `PresetMissing` goes.
- [R1: the storage seam](issues/29-r1-storage-seam.md) — built: SQLite `:memory:` only; `StorageCall` key; `EngineError::Database { context, source }`; `StoryRow` is the one story entry.

## Not yet specified

- Nothing. The slices of the response and the merge are tickets 29 to 38.

## Out of scope

- The efforts of plan §10 ("Past the build").
- A streaming LLM transport with a real cancel of a running call. A non-streaming call cannot save tokens on a stop with every provider ([Decide if a stop cancels a running model call](issues/12-stop-cancels-the-model-call.md)).
- How the player reads Messages older than the story window (show earlier, paging, a separate history page). The story log shows the newest 50 Messages ([Decide what the story log shows past 50 Messages](issues/23-story-log-past-50-messages.md)); the long-term answer is a separate effort: [docs/plans/story-log-earlier-messages-investigation-plan.md](../../docs/plans/story-log-earlier-messages-investigation-plan.md).
- Mapless Worlds and the location model (the added rooms, a growing map, no location display). The owner mentioned them only as context in [Rethink Messages and rooms](issues/25-messages-rooms-and-mapless-worlds.md) and does not follow them up; subplans B and C stay as they are.
