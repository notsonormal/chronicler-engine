# Cut provider text by characters, not by bytes

Type: task (AFK, decide and do)
Status: open
Blocked by: —

## Question

Make every cut of a string fall on a character boundary (llm-07). Choose the fix under the enforcement order of the map Notes, and put it in place.

Today `&error_body[..500]` (`src/adapters/driven/llm/transport/utils/response.rs:140`) and `&response[..response.len().min(200)]` (`src/application/agents/quantifier/utils/orchestration.rs:110`) slice by bytes. A provider error body longer than 500 bytes with a multi-byte character at byte 500 panics the model call.

The candidate fixes: the crate lint `clippy::string_slice`, which forbids every string slice at build time (the weekly-retro proposal); or one helper that cuts by characters, used at both sites.

## Context

- Finding in full: llm-07 in [the llm report](../assets/llm.md). The coordinator confirmed it in the code.
- **R4.** This is the byte-500 panic of the weekly retro, routed here (map Notes). The retro counted 5 string slices in `src/` outside the tests. Two can panic (the sites above). Three slice at an index that `find` returned, so they are safe today: `quantifier/utils/parser.rs:95` and `:120`, and `domain/model/action.rs:27`. The lint flags all five, and also `src/test_support/fixtures.rs:648` ([the retro ticket](../../weekly-retro-2026-11-04/issues/05-grilling-product-fault-routes.md), Context). The crate lints are the attributes at the top of `src/lib.rs`.
- A panic in a Job also leaves no failure line. [Write a failure line when a Job panics](10-task-failure-line-after-a-panic.md) fixes that part.
- The six end conditions of the map Notes apply.

## Answer
