---- MODULE JobLife ----
\* The life of one Job on one Game, as the dashboard shows it: who holds the
\* Game, which step the Job shows, and when the Options run follows a Narration.
\*
\* The model follows docs/diataxis/reference/game_flow.md and
\* src/application/pipeline/job.rs. It leaves out the room check, the Input save
\* and the Retry of a typed Input.
EXTENDS Naturals

CONSTANTS OptionsOn   \* whether the Options run follows a saved Narration

VARIABLES holder, step, narrationSaved, optionsTried, pendingFailure, failure, stopped

vars == <<holder, step, narrationSaved, optionsTried, pendingFailure, failure, stopped>>

Sides == {"idle", "job"}
Steps == {"none", "narrating", "quantifying", "generating event", "options"}

TypeOK ==
  /\ holder \in Sides
  /\ step \in Steps
  /\ narrationSaved \in BOOLEAN
  /\ optionsTried \in BOOLEAN
  /\ pendingFailure \in BOOLEAN
  /\ failure \in {"none", "shown"}
  /\ stopped \in BOOLEAN

Init ==
  /\ holder = "idle"
  /\ step = "none"
  /\ narrationSaved = FALSE
  /\ optionsTried = FALSE
  /\ pendingFailure = FALSE
  /\ failure = "none"
  /\ stopped = FALSE

\* The player sends an Action on an idle Game. The accept clears the failure line.
YouSendAnAction ==
  /\ holder = "idle"
  /\ holder' = "job"
  /\ step' = "narrating"
  /\ narrationSaved' = FALSE
  /\ optionsTried' = FALSE
  /\ pendingFailure' = FALSE
  /\ failure' = "none"
  /\ stopped' = FALSE

\* The narrator's text is saved: the story shows it, and the Game rules read it next.
TheNarrationIsSaved ==
  /\ holder = "job"
  /\ step = "narrating"
  /\ step' = "quantifying"
  /\ narrationSaved' = TRUE
  /\ UNCHANGED <<holder, optionsTried, pendingFailure, failure, stopped>>

\* The state update fired a one-shot Trigger, so one more generation follows.
ATriggerFires ==
  /\ holder = "job"
  /\ step = "quantifying"
  /\ step' = "generating event"
  /\ UNCHANGED <<holder, narrationSaved, optionsTried, pendingFailure, failure, stopped>>

TheStateUpdateFiresNoTrigger ==
  /\ holder = "job"
  /\ step = "quantifying"
  /\ step' = "options"
  /\ UNCHANGED <<holder, narrationSaved, optionsTried, pendingFailure, failure, stopped>>

TheEventNarrationIsSaved ==
  /\ holder = "job"
  /\ step = "generating event"
  /\ step' = "options"
  /\ narrationSaved' = TRUE
  /\ UNCHANGED <<holder, optionsTried, pendingFailure, failure, stopped>>

\* The whole Job is the Options run: `/options` on an idle Game.
TheOptionsRunEnds ==
  /\ holder = "job"
  /\ step = "options"
  /\ holder' = "idle"
  /\ step' = "none"
  /\ optionsTried' = TRUE
  /\ failure' = IF pendingFailure THEN "shown" ELSE failure
  /\ UNCHANGED <<narrationSaved, pendingFailure, stopped>>

\* A Job with no Options run due: the toggle is off.
TheJobEndsWithNoOptionsOwed ==
  /\ holder = "job"
  /\ ~OptionsOn
  /\ holder' = "idle"
  /\ step' = "none"
  /\ UNCHANGED <<narrationSaved, optionsTried, pendingFailure, failure, stopped>>

\* A part fails. A Narration that was saved still earns its Options run, so the
\* failure line is written after that run.
APartFailsAndTheOptionsFollow ==
  /\ holder = "job"
  /\ OptionsOn
  /\ narrationSaved
  /\ step' = "options"
  /\ pendingFailure' = TRUE
  /\ UNCHANGED <<holder, narrationSaved, optionsTried, failure, stopped>>

APartFailsAndTheJobEnds ==
  /\ holder = "job"
  /\ ~(OptionsOn /\ narrationSaved)
  /\ holder' = "idle"
  /\ step' = "none"
  /\ failure' = "shown"
  /\ UNCHANGED <<narrationSaved, optionsTried, pendingFailure, stopped>>

\* The player presses Stop. The Game is idle at once, and the Job writes nothing more.
YouPressStop ==
  /\ holder = "job"
  /\ holder' = "idle"
  /\ step' = "none"
  /\ failure' = "none"
  /\ stopped' = TRUE
  /\ UNCHANGED <<narrationSaved, optionsTried, pendingFailure>>

Next ==
  \/ YouSendAnAction
  \/ TheNarrationIsSaved
  \/ ATriggerFires
  \/ TheStateUpdateFiresNoTrigger
  \/ TheEventNarrationIsSaved
  \/ TheOptionsRunEnds
  \/ TheJobEndsWithNoOptionsOwed
  \/ APartFailsAndTheOptionsFollow
  \/ APartFailsAndTheJobEnds
  \/ YouPressStop

Spec == Init /\ [][Next]_vars

\* A Narration that was saved earns its Options run, unless the player stopped the Job.
OptionsFollowANarration ==
  (holder = "idle" /\ ~stopped) => (~narrationSaved \/ optionsTried)

\* A stop leaves the Game idle, whatever step the Job showed.
AStopMakesTheGameIdle == stopped => (holder = "idle" /\ step = "none")

\* A busy Game shows its step, not the failure line of an earlier Job.
AFailureLineIsNotShownWhileBusy == failure = "shown" => holder = "idle"

====
