# R1: the storage seam

Type: task (AFK)
Status: resolved
Blocked by: 27

## Question

Build §3 of [the build plan of the response](../../../../docs/plans/rust-rebuild-response-build-plan.md) on `wf/rebuild-response`. Read §1, §2 and §3 of the plan. The plan is the source of truth; this ticket records the facts that the slice measures and the result.

## Done when

The slice is **done** by §2 of the plan, and its own **Done when** checks of §3 pass. Each commit passed the commit gate, under the standing approval in the map's Notes. A gap that needs a decision stops the slice and goes to the owner as a new ticket.

## Answer

Built on `wf/rebuild-response` in one commit, after a green full gate (`logs/build_20261021_003125.log`: 189 guardrail, 1337 integration, 29 browser tests pass).

- The in-memory adapter is deleted; `Storage::new_in_memory()` opens SQLite `:memory:` and is gated to `cfg(any(test, feature = "testing"))`. `Backend` has one variant.
- `StorageCall` (38 variants) keys the failure layer; the typo test and the unconsumed-override checks went.
- `EngineError::Database { context, source }` with no `#[from]`; sites name their context through `db_context`. The HTTP refusal maps `Database` as it maps `Config` (no "Database error:" prefix).
- `StoryRow` is the one story entry; `MessageEntry`, `StoryEntry` and the view `MessageEntryView` (now `StoryRowView`) are gone.
- Finding 26: one transaction body; `clear_failure_and_put_dock` serves the three sites; serde derives in `docks.rs`; `ScriptedModel` uses `Role` (gains `Hash`).
- A Message with no Swipe is a load error (`message-without-swipe`); `db-error-context` is new; `storage-newest-game-updated-at` asserts both the id-vs-`updated_at` and the `COALESCE` crossover cases.
- Deviation from the Done-when text: `parse_datetime` (`plumbing.rs`) now wraps with `Parse`, as Content item 3 says; no `Config` wrap is left under `storage/` outside test code.
- Review findings rejected: column labels on serde errors (the plan allows the bare `Serde`).
