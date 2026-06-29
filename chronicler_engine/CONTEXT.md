# Chronicler Engine

Rust interactive fiction / text-adventure engine. HTTP + WebSocket server with an HTMX dashboard, LLM-powered narrative generation, and data-driven game state loaded from JSON scenario configs.

## Language

**Game**:
A single playthrough session bound to one scenario, holding current state, message history, and pending actions.
_Avoid_: Session, run, match

**Scenario**:
A bundled definition of world, characters, locations, prompts, and triggers loaded from JSON that seeds a Game.
_Avoid_: Module, campaign, story

**World**:
The static setting of a Scenario — locations, NPCs, items, and mapped relationships. World data is authored; game state mutates at runtime.
_Avoid_: Map, setting, environment

**Character**:
A person or actor in the world. May be player-controlled or NPC, defined by a character sheet.
_Avoid_: Person, actor, avatar

**Action**:
A semantic command issued by the player (e.g. move, speak, use) that enters the action pipeline for resolution.
_Avoid_: Command, input, verb

**Action Pipeline**:
Ordered sequence of phases that validates and resolves an Action, mutating game state and producing narrative output.
_Avoid_: Pipeline, command processor

**Trigger**:
A condition attached to world state or events whose evaluation fires scripted effects.
_Avoid_: Event, hook, callback

**Narrative**:
LLM-generated prose rendered in response to resolved Actions and trigger context.
_Avoid_: Story, text, output

**Quantifier**:
Agent that evaluates narrative state into structured metrics (e.g. confidence, scene intensity).
_Avoid_: Scorer, evaluator

**Message**:
A single entry in conversation history — player input, narrative output, or system event.
_Avoid_: Line, entry, chat

**Snapshot**:
Serialized game state blob used to persist and restore a Game across runs.
_Avoid_: Save, checkpoint, dump

## Notes

- Terms map 1:1 to symbols in `src/` per the semantic-mapping documentation strategy.
- Implementation decisions live in `docs/adr/`, not here.
