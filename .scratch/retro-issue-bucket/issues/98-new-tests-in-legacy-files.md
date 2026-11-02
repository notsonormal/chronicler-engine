# New tests land in legacy-list files, and no check stops them

Type: grilling
Status: open
Blocked by: —

## Question

`tests/STRATEGY.md` ("The one rule") says that the legacy list "only shrinks", and that a module that is not a box or a pure domain function "has no unit test of its own". But nothing stops a new test in a file that is already on the list. Must the guardrail pin the test count of each legacy file, so that a new test there fails the build?

## Context

Source: the coordinator session of 2026-11-01 that ran `/simple-after-plan` on prompt-flexibility tickets 22 and 23. Standards review session `847d99c1`.

- The Standards review found 4 breaches that no tool checks. Tickets 22 and 23 added new behaviour tests to legacy files: `runner_tests.rs` (2), `run_tests.rs` (2), `prompt_preset_service_tests.rs` (2) and `plumbing_tests.rs` (the migration).
- Two of the service tests repeated refusals that `domain/model/prompt_preset_tests.rs` already pins. The rule is "Within one tier, one behaviour has one test". The coordinator deleted one of the two.
- The tickets told the implementers where to put most of these tests. Ticket 23 decided that `runner_tests.rs` is "the only place" for the `Err` path. The map's migration rule names `plumbing_tests.rs`. So the plan, not the implementer, picked the files. A check would show that choice when the plan is written.
- `unit_test_allowlist.rs` says: "The list holds files, not test counts." `tests/http/requires_migration/` already pins its size (`REQUIRES_MIGRATION_TEST_COUNT`). Retro 02 notes that parallel tickets conflict on a shared counter like that one.

## Options to decide

- **A.** Pin a test count for each legacy file in the guardrail. A count may only go down. A file that the rules allow, such as `plumbing_tests.rs` for migrations, gets a written exception.
- **B.** Keep files only, and add a review rule to `CODING_STANDARDS.md`: a new test in a legacy file needs a reason in the ticket.
- **C.** No change. The Standards review catches it.

## Recurrence (2026-11-02, layout-trials ticket 15)

Source: `/retro` of the coordinator session of [Try the history in the ME format](../../layout-trials/issues/15-history-in-me-format.md). Build session `f9547a3c`, Standards review session `2eafa64c`.

- **known** The implementer added `complete_records_the_payload_of_a_failure_after_the_send` to `src/application/llm_recorder_tests.rs`. It also rewrote three merge tests in `src/application/ports/llm_provider_tests.rs` for the new `[ASSISTANT]` header and the empty-part rule. Both files are on the legacy list.
- **known** This time the ticket did not name these files. The implementer picked them.
- **known** The Standards review caught both. The fix moved the payload assert to the seam case `narrator-call-record-failure` and the merge rules to CASE tags in `chat_completions_tests.rs`, and deleted the legacy tests.
