---- MODULE Autoplay ----
\* One Autoplay run on one Game, as the dashboard shows it: the run holds the Game
\* between its Jobs, the progress counts the cycles, and the run ends on its budget,
\* on a failure or on a stop.
\*
\* The model follows docs/diataxis/reference/game_flow.md and
\* src/application/autoplay/driver.rs. It leaves out the pause point, the Options
\* run of a Job, and the restart that loses every holder.
EXTENDS Naturals

CONSTANTS Budget   \* the number of cycles that the player gave

VARIABLES holder, step, done, budget, ending

vars == <<holder, step, done, budget, ending>>

TypeOK ==
  /\ holder \in {"idle", "run"}
  /\ step \in {"none", "writing"}
  /\ budget = Budget
  /\ done \in 0..budget
  /\ ending \in {"none", "budget", "failed", "stopped"}

Init ==
  /\ holder = "idle"
  /\ step = "none"
  /\ done = 0
  /\ budget = Budget
  /\ ending = "none"

YouStartAutoplay ==
  /\ holder = "idle"
  /\ holder' = "run"
  /\ step' = "none"
  /\ done' = 0
  /\ ending' = "none"
  /\ UNCHANGED budget

\* The run's own accept passes while the run holds the Game.
TheRunAcceptsAJob ==
  /\ holder = "run"
  /\ step = "none"
  /\ done < budget
  /\ step' = "writing"
  /\ UNCHANGED <<holder, done, budget, ending>>

\* The Job ends with the story at a cycle boundary, so the run played one cycle.
AJobEndsACycle ==
  /\ holder = "run"
  /\ step = "writing"
  /\ step' = "none"
  /\ done' = done + 1
  /\ UNCHANGED <<holder, budget, ending>>

\* The Job ends with the story waiting for a narration, so no cycle counts.
AJobEndsMidCycle ==
  /\ holder = "run"
  /\ step = "writing"
  /\ step' = "none"
  /\ UNCHANGED <<holder, done, budget, ending>>

TheBudgetIsReached ==
  /\ holder = "run"
  /\ step = "none"
  /\ done >= budget
  /\ holder' = "idle"
  /\ step' = "none"
  /\ ending' = "budget"
  /\ UNCHANGED <<done, budget>>

\* A Job of the run ended with a failure line, so the run ends.
AJobOfTheRunFails ==
  /\ holder = "run"
  /\ step = "writing"
  /\ holder' = "idle"
  /\ step' = "none"
  /\ ending' = "failed"
  /\ UNCHANGED <<done, budget>>

\* The player presses Stop: the holder goes, and the run's loop ends.
YouPressStop ==
  /\ holder = "run"
  /\ holder' = "idle"
  /\ step' = "none"
  /\ ending' = "stopped"
  /\ UNCHANGED <<done, budget>>

Next ==
  \/ YouStartAutoplay
  \/ TheRunAcceptsAJob
  \/ AJobEndsACycle
  \/ AJobEndsMidCycle
  \/ TheBudgetIsReached
  \/ AJobOfTheRunFails
  \/ YouPressStop

Spec == Init /\ [][Next]_vars

\* An Autoplay run never plays more cycles than the player gave it.
TheBudgetIsNeverPassed == done <= budget

\* A run that ends on its budget played every cycle that the player gave.
TheBudgetIsPlayedInFull == ending = "budget" => done = budget

\* The dashboard shows no step once the run has ended.
AnEndedRunShowsNoStep == ending # "none" => (holder = "idle" /\ step = "none")

====
