# Replace `System` Messages with an error channel outside the Message history

Type: grilling
Status: open
Blocked by: —

## Question

`MessageType::System` Messages were meant only to show errors and notices to the player. But they live in the Message history, so the LLMs read them and they act as the last Message of a generation. Which channel shows these errors and notices instead, so that no notice reaches an LLM and no notice changes which Message owns the Retry, Retrigger and Swipe controls?

## User constraints (stated 2026-10-11)

- No `System` notice goes to an LLM. The notices were only a way to show errors.
- The user is open to removing them from the Message history completely: "a pretty awkward fit within the message history structure … probably isn't actually a good solution."

## Context

**Where `System` Messages come from** (5 places):

| Source | Text | File |
|---|---|---|
| NPC-detection fallback | "[System] NPC detection uncertain — using room defaults" | `src/application/pipeline/pipeline_run.rs` |
| Options failure (on-demand) | the failure message | `src/application/pipeline/action_pipeline/options.rs` (`persist_options_failure`) |
| Options unavailable after a generation | "[System] Options agent is not available" | `action_pipeline/options.rs` (`rewrite_options_after_turn`) |
| Options error after a generation | "[System] Options generation failed: {e}" (raw engine text) | `action_pipeline/options.rs` (`rewrite_options_after_turn`) |
| Movement to an unknown room | "[System] Entered unknown location: {id}" | `src/domain/model/state/game_state.rs` (`attempt_movement`) |

A failed Trigger continuation used to add one too. [Report a failed trigger continuation without a story-log Message](../../dashboard-ui-review-phase2/issues/10-report-failed-trigger-without-message.md) removed it; the status line and the banner report that failure now.

**Where they leak** (known, from the code):

- Prompts: the Narrator (`src/application/prompting/assembler.rs`, `System: …`), the Quantifier (`agents/quantifier/prompt.rs`) and Options (`agents/options/prompt.rs`) all render them.
- Controls: a trailing `System` Message is the last Message, so `show_retrigger` (`NarrativeLogTemplate::new`, `src/adapters/driving/http/templates.rs`) is false and the Swipe controls are absent. `retrigger.rs` and `MessageService::switch_swipe` use `messages.last()`. Ticket 10 already made the Retry anchor and target skip them (`Message::is_input_or_narration`).
- Storage: each one is saved with a Snapshot (`save_message_and_snapshot`), so it takes part in the Snapshot chain that Retry and Swipe switch restore.

**Channels that already exist:** the status line (`GenerationStatus::Error`, one sentence per `GenerationFailureKind`, details disclosure), the degraded-role banner (reads the LLM Messages rows), and the tracing log.

**Options to weigh** (starting points, not a decision):

| Option | Effect |
|---|---|
| Delete `MessageType::System`; route each source to the status line, the banner or the log | Clean history, prompts and controls; the story shows no notice |
| Keep notices on screen, but store them outside the Message history (for example a per-Game notice list) | The player still sees them; new storage and rendering |
| Keep `System` Messages, but filter them out of every prompt and every "last Message" rule | Smallest change; the awkward fit stays |

Questions for the grilling: does each source need the player to see it at all (the NPC fallback is not a failure; the unknown room is a world-data gap)? Do old Games with stored `System` Messages need a migration or only a render rule? Does `CONTEXT.md`'s Message entry ("… or system log") change?

## Related

[Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md) — the first ticket of this map. This ticket does not wait for it, because the user's rule (no notice reaches an LLM) holds whatever the model says.

## Source

The dashboard UI review phase 2: follow-ups F10-1 and F10-2 in [Follow up on small issues found in the phase-2 re-review](../../dashboard-ui-review-phase2/issues/13-follow-up-phase-2-re-review-p3s.md), and the ticket-10 Answer.
