# Chronicler Engine

An interactive fiction engine that runs a player's narrative through an LLM-driven pipeline, persisting game state across sessions.

## Language

**Game**:
A concrete playthrough session bound to one World and one Persona, holding its story (Messages with their Swipes), its dock record and its failure line. It carries a stable generated name and a renameable display name.
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
A semantic command issued by the player and resolved by a Job.
_Avoid_: Command, input, verb

**Job**:
The background work that one accepted Action, Retry, Retrigger or Options request starts. It can hold a Narration and its event Narration; while it runs, its Game is busy.
_Avoid_: Work, turn

**Job entry**:
The record in memory that says a Game is busy. It holds the id of the Job and the step that runs.
_Avoid_: Busy flag, lock, slot

**Step**:
One named part of a Job's work that the status shows while the Job runs: narrating, quantifying, generating an event, or the Options run.
_Avoid_: Phase, stage, hook

**Trigger**:
A condition attached to world state or events whose evaluation fires scripted continuation narrations.
_Avoid_: Event, hook, callback

**Narrative**:
LLM-generated prose rendered in response to resolved Actions and trigger context.
_Avoid_: Story, text, output

**Quantifier**:
The analysis-hook Agent that analyzes narration to detect NPCs in area, player movement, and NPC enter/leave events.
_Avoid_: Scorer, evaluator

**Game rules**:
The code that applies the analysis of a generation to the Game's state: movement, Characters present, encounters, and which Trigger fires. It makes no LLM call; the World supplies the map, the Characters and the Triggers.
_Avoid_: World rules, engine commit

**Agent**:
An LLM helper that runs at a fixed hook of a generation. The Quantifier is one Agent.
_Avoid_: Bot, assistant, operator

**Hook**:
A fixed point in a generation where the Job runs the Agents registered for it: the analysis hook and the Options hook. Each hook has one kind of output and one place where that output lands. The before-narration and rewrite hooks come with their first Agent.
_Avoid_: Phase, stage, step (a step is one named part of a Job's work; a hook is a point where Agents run)

**Message**:
A single entry in a Game's story: a typed Input, an Impersonate Input, a Narration, or an event Narration. Each Message has its own Swipe set.
_Avoid_: Line, entry, chat

**Swipe**:
One of the texts of a Message. Every Message has at least one Swipe. A Swipe carries the inputs of its generation and its Swipe state.
_Avoid_: Variant, version, alternate

**Retry**:
Redoing the last generation as a new Swipe on the last Message.
_Avoid_: Regenerate, reroll, resend

**Swipe state**:
The part of a Game's state that one Swipe holds, apart from the offered set: the room, the added rooms, the Characters present, the encounter records, the fired one-shot Triggers and the trigger context. Every Swipe has one.
_Avoid_: Snapshot, save, checkpoint

**Start state**:
The state from which a generation of a Message starts: the Swipe state of the active Swipe of the Message before it, or, for the opening Narration, the starting state of its first Swipe.
_Avoid_: Previous state, base state

**Current state**:
The Swipe state of the active Swipe of the last Message.
_Avoid_: Live state

**Pre-analysis state**:
The Swipe state that a new Swipe of an analysed generation holds from the save of its text until its analysis updates it: the start state, with no trigger context. A user regen keeps its start state instead. An event Narration also marks its own one-shot Trigger fired in it. A Swipe keeps the state when a stop or a restart cuts the analysis off.
_Avoid_: Draft state, provisional state

**Event record**:
The trigger context from which an event Narration was written: which Trigger fired, its name and its direction. The Message of the event Narration keeps it.
_Avoid_: Trigger log, event log

**Save unit**:
One write to storage that lands whole or not at all.
_Avoid_: Transaction, batch

**Dock record**:
The record of a Game that holds its offered set, the Options on offer, and at most one notice.
_Avoid_: Options record, dock state

**Notice**:
A reason line, with its raw text behind a disclosure, that the dock shows in place of the offered set when an Options run wrote none. A notice is not a Message, so no story window holds one.
_Avoid_: System message, warning, log line

**Failure line**:
The short line on the Game that names why its last Job or its last refused write failed, with the raw text behind a disclosure. The status shows it when the Game is not busy.
_Avoid_: Error status, error message, failure notice

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
Pre-written player inputs the engine offers after a non-impersonate narration message or on demand (`/options`).
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
