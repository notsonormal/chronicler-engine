# Design: game-state lifecycle

## 1. Summary

The lifecycle keeps every Game as a **story** (an ordered list of Messages), a **game state** (room, characters, triggers, options), and a **status**. Each Message has one or more **Swipes**, and the active Swipe of the last Message is what the player sees. Every Swipe that comes from a generation stores the game state as that generation left it, so a swipe switch or a delete can roll the world back to a precise point.

All engine work is done inside a single task per Game. A per-Game lock and a persisted `busy` status make sure only one generation, Retry, Retrigger or Options run is in flight at a time. HTTP requests that start work return immediately; the task uses the narrator, Quantifier, world rules, Options call and prompt-assembly black boxes and then commits one whole unit of writes. If the process restarts, the start-up pass clears every `busy` flag, so no Game can stay locked by dead work.

A Retry redoes the last generation by rolling the game state back to the snapshot stored just before that generation, writing one new Swipe on the same Message, and re-running the same black-box steps. A Retrigger writes a new event Narration from the stored trigger context. Notices are kept outside the story so they can never reach an LLM prompt.

Notable choices made because the spec is silent or conflicted:

- After a swipe switch the game state is restored from the new active Swipe, but the **offered set is cleared** (R-38 overrides R-36 for options).
- A Retry whose redo fires a Trigger **does** add a new event Narration Message; the Retry itself only adds the Swipe.
- When a free-text Action’s Narration fails, the **Input stays** in the story, and the next Retry on it is a user regen.
- Notices are **not Messages** and do not affect controls.
- Edit, delete and swipe switch are **refused while the Game is busy**.

---

## 2. What the design keeps

### 2.1 Per Game

A Game is one record for one playthrough of one World with one Persona.

| Field | What it holds | Who writes it | When it changes |
|---|---|---|---|
| `id` | Unique Game identifier. | Created at `POST /games` or reset. | Never. |
| `worldId`, `personaId` | The World and Persona the Game belongs to. | Created with the Game. | Never. |
| `alwaysOnToggle` | Copied from the World at creation. | Created with the Game. | Never. |
| `story` | Ordered list of Message ids. | Action handlers, generation workers, Retry/Retrigger workers, delete, reset. | After every Message add or delete. |
| `gameState` | Current room; rooms added to the map; present Characters; encounter record of each Character; fired one-shot Triggers; stored trigger context; offered set. | World rules during generations; swipe switch; delete; reset; startup after restart. | After every generation, swipe switch, delete, reset, or restart cleanup. |
| `status` | `idle`; or `busy` with the running step name and work id; or `failure` with a failure kind, raw text and timestamp. | Request handlers set `busy`; workers set `idle` or `failure`; swipe switch/edit/delete/switch/reset set `idle`; startup clears `busy`. | At the start and end of every work unit, and on several non-work requests. |
| `notices` | Short-lived notices to the player, separate from the story. | Workers when Options fails or is missing; possibly other transient UI events. | Added when needed; cleared on next successful generation or explicit dismiss. |
| `version` / `workId` | A tag used to confirm that a running worker is still the legitimate owner before it writes. | Set when work starts; cleared when work ends or is cancelled. | With every status transition. |

The **active Game** is not a field inside the Game record. It is a session/dashboard pointer that tells request handlers which Game to act on. A worker is bound to a concrete Game id when it starts, so a later switch of the active Game cannot redirect its writes.

### 2.2 Per Message

| Field | What it holds | Who writes it | When it changes |
|---|---|---|---|
| `id` | Unique Message identifier. | Created with the Message. | Never. |
| `gameId` | The Game it belongs to. | Created with the Message. | Never. |
| `ordinal` | Position in the story. | Created with the Message. | Never. |
| `kind` | `opening`, `input`, `narration`, or `event`. | Created with the Message. | Never. |
| `swipes` | List of Swipe ids. | Created with the first Swipe; Retry and user regen append a Swipe. | When a Retry or user regen succeeds. |
| `activeSwipeIndex` | Index of the Swipe currently shown. | Retry makes the new Swipe active; swipe switch changes it; a user regen makes the new Swipe active. | On Retry success or swipe switch. |
| `eventHeader` | For `event` Messages, the name of the Trigger. | Created with the event Message. | Never. |
| `sourceTriggerId` | For `event` Messages, the Trigger that produced it. | Created with the event Message. | Never. |

### 2.3 Per Swipe

A Swipe is one version of one Message.

| Field | What it holds | Who writes it | When it changes |
|---|---|---|---|
| `id` | Unique Swipe identifier. | Created with the Swipe. | Never. |
| `text` | The displayed text. | Created with the Swipe; edit changes it. | On edit or on creation. |
| `storedGuide` | The guide text, if this Swipe was produced by a guided Action. | Created with the Swipe. | Never. |
| `storedImpersonateDirection` | The direction, if this Swipe was produced by `/impersonate`. | Created with the Swipe. | Never. |
| `isImpersonated` | True if the narrator wrote this Swipe as the Persona. | Created with the Swipe. | Never. |
| `generationType` | `typed` (player text, no generation), `narration`, `guided`, `continue`, `impersonate`, `user-regen`, or `event`. | Created with the Swipe. | Never. |
| `roomReport` | The room the generation arrived in, or none. | Set when the world rules report a movement; for the opening Swipe it is the starting room. | Never after creation. |
| `postGenerationState` | A full snapshot of the Game’s game state after this Swipe’s generation. For a typed input it is the state before the Input. | Created with the Swipe. | Never. |
| `triggerContext` | For an `event` Swipe, the Trigger id and direction that produced it. | Created with the event Swipe. | Never. |

The `postGenerationState` snapshot is the key to R-36, R-55 and delete. It contains every mutable part of the Game state so that restoring it exactly reproduces the world at that moment.

### 2.4 Per running work unit (transient)

| Field | What it holds | Who writes it | When it changes |
|---|---|---|---|
| `workId` | Identifier of this task. | Created when the request is accepted. | Never. |
| `gameId` | The Game the task belongs to. | Bound at acceptance. | Never. |
| `step` | `narrating`, `quantifying`, `generating-event`, or `options`. | Updated as the worker moves through black-box calls. | During the work. |
| `cancelled` | Set if `/status/reset-generating` aborts the task. | Set by the reset route. | Only when aborted. |

The work unit lives only in the process memory. On restart it disappears; the database `status` is the authoritative record of whether a Game looked busy.

---

## 3. States

### 3.1 States of a Game

A Game is always in one of three logical states, recorded in `Game.status`.

| State | What it holds |
|---|---|
| **Idle** | `status = idle`. No worker holds the Game lock. The Game accepts new work and non-work mutations. |
| **Busy(step)** | `status = busy(step, workId)`. A worker is running. `step` is one of `narrating`, `quantifying`, `generating-event`, or `options`. Requests that start work are told “Still thinking…”. Edit, delete and swipe switch are refused. |
| **Failure(kind)** | `status = failure(kind, raw, ts)`. The previous generation ended in failure. The story shows the last committed Message. A new request either clears the failure or replaces it with a new outcome. |

### 3.2 State transitions

| From | Event | To | Details |
|---|---|---|---|
| Idle | `POST /action`, `POST /swipe/new`, or `POST /retrigger` accepted | Busy(step) | Lock acquired, `workId` written, `200` returned immediately. |
| Busy | Another `POST /action`, `/swipe/new`, or `/retrigger` | Busy (no change) | Answer `200` “Still thinking…”. Nothing starts. |
| Busy | `POST /message/:id/swipe/:index`, `POST /history/:id`, or `POST /history/delete` | Busy (no change) | Answer `409 Conflict`. The running work is not interrupted. |
| Busy | `POST /reset` or `POST /games` that replaces this Game | Cancelling then idle/deleted | Reset cancels the worker, deletes the Game and creates a fresh one. Create leaves the old worker running for its own Game id. |
| Busy | Worker completes successfully | Idle | Final unit commits the new Swipe/Message, updated state, and `idle`. |
| Busy | Worker fails | Failure | Final unit commits the failure reason and raw text; no invalid Message/Swipe is committed. |
| Failure | Any successful mutation (new work, edit, delete, swipe switch, switch, reset) | Idle or Busy | The old failure is cleared. |
| Any | `POST /status/reset-generating` | Idle | Cancels running work if any and forces status to `idle`. |
| Any | Process restart | Idle | Start-up pass clears every `busy` status and discards in-memory locks. |

### 3.3 States of one piece of engine work

A single request that starts work goes through these states.

| State | What it holds |
|---|---|
| **Requested** | The HTTP request has arrived and is waiting for the Game lock. |
| **Accepted** | Lock acquired; `status` set to `busy(step, workId)`; `200` already sent. |
| **Narrating** | The narrator call is in progress. |
| **Quantifying** | The Quantifier is analysing the new text. |
| **Applying rules** | The world rules are updating game state and deciding whether a Trigger fires. |
| **Generating event** | An event Narration is being written because a Trigger fired. |
| **Options** | The Options call is running. |
| **Persisting** | The worker is committing one unit of writes. |
| **Ended** | The worker releases the lock and the Game is `idle` or `failure`. |

### 3.4 Transitions of one piece of work

| From | Cause | To | Result |
|---|---|---|---|
| Requested | Game is busy | Rejected | `200` “Still thinking…”. |
| Requested | Request invalid | Rejected | `400` or `500` as required. |
| Requested | Lock acquired | Accepted | `200` returned; `busy` persisted. |
| Narrating | Narrator failure or empty text | Failure | No Message/Swipe added; status failure; lock released. |
| Narrating | Current room not on map | Failure | No text written; status names the invalid room. |
| Narrating | Success | Quantifying | Text produced; next step. |
| Quantifying | Fallback result | Applying rules | Message/Swipe still added; status will be `idle`; no movement. |
| Applying rules | No Trigger | Options/Idle | State updated; no event Message. |
| Applying rules | Trigger fires | Generating event | Trigger context stored; event Narration generated. |
| Generating event | Narrator failure | Failure | Firing text stays; no event Message; trigger context stays stored; status reports event narration failed. |
| Generating event | Success | Options/Idle | Event Message added; status `idle`. |
| Options | Failure or no Options call | Idle | Offered set handled as in R-29/R-33; notice shown. |
| Options | Success | Idle | Offered set replaced. |
| Any | Restart | Ended (lost) | Work disappears; startup clears `busy`. |

---

## 4. Swipes

### 4.1 What binds to a Swipe

Every Swipe binds to exactly one attempt to produce or reproduce a piece of text:

- A free-text or continue Action creates one Narration Swipe.
- A guided Action creates one Narration Swipe with `storedGuide` set.
- An Impersonate Action creates one Input Swipe with `storedImpersonateDirection` and `isImpersonated = true`.
- A Retry creates one new Swipe on the same Message as the generation it redoes.
- A user regen (Retry on a typed Input) creates one new Input Swipe with `generationType = user-regen`.
- An event Narration creates one event Swipe with `triggerContext` and `eventHeader`.
- The opening Narration creates one opening Swipe with the World’s opening text.

Each Swipe also carries:

- The `roomReport` that belongs to its own generation.
- The `postGenerationState` snapshot that belongs to its own generation (or, for a typed input, the state before it).

### 4.2 What a swipe switch gives back

`POST /message/:id/swipe/:index` is allowed only when `:id` is the **last Message** (R-35). The design then:

1. Validates that `:index` exists in that Message.
2. Sets `activeSwipeIndex` to `:index`.
3. Reads `postGenerationState` from the newly active Swipe and writes it into `Game.gameState`, **except** for the offered set, which is cleared.
4. Sets `Game.status` to `idle`.
5. Returns the story log.

This satisfies R-36 for all world state and R-25 for the room report. The offered-set behaviour is the resolution of OQ-01: R-38 is treated as the specific rule for the dock, so after every swipe switch the dock shows no offered set.

A swipe switch to the first Swipe of a typed Input has no generation to replay. That Swipe’s snapshot is the state before the Input was added, so the switch restores that prior state.

---

## 5. Retry and Retrigger

### 5.1 Shared Retry behaviour

A Retry (`POST /swipe/new`) first checks:

- Is a Game active? If not, `400` (R-45).
- Is the Game busy? If yes, `200` “Still thinking…”.
- Does the story contain at least one Input? If not, `400` (R-44).
- Can the data needed for the last generation be read? If not, `500` data-integrity failure (R-46).

If accepted, the Retry rolls the game state back to the snapshot stored just before the last generation, runs the generation again, and adds exactly one new Swipe to the last Message. Existing Swipe texts are never changed (R-42).

If the redo is a generation that can fire a Trigger (Narration answering an Input, guided Narration, continue Narration, or Impersonate Input) and the world rules report that a Trigger fires, the engine writes a new event Narration Message after the redone Message. This is the resolution of OQ-02.

### 5.2 Retry by kind of last generation

| Kind of last generation | Part of the story the narrator reads | Game state the redo starts from | Steps that run again | What the request adds to the story | What a failure leaves |
|---|---|---|---|---|---|
| **Narration that answers an Input** | The full story up to and including the Input, using the Input’s currently active (possibly edited) text. | The `postGenerationState` of the active Swipe of the Input Message — i.e., the state just before the Narration was first written. | Narrator call for Narration; Quantifier; world rules; possible event Narration; Options if always-on. | One new Swipe on the Narration Message. If the redo fires a Trigger, a new event Message is also added. | No new Swipe; no new event Message; existing Swipes unchanged; state unchanged; status failure. |
| **Guided Narration** | The story before the guided Narration, plus the `storedGuide` of the active Swipe. | The `postGenerationState` of the active Swipe of the previous Message. | Same as above, with the guide. | One new Swipe on the guided Narration Message; possible event Message. | No new Swipe; state unchanged; status failure. |
| **Continue Narration** | The story before the continue Narration. | The `postGenerationState` of the active Swipe of the previous Message. | Same Narration pipeline, but no player text and no guide. | One new Swipe on the continue Narration Message; possible event Message. | No new Swipe; state unchanged; status failure. |
| **Event Narration** | The trigger context stored on the event Swipe, plus the story up to the Narration that fired the Trigger. | The `postGenerationState` of the active Swipe of the Message that fired the Trigger — i.e., the state before the replaced event Narration was written. | Narrator call for event; Quantifier (no movement); world rules (updates characters/encounters). | One new Swipe on the event Message. No movement; no new event Message. | No new Swipe; status reports event narration failed; the firing Narration stays; trigger context stays stored; state unchanged; a one-shot Trigger still cannot fire again. |
| **Impersonated Input** | The story before the Impersonate Input, plus the `storedImpersonateDirection` of the active Swipe. | The `postGenerationState` of the active Swipe of the previous Message. | Narrator call for Impersonate; Quantifier; world rules; possible event Narration. | One new Swipe on the Input Message; possible event Message. | No new Swipe; existing Input Swipes unchanged; state unchanged; status failure. |
| **Typed Input / user regen** | The story before the Input. | The `postGenerationState` of the active Swipe of the previous Message. | Narrator call for user-regen only. No Quantifier, no world rules, no movement, no Trigger. | One new Swipe on the Input Message. | No new Swipe; state unchanged; status failure. |

### 5.3 Retrigger after each kind of Retry

Retrigger is allowed when:

- A trigger context is stored, **and**
- The last Message is a Narration that is **not** an event Narration.

Therefore:

- After a Retry on a **Narration** (free-text, guided, continue) that ends with the last Message still being a non-event Narration and a stored trigger context, Retrigger is allowed.
- After a Retry on an **event Narration** that succeeds, the last Message is an event Narration, so Retrigger is not allowed.
- After a Retry on an **event Narration** that fails, no new Swipe is added, the last Message remains the firing Narration, and the trigger context is still stored, so Retrigger is still allowed.
- After a Retry on any **Input** (impersonated or typed), the last Message is an Input, so Retrigger is not allowed.

### 5.4 Retrigger behaviour

`POST /retrigger` checks the same busy and data-integrity rules as a Retry, plus the conditions in R-65. If allowed:

1. The worker starts from the game state as the request finds it (R-63).
2. It runs a narrator call for an event Narration from the stored trigger context.
3. The Quantifier analyses the event text, but movement is ignored (R-14).
4. The world rules update the Characters present and the encounter record.
5. One new event Message is added; no Swipe is added (R-64).
6. If the narrator fails, no Message is added, the status reports the event narration failed, the trigger context remains stored, and the story log still offers Retrigger on the last Narration (R-66, R-67).

Because Retrigger starts from the current state rather than rolling back, it can be used to recover an event after a failed event generation or after later Actions have moved the story forward.

---

## 6. Writers

The table below covers every request in §2 and every background work piece. “Its Game” means the Game that the request or work was bound to when it started.

| Writer | May it run while the Game is busy? | What it does to running work | Which Game it writes to | How lost updates are prevented |
|---|---|---|---|---|
| `POST /action` (any kind) | No | Refused with `200` “Still thinking…”. | Active Game at request time. | Per-Game lock; `busy` status; worker `workId` check. |
| `POST /swipe/new` | No | Refused with `200` “Still thinking…”. | Active Game at request time. | Per-Game lock. |
| `POST /retrigger` | No | Refused with `200` “Still thinking…”. | Active Game at request time. | Per-Game lock. |
| `POST /message/:id/swipe/:index` | No | Refused with `409 Conflict`. | Active Game at request time. | Per-Game lock. |
| `POST /history/:id` (edit) | No | Refused with `409 Conflict`. | Active Game at request time. | Per-Game lock. |
| `POST /history/delete` | No | Refused with `409 Conflict`. | Active Game at request time. | Per-Game lock. |
| `POST /reset` | N/A (resets active Game) | Cancels any worker for the active Game before deleting it. | The active Game being reset, then a fresh Game. | Cancellation + `workId` mismatch drops stale writes. |
| `POST /games` (create) | Yes | Does not stop any previous active Game’s worker. | Creates a new Game; becomes active. | New Game id; old worker still writes to old Game id. |
| `POST /games/:id/switch` | Yes | Does not stop any running worker. | Changes the active Game pointer only. | Workers are bound to Game id, not active pointer. |
| `POST /status/reset-generating` | Yes (it interrupts) | Cancels the running worker for the active Game. | Active Game at request time. | Sets `cancelled`; final write discarded if `workId` no longer matches. |
| Background Narration generation | Only while busy | Is the work itself. | The Game id it was bound to. | Holds the lock; writes one unit; releases lock. |
| Background Impersonate generation | Only while busy | Is the work itself. | The Game id it was bound to. | Holds the lock; writes one unit; releases lock. |
| Background user-regen generation | Only while busy | Is the work itself. | The Game id it was bound to. | Holds the lock; writes one unit; releases lock. |
| Background event Narration generation | Only while busy | Is the work itself. | The Game id it was bound to. | Holds the lock; writes one unit; releases lock. |
| Background Options run | Only while busy | Is the work itself. | The Game id it was bound to. | Holds the lock; writes one unit; releases lock. |

### 6.1 Lost-update prevention in detail

1. **Per-Game lock.** A single in-memory mutex per Game id guarantees that only one request handler or background worker is inside the critical section for that Game at any moment.
2. **Status check.** Before starting, the handler reads the persisted `status`. If it is `busy`, the request returns “Still thinking…” without taking the lock. If it is `idle` or `failure`, the handler takes the lock, generates a fresh `workId`, and writes `busy(step, workId)`.
3. **Unit writes.** Every commit to storage includes every record that must land together: the Message, its Swipe, and the resulting game state (and status). Storage commits the unit whole or not at all (R-90).
4. **Worker ownership check.** Before a worker’s final write, it compares the current persisted `workId` with its own. If they differ — because `/status/reset-generating`, reset, or restart cancelled it — the worker discards its result and releases the lock.
5. **Restart safety.** In-memory locks are lost on restart. A start-up pass sets every `busy` status to `idle`, so no Game can appear busy after a restart (R-92). Any worker that was running is gone.

---

## 7. Notices

### 7.1 Kinds of notice

The design keeps two kinds of notice:

- **Options failure:** the Options call returned no usable answer.
- **No Options call:** the Game/World has no Options call configured.

The design does **not** show notices for analysis fallback or for movement to a room not on the map. Those are treated as normal outcomes.

### 7.2 Where notices go

Notices are stored in `Game.notices`, a list separate from the story. The dashboard shows them in a dedicated notice rail above the story log. Options-related notices are also echoed near the options dock. They are not Messages, so they do not get an Edit control, a Delete control, or a Swipe control.

### 7.3 Who sees them and how long they stay

The active player sees notices on the dashboard. A notice remains until:

- the player dismisses it, or
- the next successful generation starts, at which point the worker clears the notice queue.

This keeps the notice present long enough to be read but prevents stale notices from accumulating.

### 7.4 Keeping notices out of LLM prompts

No notice text is ever given to prompt assembly. Prompt assembly receives only:

- the World,
- the Persona,
- the current game state,
- the story Messages (active Swipe text, stored inputs, trigger context),
- the player’s current command (for an Action),
- the preset.

Because notices are not in the story and are never passed to prompt assembly, R-82 is satisfied.

---

## 8. The room report

### 8.1 Where the room is kept

The room report is a field on the **Swipe**: `roomReport`. It is set when the Swipe is created and never changed.

- For an **opening Narration**, `roomReport` is the World’s starting room.
- For a **Narration that moves the player**, `roomReport` is the destination room.
- For a **Narration that does not move the player**, `roomReport` is none.
- For an **Input** of any kind, `roomReport` is none.
- For an **event Narration**, `roomReport` is none.

### 8.2 How the story log gets it

The story log reads each Message’s active Swipe and displays that Swipe’s `roomReport`. Because each Swipe stores only the room its own generation arrived in (R-23), switching the active Swipe immediately changes the displayed room report for that Message (R-25). Event Narrations and Inputs never show a room report (R-24, R-22).

The room report is not part of the game state used by the world rules; it is a display value derived from the movement reported by the world rules during generation.

---

## 9. Restart and cut-off work

### 9.1 What a restart finds

When the process stops, all in-memory state is lost: running workers, locks, and transient work records. The database still holds whatever whole units were committed before the stop.

### 9.2 What a restart does

On start-up:

1. The engine loads every Game whose records are in storage.
2. For every Game with `status = busy`, it sets `status = idle` and clears `workId`.
3. It discards any in-memory lock table; locks are rebuilt on demand.

After this, no Game is busy (R-92).

### 9.3 What the player sees

The player sees the story and state exactly as they were committed to the database. If a free-text Action had committed its Input but had not committed the Narration when the process stopped, the story now shows an Input with no following Narration. The status is idle; no failure is reported for the cut-off work. The player can continue with a new Action, delete the Input, or Retry it (which will be a user regen).

A switch to a Game whose earlier work was cut off finds it idle and ready for an Action (R-80).

---

## 10. Requirement trace

| Requirement | Design section | Forced or free |
|---|---|---|
| R-01 Action kinds | §3 request parsing; §5 tables | Forced |
| R-02 Free text adds Input then Narration | §3.4; §5 Narration answering Input | Forced |
| R-03 Continue adds Narration, no Input | §3.4; §5 Continue row | Forced |
| R-04 Guided adds Narration, stores guide | §2.3 `storedGuide`; §5 Guided row | Forced |
| R-05 Impersonate adds Input | §3.4; §5 Impersonated Input row | Forced |
| R-06 Offered option as free text | §3 action handler | Forced |
| R-07 One Action at most one Narration; event separate | §3.4 Applying rules → Generating event | Forced |
| R-08 Successive Inputs kept in order | §2.1 `story` ordered list | Forced |
| R-09 Request answers before work ends | §3.2 Accepted state returns `200` immediately | Forced |
| R-10 Status idle or failure after work | §3.1 Ended state | Forced |
| R-11 Narration/Impersonate analysed and rules applied | §3.4 Quantifying → Applying rules | Forced |
| R-12 Analysis fallback ends well | §3.4 fallback path | Forced |
| R-13 Trigger fires → event Narration | §3.4 Applying rules → Generating event | Forced |
| R-14 Event analysis no movement | §5 Event row; §3.4 | Forced |
| R-15 No trigger → no event | §3.4 Applying rules | Forced |
| R-16 One-shot Trigger at most once | §2.1 `gameState.firedOneShots` | Forced |
| R-17 Event Narration failure | §3.4 Generating event failure; §5 Event row | Forced |
| R-18 User-regen no movement | §5 Typed Input row | Forced |
| R-19 Narrator failure/empty text | §3.4 Narrating failure | Forced |
| R-20 Invalid room fails before text | §3.4 room validation before narrator | Forced |
| R-21 Status failure line + raw disclosure | §2.1 `status.failure`; §3.1 Failure state | Forced |
| R-22 Room report rules | §8.1 | Forced |
| R-23 Each Swipe its own room | §8.1 `roomReport` on Swipe | Forced |
| R-24 Event no room | §8.1 | Forced |
| R-25 After switch room of new active Swipe | §4.2; §8.2 | Forced |
| R-26 `/options` makes new offered set | §3.4 Options state | Forced |
| R-27 `/options` with nothing to work from fails | §3.4 Options validation | Forced |
| R-28 `/options` adds no Message, replaces old set | §3.4 Options state | Forced |
| R-29 Options failure leaves set, tells player | §3.4 Options failure; §7 | Forced |
| R-30 Options on demand/always-on; Input gen leaves set | §3.4 Options transitions; §5 Input rows | Forced |
| R-31 Always-on after each Narration | §3.4 Options after Narration | Forced |
| R-32 Toggle off empties set after Narration | §3.4 | Forced |
| R-33 Toggle on fail/no call empties + notice | §3.4; §7 | Forced |
| R-34 Every Message ≥1 Swipe | §2.2 `swipes` list | Forced |
| R-35 Only last Message switchable | §4.2 validation | Forced |
| R-36 After switch game as Swipe left it | §4.2 restore `postGenerationState` | Forced |
| R-37 After switch status idle | §4.2 | Forced |
| R-38 After switch dock no set | §4.2 clear offered set; §11 OQ-01 | Forced (chosen resolution) |
| R-40 Retry redoes last generation with stored inputs | §5.1 | Forced |
| R-41 Retry adds one Swipe, same Message | §5.1; §5.2 | Forced |
| R-42 Retry never changes existing Swipe text | §5.1 | Forced |
| R-44 Retry no Input → 400 | §5.1 | Forced |
| R-45 Retry no Game → 400 | §5.1 | Forced |
| R-46 Retry/Retrigger data missing → 500 | §5.1; §5.4 | Forced |
| R-47 Failed Retry no Swipe leaves state | §5.2 failure column | Forced |
| R-48 Failed Retry with Swipe keeps redo world | §5.2; §5.1 event-failure note | Forced |
| R-49 Narrator fail/empty/invalid room → no Swipe | §5.2 failure column | Forced |
| R-50 Retry never busy | §3.1; §5.1 | Forced |
| R-51 Redo uses Input text as now | §5.2 Narration answering Input row | Forced |
| R-52 Redo analysed, can move/fire | §5.2 | Forced |
| R-53 Guided redo uses stored guide and room | §5.2 Guided row | Forced |
| R-54 Event redo new Swipe on event Message | §5.2 Event row | Forced |
| R-55 Event redo starts before replaced event | §5.2 Event row; §2.3 snapshots | Forced |
| R-56 Event redo uses trigger context | §5.2 Event row; §2.3 `triggerContext` | Forced |
| R-57 Event redo no move | §5.2 Event row | Forced |
| R-58 Event redo failure specifics | §5.2 Event row | Forced |
| R-59 Impersonate redo uses stored direction/current preset | §5.2 Impersonated Input row | Forced |
| R-60 Impersonate redo fresh, can move/fire | §5.2 Impersonated Input row | Forced |
| R-61 Typed Input redo is user regen | §5.2 Typed Input row | Forced |
| R-62 Retrigger allowed/context/last non-event Narration | §5.4 | Forced |
| R-63 Retrigger starts from found state, no move | §5.4 | Forced |
| R-64 Retrigger adds one Message, no Swipe | §5.4 | Forced |
| R-65 Retrigger 400 cases | §5.4 | Forced |
| R-66 Retrigger event failure | §5.4 | Forced |
| R-67 Trigger context survives failed event | §2.1 `gameState.storedTriggerContext`; §5.4 | Forced |
| R-70 Busy → still thinking for Action/Retry/Retrigger | §3.1; §6 | Forced |
| R-71 Status names step | §2.1 `status.busy.step`; §3.1 | Forced |
| R-72 Status/dock/debug report current busy; cut-off not busy | §3.1; §6; §9 | Forced |
| R-73 No lost updates | §6.1 | Forced |
| R-74 Writes only to bound Game | §2.1 active pointer vs Game id; §6 | Forced |
| R-75 Edit replaces active Swipe text | §3.1 transitions; §6 | Forced |
| R-76 Delete last Message | §3.1; §6 | Forced |
| R-77 Reset removes active Game, fresh Game with opening | §3.1; §6 | Forced |
| R-78 Create Game active, dashboard reloads | §3.1; §6 | Forced |
| R-79 Switch Game active, dashboard reloads | §3.1; §6 | Forced |
| R-80 Switch to cut-off Game idle | §9 | Forced |
| R-82 No notice in prompts | §7.4; §2.3 separate from story | Forced |
| R-83 Options fail/missing notice | §7.1; §7.2 | Free (where shown) |
| R-90 Unit writes of Message, Swipe and state | §2.1; §6.1 | Forced |
| R-91 Needed data exists | §2.3 snapshots and trigger context; §5.1 | Forced |
| R-92 Restart no busy | §9 | Forced |

---

## 11. Assumptions

1. **OQ-01 — offered set after swipe switch.** R-38 is treated as the specific rule. After a swipe switch the game state (room, characters, triggers, trigger context) is restored from the new active Swipe, but the offered set is cleared and the dock shows no set. The general statement in R-36 is read as referring to world/gameplay state, not to the transient dock controls.

2. **OQ-02 — Retry that fires a Trigger.** If a Retry’s redo is a generation that can fire a Trigger and the world rules report that one fires, the engine writes a new event Narration Message after the redone Message. The Retry itself still adds only one Swipe to the existing Message.

3. **OQ-03 — which notices to show.** Only the Options-failure and missing-Options notices are shown. Analysis fallback and movement to an off-map room are not surfaced as notices. A failed or missing Options run at the end of a generation does **not** turn that generation into a failure; the status remains `idle`.

4. **OQ-04 — notices are not Messages.** Notices live in a separate `Game.notices` list. They do not change which Message is “last,” do not receive Edit/Delete/Swipe/Retrigger controls, and are never included in prompt assembly.

5. **OQ-05 — Retry when the story has no Input.** R-44 is enforced strictly: if the story contains no Input, `POST /swipe/new` returns `400`. If the story contains at least one Input, a Retry can redo any last generation, including a continue or guided Narration. A continue redo is a fresh continue generation with no player text.

6. **OQ-06 — Input stays when its Narration fails.** For a free-text Action, the Input Message is committed before the Narration generation runs. If the Narration fails, the Input remains in the story, and the next Retry on that Input is a user regen.

7. **OQ-07 — `/status/reset-generating`.** This is a debug/admin recovery route. If the active Game is busy, it cancels the worker and sets the Game idle. If there is no active Game, it returns `failed`; otherwise it returns `reset`. It does not delete story data.

8. **OQ-08 — edit, delete, swipe switch on a busy Game.** These requests are refused with `409 Conflict`. They do not wait and they do not interrupt the running work.

9. **OQ-09 — leaving a busy Game.** Switching to another Game does not stop the worker; it continues writing to its original Game id. Reset removes and cancels work for the active Game. Creating a new Game leaves any previous active Game’s worker running for its own Game.

10. **OQ-10 — World with no opening text.** The new Game still has an opening Narration Message whose text is empty. It is not generated, the Game is not busy, and a Retry returns `400` because the story has no Input.

11. **OQ-11 — delete and game state.** Deleting the last Message reverts the game state to the `postGenerationState` snapshot of the new last Message (or the initial state if the story becomes empty). This includes room, characters, encounters, fired one-shot Triggers, trigger context, and offered set. Any Message may be deleted.

12. **OQ-12 — cut-off work after restart.** The player sees whatever whole units were committed before the stop. Cut-off work is not reported as a failure. If an Input was committed without its Narration, the story shows that Input and the status is idle.

13. **OQ-13 — clearing a failure.** A failure is cleared when any successful mutation occurs: the start of new work, an edit, a delete, a swipe switch, a Game switch, a reset, or `/status/reset-generating`.

14. **OQ-14 — lifetime of stored trigger context.** The stored trigger context remains until it is overwritten by a newly fired Trigger, or until the game state is reverted by a delete or swipe switch. Later Actions that do not fire a Trigger do not clear it.

15. **OQ-15 — room report of an Impersonate.** An Impersonate generation produces an Input Message, so it reports no room. Movement caused by an Impersonate is reflected in the game state and will be reported by the next Narration that moves the player.

16. **OQ-16 — room report of the opening.** The opening Narration reports the World’s starting room.

17. **OQ-17 — Swipe with no generation.** The first Swipe of a typed Input stores the game state as it was before the Input was added. A swipe switch to that Swipe restores that prior state.

18. **Always-on options after an event Narration.** When the always-on toggle is on, an event Narration also triggers an Options run, because an event Narration is a Narration and R-30 only exempts Input generations.

19. **“No Options call” detection.** A Game is considered to have no Options call when its World or configuration lacks one. This is treated as an Options failure for the purposes of R-29 and R-33.

20. **Free-text Action writes the Input before the Narration.** This lets the Input survive a Narration failure and matches the decision in OQ-06. Each write is still a whole unit: the Input Message with its first Swipe and the unchanged game state are one unit; the Narration Message with its Swipe and the new game state are a second unit.

21. **User-regen does not analyse.** A Retry on a typed Input only calls the narrator for a new version of the player’s words. No Quantifier or world-rules step runs, so the player cannot move and no Trigger fires.

---

## 12. Questions for the owner

All open questions from §6 have been answered in §11. No further questions remain.
