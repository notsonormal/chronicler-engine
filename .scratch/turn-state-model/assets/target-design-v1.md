# Target design: the game-state lifecycle

Ticket: [Write the target design of the game-state lifecycle in an open session](../issues/22-write-target-design.md).
Spec: [`clean-spec.md`](clean-spec.md) version 3. Rulings: [`clean-spec-rulings.md`](clean-spec-rulings.md).

Status: draft, for the owner's acceptance.

This design holds no code and no language types. It states records, rules and orders of steps. The plan after ticket 23 chooses the code shape (owner, ticket 22, round 2, Q4).

## 0. How to read this design

Each choice has an id (D-nn). §9 gives the source of each choice. The sources are:

| Label | Meaning |
|---|---|
| R-nn, E-nn | A requirement or an environment constraint of the spec. |
| OR-nn | An owner ruling in `clean-spec-rulings.md`. |
| T22 Rn Qm | The owner's answer to question m of round n of this grilling. |
| Opus, GLM, Kimi | The walled clean design of that model ([Opus](clean-design-opus.md), [GLM](clean-design-glm.md), [Kimi](clean-design-kimi.md)). |
| ST, Marinara | Prior art: [research 13](research-13-sillytavern-swipe-state.md) and [research 14](research-14-marinara-swipe-state.md). |
| code | Today's engine, from research 04–12 or read at commit `9cbf3112`. |
| premise | A finding of this map's research about today's defects. |

Facts carry a label where it matters: **known**, **inferred** or **guessed**.

## 1. Terms

These terms are new or changed. `CONTEXT.md` does not change now. It describes the current engine (ticket 22, Context).

**Swipe state.** The game state that one Swipe holds, other than the offered set: the room, the added rooms, the Characters present, the encounter records, the fired one-shot Triggers and the trigger context. Every Swipe has one. [D-01; T22 R2 Q5]

**Start state.** The state from which a generation of a Message starts. For the opening Narration, it is the Swipe state of its first Swipe, which is the Game's starting state. For every other Message, it is the Swipe state of the active Swipe of the Message before it. [D-02; T22 R2 Q5]

**Current state.** The Swipe state of the active Swipe of the last Message. [D-01; T22 R2 Q5]

**Pre-analysis state.** The Swipe state that the new Swipe of an analysed generation holds from the save of its text until its analysis updates it: the start state, with no trigger context. For an event Narration, its Trigger is also marked fired if it is one-shot. When a stop or a restart cuts the analysis off, the Swipe keeps this state. [D-04; T22 R3 Q1; OR-53]

**Work.** The engine work that one accepted request starts: an Action, a Retry or a Retrigger. One work can run more than one generation and at most one Options run.

**Work entry.** The record in memory that says that a Game is busy. It holds the work's id, the name of the step that runs, and a way to cancel the LLM call that runs. [D-05]

**Save unit.** One write to storage that lands whole or not at all (§3.6 of the spec). [D-04]

**Dock record.** The record of a Game that holds the offered set and one notice field. [D-06]

**Event record.** The trigger context from which an event Narration was written: which Trigger fired, its name and its direction. The event Narration's Message keeps it. [D-14]

**Snapshot.** This word stays the term for today's code: a saved copy of the whole state, with the status in it. Today one free-text Action saves about eight Snapshots, and only two of them belong to a Message (research 04; research 11, finding 9). The design has no Snapshot.

## 2. Records

### 2.1 What storage holds

| Record | Holds | Written by |
|---|---|---|
| Game | The World, the Persona, the prompt presets, the always-on toggle, the names. The **failure line**: none, or a failure kind with its raw text. | create, reset, the accept step of work, the work, stop, swipe switch, delete |
| Dock record (one per Game) | The offered set (a list, which can be empty) and the notice field (none, or a reason line with its raw text). | the work, swipe switch, delete, stop, create, reset |
| Message | The Game, the place in the story, the kind (typed Input, Impersonate Input, Narration, event Narration), the active Swipe. An event Narration also holds its event record. | create, reset, the work, delete |
| Swipe | The Message, the number, the text, the stored inputs (the guide, the impersonate direction, the impersonated mark), the maker (the World's opening text, the player, or the narrator), the room report (a room or none), the Swipe state. | create, reset, the work (text save and state update), edit (text only) |

No other record holds game state. No saved record holds the busy mark or the step name. [D-01; D-05]

### 2.2 What memory holds

| Record | Holds | Lost at a restart |
|---|---|---|
| Work entry (at most one per Game) | The work id, the step name, the cancel handle | yes, so no Game is busy after a restart (R-92) |
| Per-Game guard | Whether a short check-and-write for that Game runs now | yes |
| Active-Game pointer | Which Game is active | yes. The process sets it at start, as today's code does: the newest Game of the configured World (code, `bootstrap::init_game::resolve_game_id`). This is outside the spec. [D-26] |

### 2.3 Values that are derived, not stored

| Value | Rule | Requirements that the rule makes true |
|---|---|---|
| Current state | The Swipe state of the active Swipe of the last Message | R-36, R-39, R-47, R-76 (state part) |
| Start state | §1. It cannot change while a later Message exists, because only the last Message can switch its Swipe (R-35) and an edit changes no state (R-75). | R-43, R-55, R-58 |
| Busy | A work entry exists for the Game | R-72, R-92 |
| Status | If busy: the step name of the work entry. Else, if the Game holds a failure line: the failure line. Else: idle. | R-10, R-21, R-37, R-71, R-72 |
| Retrigger allowed | The last Message is a Narration that is not an event Narration, or an Impersonate Input, and the current state holds a trigger context | R-17, R-36, R-62, R-65 |
| One-shot Trigger fired | The Trigger is in the fired set of the start state of the generation that evaluates it | R-16, OR-13 |
| Room shown on a Message | The room report of its active Swipe | R-23, R-25 |
| Event header | The Trigger name in the Message's event record | R-13 |

## 3. Writes

### 3.1 One writer per Swipe state

Only the work that makes a Swipe writes its Swipe state. It writes it at most twice: the pre-analysis state at the text save, and the analysed state at the state update (§4.3). After that work ends, nothing changes it. An edit changes the text of a Swipe and nothing else (R-75), and an edit is refused while the Game is busy (R-70). A swipe switch and a delete change which Swipe state is current. They do not write a Swipe state. So no writer can overwrite game state that another writer saved (R-73). [D-03]

### 3.2 The save units

| Unit | Writes | Requirements |
|---|---|---|
| Create, reset | The Game record; an empty dock record; the opening Narration with one Swipe: the World's opening text (which can be empty), the starting state (the starting room; the starting Characters, each met once and met now; no added rooms; no fired Triggers; no trigger context), and the starting room as its room report. A reset first removes the active Game with all its records. | R-25a, R-39, R-77, R-78, OR-06, OR-43, OR-46 |
| Accept | The failure line becomes none. (The work entry is made in memory under the same guard.) | R-21a, D-20 |
| Accept refused with 500 | The failure line names the missing data. No work entry. | R-46, R-91 |
| Input save (free text) | The Input Message with one Swipe: the command text, maker the player, the current state as its Swipe state, no room report. | R-02, R-22, R-39 |
| Text save | A new Message, or a new Swipe on the last Message, made active. Its text, its stored inputs, its first Swipe state (the pre-analysis state; for a user regen, the start state) and its room report (none; for a Swipe of the opening, the starting room). When the new text is a Narration of any kind: the offered set becomes empty and the notice becomes none. When it is an Input (Impersonate, user regen): the dock record does not change. | R-11a, R-13, R-25a, R-30, R-32, R-41, R-64, R-85, R-90 |
| State update | The Swipe state and the room report of the Swipe that the text save made. | R-11, R-11a, R-22, R-23 |
| Failure line | The failure kind and raw text of a step that failed. | R-17, R-19, R-20, R-21 |
| Options save | Success: the new offered set; the notice becomes none. Failure or no Options call after a Narration: the offered set stays empty, and the notice holds the reason. Failure or no Options call for `/options`: the offered set stays as it was, and the notice holds the reason. | R-28, R-29, R-33, R-83, R-85, OR-41 |
| Swipe switch | The active Swipe of the last Message; the offered set becomes empty; the notice becomes none; the failure line becomes none. | R-35, R-37, R-38, R-21a, R-85 |
| Delete | The last Message and its Swipes go; the offered set becomes empty; the notice becomes none; the failure line becomes none. Refused when the story holds one Message. | R-76, R-21a, R-85 |
| Edit | The text of the active Swipe of one Message. | R-75 |
| Stop | The failure line becomes none, and the notice becomes none. When busy, the work entry goes in memory under the same guard. | R-81, R-85 |

The dock shows the notice when the notice field holds one. Else it shows the offered set (Opus §7).

### 3.3 One writer at a time

- **Every write names its Game.** A request reads the active-Game pointer once, when it arrives. The request and its work use that Game id for every write. Nothing reads the pointer again. (Today every write targets one process-wide Game id, read at the time of each storage call: research 05 §0, known.) [D-09; R-74, R-90]
- **The per-Game guard.** Every request that writes a Game, and every save unit of a work, takes that Game's guard. It holds the guard for one check and one save unit. It never holds the guard across an LLM call. [D-07; R-73]
- **The work id check.** Each save unit of a work checks, under the guard, that the Game's work entry still holds the work's id. If not, the unit is refused, and the work ends with no further write. A stop and a reset remove the work entry under the guard. So their order against a save unit is always clear: the unit lands before, and stays, or it is refused. [D-08; R-74a, R-81]
- **Busy refusals.** While a Game is busy, an Action, a Retry, a Retrigger, an edit, a delete and a swipe switch for that Game answer `200` "Still thinking..." and write nothing. This check comes first, before every `400` or `500` check. [R-70, OR-29, OR-33]
- **Requests that do not wait.** A create and a switch do not write the busy Game. A reset removes it. A stop ends its work. [R-74a; E-05]

### 3.4 Writers

| Writer | When it may write | What it writes |
|---|---|---|
| Create, reset | always | §3.2 |
| Switch | always | the active-Game pointer only |
| Accept (Action, Retry, Retrigger) | Game idle | work entry; failure line |
| The work | its work entry holds its id | Input save, text saves, state updates, failure line, Options save |
| Swipe switch, delete, edit | Game idle | §3.2 |
| Stop | always | §3.2, and the work entry goes |
| Restart | — | nothing. Memory is lost. |

## 4. Work

### 4.1 Accepting a request

1. Read the active-Game pointer once. No Game: `400` (R-45), or "failed" for a stop.
2. Take the guard. When a work entry exists: `200` "Still thinking..." (R-70).
3. Run the `400` checks of the request (R-27, R-44, R-65).
4. Run the `500` checks: the data that the work needs exists and can be read (R-46, R-91; §4.5).
5. Make the work entry, with its first step name. Write the accept unit. Give back the guard.
6. Answer `200` at once (R-09). Run the work in the background.

### 4.2 The plans

A plan is the order of steps that one request runs. This table is behaviour. It does not say how the code holds it (T22 R2 Q4).

| Request | Steps, in order | Options run at the end |
|---|---|---|
| Free text | room check; Input save; Narration generation; event generation, if a Trigger fired | when a Narration was saved and the toggle is on |
| Continue, `/guide` | Narration generation; event generation, if a Trigger fired | same |
| `/impersonate` | Impersonate generation; event generation, if a Trigger fired | when an event Narration was saved and the toggle is on |
| `/options` | Options run | — |
| Retry | the generation of §4.5, as a new Swipe on the last Message; event generation as a new Message, if a Trigger fired | when a Narration was saved and the toggle is on |
| Retrigger | event generation as a new Message | when it was saved and the toggle is on |

- When a step fails, the generations after it do not run. The Options run still runs when a Narration was saved (R-31, OR-36).
- With the toggle off, there is no Options run. The text save of the Narration already emptied the offered set (R-32).
- A Retry, a Retrigger and an event generation add no Input.
- An offered option that the player clicks is free text, also when it starts with "/" (R-06, OR-40).

### 4.3 The steps of one generation

Every generation of every kind runs these steps. The kind changes the inputs, and which steps it skips.

| Step | Status shows | What it does |
|---|---|---|
| 1. Room check | "narrating" ("generating-event" for an event) | The room of the start state must be on the Game's map, with its added rooms. If not: failure line "invalid room", nothing saved (R-20). |
| 2. Narrator call | "narrating" or "generating-event" | Prompt assembly gets the inputs of §4.6. Failure or empty text: failure line, nothing saved (R-19, R-49). |
| 3. Text save | "narrating" or "generating-event" | One save unit (§3.2), with the work id check. The story log shows the text from now on (R-11a). A user regen ends here (R-18). |
| 4. Analysis | "quantifying" | A failed or unusable answer gives the fallback result (R-12). |
| 5. World rules | "quantifying" | Applied to the start state. For an event generation, the movement is taken out of the analysis first, and a Trigger that the rules report is ignored (R-14, R-57; D-12). A contradiction: the Swipe keeps its pre-analysis state, and the failure line is written (R-20, OR-45). |
| 6. State update | "quantifying" | One save unit (§3.2), with the work id check. |

**The Swipe state that each kind leaves at the end of its generation** [D-13]. Before the state update, an analysed kind holds its pre-analysis state (§1).

| Kind | Swipe state | Trigger context | Fired set | Room report |
|---|---|---|---|---|
| Narration, Impersonate | the world rules' new state | the Trigger that fired, or none | not changed | the room that the world rules moved the player to, or none. The opening: the room after it (R-25a). |
| Event Narration | the world rules' new state, with no movement | none | the event's Trigger is added, if it is one-shot | none (R-24) |
| User regen (no state update) | the start state | as in the start state | not changed | none |
| Typed Input (Input save, no analysis) | the current state | as in the current state | not changed | none |

So a one-shot Trigger counts as fired only from its event Narration onward. A delete of that event Narration gives back a state in which it did not fire (R-16, OR-05, OR-13). Each analysed generation leaves its own trigger context (R-17a, OR-26).

**After the state update:** when the world rules found a Trigger, the event generation starts. Its start state is the Swipe state that the update saved. Its event record is that Swipe state's trigger context (R-13). When the event generation fails, the failure line says that the event narration failed. The Narration that fired the Trigger stays, with its trigger context, so Retrigger is allowed on it (R-17, R-67).

### 4.4 The states of one generation

| Where the generation is | Status | Saved so far by this generation | After a stop or a restart |
|---|---|---|---|
| Room check, narrator call | "narrating" or "generating-event" | nothing | nothing of this generation. A stop cancels the call. |
| Text save | "narrating" or "generating-event" | the unit, whole or not at all | if a stop takes the guard first, the unit is refused, and nothing stays. Else the text stays. |
| Analysis, world rules | "quantifying" | the text, with its pre-analysis state and its dock change | the text stays, with its pre-analysis state: the player has not moved, and no Trigger fired. A Retry can redo it. |
| State update | "quantifying" | the update, whole or not at all | if a stop takes the guard first, the update is refused, and the pre-analysis state stays. Else the analysed state stays. |
| After the state update | the next step | the text, with its analysed Swipe state and room report | all of it stays |

The work has the same states one level up. Between two generations, and during the Options run, everything that an earlier step saved stays after a stop or a restart (R-81, R-93, OR-51, OR-53). After the last step, the work removes its work entry under the guard, if the entry still holds its id.

### 4.5 Retry, by the kind of the last Message

| Last Message | Generation | Start state | Stored inputs used | Data that must exist (else `500`) |
|---|---|---|---|---|
| Opening Narration | Narration | the Swipe state of its first Swipe (the starting state) | none | that Swipe state |
| Other Narration (after an Input, a continue, a guided Narration) | Narration | the Swipe state of the active Swipe of the Message before | the active Swipe's guide, if any | that Swipe state |
| Event Narration | event | the same | the Message's event record | that Swipe state; the event record |
| Impersonate Input | Impersonate | the same | the active Swipe's direction; the Game's impersonate preset as it is now | that Swipe state; the direction |
| Typed Input | user regen | the same | the text of the active Swipe as it is now | that Swipe state |

- The redo reads the story before the last Message, with every edit (R-51).
- A Retry on an empty story answers `400` (R-44). After a Game starts, its story is never empty (R-76).
- The new Swipe is made active in its text save (R-41). A failed redo adds no Swipe, and the current state stays as the request found it (R-47, R-49). A redo that was saved stays when a later step fails (R-48).
- Sources: R-40 to R-61; Opus §5; Marinara: a regenerate is grounded in the state before the Message that it replaces (`resolveRegenerationGameStateAnchor`).

**Retrigger.** The `400` checks of R-65. Start state: the current state. Event record: the current state's trigger context. The event Narration is a new Message at the end of the story (R-62 to R-67).

### 4.6 What each LLM call reads (a free point of the spec)

| Call | Reads | Source |
|---|---|---|
| Narrator | The World, the Persona, the start state, the story before the place where the new text goes (the active Swipe text of each Message, with its kind), the stored inputs, the event record for an event, the Game's preset for the kind of text. Prompt assembly drops the oldest Messages to fit the connection's budget. | code (`fit_messages_to_context`); GLM A1 |
| Analysis | The new text; the player's text that led to it (the Input before it, if any); the room of the start state; the Characters that can be in that room; the 4 most recent Messages before the new text | code (Quantifier window of 4; AD-12) |
| Options | The current room, the Characters present, the 4 most recent Messages | code (Options window of 4; AD-12) |

No call gets the dock record, a notice, the failure line or the status. Notices are not Messages, so no story window can hold one (R-82, R-84). [D-17]

### 4.7 The failure lines (a free point of the spec)

One short line per failure kind. The raw text goes in the disclosure (R-21). The words come from today's code where a line exists (`generation_failure_summary`). A line changes where the spec asks for more, or where today's words use the deprecated word "turn". [D-18]

| Kind | Line | Source |
|---|---|---|
| No answer | "The language model could not be reached." | code |
| Prompt too long | "The story was too long for the language model." | code, changed: "turn" |
| Unusable answer | "The language model's answer could not be read." | code |
| Empty text | "The language model gave an empty answer." | new, R-19 |
| Output budget spent | "The model used its budget on reasoning. Try another model." | code |
| Preset missing | "The active prompt preset is missing." | code |
| Invalid room | "The room <name> is not on the map." | new, R-20. Today: "The current scene could not be found." |
| World-rules contradiction after the text | "The text was saved, but the game state could not be updated." | new, R-20, OR-45 |
| Event narration failed | "The event narration failed." The disclosure also holds the line of the narrator failure. | new, R-17 |
| Missing data for a Retry or a Retrigger | "The data for this request is missing: <what>." | new, R-46 |
| Save failed | "The new text could not be saved." | code, changed: "turn" |
| Internal error | "The engine failed with an internal error." | OR-47. Today: "The last turn failed to generate." |

## 5. Stop, restart, switch and reset

**Stop** (`POST /status/reset-generating`) [D-10; D-11; R-81; OR-51; OR-52; OR-53]:

- Busy: under the guard, remove the work entry and write the stop unit. Cancel the LLM call that runs, when its connection can cancel it. In all cases, the work id check refuses the work's later saves, so a late answer changes nothing. Answer "reset". The Game is idle at once, and the status shows no failure.
- What stays: every save unit that landed before. Examples: the Input of a free-text Action; a Narration whose analysis ran when the stop came, with its pre-analysis state; the Narration that fired a Trigger, with its trigger context, so Retrigger is allowed on it; after a Narration, an empty offered set; for an `/options` run, the offered set as it was.
- Idle: write the stop unit. Answer "reset".
- No Game: answer "failed".

**Restart** [D-05; D-24; R-92; R-93]: memory is lost, so no Game is busy and no work runs. Every save unit that landed stays. The failure line and the dock record stay (R-21a, R-85). The engine needs no start-up pass and no "heal" step.

**Switch and create while busy** [R-74a]: the work goes on for its own Game, because its writes name that Game (§3.3). When the player comes back, that Game shows its status.

**Reset while busy** [R-74a]: the reset removes the Game and its work entry under the guard. The work's next save unit is refused, and the work ends with no write.

## 6. What each view reads

| View | Reads |
|---|---|
| Story log | The Messages in order: the active Swipe text, its room report, the event header, the Swipe count. Delete when the story holds more than one Message. Retrigger when §2.3 allows it. |
| Dock | The notice, if the dock record holds one; else the offered set. Controls off while busy. |
| Status line, `GET /status/generating` | The status rule of §2.3 |
| Debug route | Busy (§2.3) |
| Input box | The Stop button while busy |
| Header banner | The LLM call log: the newest call of each role, across all Games (R-87, OR-50; code, `GameViewQuery::role_health`) [D-23] |

## 7. Trade-offs and assumptions

- **A full Swipe state per Swipe.** Storage grows with the number of Swipes times the size of the state. The state is small: rooms, Characters, encounter counts, Trigger ids (inferred). In return, R-36, R-39, R-55 and R-76 hold with no replay and no restore step. Marinara makes the same trade: one state row per message and Swipe (research 14). [D-01]
- **The text shows at once, and the state follows.** The story log shows a new text as soon as the narrator call returns (R-11a). Its room report and its game state follow after the analysis, a few seconds later. So one writer writes the Swipe state twice (D-03). A stop or a restart during the analysis leaves a text whose analysis did not run: the player did not move, and no Trigger fired. The player can Retry it, or play on.
- **Memory-only busy.** A second process would not see the busy mark. E-01 says that one process uses the database.
- **Cancel.** The design does not need the LLM connection to support cancel. The work id check makes a late answer harmless. Cancel only saves the cost of the call.
- **Window of 4.** Today's value. The spec says "a short window" and gives no size.

## 8. Spec observations

These do not change the design. Ticket 23 or the plan can take them up.

1. **R-76: "the opening Narration too".** The opening is always the first Message. A delete removes only the last Message, and never the only Message. So a delete can never remove the opening (inferred).

## 9. Decision log

Ticket 23 compares each row with the code.

| # | Choice | Source | Rejected |
|---|---|---|---|
| D-01 | Each Swipe holds a full Swipe state. The current state is derived. No live game-state record exists. | R-39; T22 R1 Q1; Opus §2.6; Kimi (a state per Swipe); Marinara (state per message and Swipe); premise: research 12 §4c (most fixed defects came from pointer timing) | A live state plus copies, which each swipe switch, delete and stop must copy back (GLM; code) |
| D-02 | The start state is derived from the Message before. The opening's start state is its first Swipe's Swipe state. | R-35, R-39, R-43, R-55; Opus §5; Marinara (`resolveRegenerationGameStateAnchor`) | A stored state-before record per Message (GLM). It repeats what the Message before holds. |
| D-03 | Only the work that makes a Swipe writes its Swipe state: at the text save, and at the state update. After that work ends, nothing changes it. | R-70, R-73, R-90; inferred from D-01 and D-04 | — |
| D-04 | A generation saves its text at once, with the pre-analysis state. After the analysis and the world rules, a state update saves the analysed state on the same Swipe. A stop or a restart between the two keeps the text with its pre-analysis state. | T22 R3 Q1; OR-53; R-11a, R-90; GLM (save before the analysis); code (the Narration is saved before the Quantifier, research 04) | Save once at the end (Opus; Kimi), which shows the text one Quantifier call late; show the text from memory until the end; save first, and delete the text on a stop |
| D-05 | The busy mark, the step name and the cancel handle live in memory only. The failure line is on the Game record. | R-21a, R-72, R-92, E-01; T22 R2 Q3; Opus | A saved busy mark with a start-up pass (Kimi); the status inside each saved state (code: F-6, F-7) |
| D-06 | A dock record per Game: the offered set and one notice field. The dock shows the notice in place of the set. | R-29, R-33, R-38, R-83, R-85, OR-49; Opus §7 | A notice list next to the story log (GLM); notices as Messages (code) |
| D-07 | A per-Game guard, held for one check plus one save unit, never across an LLM call. | R-70, R-73, E-03; Opus §2.7 (a per-Game lock) and §6 (W3); Kimi (per-Game lock); Marinara (one generation per chat, registered at once) | — |
| D-08 | Each save unit of a work checks the work id. A stop and a reset remove the work entry. | R-74a, R-81; Kimi (work id check); Marinara (identity-checked end of an aborted run) | Reading the busy mark only at the start (code: research 05, pairs 2–9) |
| D-09 | A request reads the active-Game pointer once. Every write names the Game that it captured. | R-74, R-90; all three clean designs | One process-wide Game id, read at each storage call (code: research 05 §0) |
| D-10 | A stop drops only the step that runs: a narrator call, an analysis or an Options run. What the work saved stays. | OR-51, OR-52, OR-53; T22 R2 Q2; T22 R3 Q1 | Undo the whole request (OR-31 as written); undo every generation but keep the Input |
| D-11 | A stop cancels the LLM call when it can. The work id check makes a late answer harmless in all cases. | R-09, R-81; ST (`stopGeneration`); Marinara (`POST /api/generate/abort`) | — |
| D-12 | For an event generation, the movement is taken out of the analysis before the world rules, and a Trigger that they report is ignored. | OR-15 (free); GLM §6.1g; Kimi §5.4; Opus A-2 | Tell the Quantifier not to report a movement (a prompt rule that a model can break) |
| D-13 | The Swipe state of each kind, as in the table of §4.3 | R-16, R-17a, R-18, R-22, R-24, R-25a, R-39; OR-05, OR-26 | — |
| D-14 | The event Narration's Message keeps its event record. | R-54, R-56; Opus (event record) | Reading the trigger context from the state again (fails after a delete, R-56) |
| D-15 | Every Narration save empties the offered set and clears the notice. An Input save leaves the dock record. | R-30, R-31, R-32, R-85 | — |
| D-16 | The plans of §4.2 and the steps of §4.3 are the behaviour. The code shape is the plan's choice. | T22 R2 Q4 (Option C) | One step runner (Option A); one function per request (Option B, code) |
| D-17 | What each LLM call reads, as in §4.6 | free point; code; GLM A1 | A window of 8 (GLM A1) |
| D-18 | The failure lines of §4.7 | free point; R-21; code (`generation_failure_summary`); T22 R3 Q2 (accepted) | — |
| D-19 | The room report is on the Swipe. It is written with the Swipe state, at the text save and at the state update. | R-22 to R-25a; T-03; code (commit `72cfffdc` put the arriving room on the Swipe) | A room report taken from the Swipe state (it cannot tell "moved to the same room" from "did not move") |
| D-20 | The accept unit clears the failure line in storage. | R-21a. Inferred: else a restart during new work shows the old failure again. | Clear it at the end of the work |
| D-21 | A `500` check failure writes the failure line and makes no work entry. | R-46, R-91 | — |
| D-22 | The terms Swipe state, start state, current state and pre-analysis state. "Snapshot" stays the term for today's code. | T22 R2 Q5; T22 R3 Q1 (pre-analysis state) | Snapshot with a new meaning; no noun |
| D-23 | The header banner reads the LLM call log. | R-87, OR-50; code (`GameViewQuery::role_health`) | — |
| D-24 | No start-up pass and no "heal" step. | follows from D-05; premise: F-7 | A start-up pass that clears busy marks (Kimi) |
| D-25 | One create unit for create and reset. No LLM opening. | R-77, OR-06, OR-46, R-25a | An LLM opening at each start (code; S-3, AD-10) |
| D-26 | The active-Game pointer stays as today. | outside the spec; code (`resolve_game_id`) | A saved Active-Game record (Opus §2.3) |

## 10. The 36 known problems

The rows of [research 16](research-16-clean-room-collapse.md) §2, and row W. Values:

- **settled:** the design states behaviour that excludes the problem;
- **moot:** the thing that has the problem does not exist in the design;
- **open:** the design does not decide it. The reason is "outside" (outside the lifecycle, the same as "untouched" at scope B in research 16) or "code shape" (the owner left the code shape to the plan, T22 R2 Q4).

| ID | Value | How the design treats it |
|---|---|---|
| AD-01 | open (outside) | Storage is a black box. The design needs only two things from it: a write names its Game, and a save unit lands whole (§3.6 of the spec). |
| AD-02 | open (outside) | The storage seam. |
| AD-03 | open (outside) | The World, Persona, Settings and Preset services. |
| AD-04 | open (code shape) | The design has one order of steps per plan (§4.2) and one set of generation steps (§4.3). How the code holds them is the plan's choice (D-16). Research 16 had this cell as moot. It is open because the owner chose to leave the code shape out. |
| AD-05 | moot | The pointer rule and the Retry anchor do not exist. The start state is derived (D-02). The record load and the edits are plain reads and writes. |
| AD-06 | open (outside) | Prompt assembly. No arrival exists, so the only production caller of `build_narration_prompt` goes (D-25). |
| AD-07 | open (outside) | Where an Agent gets its preset. |
| AD-08 | open (code shape) | The design has one work entry and one guard per Game (D-05, D-07). How many files hold them is the plan's choice. Research 16 had this cell as moot. |
| AD-09 | moot | No saved record holds the status. The step name is in the work entry, and the failure line has the units of §3.2 as its only writers (D-05). |
| AD-10 | moot | No arrival generation exists (D-25). |
| AD-11 | open (code shape) | The design has one status rule (§2.3). The copies of the branch are code shape. |
| AD-12 | open (outside) | The Agents are black boxes. The design keeps the window of 4 (D-17). |
| AD-13 | open (outside) | The Agent registry. |
| AD-14 | open (outside) | An HTTP view. |
| AD-15 | open (outside) | The HTTP error policy. §4.7 adds failure kinds that the views must show. |
| AD-16 | open (outside) | The composition root. The design has one create unit (D-25). |
| AD-17 | open (outside) | The LLM transport. |
| T-01 | settled | §2 to §5 state the records, the states of a generation and of a work, and the writer rule. Ticket 01 can take this design as its candidate. |
| T-02 | settled | Notices are in the dock record, not in the story (D-06). No call reads the dock record (§4.6). The only notice is the R-83 notice (R-86). |
| T-03 | settled | The room and the room report are both on the Swipe, and the same save units write both (D-01, D-19). A swipe switch gives back the room of the active Swipe. |
| S-1 | settled | The analysis updates the Swipe state of its own Swipe (D-04). Today the pointer never moves after its first save (research 04, R2). A Swipe keeps its pre-analysis state only when a stop or a restart cut its analysis off. |
| S-2 | settled | A re-impersonate runs the analysis and the world rules (R-60; §4.5). |
| S-3 | moot | No arrival exists (D-25). The create unit writes the opening before any work can start. |
| F-1 | settled | The accept step makes the work entry under the guard (§4.1; D-07). |
| F-2 | settled | Each step sets its own step name (§4.3; R-71). |
| F-3 | settled | A save unit that fails ends the work with the "could not be saved" line. No part of it lands (§4.7). |
| F-4 | open (outside) | The room lookups build the inputs of the narrator call and the analysis, which are black boxes. The design has one room check (§4.3, step 1). |
| F-5 | settled | The start state of a guided Retry is the state of the Message before (D-02). |
| F-6 | settled | A Swipe state holds no status (D-05). |
| F-7 | settled | The busy mark is in memory only, so a restart cannot leave a stale one (D-05, D-24). |
| F-8 | settled | An event redo starts from the Swipe state of the Message that fired the Trigger (§4.5; R-55). The fixture drift is test code, which the design does not cover. |
| F-9 | settled | A failure goes to the failure line, never into the story (§3.2; R-17, R-84). |
| F-10 | settled | The room report is written with the Swipe state of its own Swipe (D-19). |
| O-1 | settled | Each Swipe state holds the trigger context of its own generation (D-13; R-17a). |
| O-2 | settled | A Retry starts from the state of the Message before the last Message, never from an earlier Input (D-02; R-40). |
| O-3 | settled | The `500` check reads the same records that the redo uses (§4.5; D-21). |
| W (extra) | settled | Pairs 2–4: every write names its captured Game (D-09). Pair 5 (`/status/reset-generating` frees the slot of a running task): the stop removes the work entry, and the work id check refuses the work's later saves (D-08). Pairs 6–7 (pre-claim copies, heal): a Swipe state has one writer, the work that made its Swipe (D-03), and no heal exists (D-24). Pairs 8–9: an edit, a delete and a swipe switch are refused while busy (R-70), under the guard (D-07). |

**Counts** (36 rows; W is not counted, as in research 16):

| Value | This design | Research 16 at scope B |
|---|---|---|
| settled | 17 | 18 |
| moot | 4 | 5 |
| open | 15: 12 outside, 3 code shape | 0 |
| untouched | — | 13 |

The differences: S-3 moves from settled to moot, because arrival goes (D-25). AD-04 and AD-08 move from moot to open (code shape), because the owner left the code shape to the plan (D-16). AD-11 was untouched, and is open (code shape) here. The other 12 untouched rows are open (outside).
