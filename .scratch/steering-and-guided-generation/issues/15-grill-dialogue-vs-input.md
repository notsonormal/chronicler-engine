# Grill: the Dialogue-vs-Input message-type distinction

Type: grilling
Status: claimed
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
