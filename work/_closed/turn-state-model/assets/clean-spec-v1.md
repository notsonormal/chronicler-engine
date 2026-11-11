# Behaviour spec: the game-state lifecycle

## 0. About this spec

This spec states what one part of an interactive fiction engine must do. The engine tells a story with LLM calls. The part is the **game-state lifecycle**: what each player request does to a Game's story and to its game state, what a Retry and a swipe switch give back, what may change a Game while the engine works on it, and what the player sees when a step fails.

The spec states behaviour that a player or an HTTP client can observe. It does not state how the engine gets that behaviour.

- **Requirements** have the form R-nn (§4). The numbers have gaps between groups. Each requirement ends with its sources in square brackets. The sources are for traceability only. You cannot read them, and you do not need them.
- **Open questions** have the form OQ-nn (§6). No source gives that behaviour. This spec does not answer them.
- **Black boxes** (§3) are parts outside the lifecycle. A design uses them through their contracts and does not change them.
- **Environment constraints** (§5) are facts about where the engine runs.

Source labels:

| Label | Meaning |
|---|---|
| `[Actions 1.4]` | Scenario 1.4 of the feature spec "Actions". The other feature specs are Retry, Retrigger, Swipe Switching, Options, Story Log, Reset, Games, Dashboard and Failure Display. `I.n` is a property of a feature spec. |
| `[PD-n]` | Prior decision n of the project's decision log. |
| `[owner, …]` | A ruling of the product owner. |
| `[Glossary: …]` | The project glossary entry of that term. |
| `[doc: …]` | A project design document. |
| `[general]` | A correctness property that needs no ruling. |

## 1. Terms

Use these words with these meanings. This spec does not use the word "turn".

**World.** An authored template that does not change during play. It has a map of rooms, Characters, the Triggers of each Character, and the opening of a new Game: a starting room, the starting Characters, and an opening text. The opening text can be empty. [Glossary: World, Scenario]

**Persona.** The character that the player plays. It is fixed for the life of a Game. [Glossary: Persona]

**Character.** A non-player character of the World. [Glossary: Character]

**Game.** One playthrough of one World with one Persona. A Game has a story and a game state. Many Games are stored. One Game at a time is the **active Game**. [Glossary: Game]

**Story.** The ordered list of a Game's Messages. The story log on the dashboard shows it.

**Action.** A command from the player, sent through `POST /action` (R-01). [Glossary: Action]

**Guide.** A short text that steers what the narrator writes in one generation. The player gives it with `/guide`. It does not become an Input. [Glossary: Guided Generation; doc: AI Steering]

**Impersonate.** The narrator writes the next text as the player's Persona, from a short direction that the player gives with `/impersonate`. The text becomes an Input. [Glossary: Impersonate; doc: AI Steering]

**Message.** One entry in the story: an Input, a Narration or an event Narration. Every Message has one or more Swipes. [PD-6; Glossary: Message] Whether a notice is a Message is not decided (R-82, OQ-04).

**Input.** A Message that holds the player's words. The player types it, or the narrator writes it for the player (an Impersonate or a user regen).

**Narration.** A Message in which the narrator tells the story. An Input that the narrator writes for the player is not a Narration.

**Opening Narration.** The first Narration of a new Game. It holds the World's opening text. No LLM call produces it. [Reset 7.1; owner, 2026-10-12]

**Event Narration.** A Narration that a fired Trigger causes. The story log shows the name of the event on it. This name is the **event header**.

**Swipe.** One version of a Message. A Message shows one Swipe, its **active Swipe**. A Swipe keeps its text and the inputs that produced it: the guide, the impersonate direction, and whether it is impersonated. [PD-6; Glossary: Swipe, Guided Generation]

**Game state.** Everything about a Game that play changes, other than the story text:

- the room the player is in;
- the rooms that play added to the map;
- the Characters present in the scene;
- the encounter record of each Character: how many times the player met it, and whether the player meets it now;
- which one-shot Triggers fired;
- the stored trigger context;
- the offered set of Options.

**Status.** What the dashboard shows about the active Game's work: idle; busy, with the name of the step that runs; or the failure of the last generation.

**Generation.** The engine writes one piece of story text with one narrator call, and then does what that kind of text needs (§4.2). A generation adds one Message or one new Swipe. The kinds are:

- a Narration, for a free-text, continue or guided Action;
- an Impersonate Input;
- a user-regen Input;
- an event Narration.

**Busy.** A Game is busy from the moment it accepts a request that starts engine work (an Action, a Retry or a Retrigger) until that work ends. One Action can run more than one generation: a Narration, then an event Narration. An `/options` Action runs an Options run and no generation.

**Analysis.** The Quantifier call (§3.2). It reads new story text and reports the Characters present, a movement and a confidence. [Glossary: Quantifier]

**World rules.** The black box (§3.4) that applies an analysis to the game state and finds the Trigger that fires.

**Trigger.** An authored condition on a Character. When it fires, the narrator writes an event Narration from the Trigger's direction. A Trigger is one-shot or repeating (R-16). [Glossary: Trigger; Actions 1.8]

**Trigger context.** What the narrator needs to write the event Narration of a fired Trigger: which Trigger fired, and its direction.

**Options.** Short player inputs that the engine offers. The **offered set** is the current list. The dock on the dashboard shows it. [Glossary: Options]

**Always-on toggle.** A setting of a Game. When it is on, the engine makes a new offered set after each Narration. A Game takes the setting from its World when the Game is created. [Options 24.6]

**Retry.** The player's request `POST /swipe/new`. It redoes the last generation as a new Swipe. [PD-6; Glossary: Retry]

**Retrigger.** The player's request `POST /retrigger`. It writes an event Narration from the stored trigger context, as a new Message.

**Swipe switch.** The player's request to make another Swipe of the last Message its active Swipe.

**Notice.** A short report to the player about work that failed or fell back, when the request itself did not fail.

**Room report.** The name of a room that the story log shows on a Message.

**Restart.** The process stops, on purpose or by a crash, and starts again.

## 2. The HTTP interface

Each route below acts on the Game that is active when the request arrives, except the routes that carry a Game id.

| Route | Request | Answer |
|---|---|---|
| `POST /action` | an Action: `command=<text>` | starts the work and answers at once. `200` "Still thinking..." when the Game is busy (R-70) |
| `POST /swipe/new` | Retry | `200` at once when it starts. `200` "Still thinking..." when the Game is busy. `400` for a user error. `500` for a data-integrity failure |
| `POST /retrigger` | Retrigger | `200` at once when it starts. `200` "Still thinking..." when the Game is busy. `400` for a user error. `500` for a data-integrity failure |
| `POST /message/:id/swipe/:index` | swipe switch to Swipe `:index` of Message `:id` | the story log |
| `POST /history/:id` | edit: `text=<new text>` replaces the text of the active Swipe of Message `:id` | a short confirmation |
| `POST /history/delete` | delete the last Message | an empty `200` |
| `POST /reset` | remove the active Game and start a fresh one | the dashboard reloads |
| `POST /games` | create a Game from a World and a Persona. It becomes the active Game | the dashboard reloads |
| `POST /games/:id/switch` | make Game `:id` the active Game | the dashboard reloads |
| `POST /status/reset-generating` | make the active Game idle (OQ-07) | "reset" or "failed" |
| `GET /status/generating` | the status | "idle", the name of the busy step, or a failure line |
| `GET /fragment/story-log` | the story log | — |
| `GET /fragment/options-dock` | the dock | — |
| a debug route | whether the active Game is busy | "true" or "false" |

## 3. Black boxes

### 3.1 The narrator call

- **Inputs:** a prompt from prompt assembly (§3.5), and the Game's prompt preset for this kind of text. The call uses the preset that is active for the Game when the call starts [Retry 22.4].
- **Kinds of text:** a Narration; an Impersonate Input, which the narrator writes as the Persona; a user-regen Input, which is a new version of the player's own Input; and an event Narration, from a trigger context.
- **Result:** one text.
- **Failures:** no answer (a timeout, a transport failure or a non-2xx answer); the prompt is too long for the connection; an unusable answer (empty or not parseable); the model spent its whole output budget before it wrote an answer; the Game's preset is missing. Each failure has a raw text. [Actions 2.2, 2.3; Failure Display 38.3, 38.5, 38.7]
- The call writes its own record to an LLM call log. A design does not do this work.

### 3.2 The Quantifier call (analysis)

- **Inputs:** the new text; the player's text that led to it, if any; the current room; the Characters that can be in that room; and a short window of recent story.
- **Result:** the Characters present in the room; a movement, which is a destination or none; and a confidence (high, medium or low).
- **Failures:** a failed call or an unusable answer gives a fallback result: no movement, and the Characters that the scene holds now.

### 3.3 The Options call

- **Inputs:** a short window of recent story, the current room and the Characters present.
- **Result:** a list of short player inputs. A numbered list in the answer becomes plain texts.
- **Failures:** no answer, or no option could be read from the answer. Each failure has a raw text.
- **Not available:** a Game can have no Options call. When it has none, no call is made, and the player is told that (R-29, R-83).

### 3.4 The world rules

- **Inputs:** the game state, an analysis result, and the World's map and Characters.
- **Result:** a new game state, and at most one Trigger that fires.
  - A movement to a room on the map moves the player there.
  - A movement to a destination that the map does not have adds a new room to the Game's map and moves the player there.
  - The Characters present become the Characters of the analysis. A Character that comes into the scene is met one more time and is met now. A Character that leaves is no longer met now. [Actions 1.6]
  - A Trigger fires when its conditions hold in the new game state (a room, an encounter condition), and, for a one-shot Trigger, when it did not fire before. [Actions 1.4, 1.7, 1.8; Retry 9.5]
- A one-shot Trigger can fire again while no event Narration for it has been written.
- **Failures:** when the game state contradicts the World, the generation fails before any text is written, for example when the player's room is not on the map (R-20).

### 3.5 Prompt assembly

- It builds the prompt of each LLM call from: the World; the Persona; the game state; the part of the story that it is given; the player's text; the stored inputs of the generation (guide, impersonate direction); the trigger context; and the Game's preset for that kind of call.
- It renders whatever it is given, with the kind of each entry. It removes nothing, so text that must not reach an LLM must not be given to it (R-82).

### 3.6 Storage

- One database, used by one process [PD-17].
- It keeps Games, their stories (Messages and Swipes), and the records of game state that a design needs. The design chooses those records.
- It reads and writes for a named Game. It can take a unit of several writes, which lands whole or not at all.
- R-90 and R-91 state what the design must get from storage.

### 3.7 The HTTP views

- The story log shows each Message with its active Swipe, its room report, its event header and its Swipe count. Each entry has an Edit control. The Delete, Swipe and Retrigger controls are on the last entry only.
- The dock shows the offered set. Its controls are off while the Game is busy [Dashboard 39.3].
- The status line shows the status (R-21).
- The header banner reports a degraded LLM role. [Failure Display 38.1]
- The views read what the design gives them. A design states what each view reads.

## 4. Requirements

### 4.1 Actions

**R-01.** An Action has one of these kinds: free text; empty text (a continue); `/guide <text>`; `/impersonate <direction>`; `/options`. Other text that starts with "/" is free text. [Actions 1.9, 1.10, 1.12; doc: AI Steering]

**R-02.** A free-text Action adds one Input with the command text, and then one Narration. The Input is before its Narration in the story. [Actions 1.1, 1.2, I.2]

**R-03.** A continue adds one Narration and no Input. [Actions 1.5]

**R-04.** A guided Action adds one Narration and no Input. The guide steers that one Narration. The Narration's Swipe keeps the guide. [Actions 1.10; Glossary: Guided Generation; doc: AI Steering]

**R-05.** An Impersonate Action adds one Input. The narrator writes it as the Persona, from the direction. Its text is not the command text. Its Swipe keeps the direction and the mark that it is impersonated. [Actions 1.9; Retry 22.3; doc: AI Steering]

**R-06.** When the player uses an offered option, its text goes in as a free-text Action. [Options 24.13]

**R-07.** One Action adds at most one Narration of its own. An event Narration is a separate Message. [Actions I.3, 1.4]

**R-08.** Successive Actions keep each Input, in order, with its text. [Actions 6.1, 6.2, 6.3]

**R-09.** A request that starts work answers before the work ends. A slow LLM answer does not stop other requests. The Game stops being busy soon after the last call of its work returns. [Actions 3.3]

**R-10.** When the work ends, the status is idle or a failure. It is never busy. [Actions I.1; Retry I.1]

### 4.2 What a generation does

**R-11.** A Narration generation (free text, continue or guided) and an Impersonate generation are analysed, and the world rules apply the result. So each one can move the player, change the Characters present and the encounter record, and fire a Trigger. [Actions 1.4, 1.6, 1.14; PD-2]

**R-12.** When the analysis fails or falls back, the generation still ends well: the status is idle, and the Characters present stay as they were. [Actions 2.5]

**R-13.** When a generation fires a Trigger, the narrator writes an event Narration from the trigger context. The event Narration is a new Message after the text that fired the Trigger. It shows the event header. [Actions 1.4]

**R-14.** The analysis of an event Narration changes the Characters present and the encounter record. It never moves the player. [Actions 1.6; Retry 10.2, 10.5; Retrigger 13.2]

**R-15.** A generation that fires no Trigger adds no event Narration. [Actions 1.7]

**R-16.** A one-shot Trigger fires at most once in a Game. [Actions 1.8]

**R-17.** When an event Narration fails: the text that fired the Trigger stays in the story; no Message is added for the failure; the status reports that the event narration failed; the trigger context stays stored; and the story log offers Retrigger on the last Narration. [Actions 2.4; PD-13; Retrigger I.9]

**R-18.** A user-regen generation does not move the player. [Retry 22.2]

### 4.3 Narration failures

**R-19.** When the narrator call fails or gives empty text, no Message and no Swipe is added for that text. The status shows the failure. For empty text, the failure says that the text was empty. [Actions 2.2, 2.3, I.4; Retry 11.5, 11.6]

**R-20.** When the room the player is in is not on the map, the generation fails before any text is written. The status names the invalid room. [Actions 2.1; Retry 11.7]

**R-21.** The status shows a failure as one short line that the failure's kind selects, and keeps the raw text inside a disclosure. A missing preset and a spent output budget each have their own line. [Failure Display 38.3, 38.5, 38.7]

### 4.4 Room reports

**R-22.** A Narration that moves the player reports the room it arrives in. The Input of that Action reports no room. A Message whose generation did not move the player reports no room. [Actions 1.14; PD-14]

**R-23.** Each Swipe reports only the room that its own generation arrived in. A redo that does not move the player reports no room, also when a sibling Swipe reports one. [Retry 9.2, 9.8, 22.5]

**R-24.** An event Narration reports no room, on each of its Swipes. [Retry 10.8]

**R-25.** After a swipe switch, the Message reports the room of its new active Swipe. [Retry 9.8]

### 4.5 Options

**R-26.** `/options` makes a new offered set. The dock shows one button for each option. [Options 24.1]

**R-27.** `/options` on a Game whose story has nothing to work from fails at once. The answer names the reason, and the dock shows no set. [Options 24.2]

**R-28.** `/options` adds no Message to the story. A new set replaces the old set. [Options 24.4]

**R-29.** When an `/options` run fails, or the Game has no Options call, the offered set stays as it was. The player is told (R-83). [Options 24.5, 24.18]

**R-30.** The engine offers Options on demand, and after each Narration when the always-on toggle is on (R-31). A generation that writes an Input (Impersonate, Impersonate redo, user regen) leaves the offered set as it was. [Glossary: Options; Options 24.15]

**R-31.** With the always-on toggle on, the engine makes a new offered set after each Narration. [Options 24.6]

**R-32.** With the toggle off, a Narration empties the offered set. [Options 24.16]

**R-33.** With the toggle on, when the new set fails or the Game has no Options call, the offered set is empty after the Narration. The player is told (R-83). [Options 24.17, 24.20]

### 4.6 Swipes and the swipe switch

**R-34.** Every Message has at least one Swipe. A Swipe keeps the inputs that produced it. [PD-6]

**R-35.** Only the last Message can have its active Swipe switched. [doc: Storage Design]

**R-36.** "After a swipe switch, the game is as that Swipe's generation left it." [owner, ticket 17; PD-1]

**R-37.** After a swipe switch, the status is idle. [Swipe Switching 36.1]

**R-38.** After a swipe switch, the dock shows no offered set. [Swipe Switching 36.2] R-36 and R-38 disagree when the generation left an offered set: see OQ-01.

### 4.7 Retry: every kind

**R-40.** "A new Swipe redoes the last generation, with its stored inputs." [PD-6, as the owner confirmed it in ticket 17]

**R-41.** A Retry adds exactly one Swipe, to the Message that holds the last generation, and makes it the active Swipe. The Message stays the same Message. Each further Retry adds one more Swipe. [Retry 9.1, 10.1, 22.1, 22.2, I.5; Actions 6.2, 6.3]

**R-42.** A Retry never changes the text of a Swipe that exists. [Retry I.4]

**R-44.** When the story holds no Input, a Retry answers 400 and starts nothing. [Retry 11.1] See OQ-05.

**R-45.** When no Game exists, a Retry answers 400. [Retry 11.2]

**R-46.** When data that a Retry or a Retrigger needs is missing or cannot be read, the request answers 500. The status shows a failure that names what is missing. This is a data-integrity failure, not a user error (400). [Retry 11.3, 11.4, I.7; Retrigger I.7]

**R-47.** A failed Retry that added no Swipe leaves the room, the encounters and the fired Triggers as the Retry request found them. [Retry I.9, 11.9; PD-11; PD-16]

**R-48.** A failed Retry that added a Swipe keeps the world that its redo wrote. [Retry I.9, 11.10; PD-16]

**R-49.** When the narrator call of a redo fails or gives empty text, or the room is invalid, the Retry adds no Swipe, and the Swipes that exist stay as they were. [Retry 11.5, 11.6, 11.7]

**R-50.** A Retry never leaves the Game busy. This also holds for a Retry after a delete. [Retry I.1; Story Log 8.3]

### 4.8 Retry: by the kind of the last generation

**A Narration that answers an Input.**

**R-51.** The redo uses the Input's text as it is now, with any edit. The Retry does not change the Input. [Retry 9.3, 9.4, I.4]

**R-52.** The redo is analysed again. It can move the player, and the Triggers are evaluated again. [Retry 9.2, 9.5, 9.6]

**A guided Narration.**

**R-53.** The redo uses the stored guide. Its Swipe reports the room that the redo arrives in, or no room. [Retry 22.5; PD-6]

**An event Narration.**

**R-54.** The redo writes a new event Narration for the same Trigger, as a new Swipe on the event Narration. The Narration that fired the Trigger stays in the story. This also holds for an event Narration that a Retrigger wrote. [Retry 10.1, 10.4]

**R-55.** The redo's analysis applies to the Characters present and the encounter record as they were before the replaced event Narration was written. What the replaced event Narration added does not carry over. A swipe switch before the Retry does not change this. [PD-11; Retry 10.5, 10.7]

**R-56.** The redo uses the trigger context of the event that it redoes, also after later Messages were deleted. [Retry 10.6]

**R-57.** The redo never moves the player. [Retry 10.2; R-14]

**R-58.** When the redo fails: the status reports that the event narration failed; the story keeps the Narration that fired the Trigger; no Message is added for the failure; the player stays in the room where the Retry request found them; and a one-shot Trigger still cannot fire again. [Retry 11.8]

**An impersonated Input.**

**R-59.** The redo writes a new Impersonate Input, as a new Swipe on that Input, from the stored direction. It uses the Game's impersonate preset as it is now. The new Swipe keeps the direction and the mark that it is impersonated. [Retry 22.1, 22.3, 22.4]

**R-60.** The redo runs as a fresh Impersonate runs: its analysis can move the player, and it can fire a Trigger. [PD-2]

**An Input that the player typed (no stored inputs).**

**R-61.** The narrator writes a new version of the player's Input, as a new Swipe on that Input (a user regen). [Retry 22.2]

### 4.9 Retrigger

**R-62.** A Retrigger is allowed when a trigger context is stored and the last Message is a Narration that is not an event Narration. It writes a new event Narration from the stored trigger context, and adds it as a new Message at the end of the story. The story before it stays. [Retrigger 13.1]

**R-63.** A Retrigger starts from the game state as the request finds it. It never moves the player. Its event Narration changes the Characters present and the encounter record (R-14). [Retrigger 13.1, 13.2; doc: Game Flow]

**R-64.** A Retrigger adds exactly one story Message, and adds no Swipe. [Retrigger I.3]

**R-65.** A Retrigger answers 400 and starts nothing when: no trigger context is stored; the story is empty; the last Message is an Input; the last Message is an event Narration; or no Game exists. [Retrigger 14.1, 14.2, 14.3, 14.4, 14.6]

**R-66.** When the event Narration of a Retrigger fails: the status reports it; no Message is added; the story still ends in the same Narration; and the story log still offers Retrigger on it. [Retrigger 14.5]

**R-67.** A stored trigger context survives a failed event Narration. So a Retrigger can recover the event without a new Action. [Retrigger I.9]

### 4.10 One piece of work at a time

**R-70.** While a Game is busy, an Action of any kind (`/options` too), a Retry or a Retrigger for that Game answers `200` "Still thinking..." and starts nothing. [Retry 12.1, I.8; Retrigger 15.1, I.8]

**R-71.** While a Game is busy, the status names the step that runs: "narrating", "quantifying", "generating-event" or "options". [Dashboard 39.1; doc: Game Flow]

**R-72.** The status, the dock and the debug route report whether work runs for the Game now. Work that was cut off never makes a Game look busy. [Dashboard 39.1, 39.3; doc: Two State Channels]

**R-73.** No write is lost. A write by one writer never undoes a later write by another writer to the same Game. [general]

**R-74.** Each piece of work, and each request, writes only to the Game that it belongs to. A change of the active Game during the work does not move its writes to the new active Game. [owner, ticket 17; general]

### 4.11 Edit, delete, reset, create and switch

**R-75.** An edit replaces the text of the active Swipe of one Message. A later generation reads the edited text. [Retry 9.4]

**R-76.** A delete removes the last Message. The story log shows one entry fewer. The deleted text does not come back, and the next Action works as usual. [Story Log 8.1, 8.2, 8.4]

**R-77.** A reset removes the active Game and its story, and makes a fresh Game the active Game. The fresh Game's story holds only the World's opening Narration. The next Action adds the first Input. [Reset 7.1, 7.2]

**R-78.** Creating a Game from a valid World and Persona makes the new Game the active Game, and the dashboard reloads. [Games 17.1]

**R-79.** Switching to a Game makes it the active Game, and the dashboard reloads. [Games 18.1]

**R-80.** After a switch to a Game whose earlier work was cut off, that Game is idle and accepts an Action. [Games 18.3]

### 4.12 Notices

**R-82.** No notice reaches an LLM. No prompt holds the text of a notice. [owner, 2026-10-11]

**R-83.** The player is told, with the reason, when an Options run fails or the Game has no Options call. The design chooses where the player sees it. [Options 24.5, 24.17, 24.18, 24.20; owner, ticket 17]

### 4.13 Storage, records and restart

**R-90.** "A write names its Game, and one save of a Message, its Swipe and its state is one unit." No reader and no restart sees a Message without its Swipe, or a Swipe without the game state that goes with it. [owner, ticket 17]

**R-91.** While a Message exists, whatever a swipe switch or a Retry needs for it exists too. Missing data is a data-integrity failure (R-46). [Retry I.7; Retrigger I.7]

**R-92.** After a restart, no Game is busy. Work that a restart cut off does not lock any Game. [PD-17; Dashboard 39.1; Games 18.3]

## 5. Environment constraints

**E-01.** One process uses one database. What the process knows only in memory is lost at a restart. Nothing coordinates two processes. [PD-17]

**E-02.** The process can stop at any moment, by a crash, a kill or a planned stop, and start again. Work that runs at that moment ends without warning.

**E-03.** HTTP requests arrive at the same time as other requests. The dashboard polls the status and the story log every few seconds. The player can click any control at any time. A second browser tab can send requests too.

**E-04.** An LLM call takes from under a second to minutes, and any call can fail. The narrator, the Quantifier and the Options call can use different connections. One of them can fail while the others work.

**E-05.** The player can switch, create or reset at any time, also while the active Game is busy.

**E-08.** A design may discard the Games that are saved when it is put into use. It does not need to keep them playable. [owner, 2026-10-12]

## 6. Open questions

No source gives these behaviours. This spec does not answer them.

**OQ-01. The offered set and the swipe switch.** R-36 says that after a swipe switch the game is as that Swipe's generation left it. R-38 says that after a swipe switch the dock shows no offered set. When the generation left an offered set, which holds? Is the offered set part of what a swipe switch gives back?

**OQ-02. A Retry that fires a Trigger.** One source says that a Retry never adds a Message. Another expects an event Narration after a Retry whose redo fires a Trigger. [Retry I.2, 9.5] Which holds?

**OQ-03. Which notices the player must see.** The sources require a notice for a failed or missing Options call (R-83). No source says whether the player must see that:

- the analysis fell back (R-12);
- the player moved to a destination that the map does not have (§3.4).

Also: does a failed or missing Options call at the end of a generation make that generation a failed one on the status?

**OQ-04. Notices and the last Message.** If a notice is shown in the story, may it change which Message a Retry, a Retrigger, a swipe switch or a delete acts on, or where the story log puts their controls? May the player edit or delete a notice?

**OQ-05. A Retry with no Input in the story.** R-44 refuses a Retry when the story holds no Input. R-40 says that a Retry redoes the last generation. Can the player Retry a Narration that has no Input before it, for example a continue or a guided Narration early in a Game?

**OQ-06. An Input whose Narration failed.** When the Narration of a free-text Action fails, does the player's Input stay in the story? If it stays, it is the last Message, and a Retry on it is a user regen (R-61): the narrator rewrites the player's Input. Is that the wanted result, or must a Retry then try the Narration again?

**OQ-07. The idle-reset route.** `POST /status/reset-generating` makes the active Game idle. No source describes it, and the dashboard has no control for it. Does the design need it? If it does, what does it do when work runs for the Game?

**OQ-08. Edit, delete and swipe switch on a busy Game.** For each of these three requests on a busy Game: is it allowed, refused (with which answer), or held until the work ends? If it is allowed, what happens to the work that runs?

**OQ-09. Leaving a busy Game.** When the player resets, creates or switches while the active Game is busy, does the work stop, or does it finish for its own Game? What does that Game show when the player comes back to it? (R-74 says where its writes go.)

**OQ-10. A World with no opening text.** What does a new Game of such a World show first? If the narrator writes an opening:

- when does it run: when the Game is created, or later;
- is the Game busy while it runs, and does the status show it;
- can the player Retry it?

**OQ-11. Delete and the game state.** Does a delete of the last Message change the game state, for example move the player back, change the encounters, the fired Triggers, the trigger context or the offered set? May the player delete a Message of any kind?

**OQ-12. Cut-off work.** After a restart cuts off work, what does the player see? Is it reported? What is left in the story, for example an Input with no Narration?

**OQ-13. How long a failure stays on the status.** Which requests clear a failure that the status shows: the next generation, a swipe switch, an edit, a delete, a restart, a switch of Game?

**OQ-14. How long a trigger context stays stored.** The sources say only that it survives a failed event Narration (R-67). Does a later Action end it? After a delete of the event Narration, may a Retrigger write that event again?

**OQ-15. The room report of an Impersonate.** When an Impersonate generation moves the player, which Message reports the room?

**OQ-16. The room report of the opening.** Does the opening Narration of a new Game report the starting room?

**OQ-17. A Swipe that no generation wrote.** The first Swipe of an Input that the player typed has no generation. What does a swipe switch to that Swipe give back (R-36)?
