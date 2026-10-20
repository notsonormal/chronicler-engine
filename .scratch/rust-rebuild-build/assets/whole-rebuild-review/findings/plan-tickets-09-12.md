# Plan check: tickets 09 to 12

Source: scout `verify-tickets-09-12` (session ea8d7b86). Lead check added below.

## Result

All four tickets match their chosen option in the tree at `8036bbc3` (known, by the scout's file reads).

| Ticket | Option | Status | Key evidence |
|---|---|---|---|
| 09 legacy prompting tests | D | done | `token_budget_tests.rs` and `utils/context_tests.rs` deleted (15 tests); off `LEGACY_UNIT_TEST_FILES`; `CASE narrator-history-cap` in `narrator_tests.rs` |
| 10 Connection form budget | B | done | scenarios 20.25 to 20.28 in `settings.md`, tests in `tests/http/settings.rs`; `parse::<u32>()` with `> 0`; no `max_tokens` form field |
| 11 test-pattern ignored | A | done | only `#[ignore]` left: `tests/llm/flow_llm_tests.rs` |
| 12 stop and model call | C | done | `game_flow.md` "Stop and Reset" sentence; `JobWriter::hold` id check; `run_step` → `require_alive()` |

## Lead check

- The gate counts the tickets record (1335 passed + 2 skipped, browser 29) match the baseline gate `logs/build_20261019_213433.log` (known).
- Accepted gap (ticket 09): the 4-chars-per-token rate of `estimate_tokens` has no direct test. Known, recorded in the ticket.

## Findings to carry

- None that block. One accepted gap (above).
