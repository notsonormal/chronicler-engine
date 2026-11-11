# The verify-gate model against the specs and against `game_flow.md`

Written 2026-11-05. Read-only work: no cargo, no `build.py`, no TLC.
Model: `.agents/skills/verify-gate/model/VerifyGate.tla`. Rules and the action table: `.agents/skills/verify-gate/SKILL.md`.

Labels: **known** = read in the named file. **Inferred** = a conclusion from a full-text search of `docs/specs/`, not a sentence the spec states.

`docs/specs/` has 31 files. The searches for Finding A covered all of them (`rg -n -i` over `docs/specs/*.md` for `busy`, `409`, `delete`, `switch`, `reset-busy`, `start state`, `Options failure`, `unreachable`, `cannot read`).
The searches for Finding B read `/workspace/chronicler-engine/docs/diataxis/reference/game_flow.md` in full (171 lines).

---

## Finding A — every model branch and the scenario that makes it happen

### The requests

| Action | Branch | Scenario | Quote |
|---|---|---|---|
| `YouStartAJob` | `k = "action"` (a plain Action Job on an idle Game) | actions.md 1.1 | "When the client POST /action with command="look" … Then … the story log gained exactly 2 messages" |
| `YouStartAJob` | `k = "options"` (a plain Options Job; `pc = "optstart"`, `opt = "ondemand"`) | options.md 24.1 | "When the client POST /action with command="/options" … Then GET /screen?parts=options-dock renders the options strip with one button per generated option" |
| `YouStartAJob` | `holder[g].kind = "none"` precondition (the busy refusal) | actions.md 3.4, 3.5; autoplay.md 41.8 | 3.5: "Then one Action is accepted and the other is refused with 409 and the "Still thinking..." line" |
| `TheAcceptCannotRead` | the plan names a missing record: the Persona | actions.md 2.6 | "Then the response is 500 Internal Server Error And the response carries the line "The data for this request is missing: the persona."" |
| `TheAcceptCannotRead` | the plan names a missing record: the start state | failure_display.md 38.15 | "Then the response is 500 And the response carries the missing-data line that names the start state" |
| `TheAcceptCannotRead` | the accept's story read fails (a plain Job) | **no scenario** (inferred) | Only the run's version exists: autoplay.md 41.23 "the status display answers with the failure line of the unreadable story". failure_display.md 38.14's "the story" row is a read after the Narration call, not the accept. |
| `YouStartAutoplay` | idle Game, run 1 | autoplay.md 41.1 | "When the client POST /autoplay with cycles=2 Then the response is 200 with an empty body" |
| `YouStartAutoplay` | `require_idle` fails because a run holds the Game | autoplay.md 41.8 | "Given a Game that a run holds When the client POST /autoplay with cycles=1 Then the response status is 409" |

### The stop (`YouPressStop`)

| Branch | Scenario | Quote |
|---|---|---|
| a Job held at the **narrator** call | dashboard.md 39.4, row "Narration / the narrator call" | "Then the stop answers "reset" And the Game is idle And the status display answers with no failure line … the Input of the Job, and no Narration" |
| a Job held at the **analysis** call | dashboard.md 39.4, row "Narration / the analysis call" | "the Input of the Job and the Narration, with no room on the Narration and no event Narration" |
| a Job held at the **Options** call | dashboard.md 39.4, row "Narration / the Options call" | "the Input of the Job, the Narration, and the room the Narration arrived in" |
| an **Impersonate** Job at the narrator call | dashboard.md 39.4, row "Impersonate / the narrator call" | "no Message of the Job" |
| an **Impersonate** Job at the analysis call | dashboard.md 39.4, row "Impersonate / the analysis call" | "the Input of the Job and no Narration" |
| a **Retry of a typed Input** at the narrator call | dashboard.md 39.4, row "Retry of a typed Input / the narrator call" | "the Messages of the story before the Retry" |
| an **event Narration** at the event narrator call | dashboard.md 39.4, row "event Narration / the event narrator call" | "the Narration that fired the Trigger, and no event Narration" |
| an **event Narration** at the analysis call | dashboard.md 39.4, row "event Narration / the analysis call" | "the one-shot Trigger fired in the state of the event Narration" |
| a Job of a **run**, at the narrator call | autoplay.md 41.6 | "Then the stop answers "reset" And the Game is idle at once … And no further Job of the run starts And the client can start a Job on the Game" |
| a **run between two Jobs** | autoplay.md 41.7 | "Given a run between its first Job and its second Job … Then the status display answers the status "idle"" |
| a stop that clears the **failure line of a Job that already ended** (idle Game, `failLine = j`, `j ∉ cut`) | **no scenario** (inferred) | 39.4 stops a running Job. options.md 24.5 and 24.18 stop an idle Game, but only assert the dock: "Then the dock renders set A". |
| a stop **after the failure line landed and before the `JobWriter` drop** | **no scenario** (inferred) | 39.4's table stops at model calls, so the stop is always before the save. |
| a stop **after the Options failure landed and before the drop** | **no scenario** (inferred) | options.md 24.5's stop is after "the Game is no longer busy", so the Job has already ended. |
| no active Game | **not modelled** (the action needs `g ∈ exists`); the route refuses: dashboard.md 39.10 | "Then the response status is "400 Bad Request" And the response body mentions "No active game"" |
| the dock read fails | outside the model by design (SKILL.md *Scope*); dashboard.md 39.8 | "Then the response status is "500 Internal Server Error" … And the status display still answers with the step of the Job" |

### Reset, delete and switch

| Action | Branch | Scenario | Quote |
|---|---|---|---|
| `YouResetTheGame` | the active Game has a plain Job | reset.md 7.3 | "Then the new Game's story log holds its opening Narration And the new Game's story log holds no Input and no Narration of the Job the reset ended" |
| `YouResetTheGame` | the active Game has a run | autoplay.md 41.15 | "When the client POST /reset … And no further Job of the run starts" |
| `YouResetTheGame` | an idle active Game | reset.md 7.1, 7.5 | 7.1: "Then the active Game's story log holds no Input And it holds only its opening Narration" |
| `TheResetMovesThePointer` | the pointer moves after the hold | reset.md 7.5 | "And the active Game is a new Game" |
| `TheResetMovesThePointer` | the window between the reset hold and the pointer move | **no scenario** (inferred) | The window is what enables the `DeleteNotActive` break. No scenario resets and then acts on the pointer before it moves. |
| `YouDeleteAGame` | a non-active Game with no Job | games.md 19.1 | "When the client POST /games/{id2}/delete Then the response status is "200 OK"" |
| `YouDeleteAGame` | the active Game is refused | games.md 19.2 | "Then the response status is "400 BAD_REQUEST" And the response body mentions "Cannot delete the active game"" |
| `YouDeleteAGame` | a non-active Game that a **plain Job** holds | **no scenario** (inferred) | games.md 19.1 has no Job; story_log.md 8.7 refuses a story write, not a delete. |
| `YouDeleteAGame` + `TheDeleteRemovesTheGame` | a non-active Game that a **run** holds (its Job in flight) | autoplay.md 41.16 | "Given a run on a Game that is not the active Game, whose first Job is held at its narrator call … Then no further Job of the run starts" |
| `YouDeleteAGame` + `TheDeleteRemovesTheGame` | the check passes, a switch moves the pointer, the delete then removes the **active** Game | **no scenario** — the known finding | 19.2's scenario has one request and no second request between the check and the delete. |
| `YouSwitchTheGame` | while a plain Job runs | games.md 18.4 | "When both Jobs' model answers arrive And the client switches back to Game A Then Game A's story log holds its own Narration and no Narration of Game B" |
| `YouSwitchTheGame` | while a run holds the Game | autoplay.md 41.14 | "Then the story log of the first Game holds the Messages of the run" |
| `YouSwitchTheGame` | a Game that does not exist | games.md 18.2 | "Then the response status is "400 BAD_REQUEST" And the response body mentions "Game not found"" |

### The story writes and the config write

| Action | Branch | Scenario | Quote |
|---|---|---|---|
| `YouEditAMessage` | idle, the edit lands | story_log.md 8.15 | "Then the response is 200 with an empty body And the client GET /screen?parts=story-log shows the new text on that Message" |
| `YouEditAMessage` | `require_idle` fails | story_log.md 8.7 | "When the client POST /history/:id with a new text … Then each response is 409 carrying the "Still thinking..." line" |
| `YouSwitchOrDeleteASwipe` | idle switch | swipe_switch.md 36.1; 36.2 for the dock | 36.1: "Then the response is 200 with an empty body And the generation status is idle" |
| `YouSwitchOrDeleteASwipe` | idle delete of the last Message | story_log.md 8.1, 8.4 | 8.4: "Then the response is 200 with an empty body And the client GET /screen?parts=story-log renders exactly one fewer .log-entry" |
| `YouSwitchOrDeleteASwipe` | `require_idle` fails, switch | swipe_switch.md 36.3 | "Then the response is 409 carrying the "Still thinking..." line And the active Swipe of that Message does not change" |
| `YouSwitchOrDeleteASwipe` | `require_idle` fails, delete | story_log.md 8.7 | "And the story log still holds every Message of the Job" |
| `YouSwitchOrDeleteASwipe` | the refusal of a Message that is no longer last | out of the model (a second 409 kind): swipe_switch.md 36.6, story_log.md 8.17 | 36.6: "Then the response is 409 Conflict And the first line of the body is "The last Message changed…"" |
| `YouChangeTheConfig` | idle | games.md 20.7 | "When the client POST /games/{game_id}/posture … Then the response status is "200 OK"" |
| `YouChangeTheConfig` | a plain Job holds the Game (`kind ≠ "run"` passes) | **actions.md 6.4** | "And the Job's narrator call is held When the client sets the Game's system preset to a second preset … Then the LLM call record of the event Narration shows the system prompt of the second preset" |
| `YouChangeTheConfig` | a run holds the Game (`kind = "run"` is refused) | autoplay.md 41.12 | "Then each response status is 409 … And the Game's config is as it was" |
| `YouChangeTheConfig` | the run has ended, a config write then passes | autoplay.md 41.12, second half | "When the client POST /status/reset-busy … And the client POST /games/:id/posture … Then the Game's posture is the new one" |

### The Job steps

| Action | Branch | Scenario | Quote |
|---|---|---|---|
| `AJobStartsAGeneration` | `Own` true, an **Action** first generation (the Input save) | actions.md 1.2 | "Then the story log gained 2 messages And the player-input message appears in the story log before the Narration message" |
| `AJobStartsAGeneration` | `Own` true, `kind ∈ {"impersonate","continue"}`, first generation (no input at this step) | autoplay.md 41.1; actions.md 1.9 | 41.1: "the narrator call of each Continue Job carries the Impersonate Input that it answers"; 1.9 writes the Input at the text save |
| `AJobStartsAGeneration` | `Own` true, the **event** generation | actions.md 1.4 | "Then the story log holds at least one Narration message carrying an event header (the trigger fired)" |
| `AJobStartsAGeneration` | `Own` false → `Moved` | dashboard.md 39.5, 39.4 | 39.5: "Then the story log holds no Narration of the stopped Job" |
| `TheNarratorAnswers` | any kind | actions.md 1.1 | "And the story log gained exactly 2 messages: … one Narration message" |
| `TheNarratorFails` | the main narration call | actions.md 2.2; autoplay.md 41.4 | 2.2: "Then the generation status is an error whose text is non-empty"; 41.4: "And no further Job of the run starts" |
| `TheNarratorFails` | the **event** narration call | actions.md 2.4 | "Then the generation status is an error whose text is "The event narration failed."" |
| `TheTextIsSaved` | `Own` true, `savesNarr` (`kind ∈ {"action","continue"}` or `gen = "event"`) | actions.md 1.2 | "And a Narration message whose text is "You see a small wooden room."" |
| `TheTextIsSaved` | `Own` true, `kind = "impersonate"` → `last = "input"` | actions.md 1.9, 1.18 | 1.18: "Then the story log gained exactly one player-input message" |
| `TheTextIsSaved` | `Own` true, `savesNarr` also clears the Options failure (`empties_the_dock`) | **no scenario that isolates it** (inferred) | options.md 24.16 asserts the offered set: "Then the offered set is empty". The failure clear is read only where a later Options run also writes the dock (24.22-24.24: "And the dock shows no Options failure"). |
| `TheTextIsSaved` | `Own` false → `Moved` | dashboard.md 39.4, 39.5 | 39.4: "Then the story log holds exactly the records of that row" |
| `TheAnalysisStarts` / `TheAnalysisAnswers` | the Quantifier call runs | actions.md 1.4 | "And a quantifier backend that responds with {"characters_present": ["character_1"]}" |
| `TheAnalysisAnswers` | a failed Quantifier call gives the fallback, never a failure | actions.md 2.5; autoplay.md 41.24 | 2.5: "Then the generation status is Idle And the Characters present still include "character_1"" |
| `TheStateIsSaved` | `fires = TRUE` (a Trigger fires, a second generation is ordered) | actions.md 1.4; autoplay.md 41.3, 41.21 | 41.21: "Then the story log holds … the first Impersonate Input, the event Narration of the Trigger" |
| `TheStateIsSaved` | `fires = FALSE` | actions.md 1.7, 1.8 | 1.7: "And no Narration message carries an event header (no trigger fired)" |
| `TheStateIsSaved` | `Own` true / `Own` false (`Moved`) | dashboard.md 39.4 | "the Narration, with no room on the Narration" (the state update of the stopped Job wrote nothing) |
| `TheOptionsRunFollows` | `opt = "after"` and a Narration was saved | options.md 24.6, 24.22 | 24.6: "Then the dock renders a fresh option set without any /options command" |
| `TheOptionsRunFollows` | it follows a **later part's failure** | options.md 24.22, 24.23, 24.24 | 24.22: "Then the dock renders a fresh option set … And the status names the event narration failure" |
| `TheJobSkipsTheOptionsRun` | `narr = false` (an Impersonate Job with the always-on toggle) | options.md 24.15 | "Then the offered set still holds set A And the dock still renders set A" |
| `TheOptionsRunStarts` | the Options call runs | options.md 24.1, 24.6 | 24.1: "renders the options strip with one button per generated option" |
| `TheOptionsCallEnds` | `ok = TRUE` | options.md 24.1 | "Then GET /screen?parts=options-dock renders the options strip" |
| `TheOptionsCallEnds` | `ok = FALSE` (`OptionsResult::Unavailable`) | options.md 24.5, 24.18; autoplay.md 41.5 | 24.18: "Then the dock shows the Options failure saying the options agent is not available" |
| `TheOptionsAreSaved` | `Own` true, `unavailable` → `optFail = j` | options.md 24.5, 24.17, 24.20; autoplay.md 41.5 | 24.17: "And the dock shows the Options failure naming the failed Options run" |
| `TheOptionsAreSaved` | `Own` true, `offered` → `optFail = 0` | options.md 24.1, 24.6 | "the dock renders a fresh option set without any /options command" |
| `TheOptionsAreSaved` | `Own` false → `Moved` | dashboard.md 39.4, row "the Options call" | "the Input of the Job, the Narration, and the room the Narration arrived in" |
| `TheFailureLineIsSaved` | `Own` true | actions.md 2.2, 2.4; autoplay.md 41.4 | 41.4: "Then the status display answers with the failure line of that Job" |
| `TheFailureLineIsSaved` | `Own` false → `Moved` | dashboard.md 39.4, 39.5 | 39.5: "And the story log holds no Narration of the stopped Job" |
| `TheJobEnds` | a plain Job (the holder goes to idle) | dashboard.md 39.5 | "Then the story log holds the Input and the Narration of the second Job" |
| `TheJobEnds` | a run's Job (the holder stays, `job = NoJob`) | autoplay.md 41.7, 41.13 | 41.13: "Then the status display answers the status "autoplay" with the label "Autoplay: cycle 1/2"" |
| `TheJobEnds` | `holder[g].job ≠ j` (an old Job's drop frees no later Job) | autoplay.md 41.17; dashboard.md 39.5 | 41.17: "Then the status display answers with the progress of run B" |

### The Autoplay loop

| Action | Branch | Scenario | Quote |
|---|---|---|---|
| `TheRunAccepts` | `~ours` → `AutoplayAccept::Ended` (`end = "stopped"`) | autoplay.md 41.6, 41.7, 41.15, 41.16, 41.17 | 41.6: "And no further Job of the run starts And the client can start a Job on the Game" |
| `TheRunAccepts` | `ours ∧ holder.job ≠ NoJob` → `StillThinking` | **no scenario, and unreachable** (SKILL.md says TLC never reaches it) | — |
| `TheRunAccepts` | `ours ∧ ~readOk` → `NotAccepted` (the accept writes `-1` and removes the holder) | autoplay.md 41.23 | "Then the status display answers with the failure line of the unreadable story And the Game is not busy And no model call is made" |
| `TheRunAccepts` | `ours ∧ readOk ∧ d >= Budget` → `Finished` | autoplay.md 41.1, 41.21 | 41.21: "the event Narration ends the first cycle, so the run ends after two cycles" |
| `TheRunAccepts` | `ours ∧ readOk ∧ d < Budget` → `Accepted` | autoplay.md 41.1 | "Then the story log holds, after the opening Narration, an Impersonate Input, a Narration, an Impersonate Input and a Narration, in this order" |
| `TheRunAccepts` | `Accepted` with `last[g] = "input"` → a **Continue** Job | autoplay.md 41.2 | "Then the first Job of the run is a Continue Job, with no Impersonate call" |
| `TheRunAccepts` | `Accepted` otherwise → an **Impersonate** Job | autoplay.md 41.1, 41.14 | 41.14: "Then the story log of the first Game holds the Messages of the run" |
| `TheRunAccepts` | the `badAccept` ghost (`r ∈ cutRuns`) | autoplay.md 41.6 | "And no further Job of the run starts" |
| `TheRunSeesItsJobEnd` | `fail = "none"` → `pc = "accept"` | autoplay.md 41.3 | "And the run made two Impersonate Jobs and two Continue Jobs" |
| `TheRunSeesItsJobEnd` | `fail = "line"` → `jobFailed` | autoplay.md 41.4 | "Then the story log holds the Impersonate Input and no Narration of the run" |
| `TheRunSeesItsJobEnd` | any other `fail` (`"moved"`) → `stopped` | autoplay.md 41.17 | "When the model answer of run A arrives And run A ends Then the status display answers with the progress of run B" |
| `TheRunEnds` | `end = "notAccepted"` with `endIdle` | autoplay.md 41.23 | "And the Game is not busy" |
| `TheRunEnds` | `endLine = failLine[g]` for `jobFailed` | autoplay.md 41.4 | "And the status display answers with the failure line of that Job And the Game is not busy" |
| `TheRunEnds` | the run id check (`ours`) | autoplay.md 41.17 | "Then the status display answers with the progress of run B" |
| `TheRunEnds` | `endIdle` for a stopped or budget run | autoplay.md 41.1, 41.7 | 41.1: "And the Game is idle" |
| `Rest` (no deadlock) | every process ends | not a rule; SKILL.md leaves "a Job always ends" (actions.md I.1) outside the model | — |

---

## Finding B — `game_flow.md` gate sentences against the model

### Sentences that contradict the model

1. **The run's holder span.** *(inferred; a wording contradiction, not observable through HTTP)*
   > "An Autoplay run. It lasts from the start of the run to its end, also between two Jobs of the run."
   (`## The Busy State and Its Holder`)

   Contradicts `TheRunAccepts` and `TheRunEnds`. The model has a reachable state where the run has **not** ended (`run[r].pc = "end"`) and `holder[g]` is already `Idle`. `TheRunAccepts` removes the holder inside the accept, in both the `Finished` (budget) and the `NotAccepted` (unreadable record) branch, and the run record only ends at the later `TheRunEnds`. `RunHoldsItsGame` is written to skip `pc = "end"` for exactly this reason. So the holder lasts from the run's start to the accept that ends it, not to the run's end. The code agrees with the model (`accept_for_autoplay` removes the holder before it returns), so the sentence is the side that is loose. No request can tell the two apart, because the run record's end is not observable.

No other sentence of the gate sections contradicts the model's behaviour (known: I checked each against the action table above).

### Sentences that state a rule with no spec scenario (spec gaps)

1. > "A stop removes the holder, clears the failure line and clears the Options failure of the dock record."
   (`## Stop and Reset`)

   The Options-failure half has scenarios (options.md 24.5, 24.18). The **failure-line** half has dashboard.md 39.4, and 39.4 stops a **running** Job. No scenario presses the stop on an **idle** Game that carries the failure line of a Job that already ended, which is the case the model allows (`YouPressStop` needs no holder; `failLine[g] = j` with `j ∉ cut`).

2. > "A config write passes while a plain Job runs."
   (`## The Busy State and Its Holder`)

   Covered once: actions.md 6.4. It asserts the passing as a side effect of a preset read: "Then the LLM call record of the event Narration shows the system prompt of the second preset". No scenario states the 200 of the config write itself during a plain Job. *(inferred: the gap is that the rule is tested only through its effect.)*

3. > "A stop, a reset and a delete of the Game remove the holder."

   The delete half is covered for a run (autoplay.md 41.16) and not for a plain Job: no scenario deletes a non-active Game while a plain Job of that Game runs. *(inferred)*

### Not modelled (one line, per the brief)

Also outside the model, so not a gap: a restart and its loss of every holder; a step name in the holder (the model reads it from `job[j].pc`); the 400 refusals and the second 409 kinds (`LastMessageChanged`, a stale `X-Chronicler-Game`); the delete's own failure-line clear (the rows go with the Game); the `updated_at` write of a switch; a delete that removes the active Game after the check (the known finding — `game_flow.md` states no rule about it, so no doc sentence is wrong).

---

## Summary of the gaps, by kind

| Kind | Count | Items |
|---|---|---|
| A model branch with **no spec scenario** | 6 | the plain accept's story-read failure; a stop after the failure line landed; a stop after the Options failure landed; a stop clearing an ended Job's failure line; the `TheTextIsSaved` clear of the Options failure (not isolable); a delete of a non-active Game that holds a plain Job |
| A **concurrency case** the spec does not face | 2 | 19.2's active-Game check vs a switch between the check and the delete (the known finding); the window between the reset hold and the pointer move |
| A branch that is **unreachable** by design, no scenario | 1 | `TheRunAccepts` `StillThinking` (the run's own Job in flight) |
| A branch the model **leaves out** by design | 2 | dashboard.md 39.8 (the failed dock read); storage faults (24.23, 24.24, 38.11-38.15) |
| A **doc sentence contradicted** by the model | 1 | "It lasts from the start of the run to its end" (inferred; one instant, not observable) |
| A doc **rule with no scenario** | 3 | the stop's failure-line clear on an idle Game; the config write's own 200 during a plain Job (weak: actions.md 6.4 exercises the write as a side effect); the delete of a Game that holds a plain Job |
| Other | — | none found: every other gate sentence of `game_flow.md` matches its action in the model |

The two highest-value gaps are both in the delete path: the missing scenario for a delete of a non-active Game that holds a plain Job, and the missing scenario for the check-then-switch race that the `delete-race` config already breaks (`DeleteNotActive`). The second is the only one where the model reports a break in the current code.
