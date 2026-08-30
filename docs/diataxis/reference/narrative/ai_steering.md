---
diataxis: reference
title: AI Steering
---

# AI Steering

## Overview

Two steering surfaces let the player shape what the narrator generates. Both enter through the single command input as slash commands (`/guide`, `/impersonate`) and differ in what they steer and how they survive a retry.

| Surface | What it steers | Persisted to history | Output message type | Retry re-applies via |
|---|---|---|---|---|
| Guided generation | content — what the AI says | no, transient | `Narration` | the replay blob on the swipe |
| Impersonate | speaker — who says it | no, transient | `Narration` | the replay blob on the swipe |

Both surfaces are transient: their steering lives for one generation and never becomes a history entry.

## Entry: Slash Commands

The command input recognizes two case-insensitive slash commands. Anything else — an unknown slash command or plain text — is treated as a free action and carried verbatim.

| Command | Effect |
|---|---|
| `/guide <text>` | Steers the content of the next narration. Runs a guided continue with no player input. |
| `/impersonate <text>` | Forces the next narration to be written as the player's persona. Runs an impersonated continue with no player input. |

The command input's auto-suggestion palette lists the two commands; see the Dashboard reference.

## Guided Generation

Guided generation steers the *content* of the next narration. The guide text is a transient instruction; it never becomes a history entry.

The guide is the final prompt layer, rendered after `<PlayerInput>`:

```xml
<Guide>
Take the following into special consideration for your next message: {guide}
</Guide>
```

A guided turn runs the continue path with no player input and no input history entry. The guide is staged on the in-flight narrative state and recorded on the generated swipe's replay blob. A retry of that swipe re-applies the guide from the blob.

## Impersonate

Impersonate forces the next narration to be written as the player's persona. It substitutes the *speaker*, not the content — a distinct axis from guided generation.

For an impersonated turn, the impersonate preset replaces the system preset. The active impersonate preset is selected through settings. The impersonate preset acts as a voice apparatus: it writes in first person as the player character and receives the persona sheet through template variables.

The `<PlayerCharacter>` layer is dropped for an impersonated turn. The context layers stay: `<GameState>`, `<KnownNpcs>` / `<NpcsInRoom>`, `<WorldLore>`, and `<ConversationHistory>` remain. Persona data reaches the prompt through the impersonate preset, not through the dropped layer.

The impersonate output is saved as a player-voiced dialogue entry — the sender is the persona name. An optional `/impersonate <direction>` text steers the impersonated action without forcing an implausible leap; with no direction, the persona acts in character.

The replay blob on the swipe makes a retry re-impersonate using the same preset.

## Replay Blob

The replay blob is the shared mechanism for transient steering — guided generation and impersonate. It lives on the swipe and carries the conditions needed to reproduce the steering: the guide text, or the impersonate direction and selected preset.

On a fresh guided or impersonated turn, the blob is staged on the in-flight narrative state and consumed onto the new swipe when the narration is appended. On a retry, the blob is inherited from the retry-target swipe onto the new swipe, so the alternative generation re-applies the same steering.

The blob is the steering half of "reproduce this generation"; the state snapshot on the swipe is the state half.

## Mutual Exclusivity

A turn resolves one steering source at a time. The slash-command inputs take precedence; otherwise the retry-target swipe's replay blob supplies the steering.

## Retry and Re-trigger

Guide and impersonate apply on a new generation and on a retry. A re-trigger regenerates a trigger continuation as a plain follow-on narration with no player action.

On retry, the engine reads the steering from the retry-target swipe's replay blob when the in-flight inputs carry none. The retry path appends a swipe to the existing message and inherits that blob, so the alternative generation re-applies the same guide or impersonation.

## Document References

- [`./prompt_system.md`](./prompt_system.md) — the layered prompt architecture: `<PlayerInput>`, `<PlayerCharacter>`, the post-history splice, and template macros.
- [`./narration_system.md`](./narration_system.md) — the Game Master role, the `FreeAction` default, and the continue path.
- [`../game_flow.md`](../game_flow.md) — the action-pipeline phases, the retry flow, and the re-trigger path.
- [`../storage.md`](../storage.md) — the `messages` and `message_swipes` tables, the `Swipe` record, and the replay column.
- [`../frontend/dashboard.md`](../frontend/dashboard.md) — the slash-command auto-suggestion palette in the command input.
