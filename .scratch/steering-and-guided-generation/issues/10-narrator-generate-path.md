# Narrator generate-then-add path

Type: task
Status: completed
Assignee: pi-agent

## Question

Wire `/narrator <text>` to persist the narrator message AND trigger a narration generation (a continue).

Per the design synthesis (`../research/04-design-synthesis.md`, Q11):

1. `/narrator <text>` (from ticket 07's parser) persists a `MessageType::Narrator` `MessageEntry` (ticket 05) into history, then immediately triggers a narration generation.
2. The generation is a continue — no player action precedes it. Verified continue path exists (`continue_narration`, `actions.rs:27-28` → `action.rs:55`). The narrator message is in history and shapes the generation as a permanent directive (rendered bare, no prefix, per ticket 05).
3. This shares the "no-player-input generation" shape with guide (ticket 08) and plain continue.

Grounding: ST splits `/sys` (add-only) from `/sysgen` (generate-then-add); chronicler chose generate-then-add (Q11=B) on UX — a narrator direction that produces no response leaves the user wondering if it registered. Marinara has no manual narrator slash command (narrator rows are automated scene/game flows only), so it does not decide this.

Blocked by: 05 (narrator type), 07 (slash parser).

## Answer

Implemented. `/narrator <text>` now persists a `MessageType::Narrator` entry (sender `None`, rendered bare per ticket 05) into history, then triggers a continue narration so the next response is shaped by the permanent directive.

### What changed

- `src/application/pipeline/action_pipeline/action.rs`:
  - `narrator_action` now delegates to a new `process_action_with_narrator` instead of dropping the text and calling `continue_narration`.
  - `process_action_with_narrator` follows the existing `claim_and_spawn` shape: its `before_claim` heals stale state, saves state, then appends the narrator message (`add_message(text, None, MessageType::Narrator)`); the spawn task runs the continue path via `execute_action_with_replay(String::new(), None)`. The narrator message is part of history, so retry re-reads it naturally — no replay blob is needed (unlike guide/impersonate, whose steering is transient).
- `src/application/pipeline/action_pipeline/action_tests.rs`: three `#[tokio::test]` tests added — `test_narrator_action_persists_narrator_entry_then_generates` (entry text + bare sender + narration produced + Idle status), `test_narrator_action_preserves_prior_history` (prior Input preserved; Narrator appears after it; Narration after the Narrator), `test_narrator_action_rejects_concurrent_generation` (returns `ConcurrentGeneration` and does NOT persist a Narrator entry while a generation is in flight).

### Notes / deferred

- The HTTP dispatch (`actions.rs::dispatch_action`) already routed `Action::Narrator(text)` to `narrator_action` from ticket 07; no HTTP change was needed.
- Text-check on the raw `/narrator <text>` command (ticket 14, Q7) is intentionally untouched — `action_check_handler` still checks the full command string. That open question belongs to ticket 14.
- Narrator message editability/deletion (ticket 14, Q5) is a UI concern, not a pipeline concern; deferred to 14.

### Verification

`python build.py` green (fmt + clippy + guardrails + docs + 1015 lib tests + integration tests; 2 LLM tests skipped).
