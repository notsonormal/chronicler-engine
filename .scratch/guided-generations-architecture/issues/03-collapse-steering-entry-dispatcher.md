# 03 — Collapse action entry methods into one dispatcher (architecture candidate 3)

Type: grilling
Status: resolved
Blocked by: (none)

> Re-framed 2026-08-29 after ticket 06 resolved: "steering" is retired, and
> Narrator Action's removal (execution work) drops
> `narrator_action`/`process_action_with_narrator` from the entry set —
> seven methods become five. The dispatcher question stands.

## Question

Do we commit to collapsing the entry methods on `ActionPipeline`
into one `process_action(Action)` dispatcher parametrised by the existing
`Action` enum — and if so, what is the shape of the deepened entry seam?

## Background

**Friction (from `architecture-review.html`, candidate 3, Worth exploring):**
`src/application/pipeline/action_pipeline/action.rs` now exposes seven
public/crate-visible entry methods: `process_action`,
`process_action_with_guide`, `process_action_with_replay`,
`process_action_with_narrator`, `guide_narration`, `narrator_action`,
`impersonate`. After Narrator Action's removal, five remain.

The HTTP handler already maps slash commands cleanly to `Action` variants (in
`src/domain/model/action.rs`, the parser). But the pipeline then re-derives the
same mapping through method proliferation: guide → `process_action_with_guide`;
impersonate → `process_action_with_replay`. Shallow routing: the interface is
nearly as wide as the implementation because each slash command gets its own
pipeline method.

Additionally, `impersonate` directly reads
`settings.active_impersonate_prompt_preset_id` and bakes it into the stored
inputs, coupling the entry path to preset-management knowledge.

**Relationship to ticket 01 (narration-generation module).** The dispatcher
hands off to the narration-generation seam. This ticket can be grilled
independently — the dispatcher routes `Action` variants regardless of the
module's internal shape — but the grilling should confirm the hand-off point.

## What to decide

- Commit or reject the dispatcher deepening.
- If committed: the dispatcher's **interface** (one `process_action(Action)`
  method; what the `Action` enum carries — does the slash-command payload
  ride on it, or a side-channel?), its **seam** (the entry point HTTP and tests
  cross), what **implementation** moves behind it (the
  `process_action_with_*` bodies fold in), and what **tests** survive at the
  single dispatcher.
- Where preset resolution for impersonate moves — behind the dispatcher, or
  held in the prompt policy (ticket 04)?
- Whether `Action` gains payload variants or stays a pure command
  enum with the payload supplied separately.

## Answer

Resolved 2026-08-30. **Committed.**

**Interface.** One public gated entry on `ActionPipeline`:
`process_action(&GenerationGate, Action) -> Result<ProcessActionResult, EngineError>`.
Payloads ride on the existing `Action` enum — no side channel, no new
variants. The enum already carries `FreeAction(String)`, `Guide(String)`,
`Impersonate(Option<String>)`; the design targets that post-06 shape. The
empty-input → continue rule moves behind the dispatcher (the handler's
`input.is_empty()` branch folds in). The HTTP handler shrinks to
parse-plus-call; it keeps `Action::parse` regardless for the `is_steering`
text-check path.

**Seam.** The dispatcher is the public gated seam — HTTP and gated tests
cross it. `execute_action_with_replay` drops to `pub(crate)`: the sync
runner for spawn closures and sync tests, not part of the collapsed
surface (it had no production caller). `retry.rs` is untouched — it never
crossed these entries. Scope decision (Q2-A): the collapse absorbs the
gated slash-command entries only; the gated/sync execution modes stay
distinct.

**Implementation.** The `process_action_with_*` bodies fold in. The
variant→(persisted message, replay inputs) mapping concentrates in one
match behind the dispatcher. `GenerationReplay` construction — including
the impersonate preset pin — lives there, not on the domain enum, because
impersonate needs storage/settings to resolve the preset. Locality gain:
what `/guide` or `/impersonate` does becomes readable in one file instead
of split across handler and pipeline.

**Preset resolution (Q3-A).** Entry-time pinning stays: the dispatcher
reads settings and pins `impersonate_preset_id` into the replay blob, as
today, relocated inside the match. Required by 06's settled model — a
swipe stores the inputs that produced it; dropping the pin would make
retry after a preset change silently use the new preset.
`pipeline_run.rs`'s existing fallback (resolve at generation time when the
blob's preset id is `None`) stays as fallback. **Interaction passed to
ticket 04:** if 04 commits a prompt-policy module that owns preset choice,
this pin may relocate again — that is 04's decision.

**Tests (Q4-C — both layers; unit and integration overlap is accepted).**

- HTTP integration tests (`tests/http/actions.rs:687-724` region) survive
  unchanged in shape, and are extended to assert the swipe's stored replay
  inputs (they already inspect persisted messages).
- `process_action` gate-behavior unit tests (cancellation on reset,
  heal-stale, persona-missing) re-point to the dispatcher with
  `Action::FreeAction`.
- `execute_action` sync tests stay at the now-`pub(crate)` sync runner.
- The three `narrator_action` tests retire with 06's Narrator Action
  removal (execution work, not this decision).
- New unit tests drive the dispatcher with `Guide` and `Impersonate`
  variants and assert the replay inputs stored on the resulting swipe —
  coverage that does not exist today at any layer.

**Hand-off to ticket 01's module (confirmed, no decision).** The
dispatcher resolves an `Action` into generation inputs (message to
persist, replay inputs) and hands off; once 01's execution lands, that is
`narration_generation::run(state, GenerationInputs)` — the dispatcher is
exactly the "caller resolves inputs" that 01 committed to. The entry path
never runs narration phases. The sync runner remains the internal path
between them until 01 execution rewires it.

**Sequencing note (execution detail, not a decision).** If dispatcher
execution lands before Narrator Action removal, a temporary `Narrator`
arm maps to the existing narrator path; it is plumbing, removed with 06's
execution.
