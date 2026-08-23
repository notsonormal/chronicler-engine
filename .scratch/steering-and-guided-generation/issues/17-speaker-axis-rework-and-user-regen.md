# Implementation: speaker-axis type rework + user-regen

Type: task
Status: pending
Blocked by: 15

## Question

Carry the code changes resolved by ticket 15: `MessageType` encodes speaker/role (not function), impersonate output is `Input` (not `Dialogue`), `sender` is deleted, the `Dialogue` variant is removed, and a dedicated hardcoded user-regen path replaces "retry a plain Input = re-narrate."

## Scope

### 1. Impersonate output type: `Dialogue` → `Input` (overturns ticket 09)

`src/application/pipeline/pipeline_run.rs:198-205` — the impersonate branch of `phase_narrate` currently sets `(Some(persona.sheet.name), MessageType::Dialogue)`. Change to `(None, MessageType::Input)` (sender is deleted per item 2; type is `Input` per ticket 15 decision 1). The impersonate seam (`impersonate()` in `action_pipeline`) already builds the steering record; it continues to stage `pending_replay` with `impersonate: true` so retry re-impersonates.

### 2. Delete `sender` (full removal + storage migration)

- `src/domain/model/state/message_types.rs` — remove `sender: Option<String>` from `MessageEntry`; remove from `Default`; remove from the `From<&Message>` impl.
- Every `add_message` call site drops the `sender` argument: `action.rs:62` (Input), `action.rs:141` (Narrator), `pipeline_run.rs:206` (narration/impersonate), `pipeline_run.rs:251` and `:299` (System), `game_state.rs:175/206/301`, `arrival_service.rs:171`, `message_service.rs:129`. The `add_message` signature loses the parameter.
- `src/domain/model/message.rs` — `Message.sender` field removed; `From<&Message>` for `MessageEntry` drops it.
- `src/adapters/driving/http/view_models.rs` — `MessageEntryView.sender` field removed; the `log_type` match stays.
- `src/adapters/driving/http/templates.rs:25` — remove the `{% if entry.sender != "" %}<span class="sender">{{ entry.sender }}:</span>{% endif %}` from the story-log template.
- `assets/styles.css:282` — drop the `.sender` rule if it exists only for this.
- Storage: `messages.sender` column dropped via a new migration (bump `STORAGE_VERSION` / the migration version). `MessageEntry` serde round-trips without the field. The snapshot does not carry `sender`.

### 3. Remove the `Dialogue` variant

- `src/domain/model/state/message_types.rs` — remove `Dialogue` from `MessageType`.
- `src/adapters/driving/http/view_models.rs:54` — remove the `MessageType::Dialogue => "dialogue"` arm (exhaustive match will flag it).
- `src/domain/model/message_history.rs:104` — `last_ai_response_index` filter becomes `MessageType::Narration` only.
- `src/application/pipeline/action_pipeline/retry.rs:88-99` — `resolve_retry_target`'s `old_target` filter becomes `Narration` only (for the re-narrate case); the Input cases are handled by the new disambiguation (item 4).
- Any test referencing `Dialogue` updates or removes.

### 4. `Input` swipe support + three-way retry disambiguation

- `Input` messages must support swipes when they are the retry target. Today swipes are a property of all `Message`s (the `swipes` Vec exists on `Message`); verify `Input` messages can carry >1 swipe and that `push_message`/`set_replay` work on an Input. If `add_message` for Input creates a single-swipe message today, the retry path appends swipes to it.
- `src/application/pipeline/action_pipeline/retry.rs` — `resolve_retry_target` branches on the last message:
  - Last is `Narration` → existing re-narrate path (anchor on last `Input`, truncate, re-roll). The `Dialogue` arm is gone (item 3).
  - Last is `Input` with a steering record (`replay.impersonate == true`) → re-impersonate: the Input is the swipe target; append a swipe; replay the record's steering (`impersonate_direction`, `impersonate_preset_id`); stop after the swipe (no auto-narration, item 5).
  - Last is `Input` with no steering record → user-regen: the Input is the swipe target; append a swipe; run the hardcoded user-regen instruction (item 6); stop after the swipe.
- `src/domain/model/message_history.rs:104` `last_ai_response_index` — reconsider whether it needs to include `Input` when the Input is a swipe target. The re-narrate path anchors on `last_input_index` (already exists, `:111`); the Input-as-target paths use `last()` directly. Confirm the two helper roles stay clean.
- `src/adapters/driving/http/templates.rs` — swipe controls currently gate on `log_type == "narration" || "dialogue"`. Change to: show swipe controls when the last message is `Narration` OR an `Input` that carries a steering record OR an `Input` that is the last message and retryable. (The template may need a flag from the view model rather than inferring from `log_type`, since "is this Input retryable" depends on position and record, not just type.)
- The retry handler (`retry_handler` in `chat_window.rs`) calls `pipeline.retry()`; the branching moves into `pipeline.retry()` / `resolve_retry_target`, not the HTTP handler.

### 5. Stop after alternate swipe (no auto-narration)

Both re-impersonate and user-regen produce the new swipe and stop. They do not chain into narration. The player reviews swipes, picks one (or edits), and submits to trigger narration. The pipeline path for these two modes does not call `phase_narrate` again after producing the swipe; it finalizes. (Impersonate's `phase_narrate` already produces the impersonated line as its output; the change is that on *retry* of an impersonate Input, the re-impersonate produces the swipe and does not follow with a narration. Confirm the retry path does not currently chain a narration after the impersonate output — it should not under this change.)

### 6. Hardcoded user-regen instruction

Add a Rust builder (constant or fn) producing the rewrite instruction, mirroring Marinara's `buildUserMessageRegenerationInstruction`:

```
Regenerate the user's previous message as an alternate swipe.
Write only the replacement user message text.
Do not answer as the assistant, continue the assistant side, or describe what the assistant does next.

<original_user_message>
{original Input text}
</original_user_message>
```

The original Input text is the active swipe's text of the Input being retried. The user-regen path loads the impersonate preset? No — user-regen uses the **system preset** (it is a narrator-voiced instruction rewriting a player line, not an impersonate-voiced authoring). Confirm which preset user-regen loads: the instruction is narrator-voiced ("regenerate the user's message"), so the system preset is the likely base; the output is a player line. This is a sub-decision to confirm during implementation — the instruction itself is hardcoded, but it still needs a preset for `writing_style`/`output_format`. Recommendation: reuse the system preset; the hardcoded instruction overrides `role`/`instructions`. File the preset choice under this ticket if it needs a decision.

### 7. No slash command for user-regen

User-regen is retry-only. No `Action` variant, no parser entry, no auto-suggestion entry. The existing `Action::FreeAction/Guide/Narrator/Impersonate` enum (ticket 07) is unchanged. The retry handler routes to user-regen internally based on last-message + record.

## Out of scope for this ticket

- `GenerationReplay` → `SteeringRecord` rename (deferred, separate ticket).
- Spec/test plan commit (ticket 14, after this ticket re-greens the spec against the new types).
- Narrative perspective setting (ticket 16).
- NPC-as-separate-message (no current feature; `Dialogue` removal does not preclude a future type).

## Verification

- `cargo fmt`, `cargo clippy --all-targets -- -D warnings`.
- `cargo test --lib` — unit tests; update `Dialogue`/`sender` references.
- `cargo nextest run --tests` — integration tests; the impersonate integration tests (ticket 09/10) must update to expect `Input` not `Dialogue` and no `sender`.
- `python build.py` green.
- UI: verify swipe controls appear on a retried Input; verify no sender prefix renders on any message; verify impersonate output renders as an Input-styled line.
