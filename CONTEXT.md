# Chronicler Engine

An interactive fiction engine that runs a player's narrative through an LLM-driven pipeline, persisting game state across sessions.

## Language

**Game**:
A concrete playthrough session bound to one World and one Persona, holding current mutable state, message history (with swipes on the last message), and the generation gate.
_Avoid_: Session, run, match, playthrough (use Game)

**World**:
A static, authored template — locations, NPCs, maps, scenarios, global rules. Many Games can share one World. Bound to a Game via a world identifier (a display name is denormalized for display).
_Avoid_: Setting, environment, scenario (World is the template; Scenario is a World sub-concept)

**Persona**:
The player-controlled character for a Game, chosen at game creation. Bound to a Game via a persona identifier (a display name is denormalized for display). Immutable for the life of the Game row. World-independent — personas are global entities, not World properties.
_Avoid_: Player character (ambiguous), character (reserved for NPCs), avatar

**Character**:
An NPC in a World. Triggers and relationships attached as data blobs. Distinct from Persona (player-controlled, game-scoped).
_Avoid_: Person, actor, avatar, NPC (use Character)

**Scenario**:
A World sub-concept — the bundled starting state for a fresh Game (starting room, starting logs, initial NPCs, default scenario id). Lives on the World card, not a top-level entity.
_Avoid_: Campaign, story, module

**Action**:
A semantic command issued by the player that enters the action pipeline for resolution.
_Avoid_: Command, input, verb

**Action Pipeline**:
Ordered sequence of phases that validates and resolves an Action. Trigger evaluation runs **inside** engine commit. Phase methods signal success or one of several failure modes.
_Avoid_: Pipeline, command processor

**Trigger**:
A condition attached to world state or events whose evaluation fires scripted continuation narrations.
_Avoid_: Event, hook, callback

**Narrative**:
LLM-generated prose rendered in response to resolved Actions and trigger context.
_Avoid_: Story, text, output

**Quantifier**:
Post-generation Agent that analyzes narration to detect NPCs in area, player movement, and NPC enter/leave events. Uses a separate LLM connection from the storyteller.
_Avoid_: Scorer, evaluator

**Agent**:
A pipeline step that runs at a defined phase. Quantifier is one Agent.
_Avoid_: Bot, assistant, operator

**Message**:
A single entry in a Game's conversation history — player input, narration output, event continuation, dialogue, or system log. Each AI-generated Message has its own Swipe set. Only the last Message is swipeable.
_Avoid_: Line, entry, chat

**Swipe**:
An alternate version of an AI-generated Message, preserving a prior generation non-destructively. Switching swipes restores the corresponding state snapshot.
_Avoid_: Variant, version, alternate

**Snapshot**:
A serialized mutable game sub-state, message-aligned and persisted immediately with its corresponding Message. Every snapshot is immediately valid for restore. Immutable world data lives in Storage only; the orchestrator fetches each field it needs and passes it to the engine function.
_Avoid_: Save, checkpoint, dump

## Deprecated Terms

**Turn**:
Don't use. Use Message + Swipe.

## Notes

- This glossary is the single source of truth for term meanings.
- Implementation notes and historical decisions may inform term usage, but they do not override definitions here.
