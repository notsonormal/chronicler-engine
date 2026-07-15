---
diataxis: explanation
title: Message Swipe Model
---

> **Diátaxis mode:** Explanation. The reader problem solved here is *understanding*: the shape of the message aggregate — multiple swipes per message, per-swipe state binding, last-message-only swiping, independent swipe sets across narration and event. Companion to `../reference/message_model.md`, which describes the message aggregate as it is.

## Swipes and what they preserve

The Chronicler Engine's narration is an LLM call, and LLM calls are non-deterministic. The same player input can produce a strong paragraph on one run and a flat one on the next. The message aggregate carries this non-determinism directly: each retry of an AI message produces a new `Swipe` on the same `Message`, and the previous swipe is preserved.

The player navigates between swipes; the engine holds all alternatives. The narrative cost of retry — a fresh LLM call, a snapshot to restore — is paid in full. The information cost — losing the prior generation — is zero.

Storage carries the cost: a dedicated `message_swipes` table alongside `messages` holds the alternate generations, indexed by message id.

## Per-swipe state binding

A swipe is not alternate text alone. Each `Swipe` carries its own `snapshot_id` pointing at the `GameStateSnapshot` that produced it. When the player navigates to a different swipe, the engine restores the entire world state that produced that swipe's text, not just the text itself.

Narration mutates state. The quantifier runs after the narration LLM and detects NPCs and movement; it updates scene state and increments encounter counters. Two different narrations produce two different post-narration states. A model that swapped only the text would leave the world state tied to whichever swipe was generated last — a "ghost state" where the displayed text no longer matches the underlying world.

The per-swipe `snapshot_id` binds each swipe to the state that produced it. Switching swipes rewinds the world to the moment that swipe was committed. Text and state stay coherent because they were captured together.

## Last-message-only swiping

Swiping is bounded to the last message. Each message depends on the state produced by the message before it: a narration's quantifier detected NPCs the next narration assumes are present; an event header recorded a trigger firing the next message's state reflects.

A swipe on a non-last message would discard every message after it — the swipe rewinds state the subsequent messages were built on, so they cannot stand. The engine's retry operation handles that case as a single, well-named flow: roll the world back to a snapshot, soft-delete the messages that depended on it, regenerate. Carrying the same flow under two names (retry plus non-last swiping) would not gain capability.

The player's A/B comparison lives at the last message. Comparing earlier messages means rolling back everything after them via retry.

## Independent swipe sets across narration and event

A message can be a narration (the LLM's response to the player's action) or an event continuation (a trigger firing after the narration). Each message has its own swipe set: retrying a narration does not disturb the event that followed it; retriggering an event does not disturb the narration that preceded it.

The distinction lives in the last message's `event_header`. The retry path reads the header to decide which kind of retry to run — narration retry or event retrigger. Each generation keeps its grip on the previous independent of the other.

## Document References

- [ADR-017: Message Swipes](../../docs/adr/adr-017-message-swipes.md) — historical decision record for the swipe model.
- [ADR-008: SQLite Snapshot Persistence](../../docs/adr/adr-008-sqlite-snapshot-persistence.md) — supplies the `GameStateSnapshot` that each swipe references for state-consistent switching.
- `../reference/message_model.md` — reference description of the message aggregate.
