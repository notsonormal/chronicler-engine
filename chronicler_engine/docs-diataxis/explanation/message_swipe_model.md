---
diataxis: explanation
title: Message Swipe Model
---

> **Diátaxis mode:** Explanation. This document is *understanding-oriented*: it explains why a Chronicler Engine message carries multiple swipes, why only the last message is swipeable, and why each swipe keeps its own snapshot reference. It is the companion to `../reference/message_model.md`, which describes the message aggregate as it is. The reader problem solved here is *understanding*: the shape of the message model and the tradeoffs that shape encodes.

## Why swipes exist

The Chronicler Engine's narration is an LLM call, and LLM calls are non-deterministic. The same player input can produce a strong paragraph on one run and a flat one on the next. A single-text message model makes retry destructive: regenerating the narration overwrites the previous generation, and the player cannot recover a good generation that retry replaced with a worse one.

Swipes make retry non-destructive. Each retry of an AI message produces a new `Swipe` on the same `Message`, and the previous swipe is preserved. The player navigates between them; the engine holds all alternatives. The narrative cost of retry (a fresh LLM call, a snapshot to restore) is paid in full; the information cost (losing the prior generation) is zero.

The tradeoff is a more elaborate storage model (a dedicated `message_swipes` table alongside `messages`) in exchange for non-destructive A/B comparison of the LLM's output.

## State consistency: the per-swipe snapshot

A swipe is not alternate text alone. Each `Swipe` carries its own `snapshot_id` pointing at the `GameStateSnapshot` that produced it. When the player navigates to a different swipe, the engine restores the entire world state that produced that swipe's text, not just the text itself.

This matters because narration mutates state. The quantifier runs after the narration LLM; it detects NPCs and movement, updates scene state, increments encounter counters. Two different narrations produce two different post-narration states. A model that swapped only the text would leave the world state tied to whichever swipe was generated last — a "ghost state" where the displayed text no longer matches the underlying world.

The per-swipe `snapshot_id` binds each swipe to the state that produced it. Switching swipes rewinds the world to the moment that swipe was committed. The two views — text and state — stay coherent because they were captured together. The tradeoff is a snapshot per swipe; the coherence is what the design holds the storage cost for.

## Why only the last message

Swiping is limited to the last message in the history. Each message depends on the state produced by the message before it: a narration's quantifier detected NPCs in the room that the next narration assumes are present; an event header recorded a trigger firing that the next message's state reflects.

Swiping a non-last message would require discarding every message after it, because they were built on the state the swipe just rewound. That is what retry already does — roll the world back to a snapshot, soft-delete the messages that depended on it, regenerate. Allowing swiping on a non-last message would be retry under a different name, with the extra complexity of preserving and restoring history above the swiped point.

The design chose not to carry that complexity twice. The player loses the ability to A/B an earlier message without first deleting what comes after; the model gains simplicity and a single, well-named operation (retry) for the case where rolling back history is what the player actually wants.

## Why narration and event swipes are separate

A message can be a narration (the LLM's response to the player's action) or an event continuation (a trigger firing after the narration). The two kinds have independent swipe sets: retrying a narration does not disturb the event that followed it; retriggering an event does not disturb the narration that preceded it.

This independence is the point. In an earlier model — before the per-message swipes design — narration and event were coupled at the message level, and retrying a narration invalidated the event attached to it. The player who wanted a different narration also lost the event they had earned. The current model treats each message as its own swipable unit so that retrying one concern does not force the other to be redone.

The cost is that the two flows have to be distinguished in the retry path — the engine looks at the last message's `event_header` to decide which kind of retry to run. The benefit is that the player's grip on each generation is independent per message.

## Document References

- [ADR-017: Message Swipes](../../docs/adr/adr-017-message-swipes.md) — historical decision record for the swipe model.
- [ADR-008: SQLite Snapshot Persistence](../../docs/adr/adr-008-sqlite-snapshot-persistence.md) — supplies the `GameStateSnapshot` that each swipe references for state-consistent switching.
- [`../reference/message_model.md`](../reference/message_model.md) — reference description of the message aggregate.
