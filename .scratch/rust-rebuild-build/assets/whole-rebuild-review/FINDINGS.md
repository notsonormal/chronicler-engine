# Findings: review of the whole Rust rebuild

Range `bc7d6cd4..8036bbc3` plus the uncommitted fixes of this review. Sources: `findings/*.md`. Each row says which source raised it and whether the lead verified it (V = read in code by the lead; S = scout or reviewer read only).

## Already fixed in this review (uncommitted)

| Area | What | Source |
|---|---|---|
| Docs | 8 Diátaxis findings (DOC-1..7, 9), cancel-handle text (CR-2/TN-M9) in `game_flow.md`, `rust_idioms.md`, `CONTEXT.md`, ticket 12 | docs scout, CR, TN |
| Code | S-1, S-2 (vestigial `CancellationToken`, `tokio-util` dropped), S-3/27 comments, S-5 `preset_text`, 7 dead fns, S-11, S-13/29, S-14, S-15, S-21, S-22, S-23/24/31, S-25/26 | slop scout |
| Tests | Scenarios 36.5, 8.9, 8.10, 38.11, 38.12 + tier-1 tests; T-1, T-2, T-5; helpers D-1..D-6 | tests scout |
| Simplify / comments | 5 small simplifications, 2 comments | step 9/10 |
| Map | `Not yet specified` states the Destination waits for the `main` merge | step 2/3 |

Gate after the fixes: `logs/build_20261019_223126.log`, green (guardrails 189, integration 1337 + 2 skipped, browser 29). Test-police: 6 integration runs with no flake, coverage 90.6 %.

## High

| # | Finding | Sources | V |
|---|---|---|---|
| 1 | **Bare `/impersonate` answers 500** "The data for this request is missing: the impersonate direction." `Action::Impersonate(None)` reaches `require_job_data` (`gate.rs:546`), which demands a direction for every Impersonate. Design §4.5 asks for it only on a Retry of an Impersonate Input. The slash menu offers bare `/impersonate`. No test. | CR-S1; root cause TN-H1 | V |
| 2 | **Job failure lines name the wrong thing.** `impl From<EngineError> for Failure` (`job.rs:87`) makes every read error "missing data" with the raw error text as the record name (breaks §4.7 short line). `JobWriter` read misses (`state_at`, `tail_of`, dock read) become "The new text could not be saved." 34 hand-made `EngineError::Config(format!)` wraps in the new storage code also keep `from_engine_error` from classifying them. | AP-1, AP-11, CR-S2, TN-H7, TN-M7 | V |
| 3 | **Generation kind and its data are separate options** (`GenerationOrder.kind` vs `event`, `impersonate_direction`): about 7 runtime branches guard states the design calls impossible; finding 1 is one of them. Fix: data-carrying enum. | TN-H1 | S |
| 4 | **`job.rs` and `gate.rs` repeat domain rules**: the event game rules live in `Job::apply_rules` (TN-H3); the opening room report is computed in both files, and the writer drops its read errors (TN-H4, AP-5, CR-10); the room lookup exists 4 times and two copies ignore `dynamic_rooms` (AP-6, TN-M1, CR-3); the one-shot Trigger is marked fired in two places (AP-13); `Kind` maps re-implemented (AP-14, TN-M10, CR-4). | TN, AP, CR | S |
| 5 | **`gate.rs` (910 lines) holds request planning and opening content**, not only the guard and its units (TN-H5, TN-H6, AP-4). | TN, AP | S |
| 6 | **`Job::run`/`failed` hold one fact in three booleans**; the event-failure wrap runs at two sites (TN-H2). `Job::failed` skips the Options run after `Stopped::SaveFailed`, against design §4.2 (CR-S3, V at `job.rs:218`). `after_narration: bool` copies `OptionsRun` (AP-3, TN-M5, V). | TN, CR, AP | V (parts) |

Findings 3 to 6 are one refactor (thermo's "code-judo" move): data in the kind type, event rules into `game_rules`, one room lookup, `run` as one loop, planning out of `gate.rs`. It is not a slice of the build plan; it is a follow-up for the owner.

## Medium

| # | Finding | Sources | V |
|---|---|---|---|
| 7 | `InMemory` backend is half a backend (14 refusing arms, a silent no-op in `touch_game_updated_at`). Delete or keep: owner call. | TN-H8 | S |
| 8 | `testing` is a default Cargo feature: `test_support::tx_token()` is callable from any `src/` file (rule 3 guards only `TxToken::new`), and `JobTasks` ships in production. The Contradiction check only runs with `diagnostics`. | AP-12, AP-22, TN-M8, TP-10, comment-fixer | V |
| 9 | A panicking Job is invisible to tier 1: `JobTasks` logs and drops the `JoinError`; "nothing written" rows pass on a crash. | TP-2 | S |
| 10 | Stop table 39.4: the Options-call row asserts no status (spec says `options`), and no row stops at the event Narration's analysis call (one-shot Trigger in the pre-analysis state). | TP-1 (V), CR-S4 | V (TP-1) |
| 11 | `storage-newest-game-updated-at` cannot tell `updated_at` ordering from `id` ordering. | TP-3 | V |
| 12 | Retrigger event direction skips the `{{user}}` render that a fired Trigger gets. Latent: no shipped data uses a placeholder. | AP-2, TN-M2 | V |
| 13 | `AgentConfig.hook` is required by the schema and never read; dispatch is on `agent_type`. | AP-7 | V |
| 14 | `AppState.job_deps.storage` is `pub`, so HTTP can reach `Storage`; two HTTP sites start a Job. `JobDeps.storage` duplicates the writer's storage. | TN-M6, AP-21 | S |
| 15 | `preset_text` still hides a Game-row read failure inside `active_*_preset_id` (warn + registry fallback); preset choice policy lives in storage. | TN-M4 | S |
| 16 | `ActiveGame::set` writes the DB and swallows the error. | TN-M12 | S |
| 17 | Duplicate data types: `StoryRow` ≈ `MessageEntry`; `Message.message_type` derivable from `kind`; `QuantifierResult`↔`StatePatch` round trip drops fields; two status enums (debug shows `step: Narrating` for an idle Game). | AP-8, AP-9, AP-10, CR-5, TN-M13 | S |
| 18 | Test files over 1000 lines: `races.rs` 1145, `actions.rs` 1006, `swipe_new.rs` 1519. | TN-M14 | S |
| 19 | Job reads settings, tail and the Game row several times per generation. | TN-M3 | S |
| 20 | Blocking SQLite work under the global mutex on the async runtime (`Job::start`, `stop`, `busy_step`). | CR-8 | inferred |

## Low

| # | Finding | Sources |
|---|---|---|
| 21 | `GameWriter::stop` removes the entry, then a failed dock read returns early: Game idle, stop unit not written, answer "failed". | CR-S7 |
| 22 | Status handler answers "idle" when the status read fails. | CR-9, TN-L8 |
| 23 | `reset` with no active Game answers 500, design says 400 (pre-existing). | CR-S8 |
| 24 | `Job::player_text` falls back to the last message of any type; a refused state update shows "could not be saved" though the text is saved. | CR-S5, CR-S6 |
| 25 | Test-only production surface: `Tx::clear_dock`/`TxWrite::ClearDock`, `get_current_options`, Agents' `with_provider`/`Debug`, `Narrated` fields, `Narrator::assemble` pub with stale doc, `GameRuleRequest::NpcEvents`, `GameRulesOutcome` fields, walker options with no row, most `Message` Swipe accessors, `TokenBudget::fallback`, `AgentRunner::is_empty`. | AP-15/18/19/20/25, CR-6/11, TN-L1/L6, TP-9, S-4, S-7, S-19 |
| 26 | Small duplication: `transaction`/`transaction_new_game` body; `put_dock(default)`+`set_game_failure(None)` pair x3; hand-written `serde_json` in docks; `ScriptedModel::ModelRole` copies `Role`. | AP-16/17/24, CR-7, TN-L2/L3 |
| 27 | Names: `SceneMissing` (design: invalid room), `Units`; `missing`/`unreadable`/`save_failed` look like constructors but write a row. | CR-12, AP-23 |
| 28 | Tests: seam "failure text" cases check presence only; 3.5 lacks the Narration count; no storage read-failure case; CASE grammar in two implementations; `FAILURE_KINDS` can lag the enum; 21 `unwrap()`; 38.11 depends on which unit writes the dock; `story_of` uses a test-only read for counts; `dock_notice` `None` ambiguity; 39.1 four blocks in one test; `prompt_presets.rs` observes via `Storage`. | TP-4..8, TP-11, CR-13, CR-14, T-3, T-4, T-6 |
| 29 | Misc: `next_job_id` atomic outside the registry mutex; `Step` not `Copy`; hand-desugared `async_trait` in `current_game.rs`; two name-uniqueness scans; rule 13 keyed on field names; `render().unwrap_or_default()` in games handlers (pre-existing); a Message with no Swipe loads silently; `ActiveGame::set` doc repeats the wrong `updated_at` claim; `AppState` field dump in `architecture_system.md`; 7 skipped simplifications. | TN-L5/L6/L7/L9/L10, S-6, S-33, CR-15, DOC-8 |

## Recorded, not findings

The plan checks found every item done, superseded or changed with a reason. Deferred by the owner: the `main` merge (C-1); `wf/rebuild-review` lacks commits `e20b0a23`, `80fc6892`, `8036bbc3` (C-2); the I.7 start-state branch untested (C-4); no scenario for the stop and status poll with no active Game (P-1). Ticket 07 decisions (`StoryReads`/`DockReads`, the boot `GameWriter`, test-only reads) held up under review.

## Rejected or downgraded

- TN-L4 (story log keeps the oldest 50): the code at `bc7d6cd4` did the same. Not a regression.
- AP-3 high → covered in finding 6 at medium weight: no bug today.
- P-3 (`#[ignore]` anchor in `validate_feature_spec.py`): the path no longer exists.

## Tooling

- (Corrected) The gate log does hold the duplicate summary; an earlier note said it did not, from a truncated grep. The step has no row in the Step Timing Summary, so it looks as if it did not run.
- `prepare_review_bundle.py --ref` diffs against `HEAD`, so it misses uncommitted work; this review used a second `--uncommitted` bundle.
