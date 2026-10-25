# Design: the game-state lifecycle

## 1. Summary

The design keeps one record per Game. A Game holds its World and Persona references, its prompt presets, a **status** field (idle, busy with a step name, or a failure), an **offered set** of Options, one **stored trigger context** slot, the **live game state** (room, added rooms, present Characters, encounter records, fired one-shot Triggers), the **story** (ordered Messages, each with ordered Swipes), and a **notice list** that is never part of the story.

Every Message carries two game-state snapshots: the state its generation **started from** (one per Message, shared by all its Swipes) and, on each generation-written Swipe, the state its generation **left behind**. These two snapshots carry requirement R-36 ("after a swipe switch, the game is as that Swipe's generation left it") and requirement R-55 (a Retry of an event Narration starts from the state before the replaced event was written) without any replay logic: a swipe switch restores the active Swipe's end state; a Retry restores the Message's start state and re-runs the generation.

All engine work (Action, Retry, Retrigger, `/options`) runs in the background. The request handler validates synchronously, claims the Game busy with one atomic compare-and-set write, answers the client at once, and then runs the work. While a Game is busy, further work requests for it answer `200` "Still thinking..." and start nothing; the small synchronous requests (edit, delete, swipe switch) are refused the same way, so no writer ever interleaves with running work. Every write names the Game it belongs to; the Game id is captured when the request arrives and never re-read from the active-Game pointer, so switching, creating or resetting during work cannot move writes (R-74). A reset deletes the Game; a work item that then tries to write finds the Game gone and ends silently.

A generation is a fixed pipeline: pre-flight room check, narrator call, commit of the text with its Swipe and end-state snapshot, Quantifier call, world rules, and — when a Trigger fires — a second generation for the event Narration. Each successful generation commits in one storage unit (Message, Swipe, game state, status), so no reader and no restart ever sees a partial result (R-90). Failures leave exactly what the earlier commits put in the story, and set a failure line on the status. Notices about fallbacks and Options problems live outside the story and are never given to prompt assembly, so no notice can reach an LLM (R-82). A restart clears every Game's busy flag; cut-off work leaves only what its committed units put in the story, plus — usefully — a stored trigger context that a Retrigger can recover.

## 2. What the design keeps

### 2.1 Per Game

One **Game record** in storage. Written by the request handlers and the work item that the Game's busy claim admitted; each write names the Game's id (R-90). It holds:

| Field | Holds | Written when |
|---|---|---|
| Game id | An identifier used by every route that carries an id and by every write | At creation |
| World reference | Which authored World the Game plays | At creation; never again (a World does not change during play) |
| Persona | The character the player plays | At creation; never again |
| Prompt presets | The preset record for each narrator kind (Narration, Impersonate, user-regen, event Narration) | At creation; replaceable by out-of-scope configuration. Read fresh at each narrator call, so a preset that became active mid-work is used at the next call (§3.1, R-59) |
| Always-on toggle | Whether a new offered set is made after each Narration | At creation, taken from the World; changed only by the Options feature's own control (outside this design) |
| Status | `idle`, or `busy` with the step name (`narrating`, `quantifying`, `generating-event`, `options`), or `failure` with a kind, one short line, and the raw text | Claim sets busy; each step sets its name; the end of work sets idle or the failure |
| Offered set | The current list of short player inputs, or none | The Options step of a work item; a swipe switch clears it |
| Stored trigger context | None, or one context: which Trigger fired and its direction | Set when the world rules report a fired Trigger; overwritten when another Trigger fires; never cleared by later play (OQ-14) |
| Live game state | The room; the rooms play added to the map; the Characters present; the encounter record of each Character (times met, met now); the ids of fired one-shot Triggers | Committed by each successful generation, and by a swipe switch |
| Story | The ordered list of Messages | Appended by generations; changed by edit and delete |
| Notices | A short list of notices (kind, reason, time) | Work that fell back or whose Options step failed; cleared when the next engine work on this Game starts |

### 2.2 Per Message

One **Message record**. Written once by the generation that opened it, then touched only by edit, delete and swipe switches. It holds:

- **id** — used by the swipe-switch and edit routes.
- **kind** — Input, Narration, or event Narration.
- **event header** — for an event Narration, the name of the fired Trigger; shown by the story log.
- **Swipes** — an ordered list; every Message has at least one (R-34). One of them is the **active Swipe**.
- **State-before snapshot** — the game state, without the offered set, as the generation that opened this Message started from. Written once, when the Message is created, and never changed. All Swipes of the Message share it, because every redo of that generation starts from the same point. This is what a Retry restores (R-55 pins it for event Narrations; the design uses it for every kind, see §5).

### 2.3 Per Swipe

One **Swipe record**. Written once, when its generation commits or when the player's Input is taken in; never changed afterwards (R-42). It holds:

- **text** — the text of this version.
- **stored inputs** — the guide (for a guided Narration), the impersonate direction and the impersonated mark (for an Impersonate Input), or none (for a player-typed Input, a continue Narration, a plain Narration, and an event Narration, which instead carries the context copy below). These are exactly "the inputs that produced it" of R-34.
- **provenance** — which kind of generation wrote it: `opening`, `player-typed` (no generation), `narration`, `impersonate`, `user-regen`, or `event`. A Retry reads this to know which kind of redo to run (§5).
- **trigger-context copy** — for an event Swipe, a full copy of the trigger context that produced it. This is what a Retry of the event uses (R-56), even after the Game's context slot was overwritten by a later Trigger or the event's neighbours were deleted.
- **room report** — the name of the room this generation arrived in, or none (§8, R-22–R-25).
- **state-after snapshot** — the game state, without the offered set, as this generation left it. Present only on generation-written Swipes. This is what a swipe switch restores (R-36). The player-typed first Swipe of an Input has none (OQ-17).

### 2.4 Other records

- **Active-Game pointer.** A single storage record naming the active Game's id. Written by create, switch and reset; read once by each incoming request; never read by a running work item (R-74).
- **Work item.** A short-lived record of one piece of engine work (§3.2). Kept in the process; the durable trace of a work item is the Game's status field.
- **Notice.** See §7.

## 3. States

### 3.1 States of a Game

A Game is in exactly one of these states, held in its status field:

- **Absent.** No record. Requests that need a Game answer `400` (assumption A5; R-45 is the stated case for Retry).
- **Idle.** Holds the whole Game record; the offered set may hold options or be empty; a previous failure line may still be stored (OQ-13). Accepts any request.
- **Busy, with a step name.** Holds the whole Game record plus the name of the step that runs. Exactly one work item is running for this Game. The step is one of `narrating`, `quantifying`, `generating-event`, `options` (R-71). Accepts only "Still thinking..." answers for work requests and refusals for edit, delete and swipe switch (OQ-08).
- **Failed.** The status holds a failure: a kind, the short line that the kind selects (R-21), and the raw text inside a disclosure. The Game is not busy and accepts requests.

Transitions:

| From | To | Cause |
|---|---|---|
| Idle or failed | Busy | A work request (Action, Retry, Retrigger) passed its synchronous checks and won the atomic busy claim (§6.6) |
| Busy | Idle | The work committed its last unit; the status is set idle in the same unit as the last write, so the Game stops being busy soon after the last call returns (R-09, R-10) |
| Busy | Failed | A black box failed in a way the spec turns into a status failure (R-17, R-19, R-20, R-46, R-58, R-66); the failure line and raw text are written together with the clearing of the busy flag |
| Failed | Busy | The next work request on this Game starts (this also clears the failure, OQ-13) |
| Failed or Busy | Idle | A swipe switch on this Game (R-37), or `POST /status/reset-generating` on an idle-status Game (OQ-07) |
| Any | Idle | A restart: the startup sweep clears every busy flag (R-92). A stored failure survives the restart (OQ-13) |

### 3.2 States of one piece of engine work

One work item goes through these states:

1. **Checked.** The handler ran the synchronous checks: the Game exists, the request is well formed, the user-error conditions (R-44, R-45, R-65 and the /options preconditions) are tested, and the needed records are readable (R-46). A failure here answers 400 or 500 and no work item is created. Holds: the request's parsed content and the captured Game id.
2. **Claimed.** The handler wrote the busy flag in one compare-and-set unit: "set busy with step X, only if not busy" (assumption A6). If the write reports "already busy", the request answers `200` "Still thinking..." (R-70) and there is no work item. Otherwise the request is answered at once (R-09) and the work item exists. Holds: Game id (fixed), work kind, the request's content, and a working copy of the game state.
3. **Running, at a step.** The status field names the current step. The work item holds its working game state (the live state, or the restored state-before for a redo) and the results of finished steps. A step that made a black-box call commits its results as a storage unit before the next step starts, so a cut-off never loses a committed step (R-90).
4. **Ended, success.** The last unit wrote the final state, the offered set, and status `idle`; the busy flag cleared. Nothing else holds.
5. **Ended, failure.** The failure line, the raw text, and the cleared busy flag were written in one unit. The story holds whatever earlier units of this work committed (R-17, R-47, R-48, R-58, R-66).
6. **Cut off.** The process died while the item ran. Nothing observes this state while it happens; at the next start the sweep turns the Game's status to idle (§9). Committed units survive; uncommitted work is gone (E-01, E-02).

## 4. Swipes

**What binds to a Swipe.** The text; the stored inputs (guide, impersonate direction, impersonated mark) that produced it; its provenance; for an event Swipe, the copy of the trigger context; its room report; and its state-after snapshot. The state-before snapshot is per Message, not per Swipe (§2.2).

**What a swipe switch gives back.** The route answers with the story log (§2 of the spec's HTTP table). Before answering, it applies one storage unit to the Game:

- the active Swipe index of that Message moves to `:index`;
- if the target Swipe has a state-after snapshot, the live game state becomes that snapshot — this is R-36. If it has none (the player-typed first Swipe of an Input, or the opening Swipe), the live game state stays as it is (OQ-17);
- the status becomes idle, which also clears a stored failure (R-37);
- the offered set is cleared (R-38, per the OQ-01 answer);
- the offered set is *not* restored from the Swipe; Swipes do not store offered sets (OQ-01).

**Refusals.** A switch on a Message that is not the last Message answers `400` (R-35). A bad index or unknown Message answers `400`. Missing or unreadable records that the switch needs (the Message, its Swipes, the snapshot) answer `500` with a status failure naming what is missing (R-46, R-91). On a busy Game the switch is refused with `200` "Still thinking..." (OQ-08).

**Retry's relation to Swipes.** A Retry never changes an existing Swipe's text (R-42); it appends one Swipe to the last Message and makes it active (R-41). The Message's identity, kind, event header and state-before snapshot are unchanged.

## 5. Retry and Retrigger

The kind of the last generation is read from the provenance of the last Message's active Swipe. Every redo follows the same shape: synchronous checks, claim, restore the starting state, run the redo generation, apply the analysis, commit, end.

### 5.1 A Narration that answers an Input (free text)

- **The narrator reads:** the story up to but not including the Narration Message being redone — so it reads the Input with its current, possibly edited text (R-51), and everything before it. It does not read the old Narration it replaces, nor the sibling Swipes.
- **Starts from:** the Message's state-before snapshot. The Quantifier's "current room" is the room of that restored working state, so the redo is a faithful re-run of the generation from its own start (R-52).
- **Steps that run again:** pre-flight room check; `narrating`; commit of the new Swipe with its end-state; `quantifying`; world rules (full application: movement, Characters, encounters, Triggers — R-52); `generating-event` if a Trigger fires; the post-Narration Options step (R-31/R-32 apply to the Narration this Retry wrote — assumption A9).
- **What it adds:** one Swipe on the Narration Message (R-41); and, when the redo fires a Trigger, one event Narration Message after it (see the OQ-02 answer: R-13 and R-52 win over the source that says a Retry adds no Message).
- **Failure leaves:** if the narrator call failed, gave empty text, or the room was invalid: no Swipe, the existing Swipes untouched, the live state still exactly as the request found it (R-47, R-49 — the deferred-commit design never touched it), and the failure line on the status. If the redo's Narration committed but the event Narration failed: the new Swipe stays, the live state is the redo's end state (R-48), the status reports that the event narration failed, the context stays stored, and Retrigger is offered (R-17).

### 5.2 A guided Narration

As §5.1, except the narrator also reads the **stored guide** from the active Swipe, and the new Swipe stores the same guide (R-53). No Input exists; the guide is the only stored input (it never becomes an Input). The redo is analysed, can move the player, and the new Swipe reports the room the redo arrives in, or no room (R-53, R-23).

### 5.3 An event Narration (including one a Retrigger wrote)

- **The narrator reads:** the story up to but not including the event Message, and the **trigger-context copy** stored on the active Swipe (R-56 — a copy, so later deletes and later Trigger firings cannot take it away).
- **Starts from:** the event Message's state-before snapshot — the state as it was before the replaced event Narration was written (R-55). A swipe switch before the Retry restored some sibling's end state, but the redo ignores that and uses the snapshot, exactly as R-55 demands.
- **Steps that run again:** pre-flight room check (though the redo never moves, R-57, so a room contradiction would come from the snapshot); `narrating` for an event Narration; commit; `quantifying` with the **restricted application** (Characters present and encounter record only; movement dropped, no Trigger can fire — §6.1e); the post-Narration Options step.
- **What it adds:** one Swipe on the event Narration Message, carrying a copy of the same context. The Narration that fired the Trigger stays in the story, also for a Retrigger-written event (R-54).
- **Failure leaves:** the status reports that the event narration failed; no Message and no Swipe added; the story keeps the triggering Narration; the player stays in the room where the Retry found them (nothing was committed); and a one-shot Trigger still counts as fired, so it cannot fire again (R-58).

### 5.4 An impersonated Input

- **The narrator reads:** the story up to but not including the Input Message, plus the **stored direction** from the active Swipe. It writes as the Persona.
- **Starts from:** the Input Message's state-before snapshot.
- **Steps that run again:** pre-flight check; `narrating` (an Impersonate generation); commit of a new Swipe on that Input, keeping the direction and the impersonated mark, and using the impersonate preset as it is when the call starts (R-59); `quantifying` and full world rules — the redo runs as a fresh Impersonate runs, so it can move the player and fire a Trigger (R-60, PD-2); `generating-event` if fired. The Input-writing generation leaves the offered set as it was (R-30), so no post-Narration Options step.
- **What it adds:** one Swipe on the Input; possibly one event Narration Message.
- **Failure leaves:** as §5.1 (R-49, R-47).

### 5.5 An Input that the player typed (no stored inputs)

- **The narrator reads:** the story up to and including the Input's current active Swipe text — the material it must rewrite — plus the story before it.
- **Starts from:** no state change; the live state stays as found.
- **Steps that run again:** `narrating` only (a user-regen generation). No Quantifier call, no world rules (assumption A2; R-18's "does not move the player" holds trivially).
- **What it adds:** one Swipe on the Input, provenance `user-regen` (R-61).
- **Failure leaves:** no Swipe, state untouched, failure line on the status (R-49).

### 5.6 Retrigger

- **The narrator reads:** the whole story as the request finds it, ending with the last Narration, plus the **stored trigger context** from the Game's slot.
- **Starts from:** the live game state as the request finds it (R-63). It never moves the player.
- **Steps that run:** the checks of R-65 (synchronous 400s: no stored context; empty story; last Message an Input; last Message an event Narration; no Game); claim; `generating-event`; commit of one new Message at the end of the story; `quantifying` with the restricted application (R-14); the post-Narration Options step (the added Message is a Narration, so R-31/R-32 apply).
- **What it adds:** exactly one event Narration Message with its single Swipe (R-64 — "adds no Swipe" means no additional Swipes; the Message's own first Swipe is part of the Message, per R-34). It adds no other Message; the story before it stays.
- **Failure leaves:** the status reports the event narration failed; no Message added; the story still ends in the same Narration; the context stays stored, so a Retrigger can try again without a new Action (R-66, R-67).

### 5.7 What a Retrigger can do after each kind of Retry

- **After a Narration, guided or Impersonate redo that succeeded without firing a Trigger:** the last Message is a non-event Narration, so Retrigger is allowed iff a trigger context is stored (an earlier Trigger fired and was never overwritten). If allowed, it appends the event.
- **After a redo that fired a Trigger and whose event Narration succeeded:** the last Message is an event Narration → Retrigger answers 400 (R-65).
- **After a redo whose event Narration failed:** the last Message is the triggering Narration and the context is stored → Retrigger is allowed and recovers the event (R-17, R-67).
- **After an event-Narration redo (success or failure):** the last Message is an event Narration → 400.
- **After a user-regen Retry:** the last Message is an Input → 400.
- **After a failed Retry that added no Swipe:** the story ends where it did before, so Retrigger's answer is whatever it was before the Retry.

## 6. Writers

Every writer below obeys the same four rules, which together prevent lost updates (R-73, R-74):

1. It captures the Game id **once, when the request arrives** — for routed requests, the id in the route; for the others, the active-Game pointer read once — and never dereferences the pointer again.
2. Engine work runs only after it won the Game's atomic busy claim; a second claim for the same Game gets "Still thinking..." and writes nothing.
3. The small synchronous writers (edit, delete, swipe switch) are refused while the Game is busy, so they never interleave with running work; when the Game is idle, only one of them can be inside its storage unit at a time, and each is a single unit.
4. Every write is a unit that names its Game (R-90). Storage refuses writes to a Game id that no longer exists, which is how a reset neutralises still-running work.

| Writer | May it run while the Game is busy? | Effect on running work | Writes to | Lost-update prevention |
|---|---|---|---|---|
| Action worker (free text, continue, guided, impersonate) | No — it *is* the busy work | — | Its captured Game | Claim rule 2; captures id; unit commits |
| Options worker (`/options` Action) | No — same | — | Its captured Game | Same |
| Retry worker | No — same | — | Its captured Game | Same |
| Retrigger worker | No — same | — | Its captured Game | Same |
| Swipe switch | No — refused "Still thinking..." (OQ-08) | None | Its routed Game | Rule 3 |
| Edit | No — refused (OQ-08) | None | Its routed Game | Rule 3 |
| Delete | No — refused (OQ-08) | None | Its routed Game | Rule 3 |
| Reset | Yes, at any time (E-05) | The running work finds its Game deleted at its next write and ends silently; its writes are dropped | Deletes the old Game's records; writes the new Game and the pointer | The work's writes carry the old id, which storage rejects; the new Game has a new id (R-74) |
| Create | Yes | None on other Games' work | The new Game and the pointer | Pointer write is one unit |
| Switch | Yes | None — work continues for its own Game (OQ-09) | The pointer only | Rule 1: no work re-reads the pointer |
| `POST /status/reset-generating` | Only when the status is not busy; when busy it answers "failed" and changes nothing (OQ-07) | None | The captured active Game | Single-field unit |
| Startup sweep | Runs before requests are served | None left to affect | Every Game with a stale busy flag | One sweep pass at startup |
| Status, story-log and dock polls; debug route | Yes, always | Read-only | — | Reads see only whole committed units (R-90) |

### 6.1 The Action worker in detail

The parsing of kinds is R-01: free text; empty text (continue); `/guide <text>`; `/impersonate <direction>`; `/options`; any other `/`-prefixed text is free text. Using an offered option sends its text as a free-text Action (R-06).

**a. Pre-flight.** Before any story write: the player's room must be on the map (R-20) — for a redo, the room of the restored working state. A missing preset for the kind about to run is detected here too and fails with its own line (R-21). Nothing has been written.

**b. Input take-in.** A free-text Action commits one unit: the Input Message with its player-typed Swipe (text = the command; no room report; no state-after), plus the unchanged state (R-02, R-08). A continue, guided or impersonate Action commits no Input (R-03, R-04).

**c. `narrating`.** Prompt assembly receives: the World; the Persona; the working game state; the **whole story** (all Messages before the new text, each with its active Swipe text and kind — assumption A1); the player's text for this generation, if any; the stored inputs (guide, direction); the trigger context, for an event Narration; the preset read at call time. A notice is never among these (R-82).

**d. Narrator failure.** No Message and no Swipe for that text; the failure line per kind — including the dedicated lines for a missing preset and a spent output budget, and the "text was empty" line — with the raw text in the disclosure (R-19, R-21). The busy flag clears in the same unit. For a free-text Action, the Input committed in (b) stays (OQ-06 default).

**e. Commit of the text.** One unit: the new Message (Input's Narration, Impersonate Input, or event Narration), its Swipe (text, stored inputs, provenance, room report), the Message's state-before snapshot, the new live game state, the updated fired-Trigger set and trigger context if a Trigger fired, and the current step. The room report is the arrival room when this generation's world rules moved the player, else none (§8).

**f. `quantifying`.** The Quantifier reads the new text, the player's text that led to it, the working state's room, the Characters that can be in that room (for a room the World does not have: all the World's Characters — assumption A7b), and a window of recent story. A failure or unusable answer arrives as the black box's **fallback result**, which the work applies and reports with a notice; the generation still ends well (R-12, OQ-03).

**g. World rules.** The analysis result is applied to the working state. For a Narration or Impersonate generation the application is **full**: movement, added rooms, present Characters, encounter records, Trigger finding (R-11). For an event Narration generation it is **restricted**: the movement is dropped from the analysis result before it is given to the world rules, and a Trigger the rules report is ignored — the event's analysis changes only the present Characters and the encounter record (R-14), never moves the player (R-57), and fires nothing further (R-15 stays true because no Trigger *fires* in the restricted application). A user-regen generation runs no Quantifier at all (A2). A contradiction reported by the world rules is the missing-room condition, which the pre-flight check has already removed; if one is nevertheless reported after the text was committed, the work ends with the same failure line and the committed text stays (assumption A8).

**h. `generating-event`.** When a Trigger fired: the context goes into the Game's slot, and a second generation writes the event Narration (a fresh `narrating` step for the event kind, then its own restricted `quantifying`). Its failure is the R-17 end state; its success appends the event Message after the text that fired the Trigger, with the event header and the context copy on its Swipe (R-13). An event Narration's Swipe reports no room, always (R-24).

**i. Post-Narration Options step.** After the last Narration the work added (its own or an event's), when the work is otherwise ending well: toggle off → the offered set is emptied (R-32); toggle on → step `options`, an Options call is made when the Game has one: success replaces the set (R-31); failure or a missing Options call empties the set and issues the notice (R-33, R-83). A generation that wrote an Input (Impersonate, Impersonate redo, user regen) skips this step and leaves the set as it was (R-30). If the event Narration failed, the work ends in failure and this step does not run; the set stays as it was.

**j. End.** The last unit sets the status idle (or the failure) and clears the busy flag (R-10).

### 6.2 The Retry and Retrigger workers

They run the shape of §5, with these synchronous answers before the claim: busy → `200` "Still thinking..." (R-70); user errors → `400` and nothing starts (R-44 per OQ-05, R-45, R-65); a needed record missing or unreadable → `500` and a status failure naming it (R-46, R-91). A Retry never leaves the Game busy, also after a delete: the busy flag is cleared in every end path (R-50).

### 6.3 The small handlers

- **Edit** replaces the text of the active Swipe of the addressed Message in one unit (R-75). Later generations read the story's active Swipe texts, so they read the edited text. The Swipe's other fields do not change.
- **Delete** removes the last Message with all its Swipes in one unit (R-76). The game state does not change (OQ-11 default). The next Action appends as usual; a Retry after a delete redoes the new last generation (R-50).
- **Reset** deletes the active Game's records and creates the fresh Game in one or two units, points the active pointer at it, and answers with the dashboard reload. The fresh story holds only the Opening Narration, which is created from the World with no LLM call; the next Action adds the first Input (R-77). The fresh Game takes the same World and Persona (assumption A4).
- **Create** and **switch** write the pointer; the dashboard reloads (R-78, R-79). A switch to a Game whose work was cut off finds it idle because of the sweep (R-80).

### 6.4 The Options paths

On demand (`/options`): the checks are the busy check and the "nothing to work from" check — the run fails at once, naming the reason, when the story holds no readable text: no Messages, or only an Opening Narration whose text is empty (R-27; the precise test is assumption A10). A run that starts makes the Options call, replaces the set on success, and leaves the set and issues the notice on failure or when the Game has no Options call (R-29, R-83). `/options` adds no Message (R-28).

After each Narration: the step in §6.1i, which implements R-31, R-32 and R-33.

## 7. Notices

**Kinds.** An Options run failed (with the reason); the Game has no Options call (with that reason) — both required by R-83; and, by the OQ-03 answer, an analysis that fell back.

**Where.** A notice is an entry in the Game's notice list (§2.4). It is **not a Message**: it never enters the story, so it cannot change which Message a Retry, Retrigger, swipe switch or delete acts on, cannot be edited or deleted, and does not move the story log's controls (OQ-04). The dashboard shows a notice strip next to the story log, reading the active Game's notice list; this is where the design puts the R-83 telling.

**Who sees it.** The player looking at that Game's dashboard. A second tab polling the fragments sees it too.

**How long.** A notice of a kind replaces the previous notice of the same kind; all notices for a Game are cleared when the next engine work on that Game starts, so they always describe the most recent finished work (assumption A10). A restart does not clear them (they are in storage).

**Out of every prompt.** The notice list is not part of the story, not part of any game-state snapshot, and is never among prompt assembly's inputs (§6.1c). Prompt assembly removes nothing, so the only safe place for notice text is outside its inputs — which is where the design keeps it (R-82).

## 8. The room report

The room a generation arrives in is **kept on the Swipe that the generation wrote**, written at the commit of §6.1e from the world-rules result: the arrival room's name when that generation's application moved the player; none otherwise (R-22). Consequences the design guarantees:

- The Input of a free-text Action reports no room (it is not the generation's output).
- Each Swipe reports only its own generation's arrival, so a redo that does not move reports no room even when a sibling reports one (R-23); a swipe switch changes what the Message reports, because the story log reads the **active Swipe's** room report (R-25).
- Event Swipes are forced to no room (R-24); the restricted application cannot move the player anyway.
- An Impersonate Swipe reports its arrival room (OQ-15 answer).
- The Opening Narration's single Swipe reports the World's starting room (OQ-16 answer); it is the one report written without a generation.
- A user-regen Swipe reports no room (no analysis, no movement).

The story log's reader takes, per Message, the active Swipe's room report, its Swipe count, its text and its event header (§3.7).

## 9. Restart and cut-off work

**What a restart finds.** Storage holds only whole units (R-90), so the story and game state are never torn. What cut-off work can have left behind, all of it legitimate committed state: an Input with no Narration (committed at §6.1b before the call); a Narration whose event Narration never ran; a committed Swipe whose later steps never ran; a stored trigger context with no event Narration for it — which is exactly what Retrigger recovers (R-67); and a status field that still says busy with a stale step name.

**What it does.** Before serving requests, the sweep clears every Game's busy flag back to idle (or to the stored failure, if the Game had already recorded one and was not running — a busy flag is always cleared) (R-92). After the sweep no Game is busy, cut-off work never looks busy (R-72), and a switch to such a Game finds it idle and accepting Actions (R-80).

**What the player sees.** The story as the commits left it — for example their Input with no answer under it — and an idle status. The design keeps no record of the cut-off itself (what the process knew in memory is lost, E-01), so nothing reports it beyond the visible gap in the story (OQ-12 answer). The player can delete the leftover Input or simply act.

## 10. Requirement trace

| R | Section | Forced/free |
|---|---|---|
| R-01 | §6.1 (kinds), §6 intro | forced |
| R-02 | §6.1b | forced |
| R-03 | §6.1b | forced |
| R-04 | §6.1b, §2.3 (stored inputs) | forced |
| R-05 | §6.1b, §2.3 | forced |
| R-06 | §6.1 (option click) | forced |
| R-07 | §6.1e, §6.1h | forced |
| R-08 | §6.1b, §2.2 | forced |
| R-09 | §3.2 (claimed), §6.1j | forced |
| R-10 | §3.1 (transitions) | forced |
| R-11 | §6.1g (full application) | forced |
| R-12 | §6.1f, §7 | free (the fallback notice is the OQ-03 choice) |
| R-13 | §6.1h | forced |
| R-14 | §6.1g (restricted application) | forced |
| R-15 | §6.1g | forced |
| R-16 | §2.1 (fired-Trigger set), §6.1g | forced |
| R-17 | §6.1h (failure path) | forced |
| R-18 | §6.1g (user regen: no Quantifier) | free (skip analysis vs analyse-with-movement-suppressed) |
| R-19 | §6.1d | forced |
| R-20 | §6.1a (pre-flight) | forced |
| R-21 | §2.1 (status field), §6.1d | forced (line wording free) |
| R-22 | §8 | forced |
| R-23 | §8, §2.3 | forced |
| R-24 | §8, §6.1h | forced |
| R-25 | §8, §4 | forced |
| R-26 | §6.4 | forced |
| R-27 | §6.4 (test is A10) | free |
| R-28 | §6.4 | forced |
| R-29 | §6.4, §7 | forced |
| R-30 | §6.1i | forced |
| R-31 | §6.1i | forced |
| R-32 | §6.1i | forced |
| R-33 | §6.1i, §7 | forced |
| R-34 | §2.2, §6.1e | forced |
| R-35 | §4 (refusals) | forced |
| R-36 | §4 (state-after restore) | forced |
| R-37 | §4 | forced |
| R-38 | §4 | forced |
| R-40 | §5 (intro) | forced |
| R-41 | §5, §6.2 | forced |
| R-42 | §2.3, §6.2 | forced |
| R-44 | §6.2 — answered per OQ-05, open in §12 | free |
| R-45 | §6.2 | forced |
| R-46 | §6.2, §6.4, §3.2 (checked) | forced |
| R-47 | §5.1 (deferred commits), §3.2 | forced |
| R-48 | §5.1, §6.1e (per-generation commits) | forced |
| R-49 | §5.1–5.5 (failure leaves) | forced |
| R-50 | §6.2, §3.1 | forced |
| R-51 | §5.1 | forced |
| R-52 | §5.1 | forced |
| R-53 | §5.2 | forced |
| R-54 | §5.3 | forced |
| R-55 | §5.3 (state-before), §2.2 | forced |
| R-56 | §2.3 (context copy), §5.3 | forced |
| R-57 | §5.3 | forced |
| R-58 | §5.3 (failure leaves) | forced |
| R-59 | §5.4 | forced |
| R-60 | §5.4 | forced |
| R-61 | §5.5 | forced |
| R-62 | §5.6 | forced |
| R-63 | §5.6 | forced |
| R-64 | §5.6 | forced |
| R-65 | §5.6, §6.2 | forced |
| R-66 | §5.6 (failure leaves) | forced |
| R-67 | §5.6, §2.1 (slot) | forced |
| R-70 | §3.2 (claim), §6 intro | forced |
| R-71 | §2.1, §3.1 | forced |
| R-72 | §3.1, §9 | forced |
| R-73 | §6 intro (four rules) | forced |
| R-74 | §6 intro rule 1, §6.3 (reset) | forced |
| R-75 | §6.3 (edit) | forced |
| R-76 | §6.3 (delete; state effect per OQ-11) | free |
| R-77 | §6.3 (reset) | forced |
| R-78 | §6.3 (create) | forced |
| R-79 | §6.3 (switch) | forced |
| R-80 | §9 | forced |
| R-82 | §7 | forced |
| R-83 | §7 (placement free) | free |
| R-90 | §2, §6 intro rule 4, §3.2 | forced |
| R-91 | §6.2, §6.4 (checked state; 500 path) | forced |
| R-92 | §9 | forced |

## 11. Assumptions

Each point below is a place where the spec is silent or unclear and the design chose. OQ answers carry their reasons.

- **OQ-01, answered: R-38 holds.** After a swipe switch the dock shows no offered set, even when the Swipe's generation left one. Reason: R-38 is specific to the swipe switch and unqualified; R-36 governs the *game* the swipe switch restores (room, Characters, encounters, fired Triggers, context) — the offered set is transient guidance tied to the newest story, and Swipes do not store offered sets, so there is nothing faithful to restore. The offered set is kept outside the state snapshots.
- **OQ-02, answered: the redo can fire a Trigger, and an event Narration is added.** Reason: R-52 says explicitly that after a Narration redo "the Triggers are evaluated again" — that has no observable effect unless R-13 then applies — and R-13/R-15 are the requirements a reader can observe; the conflicting source's "never adds a Message" is about the Retry's own Swipe mechanics (R-41). The event Narration is appended at the end of the story, after the text that fired the Trigger.
- **OQ-03, answered:** (a) an analysis fallback issues a notice — otherwise the present Characters change invisibly; (b) a movement to an unmapped destination issues no notice — the room report on the Narration already names the new room, so a notice would duplicate it; (c) a failed or missing Options call at the end of a generation does **not** make that generation failed on the status — the status ends idle and the player is told by the notice, mirroring R-12's pattern that an auxiliary failure still ends the generation well.
- **OQ-04, answered: notices are not Messages.** Reason: R-82 requires that no notice reach an LLM; keeping notices outside the story makes that hold by construction, keeps the story log purely story, and leaves the controls of R-44/R-62/R-35 untouched.
- **OQ-07, answered: the route is kept and refuses while busy.** Reason: the HTTP table states the route, so the design must implement it. It answers "reset" when the active Game's status is not busy (a no-op that also clears a stored failure) and "failed" when work runs, changing nothing — clearing a busy flag under running work would break R-72's truthfulness; only the restart sweep or the work's own end may clear it.
- **OQ-08, answered: refused while busy.** Edit, delete and swipe switch on a busy Game answer `200` "Still thinking..." and start nothing. Reason: each of them writes the story or the state that the running work is reading from and writing to; allowing them would need interleaving semantics the spec does not define, and R-70's spirit is that a busy Game takes no changes. They run normally when the Game is idle or failed.
- **OQ-09, answered: the work finishes for its own Game.** Reason: R-74 fixes where the writes go, and nothing in the spec asks the engine to stop work mid-call. The Game keeps its own status the whole time; when the player switches back to it, the dashboard shows that status — busy with its step, or idle, or the failure — exactly as if the player had never left. R-80 is unaffected: it speaks of work a *restart* cut off.
- **OQ-10, answered: no narrator-written opening.** A Game of a World with an empty opening text still gets its Opening Narration Message, whose single Swipe holds empty text; the story log shows it as an empty entry, and the fresh Game simply waits for the first Action. Reason: the owner's ruling says the Opening Narration holds the World's opening text and no LLM call produces it — inventing an LLM-written opening would add a feature the spec does not ask for. So: it runs at Game creation, makes no busy period, shows nothing on the status, and cannot be retried (nothing was generated; see OQ-17).
- **OQ-13, answered: a failure stays until the Game's next work, a swipe switch on it, or the reset-generating route.** Reason: the failure is the status of the last generation on that Game; edit and delete do not end a generation, so they leave it; a switch of active Game shows the *other* Game's status and clears nothing; a restart leaves stored failures (they are still true), and only busy flags are swept.
- **OQ-14, answered: the context stays until another Trigger fires (overwriting the single slot) or the Game is removed.** A later Action does not clear it; a successful event Narration does not clear it (R-62 needs it afterwards); a delete of the event Narration does not clear it, so a Retrigger can write that event again. Reason: R-56 needs the context of an event across deletes, R-67 needs it across failures, and R-62's conditions already stop a Retrigger where it would be wrong; one slot is what the spec's singular "the stored trigger context" describes.
- **OQ-15, answered: the Impersonate Input reports the room.** Reason: the Impersonate generation's output Message *is* that Input; R-22's "the Input of that Action reports no room" speaks of the free-text Action's Input, which is not the generation's output. Reporting on the generation's own Swipe keeps R-23 and R-25 uniform.
- **OQ-16, answered: the opening reports the starting room.** Reason: the room report exists to tell the player where the scene is; the opening establishes it; and R-22's "a Message whose generation did not move the player" does not apply, since no generation ran.
- **OQ-17, answered: nothing is restored, and the switch still ends in idle with a cleared dock.** Switching to the player-typed first Swipe changes only the shown text (and a later generation reads it, R-75); the game state stays as the last work left it, the status becomes idle (R-37 is unqualified), the dock is cleared (R-38 is unqualified). Reason: R-36 speaks of what a Swipe's *generation* left; this Swipe has no generation, so there is nothing faithful to restore.
- **A1 (prompt and window sizes).** Prompt assembly receives the whole story (active Swipe texts, with kinds); the Quantifier and the Options call receive a window of the 8 most recent Messages. The spec says "short window" without a size; any faithful window meets the contracts.
- **A2 (user regen is not analysed).** R-11 enumerates the analysed generations and omits the user-regen kind; R-18 is the only statement about it. The design runs no Quantifier for it, so the game state cannot change; "does not move the player" holds.
- **A3 (restricted application).** For event Narrations the design passes the world rules an analysis result with the movement removed and ignores a reported Trigger. This uses the black boxes through their contracts (§3.2, §3.4 take "an analysis result") and is how R-14 and R-57 are met without changing the boxes.
- **A4 (reset reuses World and Persona).** "Start a fresh one" is read as a fresh Game of the same World and the same Persona.
- **A5 (no active Game).** Requests that act on the active Game answer `400` when no Game exists. R-45 states this for Retry; the design extends it to the other active-Game routes for uniformity.
- **A6 (atomic claim).** The busy claim is one compare-and-set unit ("set busy with step X only if idle"). Two simultaneous work requests for one Game: the loser gets "Still thinking..."; no double work. E-03 makes this race real (two tabs).
- **A7 (snapshot cost, characters of added rooms).** (a) State snapshots are stored whole per Message (start) and per Swipe (end); the storage cost is accepted for the sake of R-36 and R-55 being exact. (b) For an added room, "the Characters that can be in that room" is passed as all of the World's Characters, since the World authors no list for a room it does not have.
- **A8 (contradiction after text).** The only specified contradiction is the missing room, which the pre-flight check removes. If the world rules nevertheless report a contradiction after the text was committed, the work ends with that failure line and the committed text stays; the spec's "before any text is written" is honoured where it can be.
- **A9 (Options step after a Retry).** A Retry whose redo is a Narration applies the post-Narration Options rules (R-31/R-32/R-33) to the Narration it added; a Retry whose redo is an event Narration likewise; a user-regen or Impersonate redo leaves the set as it was (R-30). Reason: R-31 says "after each Narration" without limiting it to Actions.
- **A10 (notice lifetime).** Notices are per Game, replaced per kind, and cleared when the next engine work on that Game starts. Any lifetime the owner prefers can replace this without touching the rest of the design.

## 12. Questions for the owner

- **OQ-05 — a Retry with no Input in the story.** Why it matters: R-44 read literally refuses any Retry while the story holds no Input, which makes a continue-only or guided-only stretch unretryable; R-40 and R-53 say those generations are redoable. The two cannot both hold in every case. Answer I would take: a Retry is allowed whenever the last Message holds a generation-written Swipe (any kind) or is a player-typed Input; R-44's 400 applies when there is nothing to redo — an empty story, or a story ending in the Opening Narration.
- **OQ-06 — an Input whose Narration failed.** Why it matters: it decides whether the player can re-attempt the answer to their own words with Retry, or only rewrite the words. Answer I would take: the Input stays in the story (it was committed before the call), and a Retry on it is a user regen per R-61 — the narrator rewrites the player's Input; re-attempting the Narration is then done by acting again or by delete-plus-retype. If the owner prefers Retry to re-attempt the Narration, the design adds a special case: a Retry on an Input whose provenance is player-typed *and* whose Narration attempt failed adds a Narration Message instead of a Swipe.
- **OQ-11 — delete and the game state.** Why it matters: a delete of the Narration that moved the player leaves the player in the arrived-at room with the text gone; whether that is wanted shapes whether delete is a story tool or an undo tool. Answer I would take: delete changes only the story — the room, encounters, fired Triggers, trigger context and offered set stay as they are; any kind of Message may be deleted, and the opening too (an empty story then refuses Retry and Retrigger).
- **Q-4 — the degraded-role banner's source.** Why it matters: §3.7 says the header banner reports a degraded LLM role, but the LLM call log is the narrator-call black box's own record, and this spec's part owns no writer for it. Answer I would take: the banner's view reads a role-health note that the call-log side exposes; if the owner wants it sourced from this design's records instead, a small per-Game "last call role health" field would be added, written by the same commits as the failure lines.
