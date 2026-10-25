# Research: list the readers of the Message history and its Snapshot pointers

Type: research
Status: resolved
Blocked by: —

## Question

Which code reads the Message history or a Message or Swipe `snapshot_id`, which Messages does each reader take (all, last, last Input or Narration, by `MessageType`), and which state does each reader need from the pointer it reads?

## Scope

Reader groups to cover (add any you find):

1. **Pointer readers.** `MessageService::switch_swipe`, `find_retry_anchor_msg`, Retrigger, Options, debug routes, any render that loads a Snapshot. For each: the pointer it reads, the state it gets now (seed table: S0, S1, S5 …), and the state it needs.
2. **"Last Message" rules.** `messages.last()`, `Message::is_input_or_narration`, `show_retrigger` in `NarrativeLogTemplate::new` (`src/adapters/driving/http/templates.rs`), the Swipe controls, the Retry controls. For each: the effect of a trailing `System` Message, event Message or narrator Message.
3. **Prompt readers.** The Narrator assembler (`prompting/assembler.rs`), the Quantifier prompt, the Options prompt, the impersonate prompt. For each: which `MessageType`s and which fields (text, `location_header`) reach the LLM.
4. **Render readers.** The story log templates: which fields each `MessageType` shows.

## Context

- Seed: [three pipeline defects with one root](../assets/seed-evidence.md) §1.
- [Replace `System` Messages with an error channel outside the Message history](02-replace-system-messages.md) lists the known `System` leaks. Confirm the list and complete it.
- [Decide where the room a generation arrives in lives](03-decide-where-the-room-lives.md) states that no prompt reads `location_header`. Confirm it.

## Output

An asset `../assets/research-07-message-history-readers.md`: one table per reader group (reader symbol, what it selects, what it reads, what a `System` or event Message does to it, the state it needs). Label each fact known, inferred, or tested. Cite symbols, not line numbers.

## Feeds

[Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md), [Replace `System` Messages with an error channel outside the Message history](02-replace-system-messages.md), and [Decide where the room a generation arrives in lives](03-decide-where-the-room-lives.md).

## Answer

Asset: [research-07-message-history-readers.md](../assets/research-07-message-history-readers.md). Facts are known from the code unless marked. 48 labelled rows: 14 pointer readers, 16 last-Message rules, 9 prompt readers, 9 render readers.

- **Only three sites read a Message or Swipe `snapshot_id`.** `MessageService::switch_swipe`, `ActionPipeline::retry_last_response` and `ActionPipeline::check_retry_anchor`. Every other reader that needs state — every render, every fragment, `/debug/state`, `edit_history`, `delete_last`, `load_or_fresh` — calls `Storage::load_latest_snapshot`. No file under `src/adapters/driving/http/` mentions `snapshot` at all, so no render path can disagree with the latest Snapshot. `switch_swipe`'s copied `Ss` row is what makes the latest Snapshot the switched-to state; it moves no Message pointer.
- **A trailing `System` Message is the last Message to five readers.** It does not merely hide a control: `switch_swipe` returns `ApplicationError::validation("Only the last message can be swiped")`, so swiping is dead, and the Delete button moves onto the notice while the Narration before it loses it. `retrigger` rejects it, and `NarrativeLogTemplate::new` cannot set `show_retrigger` on it.
- **`Message::is_input_or_narration` is the one filter that steps over them.** `find_retry_anchor_msg`, `resolve_retry_target`, `arriving_message_mut` and `MessageHistory::last_turn_is_retryable` use it. `last_turn_is_retryable` still finds its Input, so Retry stays reachable past a notice. `edit_history` and `delete_last` have no type or position filter.
- **Every history-carrying prompt reads `text` and `message_type` only.** `LayerRenderer::render_history_layer` (Narrator, Trigger continuation, impersonate, guide, retry tails) renders **all** Messages as `Narrator: / {persona}: / System:`, so every `System` Message reaches the LLM; the Quantifier and Options prompts take the last 4. `location_header`, `event_header`, `swipe_count`, `active_swipe_index`, `id` and `timestamp` reach no prompt builder.
- **Ticket 02's source list is complete; its leak list is understated.** Five `MessageType::System` construction sites, no sixth. The ticket's `persist_options_failure` is `ActionPipeline::finalize_options_failure`. Additions to the leak list: `PipelineRun::build_trigger_request` is a fourth prompt that renders them; `last_turn_is_retryable` does not skip them; the Delete button deletes a trailing notice instead of the Narration; and `GameState::push_message` takes `pending_location` and `pending_event` **before** its type check, so a notice can carry a location or event header.
- **Ticket 03's claim holds.** No prompt reads `location_header`; the story-log template is its only reader outside the domain and storage layers.

Two findings no other ticket on this map holds (both known from the code; the consequences inferred):

1. `check_retry_anchor` validates the **anchor's** pointer, but the event branch of `retry_last_response` restores `old_target.snapshot_id()` — the event Message's own pointer. Every other mode coincides. The guard can pass and the redo then fail with `Retry failed: no snapshot found for id …`. This is an input to [Model a generation's states, its Snapshot pointers and the game-state writers](01-model-a-generations-states.md), not a ticket of its own, because the pointer model decides it; it is written up in the asset's Open points, which ticket 01 reads.
2. `AppState::render_story_log` takes the **first** `MAX_LOG_DISPLAY` (50) entries of a chronological list, while the template attaches Delete, Retrigger and the Swipe controls to `loop.last`. Past 50 Messages the controls sit on Message 50 and the newest entries render bare. This sits beyond the map's destination, so it is recorded under **Out of scope** rather than ticketed.

Four further points for later tickets, all in the asset: the four dead `MessageHistory` last-Message helpers that ticket 01 should delete rather than extend; `delete_last` leaving the deleted Message's `snapshots` rows behind; `switch_swipe` writing a copied Snapshot rather than moving a pointer (ticket 01, ticket 13, ticket 14); and the reachability of `pending_location` on a notice (ticket 03, ticket 08).
