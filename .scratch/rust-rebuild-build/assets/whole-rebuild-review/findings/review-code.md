# Review: code-review (reviewer, opus), Standards + Spec

Session 20247c8f. Lead verification marked.

## Spec
| # | Sev | Finding | Lead |
|---|---|---|---|
| CR-S1 | **high** | Bare `/impersonate` (no direction) → `Impersonate(None)` → `require_job_data` refuses with 500 "missing: the impersonate direction". The slash menu offers bare `/impersonate`; design §4.5 asks for the direction only on a Retry of an Impersonate Input. No test covers bare `/impersonate` (actions.md 1.9 uses "/impersonate hello") | verified: `gate.rs:546`, `gate.rs:480-487` |
| CR-S2 | med | Job failure lines put raw engine error text in the short line (= AP-1); design §4.7 | verified (`job.rs:87`) |
| CR-S3 | med | `Job::failed` skips the Options run after `Stopped::SaveFailed` (e.g. a state update fails after the text saved); only `EntryMoved` means the Job lost the Game; design §4.2 | verified `job.rs:218` (inferred impact) |
| CR-S4 | med | 39.4 stop table lacks the stop at the event Narration's analysis call (one-shot Trigger fired in pre-analysis state, R-16) | inferred |
| CR-S5 | low | `Job::player_text` falls back to the last message of any type (a Narration on Continue/guide); `previous_input` filters Input | inferred |
| CR-S6 | low | A refused state update writes "The new text could not be saved." though the text is saved; Contradiction line fits | inferred |
| CR-S7 | low | `GameWriter::stop` removes the entry, then `dock_of(game)?` can return early: Game idle, stop unit not written, handler says "failed" | known |
| CR-S8 | low | `reset_handler` with no active Game answers 500, design says 400 (pre-existing) | known |

## Standards
| # | Sev | Finding | Lead |
|---|---|---|---|
| CR-1 | med | = CR-S2 / AP-1 | |
| CR-2 | med | Docs still described the cancel handle removed by this review's fix pass (`game_flow.md`, `rust_idioms.md`, `CONTEXT.md`, ticket 12) | **fixed by lead** |
| CR-3 | low | Room lookup/name duplicated (= AP-6) | |
| CR-4 | low | `empties_the_dock` = `Kind::saves_a_narration` (= AP-14) | |
| CR-5 | low | Two status types; `/debug/state` shows `step: Narrating` for idle/failed Game | |
| CR-6 | low | `Tx::clear_dock` test-only (= AP-15) | |
| CR-7 | low | `transaction`/`transaction_new_game` duplicate (= AP-17) | |
| CR-8 | low | `Job::start`, `stop`, `busy_step` take the global mutex and run SQLite reads on the async runtime | inferred |
| CR-9 | low | `generating_status_handler` answers "idle" when the status read fails | |
| CR-10 | low | `opening_room_report` drops read errors (= AP-5 part) | |
| CR-11 | low | `Narrator::assemble` pub with stale doc (= AP-19) | |
| CR-12 | low | Names: `SceneMissing` (design: invalid room), `Units` | |
| CR-13 | low | Tier-1 `story_of` uses a test-only read for counts the story-log fragment already returns | |
| CR-14 | low | New 38.11 test depends on which unit writes the dock (`PutDock` injection) — fragile | |
| CR-15 | low | `ActiveGame::set` doc repeats the wrong "newest Game picked by updated_at" claim (DOC-3) | |
