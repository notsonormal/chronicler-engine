---
diataxis: reference
title: AI Steering
---

# AI Steering

## Overview

Three steering surfaces let the player shape what the narrator generates. All three enter through the single command input as slash commands (`/guide`, `/narrator`, `/impersonate`) and differ in what they steer, whether they persist, and how they survive a retry.

| Surface | What it steers | Persisted to history | Output message type | Retry re-applies via |
|---|---|---|---|---|
| Guided generation | content — what the AI says | no, transient | `Narration` | the replay blob on the swipe |
| Narrator action | a permanent author directive | yes, a `Narrator` row | `Narration` (a follow-on continue) | the history row itself |
| Impersonate | speaker — who says it | no, transient | `Dialogue` | the replay blob on the swipe |

Guided generation and impersonate are transient: their steering lives for one generation and never becomes a history entry. Narrator action is permanent: the directive is itself a history row, so later generations read it from history.

## Entry: Slash Commands

The command input recognizes three case-insensitive slash commands. Anything else — an unknown slash command or plain text — is treated as a free action and carried verbatim.

| Command | Effect |
|---|---|
| `/guide <text>` | Steers the content of the next narration. Runs a guided continue with no player input. |
| `/narrator <text>` | Persists a narrator directive in history, then runs a continue so the next narration is shaped by it. |
| `/impersonate <text>` | Forces the next narration to be written as the player's persona. Runs an impersonated continue with no player input. |

The command input's auto-suggestion palette lists the three commands; see the Dashboard reference.

## Guided Generation

Guided generation steers the *content* of the next narration. The guide text is a transient instruction; it never becomes a history entry.

The guide is the final prompt layer, rendered after `<PlayerInput>`:

```xml
<Guide>
Take the following into special consideration for your next message: {guide}
</Guide>
```

A guided turn runs the continue path with no player input and no input history entry. The guide is staged on the in-flight narrative state and recorded on the generated swipe's replay blob. A retry of that swipe re-applies the guide from the blob.

## Narrator Action

A narrator action is a permanent author directive from the omniscient voice, persisted in history and rendered distinctly.

The `/narrator <text>` command does two things in one turn: it appends a narrator entry to history with no sender and the directive text, then it runs a continue narration so the next response is shaped by the directive already in the history.

In `<ConversationHistory>`, a `Narrator` row renders as bare text — no `{sender}: ` prefix. Every other message type renders with a sender prefix (defaulting to `Narrator` when the sender is absent). The absent prefix is itself the narrator signal. `Narrator` is distinct from `System`, which carries engine notices such as the NPC-detection-uncertain message.

A retry re-reads the narrator row from history, the same way it re-reads any other history row.

## Impersonate

Impersonate forces the next narration to be written as the player's persona. It substitutes the *speaker*, not the content — a distinct axis from guided generation.

For an impersonated turn, the impersonate preset replaces the system preset. The active impersonate preset is selected by the `active_impersonate_prompt_preset_id` setting. The impersonate preset is a voice apparatus that injects the persona through `{{user}}` and the `{{persona_description}}`, `{{persona_personality}}`, and `{{persona_background}}` macros, writing in first person as the player character. The default impersonate preset ships at `data/prompt_presets/impersonate/default.json`.

The `<PlayerCharacter>` layer is dropped for an impersonated turn. The context layers stay: `<GameState>`, `<KnownNpcs>` / `<NpcsInRoom>`, `<WorldLore>`, and `<ConversationHistory>` remain. Persona data reaches the prompt through the impersonate preset's macros, not through the dropped layer.

The impersonate output is saved as a player-voiced `Dialogue` entry — the sender is the persona name. An optional `/impersonate <direction>` text steers the impersonated action without forcing an implausible leap; with no direction, the persona acts in character.

The replay blob on the swipe (the impersonate flag, the direction, and the preset id) makes a retry re-impersonate using the same preset.

## Replay Blob

The replay blob is the shared mechanism for transient steering — guided generation and impersonate. A `GenerationReplay` record on `Swipe` carries the turn's steering conditions: the guide text, or the impersonate flag with its direction and preset id.

On a fresh guided or impersonated turn, the blob is staged on the in-flight narrative state and consumed onto the new swipe when the narration is appended. On a retry, the blob is inherited from the retry-target swipe onto the new swipe, so the alternative generation re-applies the same steering.

The blob is the steering half of "reproduce this generation"; the state snapshot on the swipe is the state half. Narrator steering persists in history, so retry re-reads the history row.

## Mutual Exclusivity

A turn resolves one steering source at a time. The slash-command inputs take precedence; otherwise the retry-target swipe's replay blob supplies the steering.

## Retry and Re-trigger

Guide and impersonate apply on a new generation and on a retry. A re-trigger regenerates a trigger continuation as a plain follow-on narration with no player action.

On retry, the engine reads the steering from the retry-target swipe's replay blob when the in-flight inputs carry none. The retry path appends a swipe to the existing message and inherits that blob, so the alternative generation re-applies the same guide or impersonation.

## Document References

- [`./prompt_system.md`](./prompt_system.md) — the layered prompt architecture: `<PlayerInput>`, `<PlayerCharacter>`, the post-history splice, and the `{{user}}` / `{{persona_*}}` template macros.
- [`./narration_system.md`](./narration_system.md) — the Game Master role, the `FreeAction` default, and the continue path.
- [`../game_flow.md`](../game_flow.md) — the action-pipeline phases, the retry flow, and the re-trigger path.
- [`../storage.md`](../storage.md) — the `messages` and `message_swipes` tables, the `Swipe` record, and the replay column.
- [`../frontend/dashboard.md`](../frontend/dashboard.md) — the slash-command auto-suggestion palette in the command input.
