# Triage the small items of the review, and choose the branch to merge

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

The design tickets 19 to 25 placed the large findings. Some items of the review are still not placed. For each item, decide: **fix it in the response** (it becomes part of the build plan), or **close it** (record it and do nothing before the merge). Also decide which branch merges to `main`.

The items:

1. **Candidate S2, one preset module** (finding 15, and the inferred gap: a preset delete does not check the preset slots of each Game).
2. **Candidate M6, the shallow catalogues** (`PersonaCatalogue`, `WorldCatalogue`).
3. **Medium findings that no design ticket took:** 10, 11, 13, 16, 18, 20.
4. **Low findings:** 21 to 29, less the parts that the answers of tickets 19 to 22 already delete or close.
5. **Recorded, not findings:** C-4 (the I.7 start-state branch is not tested) and P-1 (no scenario for the stop and the status poll with no active Game).
6. **The branch to merge** (ticket 13, sub-questions 1 and 2): `wf/rebuild-core` holds every commit; `wf/rebuild-review` (`fa1a25e1`) is `main` plus two merge commits of an older `wf/rebuild-core`.

## Context

- Inputs: [FINDINGS.md](../assets/whole-rebuild-review/FINDINGS.md); [synthesis.md](../assets/architecture-review/synthesis.md) §2 (S2), §1 (M6), §3.
- Already placed: findings 1 to 7, 12, 17 and 19 (tickets 19, 20, 21, 24, 25); 8, 9 and 14 closed by [Decide what test machinery the production build carries](22-test-machinery-in-production.md).
- Skills: `/grilling`. Terms: `CONTEXT.md`.

## Answer

Resolved with the owner on 2026-10-20 (grilling, 2 rounds, 6 questions). Every item has a place: the build plan of the response ([ticket 27](27-write-the-build-plan-of-the-response.md)) takes the "fix" items; the "closed" items need no work before the merge.

Round 1:

- **Q1, S2: build it (option A).** One preset module: the preset of a Game for each hook, and the edit and delete rules leave the HTTP handler. A preset delete is refused while a Game uses the preset, the same way as a World delete ("used by N games"). Finding 15 goes with it.
- **Q2, M6: closed.** The guardrail `check_http_storage_leak` decides the shape; a merge moves the pass-through but does not remove it.
- **Q3, medium findings (all as recommended):** fix 10 (stop table 39.4: the Options-call row asserts `options`; a row stops at the analysis call of an event Narration), 11 (test data where the newer id has the older `updated_at`), 13 (delete `AgentConfig.hook` and its schema field; the Agent kind gives the hook), 16 (`ActiveGame::set` returns `Result`; the select answers with the failure line and the pointer does not move). Close 18 (the guardrail limit is 2000 non-blank lines) and 20 (no symptom; single-player server).

Round 2 (all as recommended):

- **Q4, low findings.**
  - Fix 21: `GameWriter::stop` reads the dock record before it removes the job entry (the step rule of ticket 20: load first, then save).
  - Fix 22: the status handler answers with the failure line when the status read fails, not "idle".
  - Fix 23: `reset` with no active Game answers 400, with a new scenario.
  - 24 is covered by the Job slice (ticket 20: the narrator reads a user regen's Input text from the Tail; only a failed write shows the save line). The plan adds a row that checks a refused state update.
  - 25: closed. Tickets 19 and 20 delete `GameRuleRequest::NpcEvents`, the `GameRulesOutcome` fields and the public `Narrator::assemble`; ticket 22 keeps the rest.
  - Fix 26 in the storage slice: one body for `transaction` and `transaction_new_game`; one helper for the `put_dock(default)` + `set_game_failure(None)` pair (3 sites); serde derives in place of the hand-written `serde_json` in `docks.rs`; `ScriptedModel` uses `Role` in place of `ModelRole`.
  - 27: rename `GenerationFailureKind::SceneMissing` to `InvalidRoom`. The Job slice deletes `Failure::missing` and its relatives. `Units` is closed.
  - 28: fix 5 sub-items: the seam "failure text" cases check the exact text; scenario 3.5 asserts the Narration count; `FAILURE_KINDS` is derived from the enum; 38.11 does not depend on which unit writes the dock; `tests/http/prompt_presets.rs` observes through the S2 preset module, not `Storage`. Closed: the CASE grammar in two implementations, the `unwrap()` calls in tests, `story_of` (ticket 22), the `dock_notice` `None`, the four blocks of 39.1. The storage read-failure case is `db-error-context` (ticket 20).
  - 29: fix 4 sub-items: `render().unwrap_or_default()` in `games.rs` (lines 71, 240) gives the failure line; a Message with no Swipe is a load error (`mappers/message.rs:72`); the `ActiveGame::set` doc loses the wrong `updated_at` claim (with finding 16); `architecture_system.md` loses the `AppState` field list. Closed: `next_job_id`, `Step` not `Copy`, the hand-desugared `async_trait` in `current_game.rs`, the two name-uniqueness scans, rule 13 keyed on field names, the 7 skipped simplifications.
- **Q5, recorded items.** Fix C-4: an I.7 row (a failed start-state read in `accept`) in `job-read-names-record`, through the `StorageCall` key of ticket 21. Fix P-1: a scenario and a tier-1 test each for the stop and the status poll with no active Game.
- **Q6, the branch (option A).** The response is built on `wf/rebuild-core`, which then merges to `main` with `--no-ff`, as the earlier slices did. After the merge, `wf/rebuild-review` and its worktree `../chronicler-engine-rebuild-review` are removed. Fact: `main` (`f64f5d41`) is an ancestor of `wf/rebuild-core`; `wf/rebuild-review` has only two merge commits of its own.
