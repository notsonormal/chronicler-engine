# Review: antipattern-checker (reviewer, opus)

Session 20ea0154. 25 findings: 3 high, 10 med, 12 low. Full text in that session; gist below.
Lead verification: AP-1, AP-2, AP-3, AP-7, AP-12 checked by rg (known).

| # | Sev | Finding | Where | Lead |
|---|---|---|---|---|
| AP-1 | high | Blanket `From<EngineError> for Failure` → every read error becomes "missing data" with the error text as the record name; bypasses `GenerationFailure::from_engine_error` and `GameWriter::unreadable`. Same fault gives a different line at accept vs in the Job | `job.rs:87` | verified |
| AP-2 | high | Retrigger `EventRecord.direction` not template-rendered (`{{user}}`), fired-Trigger path renders it; latent (no shipped data uses a placeholder) | `gate.rs:587` vs `game_rules.rs:119` | verified |
| AP-3 | high→med | `after_narration: bool` always equals `run == AfterNarration`; second copy of `OptionsRun` | `job.rs:211,512`, `gate.rs:747` | verified; no bug today |
| AP-4 | med | `gate.rs` (910 lines) mixes lock/units with request planning and record building; planning could be a pure domain fn | `gate.rs` | inferred |
| AP-5 | med | Opening room report decided in both `JobWriter::save_text` and `Job::apply_rules`; writer drops read errors with `.ok()?` | `gate.rs`, `job.rs` | |
| AP-6 | med | Room lookup x3, room name x3; three name versions ignore `dynamic_rooms` | `game_rules.rs`, `job.rs`, `view_query.rs`, `opening.rs` | |
| AP-7 | med | `AgentConfig.hook` required by schema, never read; dispatch is on `agent_type` | `agent.rs`, `runner.rs` | verified (no `.hook` read) |
| AP-8 | med | `StoryRow` duplicates `MessageEntry`; `StoryEntry` built from both | `message_types.rs`, `agent.rs` | |
| AP-9 | med | `QuantifierResult` → `StatePatch` → `QuantifierResult` round trip drops fields | `quantifier/agent.rs`, `runner.rs` | |
| AP-10 | med | `Message.message_type` derivable from `Message.kind`; two columns can disagree | `message.rs` | |
| AP-11 | med | `JobWriter` reads (`state_at`, `tail_of`) classify a missing record as `SaveFailed` → "could not be saved"; Job's own read gives `MissingData` | `gate.rs`, `job.rs` | |
| AP-12 | med | `TxToken` capability hole: `test_support::tx_token()` is reachable from any `src/` file (default `testing` feature); rule 3 only guards `TxToken::new` | `fixtures.rs:511`, `walker_rows.rs` | verified |
| AP-13 | med | One-shot Trigger marked fired in two places | `swipe_state.rs::pre_analysis`, `game_rules.rs` | |
| AP-14 | low | `JobPlan::first_step`, `empties_the_dock` re-implement `Kind` methods | `gate.rs` | |
| AP-15 | low | `Tx::clear_dock`/`TxWrite::ClearDock` test-only; gate writes `put_dock(default)` x4 | `transaction.rs`, `gate.rs` | |
| AP-16 | low | Two hand-written `serde_json` calls left beside `to_json`/`from_json` | `transaction.rs`, `docks.rs` | |
| AP-17 | low | `transaction` / `transaction_new_game` repeat their body | `transaction.rs` | |
| AP-18 | low | `from_config(_config, ..)` never reads config, never errs | quantifier/options agents | |
| AP-19 | low | `Narrated.backend_name/model_name` unread; `Narrator::assemble` pub with stale doc | `narrator.rs` | |
| AP-20 | low | `GameRuleRequest::NpcEvents`, `unknown_location`, `trigger_narration_landed` test-only (= S-4) | `game_rules.rs` | |
| AP-21 | low | `JobDeps.storage` duplicates `JobWriter`'s storage; test wiring re-points it | `job.rs`, `wiring.rs` | |
| AP-22 | low | `JobTasks` test-only on production spawn path (= comment-fixer flag) | `job_tasks.rs` | |
| AP-23 | low | `GameWriter::missing/unreadable/save_failed` look like constructors but write a row | `gate.rs` | |
| AP-24 | low | `ScriptedModel::ModelRole` copies `Role` | `scripted_model.rs` | |
| AP-25 | low | Walker options `TypeCheckPosition::Parameters`, `CallCountRow.file: Some` used only by tests | `walkers.rs` | |
