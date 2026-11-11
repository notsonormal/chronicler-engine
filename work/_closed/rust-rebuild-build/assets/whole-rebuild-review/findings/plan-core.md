# Plan check: the core step (C1 to C7)

Source: scout `verify-core` (session 69910b82). Full tables are in that session.

## Result

Every item of plan §9 (six areas, HTTP, deletions, §9.1 target tests, §9.2 scenarios, §9.3 commits, §9.4 test fates, §9.5 rules 3/4/5/6/7/9/13/14/15/18 and the re-points, §9.6 docs) has a tree artifact (known). Validators: feature-spec 292/292, seam cases 5 contracts / 96 cases, docs PASS. §9.7 search: only `storage/utils/plumbing.rs` migration history. No `#[ignore]` outside `tests/llm/`.

Changed, each with a reason in ticket 07: `Storage::games()` not built (`get_game` covers it); `mappers/message_tests.rs` deleted, moved to the storage seam; 12.1/15.1 wording "(no Job is accepted)"; `Snapshot` kept as one `_Avoid_` entry on Swipe state in `CONTEXT.md`; reset is two units in one hold; the `Kind` mappings live in `domain/model/utils/generation_record.rs`.

Deferred, each with a reason: the start-state branch of `swipe_new.md` I.7 has no test (no client request makes that state); `Notice.reason` stays `String`; the event Narration's maker stays `Narrator`; the boot keeps its own `GameWriter`; `MockBackend::with_narrations`/`with_prompt_responses` stay; `GameViewQuery::story_messages` and `Narrator::recorder` are test-only.

## Findings to carry

| # | Finding | Status |
|---|---|---|
| C-1 | §9.7: the core branch is not merged to `main` (`main` = `f64f5d41`). It merged into `wf/rebuild-review` (`fa1a25e1`, worktree `/workspace/chronicler-engine-rebuild-review`). | deferred by owner (ticket 07) |
| C-2 | `wf/rebuild-review` lacks the 3 later commits of this branch: `e20b0a23` (ticket 09), `80fc6892` (ticket 10), `8036bbc3` (tickets 11/12). | known (git log); owner to merge or not |
| C-3 | Test-only production code: `GameViewQuery::story_messages`, `Narrator::recorder`, `MockBackend::with_narrations`/`with_prompt_responses` | recorded deferral; check in step 6 (hacks) |
| C-4 | `swipe_new.md` I.7 start-state branch untested | recorded; carry to report |
