# Chronicler Engine

An interactive fiction engine that runs a player's narrative through an LLM-driven pipeline, persisting game state across sessions.

## Language

**Game**:
A concrete playthrough session bound to one World and one Persona, holding current mutable state, message history (with swipes on the last message), and the generation gate. It carries a stable generated name and a renameable display name.
_Avoid_: Session, run, match, playthrough (use Game)

**Display name**:
The player-facing name of a Game, distinct from its stable generated name. Free text, and changeable by the player.
_Avoid_: Title, label, nickname

**World**:
A static, authored template — locations, NPCs, maps, scenarios, global rules. Many Games can share one World, and a Game binds to its World by identifier.
_Avoid_: Setting, environment, scenario (World is the template; Scenario is a World sub-concept)

**Persona**:
The player-controlled character for a Game, chosen at game creation and fixed for the life of that Game. A Persona is a global entity, independent of any World.
_Avoid_: Player character (ambiguous), character (reserved for NPCs), avatar

**Character**:
An NPC in a World. Distinct from Persona, which is player-controlled and Game-scoped.
_Avoid_: Person, actor, avatar, NPC (use Character)

**Scenario**:
A World sub-concept: the bundled starting state for a fresh Game — starting room, starting logs, and initial NPCs.
_Avoid_: Campaign, story, module

**Action**:
A semantic command issued by the player and resolved by the Action Pipeline.
_Avoid_: Command, input, verb

**Action Pipeline**:
The ordered sequence of phases that validates and resolves an Action.
_Avoid_: Pipeline, command processor

**Trigger**:
A condition attached to world state or events whose evaluation fires scripted continuation narrations.
_Avoid_: Event, hook, callback

**Narrative**:
LLM-generated prose rendered in response to resolved Actions and trigger context.
_Avoid_: Story, text, output

**Quantifier**:
The post-generation Agent that analyzes narration to detect NPCs in area, player movement, and NPC enter/leave events.
_Avoid_: Scorer, evaluator

**Agent**:
A pipeline step that runs at a defined phase. Quantifier is one Agent.
_Avoid_: Bot, assistant, operator

**Message**:
A single entry in a Game's conversation history — player input, narration output, event continuation, or system log. Each AI-generated Message has its own Swipe set.
_Avoid_: Line, entry, chat

**Swipe**:
An alternate version of an AI-generated Message, preserving a prior generation non-destructively. A Swipe carries the player-typed inputs of its generation.
_Avoid_: Variant, version, alternate

**Retry**:
Redoing the last generation as a new Swipe on the last Message.
_Avoid_: Regenerate, reroll, resend

**Snapshot**:
A serialized mutable game sub-state, message-aligned and persisted with its corresponding Message. Every Snapshot is immediately valid for restore.
_Avoid_: Save, checkpoint, dump

**Guided Generation**:
A transient slash-command input that steers what the narrator says for one generation. Stored on the Swipe with the generation's other inputs.
_Avoid_: hint, nudge

**Impersonate**:
Forcing the next narration to be written as the player's Persona — the player speaking rather than the narrator.
_Avoid_: roleplay as, pose as, pretend (use Impersonate)

**Narrator Mode**:
Which of two narration stances a Game plays — Novel or Interactive Fiction. Set on the World and inherited at game creation; switchable per Game.
_Avoid_: style, tone, personality, narrator persona

**Options**:
Pre-written player inputs the engine offers after a non-impersonate narration turn or on demand (`/options`).
_Avoid_: suggestions, quick actions, auto-inputs

## Deprecated Terms

**Turn**:
Don't use. Use Message + Swipe.

**Narrator Action**:
Don't use. Player direction is transient (Guided Generation); lasting facts go in-world through free actions.

**Replay blob**:
Don't use. The inputs a Swipe stores about its generation have no name of their own.

## Notes

- This glossary is the single source of truth for term meanings.
- Entries define what a term IS, in at most two sentences. Behaviour, implementation details, field names, and storage mechanics belong in `docs/diataxis/` and the source, never here.
