# Review: thermo-nuclear (reviewer, opus)

Session 0ec8f36d. Verdict: **not approved**. No critical; 8 high, 14 medium, 10 low. Full text: `/home/node/.pi/agent/delegate/0ec8f36d/result.json`.

| # | Sev | Finding | Dup of | Lead |
|---|---|---|---|---|
| TN-H1 | high | `GenerationOrder` keeps `kind` and `event`/`impersonate_direction` as separate options → 7 runtime branches for "impossible" states, two write misleading lines. Fix: data-carrying `Generation` enum | (root of CR-S1) | inferred design; branches known |
| TN-H2 | high | `Job::run`/`failed`: one fact in 3 booleans, event-failure wrap at 2 sites, `|=` patch. Fix: one loop | | known |
| TN-H3 | high | `Job::apply_rules` holds event game rules (no movement, ignore Trigger, clear `trigger_context`) outside `game_rules.rs` | | known |
| TN-H4 | high | Opening room report computed in two layers; writer drops read errors | AP-5, CR-10 | |
| TN-H5 | high | `GameWriter::opening_message` builds narrative content in the writer; World/Persona read 3 times | | |
| TN-H6 | high | ~250 of 910 `gate.rs` lines are request planning; should be a pure planner | AP-4 | inferred split |
| TN-H7 | high | Job failure lines name the wrong thing (blanket `From`); `JobWriter` read misses become "could not be saved" | AP-1, AP-11, CR-S2 | verified |
| TN-H8 | high | `InMemory` backend is half a backend: 14 `no_in_memory` arms, silent no-op in `touch_game_updated_at`. Recommend deleting it | | known; owner call |
| TN-M1 | med | Room lookup x4, two miss dynamic rooms | AP-6, CR-3 | |
| TN-M2 | med | Retrigger direction not rendered | AP-2 | verified |
| TN-M3 | med | Job reads settings/tail/game row several times per generation | | |
| TN-M4 | med | `preset_text` still hides a failure through `active_*_preset_id` (`get_game().ok().flatten()` + warn); preset choice policy in storage | (S-5 partial) | |
| TN-M5 | med | `save_options` bool repeats `OptionsRun` | AP-3 | verified |
| TN-M6 | med | Two HTTP call sites start a Job; `AppState.job_deps.storage` is `pub` → HTTP can reach `Storage` | | |
| TN-M7 | med | 34 hand-written `EngineError::Config(format!..)` around rusqlite errors in new storage code; `EngineError::Database` exists; `from_engine_error` then maps them to `Other` | | verified count (34) |
| TN-M8 | med | Contradiction check only under `diagnostics` feature; `testing` is a default feature, ships test machinery | AP-12, AP-22 | |
| TN-M9 | med | Docs still describe the removed cancel handle | CR-2 | **fixed by lead** |
| TN-M10 | med | `empties_the_dock`, `first_step` re-implement `Kind` methods | AP-14, CR-4 | |
| TN-M11 | med | Boot builds a second `GameWriter`; `GameWriter::new` is `pub` | (ticket 07 recorded) | |
| TN-M12 | med | `ActiveGame::set` writes the DB and logs-and-swallows its failure | | |
| TN-M13 | med | Two status enums; debug shows `Narrating` when idle | CR-5 | |
| TN-M14 | med | `races.rs` 1145, `actions.rs` 1006, `swipe_new.rs` 1519 lines (>1k limit) | | |
| TN-L1 | low | `clear_dock`, `get_current_options` test-only, no "only tests" doc | AP-15, CR-6 | |
| TN-L2 | low | `put_dock(default)` + `set_game_failure(None)` pair repeated in 3 units | | |
| TN-L3 | low | Hand-written `serde_json` in docks | AP-16 | |
| TN-L4 | low | `STORY_LIMIT = i32::MAX` sentinel; story log keeps the oldest 50 | | lead: same as before the rebuild (old history was oldest-first + `take(50)`); not a regression |
| TN-L5 | low | `next_job_id` atomic could live inside the registry mutex | | |
| TN-L6 | low | `Step` not `Copy`; `Narrated` unused fields; stale `assemble` doc; `NpcEvents` | AP-19, AP-20 | |
| TN-L7 | low | `current_game.rs` hand-desugared async_trait; `dispatch_with_status_headers` async w/o await; DOC anchor | | |
| TN-L8 | low | status handler answers "idle" on a read error | CR-9 | |
| TN-L9 | low | Two different name-uniqueness scans for create vs reset | | |
| TN-L10 | low | Rule 13 matches by receiver field name | | |
