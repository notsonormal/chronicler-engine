# 03 — Collapse steering entry methods into one dispatcher (architecture candidate 3)

Type: grilling
Status: open
Blocked by: (none)

## Question

Do we commit to collapsing the seven steering entry methods on `ActionPipeline`
into one `process_action(Action)` dispatcher parametrised by the existing
`Action` enum — and if so, what is the shape of the deepened entry seam?

## Background

**Friction (from `architecture-review.html`, candidate 3, Worth exploring):**
`src/application/pipeline/action_pipeline/action.rs` now exposes seven
public/crate-visible entry methods: `process_action`,
`process_action_with_guide`, `process_action_with_replay`,
`process_action_with_narrator`, `guide_narration`, `narrator_action`,
`impersonate`.

The HTTP handler already maps slash commands cleanly to `Action` variants (in
`src/domain/model/action.rs`, the parser). But the pipeline then re-derives the
same mapping through method proliferation: guide → `process_action_with_guide`;
impersonate → `process_action_with_replay`; narrator →
`process_action_with_narrator`. Shallow routing: the interface is nearly as
wide as the implementation because each steering surface gets its own pipeline
method.

Additionally, `impersonate` directly reads
`settings.active_impersonate_prompt_preset_id` and bakes it into the replay
blob, coupling the entry path to preset-management knowledge.

**Relationship to ticket 01 (NarrationTurn).** The dispatcher hands off to the
narration-turn seam. This ticket can be grilled independently — the dispatcher
routes `Action` variants regardless of NarrationTurn's internal shape — but the
grilling should confirm the hand-off point.

## What to decide

- Commit or reject the dispatcher deepening.
- If committed: the dispatcher's **interface** (one `process_action(Action)`
  method; what the `Action` enum carries — does the steering payload ride on
  it, or a side-channel?), its **seam** (the entry point HTTP and tests
  cross), what **implementation** moves behind it (the
  `process_action_with_*` bodies fold in), and what **tests** survive at the
  single dispatcher.
- Where preset resolution for impersonate moves — behind the dispatcher, or
  held in the SteeringPromptPolicy (ticket 04)?
- Whether `Action` gains steering payload variants or stays a pure command
  enum with the payload supplied separately.
