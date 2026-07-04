# Chronicler Engine

Rust interactive fiction engine. Axum HTTP server with an HTMX dashboard, LLM-powered narrative generation, and game/playthrough state persisted in SQLite (seeded from JSON scenario files at startup).

## Language

**Game**:
A concrete playthrough session bound to one World and one Persona, holding current mutable state, message history (with swipes on the last message), and the generation gate.
_Avoid_: Session, run, match, playthrough (use Game)

**World**:
A static, authored template — locations, NPCs, maps, scenarios, global rules. Many Games can share one World. Bound to a Game via `world_key` (denormalized `world_name` for display).
_Avoid_: Setting, environment, scenario (World is the template; Scenario is a World sub-concept)

**Persona**:
The player-controlled character for a Game, chosen at game creation. Bound to a Game via `persona_key` (denormalized `persona_name`). Immutable for the life of the Game row. World-independent — personas are global entities, not World properties.
_Avoid_: Player character (ambiguous), character (reserved for NPCs), avatar

**Character**:
An NPC in a World, defined by an NpcCard and world-scoped. Triggers and relationships attached as JSON blobs. Distinct from Persona (player-controlled, game-scoped).
_Avoid_: Person, actor, avatar, NPC (use Character)

**Scenario**:
A World sub-concept — the bundled starting state for a fresh Game (starting room, starting logs, initial NPCs, default scenario id). Lives on the WorldCard, not a top-level entity.
_Avoid_: Campaign, story, module

**Action**:
A semantic command issued by the player that enters the action pipeline for resolution.
_Avoid_: Command, input, verb

**Action Pipeline**:
Ordered sequence of phases (snapshot, narration, post-generation agents, engine commit, trigger evaluation, trigger continuation, reconciliation, finalization) that validates and resolves an Action, mutating game state and producing narrative output. Normal play, main retry, and event retry all share these phases.
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
A trait-object pipeline step with a phase (`PreGeneration` or `PostGeneration`) and a `BackendSelector` choosing its LLM connection. Quantifier is the first `PostGeneration` agent.
_Avoid_: Bot, assistant, operator

**Message**:
A single entry in a Game's conversation history — player input, narration output, event continuation, dialogue, or system log. Each AI-generated Message has its own Swipe set. Only the last Message is swipeable.
_Avoid_: Line, entry, chat

**Swipe**:
An alternate version of an AI-generated Message, preserving a prior generation non-destructively. Each Swipe carries its own `snapshot_id` referencing the GameStateSnapshot that produced it. Switching swipes restores the corresponding snapshot.
_Avoid_: Variant, version, alternate

**Snapshot**:
A GameStateSnapshot — serialized mutable game sub-state (movement, narrative, scene, npc_encounter_log), message-aligned and persisted immediately with its corresponding Message. No `committed` flag; every snapshot is immediately valid for restore. Immutable world data is cached on AppState as Arcs and re-attached on load, not stored in the snapshot.
_Avoid_: Save, checkpoint, dump

## Deprecated Terms

**Turn**:
Removed grouping concept. ADR-012 (deleted) introduced `Turn + Swipe` to group a player input with all its AI responses (narration, event continuation, dialogue). ADR-013 removed turns because coupling responses within a turn locked retries together. Messages are now independent units; swipes live per-Message, not per-Turn.
_Don't use_ for current architecture. Use Message + Swipe.

## Notes

- Terms map to symbols in `src/domain/model/`, `src/application/`, and `src/adapters/` per the hexagonal layout (ADR-027).
- Implementation decisions live in `docs/adr/`; this glossary is the single source of truth for term meanings.
- ADRs may not redefine these terms — they may only use them. Contradictions belong here first.
