# Design: the game-state lifecycle

## 1. Summary

**The state lives in the story.** Every Swipe carries an *after-state*. This is a full copy of the game state as its generation left it: the room, the added rooms, the Characters present, the encounter records, the fired one-shot Triggers and the trigger context. The Game's current state is not a separate record. It is the after-state of the last Message's active Swipe. When the story is empty, it is the Game's opening state. Three results follow:

- A swipe switch gives back that Swipe's world just by making the Swipe active.
- A delete rolls the world back together with the text.
- A redo that saves nothing changes nothing.

The offered set is the only part of the game state that is kept apart, in a record per Game. A swipe switch empties it. On that one item, R-38 wins over R-36 (OQ-01).

**A redo starts from the base state of its Message.** The base state is the after-state of the previous Message's active Swipe. That Swipe cannot change while a later Message exists. So every redo of one Message starts from the same world, whatever swipe switches happened (R-55).

**One save is one unit.** A new Message or a new Swipe, its after-state, and its effect on the offered set land in one storage unit that names the Game (R-90).

**Busy exists only in memory.** A work registry per Game holds the running work and its step. The registry entry is set in the same critical section as the request's checks. It is cleared on every exit path. While it is set, every request that would change that Game's story gets "Still thinking...". Work binds to its Game's id when it is accepted, never to "the active Game". Each of its units first checks two things: that the Game still exists, and that the work still owns the mark. A restart empties the registry, so cut-off work never locks a Game.

**Notices stay out of the story.** The only notice is about Options. It sits on the offered-set record and shows in the dock. No LLM call is ever given that record. Failures show on a status line per Game, kept in memory.

## 2. What the design keeps

Durable records are in storage (§3.6 of the spec). Memory records are lost at a restart (E-01). The current game state is **derived**, not stored (§2.8).

### 2.1 Game record (durable, one per Game)

| Item | Holds | Written by | Changes |
|---|---|---|---|
| Identity | Game id, World, Persona | the create unit (`POST /games`, or the fresh Game of a reset) | never; a reset removes it whole |
| Always-on toggle | on or off, copied from the World | the create unit | never in this part |
| Opening state | starting room; the World's map with no added rooms; starting Characters present; their encounter records; no fired Triggers; no trigger context | the create unit | never |
| Settings read only | presets; whether the Game has an Options call | outside this part | — |

### 2.2 Offered-set record (durable, one per Game)

| Item | Holds | Written by | Changes |
|---|---|---|---|
| Options | ordered option texts, or empty | create unit (empty); Options-success unit (replace); every Narration save unit, event Narrations included (empty); swipe-switch unit (empty); delete unit (empty) | at those writes only. Input saves never touch it (R-30) |
| Options notice | none, or a reason line plus raw text | Options-failure unit; no-Options-call unit | every other write of this record clears it |

### 2.3 Active-Game record (durable, one)

It holds the id of the active Game, or none. Only the create, switch and reset units write it.

### 2.4 Message (durable)

| Item | Holds | Written by | Changes |
|---|---|---|---|
| Identity | Message id, Game id, position | its creating unit | never |
| Kind | Input, Narration or event Narration | creating unit | never |
| Active Swipe | the number of the shown Swipe | creating unit (1); Retry save unit (the new Swipe); swipe-switch unit | only while the Message is last (R-35) |
| Event record (event Narrations only) | the trigger context it was written from (which Trigger, its direction) and the event header | creating unit | never. Every Swipe of the Message uses it (R-56) |

Messages are removed by a delete (last Message only) or by a reset (the whole Game). Every Swipe is removed with its Message.

### 2.5 Swipe (durable)

| Item | Holds | Written by | Changes |
|---|---|---|---|
| Identity | Message, Swipe number 1, 2, … | its save unit | never |
| Made by | opening, typed, user regen, impersonate, narration, guided, event | save unit | never |
| Text | the text | save unit; edit | only by an edit of the active Swipe (R-42, R-75) |
| Stored inputs | the guide (guided); the direction and the impersonated mark (impersonate); nothing for the other kinds | save unit | never |
| Room report | a room name, or none (§8) | save unit | never |
| After-state | §2.6 | save unit | never |

### 2.6 After-state (part of each Swipe)

The after-state is a full copy, not a list of changes. It holds:

- the room the player is in;
- the added rooms;
- the Characters present;
- the encounter record of each Character (times met, met now);
- the fired one-shot Triggers;
- the trigger context (none, or a Trigger and its direction).

It does **not** hold the offered set, any notice, or any status.

How the after-state is made, by "made by":

| Made by | After-state |
|---|---|
| opening | the Game's opening state |
| typed, user regen | a copy of the base state. No analysis runs |
| narration, guided, impersonate | the world rules' new state from the analysis, with two rules. (a) The trigger context becomes the fired Trigger, or none when none fired. (b) A one-shot Trigger is **not** added to the fired Triggers here. It is added when its event Narration is saved |
| event | the world rules' new state from the analysis, with the movement removed before the world rules run. The Trigger of the event record is added to the fired Triggers if it is one-shot. The trigger context becomes none. Any Trigger that the world rules report is ignored |

### 2.7 Memory records

| Record | Holds | Written by | Changes |
|---|---|---|---|
| Work registry (per Game) | none, or: work id, kind, current step | set by the accepting request; step changes by the work; cleared at the work's end; dropped by a reset of that Game | lost at a restart |
| Status record (per Game) | none, or a failure: kind, short line, raw text | set at the work's end and by a 500 refusal | cleared when new work is accepted, and by a swipe switch, a delete or the idle-reset route; lost at a restart |
| Role health (per LLM role: narrator, Quantifier, Options) | the last outcome (ok, failed, fell back, not available) and its time | the work, after each black-box call | an older outcome never replaces a newer one; lost at a restart |
| Game turn (per Game) | whether a short check-and-write for that Game runs now; others queue | every request or unit that reads and then writes the Game (§6) | held for one check plus one unit, never across an LLM call |

### 2.8 Derived values (not stored)

- **Current state.** The after-state of the active Swipe of the last Message, or the opening state when the story is empty. Together with the offered-set record, it is the full game state of spec §1.
- **Base state of a Message.** The after-state of the active Swipe of the previous Message, or the opening state for the first Message. It is stable for two reasons: only the last Message can switch (R-35), and only the last Message can be deleted.
- **Busy.** The work registry has an entry for the Game.
- **Retrigger offered.** The current state holds a trigger context, and the last Message is a Narration that is not an event Narration (R-62).
- **Story holds an Input.** Some Message has the kind Input.

## 3. States

### 3.1 Stored life of a Game

| State | Holds | Entered by | Left by |
|---|---|---|---|
| Absent | — | — | a create unit |
| Live | the Game record, the offered-set record, the Messages | `POST /games`, or the fresh Game of a reset | a reset unit while it is the active Game |
| Removed | nothing | a reset unit | — |

Whether a Game is active is a separate fact. It lives in the Active-Game record.

### 3.2 Status of a Game (memory)

There are three states:

- **Idle**: no failure line.
- **Idle with failure**: a failure line and its raw text.
- **Busy(step)**: work id, kind, and step. The step is narrating, quantifying, generating-event or options.

| From | To | Cause |
|---|---|---|
| Idle, or Idle with failure | Busy(first step) | an Action, a Retry or a Retrigger is accepted. The failure line is cleared |
| Busy(step) | Busy(next step) | the work starts its next step (R-71) |
| Busy | Idle | the work ends and no generation step failed. An analysis fallback or an Options failure does not count (R-12, OQ-03) |
| Busy | Idle with failure | any narrator failure; empty text; an invalid room; an event-narration failure; a data fault found during the work; an internal error |
| Busy | entry dropped | a reset removes the Game |
| Idle | Idle with failure | a Retry, a Retrigger or a swipe switch is refused with 500 (R-46, R-91) |
| Idle with failure | Idle | a swipe switch, a delete, or the idle-reset route |
| any | Idle | a restart (memory is empty) |

A "Still thinking..." refusal, a 400 refusal, an edit and a switch of Game do not change the status.

### 3.3 Failure lines (R-21)

| Failure kind | Short line (raw text in a disclosure) |
|---|---|
| no answer | "The narrator did not answer." |
| prompt too long | "The story is too long for the narrator's connection." |
| unusable answer | "The narrator's answer could not be used." |
| empty text | "The narrator returned empty text." |
| output budget spent | "The narrator used its whole output budget before answering." |
| preset missing | "The prompt preset for ‹kind› is missing." |
| invalid room | "The player's room ‹name› is not on the map." |
| event narration failed | "The event narration failed: ‹line of the cause above›." |
| data integrity | "Data missing: ‹what›." |
| internal error | "This work stopped on an internal error." |

### 3.4 One piece of work

| State | Holds | Status shows |
|---|---|---|
| Requested | the request, and the Game that is active when the request takes that Game's turn | unchanged |
| Refused (end) | only the answer | unchanged, or a failure line for a 500 |
| Accepted | work id, Game id, kind, and what the checks read: the last Message, the base or current state, the stored inputs, the trigger context | the first step |
| Checking | the start state is checked against the World through the world rules' contradiction check | the first step |
| Input saved | free text only: the Input unit has landed | narrating |
| Narrating | the narrator call (Narration, Impersonate or user regen) | narrating |
| Generating event | the narrator call for an event Narration | generating-event |
| Quantifying | the analysis, then the world rules | quantifying |
| Saving | one storage unit | the current step |
| Making options | the Options call | options |
| Ended | the entry is cleared, and the status is Idle or Idle with failure | idle or failure |
| Abandoned | the Game is gone or the mark is not ours. Nothing is written | — |
| Cut off | nothing (a restart) | — |

Paths by kind:

- **Free text:** Accepted → Checking → Input saved → Narrating → Quantifying → Saving the Narration → [Generating event → Quantifying → Saving the event] → [Making options] → Ended.
- **Continue and guided:** the same path without "Input saved". The guided Swipe stores the guide.
- **Impersonate:** Accepted → Checking → Narrating → Quantifying → Saving the Input → [event chain] → [Making options, only if an event Narration was saved] → Ended.
- **`/options`:** Accepted → Making options → Ended.
- **Retry:** see §5.
- **Retrigger:** see §5.3.

| From | To | Cause |
|---|---|---|
| Requested | Refused | no Game (400); busy ("Still thinking..."); a user error (400); unreadable data (500) |
| Requested | Accepted | the checks pass. The mark is set in the same Game turn. The answer is 200 at once (R-09) |
| Checking | Ended (failure) | the start state contradicts the World (R-20). Nothing has been written, not even the Input |
| Checking | next step | the check passes |
| Narrating / Generating event | Ended (failure) | a narrator failure or empty text (R-19). A free-text Input stays (OQ-06) |
| Generating event | Making options | the event failed, the work saved a Narration, and the toggle is on. The failure is still reported at the end |
| Narrating | Quantifying | a text comes back (narration, guided, impersonate) |
| Narrating | Saving | a text comes back (user regen: no analysis) |
| Quantifying | Saving | a world-rules result. An analysis fallback is a result |
| Saving | Generating event | the unit landed and a Trigger fired (not after an event) |
| Saving | Making options | the unit landed, the work saved a Narration, the toggle is on, and no event is due |
| Saving | Ended | otherwise |
| Making options | Ended | a set is saved, or a notice is saved. If the Game has no Options call, no call is made and the notice is saved |
| any | Abandoned | a unit's pre-check fails (Game removed, or mark not ours) |
| any | Ended (failure) | an internal error. The exit path always clears the mark (R-10) |
| any | Cut off | a restart (§9) |

`/options` request checks run in this order:

1. Busy → "Still thinking...".
2. Nothing to work from → 400 with the reason (R-27).
3. No Options call → 200 with the reason, and the notice is saved (R-29).
4. Otherwise the work is accepted.

## 4. Swipes

**What binds to a Swipe:**

- its text;
- its "made by";
- its stored inputs (guide, direction, impersonated mark);
- its room report;
- its after-state: room, added rooms, Characters present, encounter records, fired one-shot Triggers, trigger context.

The trigger context of an event Narration binds to the Message, because every Swipe of that Message redoes the same Trigger.

**Swipe switch sequence** (`POST /message/:id/swipe/:index`):

1. Take the turn of the active Game. If there is no Game, answer 400.
2. If the Game is busy: answer the unchanged story log and change nothing (OQ-08).
3. If Message `:id` is not the last Message, or Swipe `:index` does not exist: answer 400 and change nothing (R-35).
4. Read the target Swipe and its after-state. If something is missing or unreadable: answer 500, and the status names it (R-91, R-46).
5. In one unit: set the active Swipe, empty the offered set, and clear its notice.
6. Clear the failure line, so the status is Idle (R-37).
7. Answer the story log.

**What a switch gives back:**

| Item | After the switch |
|---|---|
| Text, room report, Swipe count | those of the new active Swipe (R-25) |
| Room, added rooms, Characters present, encounter records, fired one-shot Triggers, trigger context | those of the Swipe's after-state, because the current state is derived from it (R-36). Whether Retrigger is offered follows from them |
| Offered set | empty, with no notice (R-38; OQ-01) |
| Status | Idle (R-37) |

**First Swipe of a typed Input (OQ-17).** Its after-state is a copy of the base state, so a switch to it gives back the world in which the player typed. A user regen also copies the base, so all Swipes of a typed Input give back the same world.

## 5. Retry and Retrigger

### 5.1 Common to every Retry

The request runs inside the Game's turn:

1. If no Game exists: 400 (R-45).
2. If the Game is busy: 200 "Still thinking..." (R-70).
3. If the story holds no Input: 400 (R-44; OQ-05).
4. Read the last Message, its active Swipe, its "made by" and stored inputs, its base state, and its event record if it is an event Narration. If something is missing or unreadable: 500, and the status names it (R-46).
5. Set the mark, clear the failure line, and answer 200.

The work runs these steps:

1. Check the **base state** against the World (R-20, R-49).
2. Run the steps of the kind (§5.2).
3. Save exactly one new Swipe on the same Message, numbered one past the count, as its active Swipe, in one unit (R-41). No existing text is touched (R-42).
4. If a Trigger fired, write an event Narration as a new Message (OQ-02).
5. If the work saved a Narration and the toggle is on, run Options.

If a step fails before the save, nothing has been written. The current state is the same after-state as before (R-47, R-49), and the status shows the failure. The mark is cleared on every path (R-50).

### 5.2 By the kind of the last generation

The kind comes from the last Message. An Input is typed or impersonated according to its Swipe 1. A Narration is narration, guided or event according to its "made by".

**A. A Narration that answers an Input** (made by narration, and the previous Message is an Input)

- **Narrator reads:** every Message before the Input. The Input's active text, as it is now and with any edit, is the player's text (R-51).
- **Starts from:** the base state, which is the Input's after-state.
- **Steps again:** check; narrating; quantifying, with the Input as the player's text; world rules; save; event chain; Options (R-52).
- **Adds:** one Swipe, with a room report if the redo moved the player. Possibly one event Narration Message. A new set, or an empty set.
- **On failure:** before the save, nothing changes. If the event fails after the save, the Swipe and its world stay (R-48), the context stays stored, the status reads "event narration failed", and Retrigger is offered (R-17).

**B. A continue Narration** (made by narration, and the previous Message is not an Input)

- This is like A, but with no player's text.
- It is allowed only when some Input exists in the story (R-44).

**C. A guided Narration**

- This is like A or B. The prompt also carries the active Swipe's stored guide, and the new Swipe keeps the same guide.
- The room report is the redo's own room, or none (R-53).

**D. An event Narration** (also one that a Retrigger wrote, R-54)

- **Narrator reads:** every Message before it, and the Message's own event record. This holds even after later Messages were deleted (R-56).
- **Starts from:** the base state, which is the world before this event Narration was written. A swipe switch on it does not change the base (R-55).
- **Steps again:** check; generating-event; quantifying with the movement removed (R-57); world rules, with any Trigger ignored; save the Swipe with no room report. The Trigger is marked fired if one-shot, and the context is cleared. Then Options.
- **Adds:** one Swipe. No Message.
- **On failure:** no Swipe and no Message. The current state is still the active Swipe's after-state, so the player stays where the request found them and a one-shot Trigger stays fired. The status reads "event narration failed" (R-58). The Narration that fired the Trigger stays.

**E. An impersonated Input**

- **Narrator reads:** every Message before it, and the stored direction. The call uses the impersonate preset that is active when the call starts (R-59).
- **Starts from:** the base state.
- **Steps again:** check; narrating; quantifying; world rules; save. The Swipe keeps the direction and the impersonated mark, and gets a room report if the redo moved the player. Then the event chain (R-60).
- **Options:** none, unless an event Narration was saved (R-30).
- **Adds:** one Swipe, and possibly an event Message.
- **On failure:** as in A.

**F. A typed Input** (a user regen, R-61)

- **Narrator reads:** every Message before it, and the Input's active text as it is now.
- **Starts from:** the base state. There is no analysis and no movement (R-18). The after-state is a copy of the base state, and there is no room report.
- **Steps again:** check; narrating; save.
- **Adds:** one Swipe made by user regen. The offered set is unchanged.
- **On failure:** nothing changes.

**G. The opening Narration, or an empty story**

- The story then holds no Input, so step 3 of the request answers 400.

### 5.3 Retrigger

The request runs inside the Game's turn:

1. No Game → 400.
2. Busy → "Still thinking...".
3. The story is empty, or no context is stored in the current state, or the last Message is an Input or an event Narration → 400 (R-65).
4. The context is unreadable, or its Trigger is missing from the World → 500 (R-46).
5. Otherwise: accept, clear the failure line, and answer 200.

The work:

- **Narrator reads:** the whole story, which ends in the Narration, and the stored context.
- **Starts from:** the current state as the request finds it (R-63).
- **Steps:** check; generating-event; quantifying with the movement removed; world rules, with any Trigger ignored; save a new event Message. Its Swipe 1 is made by event and has no room report. The event record is the context. The Trigger is marked fired if one-shot, and the context is cleared. Then Options if the toggle is on.
- **Adds:** exactly one Message, and no Swipe on any other Message (R-64).
- **On failure:** nothing is written. The context stays, the story still ends in the Narration, Retrigger is still offered, and the status reports the failure (R-66, R-67).

### 5.4 What a Retrigger can do after each kind of Retry

| Retry of | Outcome | Retrigger then |
|---|---|---|
| A, B, C | the redo fired no Trigger | 400: the redo's after-state has no context |
| A, B, C | fired, and the event was saved | 400: the last Message is an event Narration |
| A, B, C | fired, and the event failed | allowed: it writes that event after the Narration |
| A, B, C | failed before the save | as before the Retry |
| D | any outcome | 400: the last Message is an event Narration |
| E | no Trigger, or the event failed | 400: the last Message is an Input (see Q1) |
| E | the event was saved | 400: the last Message is an event Narration |
| F | any outcome | 400: the last Message is an Input |
| refused (busy, 400, 500) | — | unchanged |

## 6. Writers

**Mechanisms:**

- **W1. Id binding.** A request picks its Game when it takes that Game's turn. Work keeps that Game id for all of its reads and writes (R-74).
- **W2. Game turn.** Each check-and-write for a Game runs alone. The single process queues them (E-01). A turn is never held across an LLM call (R-09).
- **W3. Busy gate.** While the mark is set, no request may change that Game's story or offered set.
- **W4. Unit pre-check.** Each unit of the work first checks, inside the turn, that the Game exists and that the mark carries this work's id. If either check fails, the work is Abandoned.
- **W5. Few shared records.** Swipes are written once, and the current state is derived. Only these records can be rewritten:
  - the active Swipe (Retry, switch);
  - Swipe text (edit);
  - the offered-set record (work, switch, delete);
  - the Active-Game record (create, switch, reset).

  W3 keeps the first three to one writer at a time. Each Active-Game write is one unit that reads nothing it later rewrites.

| Request / work | While busy? | Effect on running work | Writes to | Lost update prevented by |
|---|---|---|---|---|
| `POST /action` | no: "Still thinking..." (R-70) | none | the active Game at the turn | W1–W4 |
| `POST /swipe/new` | no | none | the active Game | W1–W4 |
| `POST /retrigger` | no | none | the active Game | W1–W4 |
| swipe switch | no: unchanged story log | none | the active Game | W2, W3; the check is inside the unit |
| edit | no: 200 "Still thinking..." | none | the active Game | W2, W3; edits change text only |
| delete | no: 200 "Still thinking..." | none | the active Game | W2, W3 |
| `POST /reset` | yes (E-05) | work for the removed Game is Abandoned at its next unit. The registry entry is dropped | removes the active Game; creates a fresh Game; the Active-Game record | one unit; W4 |
| `POST /games` | yes | none; the work goes on for its own Game | the new Game; the Active-Game record | one unit; W1 |
| `POST /games/:id/switch` | yes | none | the Active-Game record | one unit; W1 |
| `POST /status/reset-generating` | allowed; answers "failed" while busy | none | the status record of the active Game | W2 |
| GET routes, debug route | yes | none | nothing | they read whole units only (R-90) |
| Generation work | it is the busy work | — | its bound Game | W1, W4, and W3 shuts out the others |
| Options run (on demand or trailing) | part of the work | — | the offered-set record of its bound Game | W1, W4 |
| Role-health writes | any time | — | role health (not per Game) | the newer outcome wins |

**What happens when the player leaves a busy Game (OQ-09).** On a switch or a create, the work finishes for its own Game. Coming back shows Busy if it still runs. Otherwise it shows its story and its final status. On a reset, the old Game is gone, and its work ends at its next pre-check.

## 7. Notices

**Kinds.** There is one kind of notice, the Options notice (R-83). It has these reasons:

- an Options run failed (on demand or trailing), with the raw text;
- the Game has no Options call (on demand, or trailing with the toggle on).

These are not notices:

- status failure lines (§3.3);
- the header banner, which reads role health and shows the roles whose last outcome was failed, fell back or not available;
- room reports.

**Where it goes.** The notice field of the offered-set record. The dock shows the reason line under the current set (the old set for an on-demand failure, an empty set after a Narration, R-29, R-33). The raw text is in a disclosure. For an on-demand `/options` with no Options call, the answer also names the reason.

**Who sees it.** Anyone who views that Game's dock, in any tab, while it is the active Game. It belongs to the Game, so it is still there when the player switches back.

**How long it stays.** Until the offered-set record is next written: the next Options result, the next saved Narration, a swipe switch, a delete, or a reset. It is durable, so it survives a restart.

**OQ-03.** There is no notice for an analysis fallback or for a new room. The banner shows a degraded Quantifier. The room report shows the new room.

**Out of every prompt (R-82).** The design gives prompt assembly, the Quantifier and the Options call only these things:

- the World and the Persona;
- a game state taken from an after-state, which has no notice field and no offered set;
- story windows built only from Messages and their active Swipes;
- stored inputs, trigger contexts and presets.

Notices are not Messages, so they never enter a story window. The offered-set record, the status record and role health are never given to any call. Because notices are not in the story, they cannot change which Message is last, and they cannot be edited or deleted (OQ-04).

## 8. The room report

**Where it is kept.** On each Swipe, written once by its save unit.

**When a Swipe reports a room.** Its kind can move the player (narration, guided, impersonate), and its after-state room differs from its base-state room. The report is the after-state room's name. For a room that the world rules added, it is the destination name.

**When a Swipe reports no room:**

- typed Swipes (R-22);
- user regen (R-18);
- event Swipes (R-24);
- the opening (OQ-16);
- any redo that did not move the player, even when a sibling Swipe reports a room (R-23).

**How the story log gets it.** For each Message, the story log reads the active Swipe's room report. After a switch, it shows the new active Swipe's report (R-25). An impersonated Input that moves the player reports the room on the Input itself (OQ-15).

## 9. Restart and cut-off work

**What a restart finds.** Storage holds only whole units. The work registry, the status records and role health are empty, so no Game is busy and every status is Idle (R-92, R-72). The Active-Game record still names the active Game.

**What it does.** Nothing special. There is no scan and no repair. Cut-off work is not reported (OQ-12). The story shows the last unit that landed:

| Cut at | What is left |
|---|---|
| before the first unit (any kind, any Retry, any Retrigger) | nothing changed |
| free text, after the Input unit | the story ends in the Input. A continue answers it; a Retry rewrites it (OQ-06) |
| after the firing Narration or Impersonate unit, before the event | the context is stored. Retrigger is offered if the last Message is a Narration |
| after the last generation unit, during Options | the offered set is empty with no notice (a Narration was saved), or unchanged (`/options`) |
| create, reset, switch, edit, delete or switch unit | whole or not at all |

A switch to such a Game finds it Idle, and it accepts an Action (R-80).

## 10. Requirement trace

| R | Met in | Forced / free |
|---|---|---|
| R-01 | §3.4 paths; A-14 | forced |
| R-02 | §3.4 free-text path (Input saved first) | free |
| R-03 | §3.4 | forced |
| R-04 | §2.5, §3.4 | forced |
| R-05 | §2.5, §5.2 E | forced |
| R-06 | §3.4; Q2 | free |
| R-07 | §3.4; A-3 | forced |
| R-08 | §3.4, §2.4 | forced |
| R-09 | §3.4, W2 | free |
| R-10 | §3.2 (mark cleared on every exit) | free |
| R-11 | §2.6, §3.4 | forced |
| R-12 | §3.2, §3.4 | forced |
| R-13 | §3.4 | forced |
| R-14 | §2.6 (movement removed) | free |
| R-15 | §3.4 | forced |
| R-16 | §2.6 (marked fired at event save) | free |
| R-17 | §2.8, §5.2 A | forced (Q1) |
| R-18 | §5.2 F (no analysis) | free |
| R-19 | §3.4 | forced |
| R-20 | §3.4 Checking | forced |
| R-21 | §3.3 | free |
| R-22 | §8 | forced |
| R-23 | §8 | forced |
| R-24 | §8 | forced |
| R-25 | §4, §8 | forced |
| R-26 | §2.2 | forced |
| R-27 | §3.4 checks | free |
| R-28 | §2.2 | forced |
| R-29 | §2.2, §7 | forced |
| R-30 | §2.2 | forced |
| R-31 | §3.4 (one run per work) | free |
| R-32 | §2.2 | forced |
| R-33 | §2.2, §7 | forced |
| R-34 | §2.5, R-90 units | forced |
| R-35 | §4 | forced |
| R-36 | §2.8, §4 | free |
| R-37 | §4 | forced |
| R-38 | §4 | forced |
| R-40 | §5 | forced |
| R-41 | §5.1 | forced |
| R-42 | §2.5 | forced |
| R-44 | §5.1 | forced |
| R-45 | §5.1 | forced |
| R-46 | §5.1, §3.3 | free |
| R-47 | §2.8 derived state | free |
| R-48 | §5.2 A | forced |
| R-49 | §5.1 | forced |
| R-50 | §3.2, §5.1 | free |
| R-51 | §5.2 A | forced |
| R-52 | §5.2 A | forced |
| R-53 | §5.2 C | forced |
| R-54 | §5.2 D | forced |
| R-55 | §2.8 base state | free |
| R-56 | §2.4 event record | free |
| R-57 | §5.2 D | forced |
| R-58 | §5.2 D | forced |
| R-59 | §5.2 E | forced |
| R-60 | §5.2 E | forced |
| R-61 | §5.2 F | forced |
| R-62 | §2.8, §5.3 | forced |
| R-63 | §5.3 | forced |
| R-64 | §5.3 (chained Trigger ignored) | free |
| R-65 | §5.3 | forced |
| R-66 | §5.3 | forced |
| R-67 | §2.6 (context rule) | free |
| R-70 | §6 | forced |
| R-71 | §3.2 | forced |
| R-72 | §2.7 (memory registry) | free |
| R-73 | §6 W2–W5 | free |
| R-74 | §6 W1, W4 | free |
| R-75 | §2.5 | forced |
| R-76 | §2.8 (rollback); A-10 | free |
| R-77 | §3.1, A-8 | forced |
| R-78 | §3.1 | forced |
| R-79 | §2.3 | forced |
| R-80 | §9 | free |
| R-82 | §7 | free |
| R-83 | §7 | free |
| R-90 | §2, §3.4 units | free |
| R-91 | §2.8 (base state derived from an existing Message) | free |
| R-92 | §9 | free |

## 11. Assumptions

**Ideas from other products.** The terms Swipe, Impersonate and Guided Generation also exist in SillyTavern. No design idea is taken from it, or from any other product. The binding of state to a Swipe comes from R-36.

**Answers to the open questions:**

- **OQ-01. R-38 wins.** A swipe switch empties the offered set. The offered set is not kept on a Swipe. *Reason:* R-38 is a concrete scenario. Options are suggestions that fit a text, and they are not world. On a Narration, R-32 already empties the set, so the conflict only bites for Inputs.
- **OQ-02. Yes.** A redo that fires a Trigger adds an event Narration as a new Message. A later Retry then acts on that event Narration. *Reason:* R-13, R-52 and R-60 apply to every generation. R-41 limits Swipes, not Messages. Leaving a fired Trigger without its event needs extra player action. This drops the source rule "Retry never adds a Message" (Retry I.2); see Q3.
- **OQ-03.** There is no notice for an analysis fallback or a new room. An Options failure does not fail the generation on the status. *Reason:* no requirement asks for these, and R-12 treats a fallback as success.
- **OQ-04.** Notices are not Messages, so the questions do not arise. *Reason:* this makes R-82 hold by construction.
- **OQ-05. R-44 holds as written.** With no Input anywhere in the story, a Retry is 400, even on a continue or a guided Narration. When an Input exists, such Narrations can be redone. *Reason:* R-44 is explicit, and it also covers the opening. R-40 gives way only here.
- **OQ-06.** The Input stays. A Retry on it is a user regen, as R-61 states. A continue writes the Narration that answers it. *Reason:* the player's words are not lost, and there is a recovery that needs no new rule.
- **OQ-07. The route is kept.** With no Game, it answers "failed". While busy, it answers "failed" and leaves the work running. Otherwise it clears the failure line and answers "reset". *Reason:* busy cannot stick (§9), and cancelling work would be a new feature.
- **OQ-08.** All three requests are refused while the Game is busy. Edit and delete answer "Still thinking...". The swipe switch answers the unchanged story log. *Reason:* the running work holds a base read of the story, and a single rule rules out every race.
- **OQ-09.** The work finishes for its own Game. A reset abandons it. *Reason:* R-74, and no cancel feature is needed.
- **OQ-10.** An empty opening text gives an empty story. The narrator writes no opening. *Reason:* no requirement asks for an LLM opening. R-27 implies that empty stories exist. A continue writes the first Narration.
- **OQ-11.** A delete rolls the game state back, because the current state is derived. It also empties the offered set. Any kind of last Message may be deleted, including the opening. *Reason:* the story and the world stay in step, consistent with R-36.
- **OQ-12.** Cut-off work is not reported. The story shows the last whole unit. *Reason:* a durable journal of work would be a new feature.
- **OQ-13.** These clear a failure line: the acceptance of new work, a swipe switch, a delete, the idle-reset route, and a restart. These do not: an edit, a switch of Game, and refusals.
- **OQ-14.** The context is replaced by the next analysed generation (narration, guided, impersonate), which sets the context to its own fired Trigger or to none. It is also cleared when its event is saved. After a delete of the event Narration, the rollback restores the context, so a Retrigger may write the event again. *Reason:* the event belongs right after the text that fired it, and §3.4 allows a one-shot Trigger to fire again while no event exists.
- **OQ-15.** The Impersonate Input's Swipe reports the room (R-23).
- **OQ-16.** The opening reports no room. No generation moved the player (R-22).
- **OQ-17.** It gives back the base state, which is the world in which the player typed.

**Other points where the spec is silent:**

- **A-1.** The world rules may themselves mark a one-shot Trigger as fired. If they do, the design removes that mark from a non-event after-state, following the note in §3.4.
- **A-2.** Movement is removed from the analysis before the world rules run for an event Narration.
- **A-3.** A Trigger reported for an event Narration is ignored. There are no chained events (R-07, R-64).
- **A-4.** "Moves the player" means that the after-state room differs from the base-state room.
- **A-5.** For a Narration, the player's text is the previous Message's active text when that Message is an Input.
- **A-6.** The trailing Options run happens once per piece of work, after its last saved Narration. A set made between a Narration and its event would be hidden behind busy, and the next set would replace it at once.
- **A-7.** The room check runs before the Input is saved. The check runs on the state the generation starts from.
- **A-8.** A reset keeps the World and the Persona. The fresh Game gets an empty set and the opening state.
- **A-9.** In the opening state, each starting Character has been met once and is met now.
- **A-10.** "At most once in a Game" (R-16) is read against the current story. A deleted event does not count.
- **A-11.** Request checks run in this order: no Game, then busy, then user error, then data integrity.
- **A-12.** These answer 400: an Action with no Game; a swipe switch on a Message that is not last or has a bad index; `/options` with nothing to work from. "Nothing to work from" means that no Message has non-empty text.
- **A-13.** Existing saved Games are discarded when the design goes into use (E-08).
- **A-14.** Command parsing trims the text. An empty text is a continue. "/options" is matched exactly. "/guide " and "/impersonate " are prefixes.
- **A-15.** An internal error ends the work with its own failure line, so that R-10 holds.

## 12. Questions for the owner

**Q1. A Trigger fired by an Impersonate whose event fails.**
- *Why it matters:* R-17 offers Retrigger on "the last Narration". But the last Message is the Input, so R-65 and §3.7 refuse a Retrigger there.
- *Default:* R-65 holds. The event can come back when a later Narration fires the Trigger again.

**Q2. Option texts that look like commands.**
- *Why it matters:* an option such as "/options" would be parsed as a command (R-01), not as free text (R-06).
- *Default:* the dock's request carries a mark that it is an option, and the engine then treats the text as free text.

**Q3. Confirm OQ-02.**
- *Why it matters:* the answer drops the source rule "Retry never adds a Message".
- *Default:* the event Narration is added.

**Q4. Confirm OQ-01.**
- *Why it matters:* if R-36 must win, each Swipe must also keep its offered set, and every trailing Options run must write onto the new Swipe.
- *Default:* R-38 wins.

**Q5. Confirm OQ-05.**
- *Why it matters:* the player cannot redo an early continue until some Input exists.
- *Default:* R-44 holds as written.

**Q6. What counts as a "degraded LLM role" for the banner?**
- *Why it matters:* role health is the only input of the banner.
- *Default:* a role is degraded when its last outcome was failed, fell back, or not available.

**Q7. Should cut-off work be reported after a restart (OQ-12)?**
- *Why it matters:* the player may find an Input with no Narration and no explanation.
- *Default:* no report.

**Q8. "Fires at most once" after a deleted event (A-10).**
- *Why it matters:* the delete rollback lets the same one-shot event be written again.
- *Default:* yes, it may be written again.

**Q9. One Options run per piece of work (A-6).**
- *Why it matters:* "after each Narration" could mean a run after the firing Narration as well as after the event.
- *Default:* one run.
