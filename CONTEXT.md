# Chronicler Engine

An interactive fiction engine: an LLM writes the story of each Game, and the engine keeps the state of each Game in storage.

## Shown in the product

The dashboard, the World files and the prompt presets use these names. Product copy uses them and no competing word.

### Games and Worlds

**Game**:
One play of a World by one Persona, with its own story. It has a stable generated name and a Display name.
_Avoid_: Session, match, playthrough, "run" (a run is an Autoplay run)

**Display name**:
The name of a Game that the player sees and can change, distinct from its stable generated name.
_Avoid_: Title, label, nickname

**World**:
A static, authored template: its map of Rooms, its Characters, its Triggers and its Scenarios. Many Games can share one World.
_Avoid_: Setting, environment

**Scenario**:
A part of a World: the starting state of a new Game. It has its starting Room, its opening text and the Characters present at the start.
_Avoid_: Campaign, module

**Room**:
A place on the map of a World. A Game can add Rooms that the map of its World does not have.
_Avoid_: Location, area

### Persona and Characters

**Persona**:
The protagonist that the player plays in a Game, chosen when the Game is made and fixed for its life. A Persona belongs to no World.
_Avoid_: Player character, avatar, "character" (a Character is never the player's)

**Character**:
A person in a World whom the player does not play.
_Avoid_: NPC, actor, avatar

### The story

**Message**:
One item of a Game's story: a typed Input, an Impersonate Input, a Narration or an event Narration. Each Message has one or more Swipes.
_Avoid_: Chat

**Input**:
A Message that speaks for the Persona: a typed Input, which the player writes, or an Impersonate Input, which the narrator writes.
_Avoid_: Free action, player text

**Narration**:
A Message that the narrator writes. An event Narration is a Narration that a fired Trigger causes, and the opening Narration is the first Message of a Game.
_Avoid_: Narrative

**Swipe**:
One of the texts of a Message, with its own Swipe state. Every Message has at least one Swipe, and exactly one of them is active.
_Avoid_: Variant, version, alternate

### What the player asks for

**Action**:
What the player submits: free text, or a slash command such as `/guide`, `/impersonate` or `/options`.
_Avoid_: Command (alone), verb

**Retry**:
Redoing the generation of the last Message as a new Swipe of it, also when the last Message is an Input.
_Avoid_: Regenerate, regen, reroll, resend

**Impersonate**:
Having the narrator write the player's next Message as the Persona, which makes an Impersonate Input.
_Avoid_: Roleplay as, pose as, pretend

**Guided Generation**:
A one-time direction from the player, given with `/guide`, that steers what the narrator writes in the next generation.
_Avoid_: Hint, nudge

**Options**:
Pre-written Actions that the engine offers to the player to pick from.
_Avoid_: Suggestions, quick actions, auto-inputs

**Narrator Mode**:
Which of two stances the narrator takes in a Game: Novel or Interactive Fiction.
_Avoid_: Style, tone, personality, narrator persona

### Autoplay and the busy state

**Autoplay run**:
A series of Jobs that the engine starts one after the other, so that the Persona plays by itself. It holds its Game from its start to its end, also between two Jobs.
_Avoid_: Run (alone), auto mode, bot, simulation

**Cycle**:
The unit of progress of an Autoplay run: one Input and the Narration that answers it. An event Narration that answers the Input takes the place of that Narration.
_Avoid_: Round, iteration, turn

**Busy state**:
The state of a Game while a Job or an Autoplay run holds it. A Game that nothing holds is idle.
_Avoid_: Busy flag, lock, generating state

### Triggers and model calls

**Trigger**:
A condition in a World that, when a Game's state meets it, causes an event Narration. A Trigger fires once unless it repeats.
_Avoid_: Trigger event, hook, callback

**Quantifier**:
The Agent that reads the new text of a generation and gives the Quantifier result: the Characters present and the Persona's destination, if any.
_Avoid_: Scorer, evaluator

**Role**:
A kind of model call that the engine makes, with its own health: Narrator, Quantifier, Options or Trigger. Each Agent has a Role, but not every Role belongs to an Agent.
_Avoid_: Agent (for the Narrator)

**Connection**:
A named way to reach one model: its provider, its address and its model. One or more Roles use each Connection.
_Avoid_: Backend, provider config

**LLM message**:
The stored record of one model call: what the engine sent, what came back and any failure. It belongs to no story.
_Avoid_: Forensics record, call log

## Used only in the code and docs

Code, tests, docs and specs use these names. These names change with the design.

### Jobs

**Job**:
The background work of one accepted request, such as an Action or a Retry. A player or an Autoplay run starts it, and while it runs its Game is busy.
_Avoid_: Turn

**Step**:
One named part of a Job that the player sees while the Job runs: narrating, quantifying, generating an event, or making Options.
_Avoid_: Phase, stage, hook

**Budget**:
The number of Cycles that an Autoplay run plays, which the player sets at its start.
_Avoid_: Limit, quota, count

### Agents and the Game rules

**Agent**:
An LLM helper that runs at a Hook and gives one kind of result. The Quantifier and the Options Agent are Agents.
_Avoid_: Bot, assistant, operator

**Hook**:
A fixed point in a Job where the Agents registered for it run, named by its position: the after-narration hook or the end-of-Job hook.
_Avoid_: Phase, stage, step, analysis hook, Options hook

**Game rules**:
The code that applies the Quantifier result of a generation to its Swipe state: movement, the Characters present, encounters, and which Trigger fires. It makes no model call.
_Avoid_: World rules, engine commit

**Characters present**:
The Characters in the same Room as the Persona, in one Swipe state.
_Avoid_: Scene, NPCs in the area, NPCs in the room

### State

**Swipe state**:
The part of a Game's state that one Swipe holds. It has the Room, the added Rooms, the Characters present, the encounters with Characters, the fired Triggers and the event record, if any. Every Swipe has one.
_Avoid_: Snapshot, save, checkpoint

**Current state**:
The Swipe state of the active Swipe of the last Message.
_Avoid_: Live state

**Event record**:
The record of a fired Trigger from which an event Narration is written: which Trigger fired, its name and its direction.
_Avoid_: Trigger context

## Deprecated Terms

**Turn**:
Don't use. Say Cycle in an Autoplay run, Message and Swipe in the story, and Job for the background work.

**Narrator Action**:
Don't use. Player direction is transient (Guided Generation). Lasting facts go in-world through Actions.

**Replay blob**:
Don't use. What a Swipe stores about its generation has no name of its own.

## Notes

- This glossary is the single source of truth for term meanings. An entry defines what a term IS, in at most two sentences. Behaviour, implementation details, field names and storage mechanics belong in `docs/diataxis/` and the source.
- A term gets an entry when it is **shared** (writers in more than one module need it), **contested** (a competing word is in the repo or its history) and **not already general** (no general programming word means exactly this). An avoided word is a competing word from the repo or its history.
- Add an entry when drift is observed. Also add one when a new concept lands and its name crosses a module boundary, or when another entry cannot be defined without it. A planned term waits in its plan until its first code lands. Remove an entry when its concept leaves the code. Candidate terms wait outside this file.
- Prefer a plain word, or a plain word with an entry name in front of it (Options failure). Make up a word only when every plain word clashes with an entry, a programming meaning or a product word. A plain word that keeps its plain meaning gets no entry, and a code name that is a plain description needs none either.
- An entry names a concept, not a place on the screen. `docs/diataxis/reference/frontend/dashboard.md` names the parts of the screen.
- An entry goes under "Shown in the product" when the dashboard, the World files or the prompt presets use its word, in any word form. Otherwise it goes under "Used only in the code and docs".
