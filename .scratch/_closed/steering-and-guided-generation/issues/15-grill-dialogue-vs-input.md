# Grill: the Dialogue-vs-Input message-type distinction

Type: grilling
Status: resolved
Blocked by: (none)

## Question

Grill the semantic distinction between `MessageType::Dialogue` and `MessageType::Input` in general, and decide what type impersonate output should be. This supersedes ticket 14's Q6, which "resolved" impersonate output as `MessageType::Input` ahead of grilling without accounting for the retry mechanism — the implementation in ticket 09 uses `MessageType::Dialogue`, contradicting Q6. This ticket reconciles the two.

## Why this needs grilling, not a quick fix

The two types look almost interchangeable — both carry a `sender` (the persona name), both render as a player-voiced line. But they diverge in two hidden places the Q6 decision never examined:

1. **Retry anchoring.** `find_retry_anchor_msg` (`src/application/message_service.rs:219`) anchors retry at the last `MessageType::Input` (unless the last message is an event continuation). History is truncated back to that Input (inclusive); the Input stays; a new narration is generated and appended. So `Input` is the *spine* of retry, not a victim of it.
2. **Swipe target.** `resolve_retry_target` (`src/application/pipeline/action_pipeline/retry.rs:90`) and `last_ai_response_index` (`src/domain/model/message_history.rs:104`) filter on `Narration || Dialogue` to pick the message a new swipe appends to. `Input` is excluded — it is the anchor, not the retryable output.

So the choice flips what *retry* does to an impersonated line:

| Impersonate output as | What retry does to it |
|---|---|
| `Dialogue` (current, ticket 09) | Appends a new swipe to it — re-impersonate, an alternative take as the player. |
| `Input` (ticket 14 Q6) | Makes it the retry anchor — re-feeds its text to the LLM as a player command and narrates a *response* to it. Re-narrate, not re-impersonate. |

Q6's "indistinguishable from a typed Input by everything downstream" is more right than ticket 09 realized and more wrong than Q6 realized: downstream is exactly where the difference lives.

## Questions to grill

1. **What is `Dialogue` for?** Today it is used for NPC speech (an AI-generated line spoken by an NPC). Is that its only purpose? Does anything else emit `Dialogue`? Read `pipeline_run.rs` and the NPC/event code paths.
2. **What is `Input` for?** Today it means "the player's typed command" — a prompt that drives the next narration. Is that its only purpose? The retry mechanism treats it as the anchor.
3. **The general distinction.** Is the type a marker of *who spoke* (player vs NPC vs narrator) or of *what the message is for* (prompt vs output)? These two axes are conflated today. Which axis should the type encode — and does the current split follow it?
4. **Impersonate specifically.** Impersonate output is AI-generated (→ wants to be retryable output) but reads as a player line (→ wants to be indistinguishable from typed input). It is genuinely both. Which axis from Q3 decides its type, and why?
5. **Retry semantics under each choice.** Confirm the table above against the code. Is "re-narrate the impersonated line as a player command" (the `Input` path) ever desirable, or is it always a bug? Is "re-impersonate, alternative take" (the `Dialogue` path) the only sensible retry for impersonate?
6. **The third option.** `Input` + a new retry mode that distinguishes impersonate-generated Input from real-player Input (via the replay blob's `impersonate: true`) and re-impersonates on retry. Does this earn its complexity, or is `Dialogue` the pragmatic answer that already works?
7. **Retry of real Inputs — is the current model even right?** The user's intuition: "the idea why we can't retry Inputs is probably wrong." Grill what *retrying an Input* should mean. Today retry anchors on the Input and re-narrates a response — it does not produce an alternative Input. Is that the correct model, or should retrying an Input offer an alternative *Input* (e.g., re-generate what the player might have typed)? This is a general question about the Input type, not specific to impersonate.
8. **If `Dialogue` stays for impersonate**, is the visual distinction (brown `--color-log-dialogue` bg, sender shown, swipe controls) acceptable, or must impersonate be visually indistinguishable from a typed player line? Q6 said "indistinguishable" — is that a hard requirement or a nice-to-have?
9. **The `sender` field is inconsistent across types.** Verified across every `add_message` call site: `Input` sets `sender = Some(player_name)` (`action.rs:64`); `Dialogue` (impersonate) sets `sender = Some(persona name)` (`pipeline_run.rs:192`); `Narration`, `Narrator`, and `System` all set `sender = None` (`pipeline_run.rs:194`, `action.rs:141`, `pipeline_run.rs:245,293`). The template renders `{{ entry.sender }}:` as a bold pink prefix whenever `sender != ""` (`templates.rs:25`, styled `assets/styles.css:282`). So a player's typed command shows e.g. "**Julian:**" above it, while the AI's narration shows nothing. This is likely a holdover from a design where the narrator was unnamed and player/NPC lines were attributed — but `Dialogue` (NPC speech) is the only other type that would naturally carry a sender, and it is currently used only for impersonate, not NPC speech. Grill: should `sender` be set consistently by *role* (narrator = none, player/NPC = name), or should `Input` drop the sender (the player is implied by context)? Is the name on Input a feature or a leftover? This is general — not impersonate-specific — but it interacts with Q8: if impersonate must be indistinguishable from typed Input, both must agree on whether to show a name.

## Forcing instance

Impersonate (ticket 09) currently saves output as `MessageType::Dialogue` with `sender = persona.sheet.name` (`src/application/pipeline/pipeline_run.rs:191-195`). Ticket 14 Q6 decided `MessageType::Input`. The implementation and the map's recorded decision conflict. This ticket resolves the conflict and, more usefully, pins the general type distinction so future features (NPC impersonation, any other "AI writes as a character" case) don't relitigate it.

## Notes for the session

- Read before grilling: `src/domain/model/state/message_types.rs` (the type + variants), `src/application/message_service.rs:219` (`find_retry_anchor_msg`), `src/application/pipeline/action_pipeline/retry.rs:90` (`resolve_retry_target`), `src/domain/model/message_history.rs:104` (`last_ai_response_index`), `src/adapters/driving/http/view_models.rs:52-57` + `templates.rs:25` (UI rendering + swipe-control gating), `src/application/pipeline/pipeline_run.rs:191-195` (current impersonate save), and every `add_message` call site for the `sender` pattern (Q9): `src/application/pipeline/action_pipeline/action.rs:62-65` (Input), `:141` (Narrator), `src/application/pipeline/pipeline_run.rs:192-199` (impersonate Dialogue + Narration), `:244` and `:292` (System).
- Supersedes ticket 14 Q6. If this ticket decides `Dialogue`, update ticket 14's Q6 answer to match. If `Input` (or `Input` + new retry mode), ticket 09's implementation must change and the retry filter needs rework.
- Out of scope: the no-immediate-narrator-follow-up after impersonate (separate observation from the same session — that is Marinara's model, logged for a future ticket if the user wants narrator auto-follow).
- Skills: `/grilling`, `/domain-modeling`.

## Answer

Grilled across six rounds. The type axis is **speaker/role**, not function. Impersonate output is `MessageType::Input` — the same type as a typed player line, because impersonate *is* the player speaking. This overturns ticket 09's implementation (which saved impersonate as `Dialogue` with a `sender`) and relocates ticket 14 Q6's `Input` decision from the function axis to the speaker axis. An implementation ticket carries the code change.

### Decisions

1. **Type axis = speaker/role (overturns the grilling's working hypothesis).** `MessageType` encodes *who is speaking* — narrator, player, system — not *what the message is for* (prompt vs output). The retry machinery keys on the type today, but the decisions below reshape that machinery so the type carries role cleanly. Impersonate and typed input are the same type because both are the player speaking.

2. **Delete `sender` entirely (full removal).** The `sender` field on `MessageEntry`, the `{{ sender }}:` template prefix, the view-model field, and the storage column all go. Voices are distinguished by `MessageType` styling alone (input vs narration vs narrator vs system). A sender/name field is not needed because the engine always has a narrator as the opposing side; a name on player lines is a leftover from a design the engine does not use. A future NPC-speech feature re-adds a name field with its own design if it arrives. Storage migration drops the column.

3. **Remove the `Dialogue` variant.** With impersonate moved to `Input`, `Dialogue` has no emitter (verified across every `add_message` call site — only the impersonate branch emitted it). A variant with no emitter leaves dead retry/swipe filter arms that mislead the next reader. If NPC-as-separate-message arrives, it is a fresh design that picks its own type; inheriting a reserved variant pre-decides that design. Remove the variant, its view-model branch, its template styling, and its `last_ai_response_index`/`resolve_retry_target` filter arms.

4. **Retry model = three-way, last-message + steering-record disambiguated.** Retry targets the last message only (already enforced by `switch_swipe` rejecting non-last messages). The steering record (the `GenerationReplay` struct on `Swipe.replay` — rename to `SteeringRecord`/`Swipe.steering` deferred to a later ticket; called "steering record" here) disambiguates the Input case:

   | Last message | Steering record | Retry mode |
   |---|---|---|
   | `Narration` | (n/a) | Re-narrate — anchor on last `Input`, truncate, re-roll narration (existing path) |
   | `Input` | present, `impersonate: true` | Re-impersonate — append swipe to the Input, replay the record's steering |
   | `Input` | absent | User-regen — append swipe to the Input, dedicated hardcoded rewrite instruction, original text fed back in tags |

   Consequences: plain Inputs store *no* steering record (absence is the user-regen signal); `Input` gains swipe support, because both re-impersonate and user-regen append swipes to the Input itself. `last_ai_response_index` and `resolve_retry_target` must treat an Input carrying a steering record (or any Input that is the last message) as a swipe target. The `Dialogue` filter arm is removed (decision 3).

5. **Retry any Input = offer an alternative player line (overturns the grilling's Q3).** Retrying a plain Input is not a re-roll of the AI's response to it; it re-authors the Input itself, offering an alternative way the player might have expressed the same intent. This re-rolls the last thing that was *generated* — for an impersonate-originated Input that was AI-generated, and for a typed Input it asks the AI for an alternative deliberately. The Q3 "retry does not re-author player input" position is overturned.

6. **Dedicated user-regen path (not the impersonate path).** A third generation mode, mirroring Marinara's `buildUserMessageRegenerationInstruction`: the prompt says "rewrite this as an alternate swipe" and includes the original Input text in tags as a rewrite target. This is distinct from impersonate, which writes a player line from scratch (persona + context, no original). The two intents differ — rewrite vs author — and a shared path would conflate two presets' worth of tuning. User-regen is **hardcoded** (a Rust instruction builder, not a `PresetType`): the rewrite contract is fixed and should not change, so it earns none of the preset/setting/panel apparatus ticket 09 built for impersonate.

7. **User-regen is retry-only; no slash command.** Reached only via the swipe/retry UI on a plain Input. No parser entry, no auto-suggestion entry. The existing retry handler branches on last-message type + record to route to the three modes; user-regen is the Input-without-record branch.

8. **Stop after the alternate swipe; no auto-narration.** Both re-impersonate and user-regen produce the new swipe and stop. The player reviews alternates, picks one (or edits), then submits to trigger narration. This matches the no-auto-narrate principle already endorsed for impersonate ("Marinara's model: it needs to allow the impersonate to be re-tried rather than assuming it will be correct the first time"). Auto-narrating would let narration lock in an Input the player has not approved.

9. **No "old narration" problem (dissolved).** Retry is last-message-only, so when an Input is the retry target there is by definition no narration after it. The truncation in the existing re-narrate path removes the Narration that is the retry target (the last message), not a later message. The three-way table (decision 4) and stop-after-swipe (decision 8) are the complete model.

10. **"Guided Generation" kept; `GenerationReplay` rename deferred.** The feature name stays (established in CONTEXT.md and docs, matches Marinara and the GG extension; "Steering" is the umbrella term and promoting it to a member name would blur the family/member distinction). The `GenerationReplay` struct / `Swipe.replay` field / `pending_replay` rename to `SteeringRecord` / `Swipe.steering` / `pending_steering` is deferred to a later ticket — the term "steering record" is used in this resolution, the rename is mechanical (no behavior change, no migration), and it does not gate the implementation ticket. CONTEXT.md's "Replay Blob" entry is left untouched pending the rename.

### What this overturns

- **Ticket 09** (impersonate saved as `Dialogue` with `sender = persona.sheet.name`): the impersonate branch of `phase_narrate` (`pipeline_run.rs:198-205`) must write `MessageType::Input` with no `sender`. The `Dialogue` variant is removed. The `sender` argument goes away across all call sites. An implementation ticket carries this.
- **Ticket 14 Q6** (decided `Input` on the function axis): the type result stands (`Input`), but the rationale moves to the speaker axis. The retry consequence Q6 recorded (Input as anchor, not swipe target) is overturned — an impersonate Input *is* a swipe target under the three-way model.

### CONTEXT.md

The "Impersonate" entry was updated during the session to reflect the `Input` output type and the no-auto-narrate, retryable properties.

### Implementation ticket to graduate

One implementation ticket carries the code changes: `MessageType::Input` for impersonate (overturning ticket 09), `sender` deletion (full removal + storage migration), `Dialogue` variant removal, `Input` swipe support, three-way retry disambiguation in `resolve_retry_target`/`last_ai_response_index`, the hardcoded user-regen instruction, and the retry-handler branching. The `GenerationReplay` → `SteeringRecord` rename is a separate later ticket (deferred per decision 10).
