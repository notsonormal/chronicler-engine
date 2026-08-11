# 05 — Refactor DefaultApplicationService to module-per-type

Type: task
Status: ready-for-agent
Blocked by: (none)

## Question

Refactor `DefaultApplicationService` so its inherent impls satisfy `guardrails_inherent_impl_locality`.

Current state:
- `DefaultApplicationService` defined in `application/application_service.rs`
- `impl DefaultApplicationService` blocks spread across:
  - `application/application_service.rs` (main impl block + `retry_last_response`, `retrigger_event`)
  - `application/action_pipeline/retry.rs` (`retry_persist_error`, `handle_retry_outcome`, `retry_event_continuation`, `retry_main_narration` — all per issue 09's parking)
  - `application/message_editing.rs` (`prepare_retry_state` only, ~24 lines total in file)

This is a **cross-module split** — the three impl files live in three different parent folders. Under the rule, this is the clearest violation in the codebase.

Target shape — single-file consolidation:
```text
application/application_service.rs  # struct DefaultApplicationService + all impls
```

If file exceeds 2000 lines (file_length guardrail), switch to folder shape:
```text
application/default_application_service/
  mod.rs                    # struct definition
  retry.rs                  # impl DefaultApplicationService { retry methods }
  message_editing.rs        # impl DefaultApplicationService { prepare_retry_state }
  ...                       # other splits
```

Constraints:
- `build.py` green at every landed step.
- Preserve `pub`/`pub(crate)`/`#[instrument(skip(self))]` signatures exactly.
- `application/mod.rs:30` currently has flat re-export `pub use utils::retry::{retrigger, retry};` for the Arc-self spawn-blocking orchestrators. Those are FREE functions in `application/utils/retry.rs` and are out of scope for this ticket — they stay as free functions (Arc<Self> is not an idiomatic method receiver). Only the `impl DefaultApplicationService` blocks move.
- Preserve `guardrails_application_storage_direct` (application_service.rs is already in the 6 grandfathered files — keep that).
- Do NOT touch trait impls.
- Do NOT touch `retry_tests.rs` (test file, unchanged).
- `message_editing.rs` becomes empty after the `prepare_retry_state` impl moves out — delete the file and remove its `mod` declaration from `application/mod.rs`.

Acceptance:
- `cargo test --test guardrails guardrails_inherent_impl_locality` reports zero `DefaultApplicationService` violations.
- Full `build.py` green.
- No new `guardrails_*` failures.
- `message_editing.rs` deleted if emptied.
