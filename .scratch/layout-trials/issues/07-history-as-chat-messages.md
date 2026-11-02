# Try the history as chat messages

Type: task (HITL)
Status: closed
Blocked by: 05, 12

## Question

Does the Narration get better when the history goes to the model as chat messages, one message for each entry, and not as one text block in the user message?

## Context

- Target part: the Narration.
- Evidence from the judge pilot (2026-11-02): role breaches. Inputs write `Narrator:` and `Julian:` turns for the other side, and Narrations write Julian (7 entries in P, 2 in Q, one of 1344 words). The history is one text block with these labels, and the model may continue the format (inferred). So the count of entries that break their role is a clear question for this trial. See [the inconsistencies of two trial stories](../assets/story-inconsistencies.md).
- Known: each Narrator-role call sends one system message and one user message. Layer 5, `<ConversationHistory>`, is one text block in the user message, with a sender label on each entry (`Narrator: `, the Persona name, `System: `). SillyTavern and Marinara Engine send the history as chat messages. See [How ST and ME let an author change a call](../../prompt-flexibility/issues/02-st-me-prompt-configuration.md).
- Known (plan review, 2026-11-01): the model port sends two messages only. `LlmProvider::complete` takes one `system_prompt` and one `user_prompt` (`src/application/ports/llm_provider.rs`), and `build_request_payload` (`src/adapters/driven/llm/transport/utils/request.rs`) sends one `system` and one `user` message. `llm_messages` stores the two texts, and `raw_request_json` holds the request as it was sent.
- So the trial code in the worktree changes the port, the request and `LlmCallRecorder::complete` to carry a list of messages. Nobody merges this code.
- Known: `scripts/trial.py` finds the history through `<ConversationHistory>` in the stored user prompt (`history_block`), and writes each call file from the two stored texts. With the history as messages, the pack would show no history and wrong "holds / drops" numbers, and no error. So before the trial, `scripts/trial.py` learns to read the messages from `raw_request_json` when a request holds more than two. This script change is kept: make it on `layout-trials`, with its tests in `scripts/tests/test_trial.py`.
- Trial choices, each stated in the answer with its reason:
  - the role of each entry (for example `assistant` for a Narration and `user` for an Input, and the role of an Event Narration);
  - the place of the data layers and of the post-history splice: before the history, or in the last user message;
  - the behaviour with `single_user_message`, a Connection flag that merges the parts into one user message.
- Messages in a fixed order keep the front of the prompt stable, so the cache should not drop (inferred). Give the cached share of both sides too.
- Models treat the message roles differently, so [Decide how to read an output trial](05-read-an-output-trial.md) decides whether this candidate also runs on a second model.
- Source: candidate 5 of [Decide which parts of a Narrator-role call an author can change](../../prompt-flexibility/issues/06-the-layout-model.md).
- How: a linked worktree from `layout-trials` with a detached HEAD, as in [Try the changing blocks after the history](03-changing-blocks-after-history.md). Run the trials that ticket 05 sets, then read the comparison with the owner, with its method.

Done when the owner keeps or drops the candidate, the reading notes extend the criteria asset, and the worktree is removed. A kept candidate gets a new task ticket (map Notes).

## Out of scope (owner, 2026-11-02)

Replaced. Its one measured target, the role breaches, gets a cheaper test with no change of the model port: [Stop the role breaches with stop sequences](14-stop-role-breaches.md). Reopen if that test fails and the label format of the history is the cause.

Later (owner, 2026-11-02): the topic returns as a new ticket, [Try the history in the ME format](15-history-in-me-format.md). The engine was modelled on ME, and nobody chose the two-message history on purpose. Ticket 14 is closed.
