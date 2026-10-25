# Review: test-police (implementer, own target dir)

Session d448fab2. Runs: integration 1337 passed x6 (one under CPU saturation), guardrails 189, coverage gate 90.6 % (10229/11293), browser 29. No flake. Artifacts `tmp/test-police/`.

| # | Sev | Finding | Lead |
|---|---|---|---|
| TP-1 | med | 39.4 stop row "Narration / the Options call" asserts no status, though the spec row says `options`; `"options"` status is asserted nowhere | verified (`races.rs:691-699`, `dashboard.md:71`) |
| TP-2 | med | A Job that panics is invisible to tier 1: `JobTasks` logs and drops the `JoinError`; `NoMessage` rows pass on a crash | known (code read by scout) |
| TP-3 | med | `storage-newest-game-updated-at` cannot tell `updated_at` from `id` ordering (winner has both) | verified |
| TP-4 | low | Three "failed call recorded with its failure text" seam cases check presence, not text | |
| TP-5 | low | 3.5 test lacks the "exactly one Narration" assertion | |
| TP-6 | low | No storage read-failure case; `docks.rs` error arms uncovered | |
| TP-7 | low | CASE grammar in two implementations (Rust guardrail, Python validator) that can drift | |
| TP-8 | low | `FAILURE_KINDS` array can fall behind `GenerationFailureKind` | |
| TP-9 | low | Agents' `with_provider` + `Debug` impls unused | |
| TP-10 | note | `JobTasks` ships in production (= AP-22) | |
| TP-11 | low | 21 new `unwrap()` in tests where neighbours use `expect` | |
