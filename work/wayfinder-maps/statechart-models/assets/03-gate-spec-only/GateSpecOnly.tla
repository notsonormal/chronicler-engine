---------------------------- MODULE GateSpecOnly ----------------------------
(***************************************************************************)
(* PROTOTYPE (ticket 03 of .scratch/statechart-models). Throwaway.         *)
(*                                                                         *)
(* The gate of one Game, written from docs/diataxis/reference/game_flow.md *)
(* and the gate scenarios of docs/specs/ only. No file of src/ was read    *)
(* before the first run.                                                   *)
(*                                                                         *)
(* One Game. Plain Jobs (a free-text Action) and Autoplay runs (an         *)
(* Impersonate Job, then a Continue Job). Each save of a Job is one atomic *)
(* step: "One hold of the registry covers one check and its transactions." *)
(* Each model call is a separate step that reads no holder: "No hold of    *)
(* the registry spans an LLM call."                                        *)
(*                                                                         *)
(* Where the docs are silent, a CONSTANT names the reading:                *)
(*   ReleaseChecksId     G1: the end of a Job or of a run removes only its *)
(*                       own job entry or holder.                          *)
(*   LoopNeedsOwnHolder  G2: the accept of a run's loop passes only while  *)
(*                       that run holds the Game.                          *)
(*   LoopReadsJobResult  G3: the loop ends on a failure line only when its *)
(*                       own last Job wrote it.                            *)
(* FALSE is the literal reading of the docs as written.                    *)
(*                                                                         *)
(* IdCheck = FALSE removes the job id check of each save (the guard).      *)
(*                                                                         *)
(* Version 2: an ended Job or loop keeps no field that no later step and   *)
(* no rule reads, so the state space stays small enough for a full check.  *)
(* The actions and the rules are those of version 1 (v1/).                 *)
(***************************************************************************)
EXTENDS Naturals, FiniteSets

CONSTANTS
  MaxJobs, MaxRuns, Budget, MaxRestarts,
  IdCheck,
  ReleaseChecksId,
  LoopNeedsOwnHolder,
  LoopReadsJobResult

JobIds == 1..MaxJobs
RunIds == 1..MaxRuns

NoHolder == [kind |-> "none", run |-> 0, job |-> 0, step |-> "none", cycles |-> 0]
NoLine == [on |-> FALSE, by |-> 0]
NoJobRec == [pc |-> "unborn", kind |-> "none", run |-> 0, narration |-> FALSE,
             failed |-> FALSE, step |-> "none"]
EndedJob(failed) == [NoJobRec EXCEPT !.pc = "ended", !.failed = failed]
NoLoopRec == [pc |-> "unborn", job |-> 0]
EndedLoop == [pc |-> "ended", job |-> 0]

VARIABLES
  holder,       \* the holder of the Game in the busy-state registry
  jobs,         \* each Job: where it is, its kind, its run, what it saved
  loops,        \* each Autoplay run's loop, and the Job it waits for
  failure,      \* the failure line of the Game, and the Job that wrote it
  optFail,      \* the Options failure of the dock record
  lastIsInput,  \* the end of the story is an Input
  stateWrites,  \* the Swipe state writes of the Swipe that each running Job made
  gameExists, gameActive,
  nextJob, nextRun, restarts,
  \* ghost variables: the history that the rules read
  cut,          \* running Jobs that a stop, a reset, a delete or a restart ended
  endedRuns,    \* running loops whose run a stop, a reset, a delete or a restart ended
  late,         \* <<Job, record>>: a save that landed after its Job was cut
  staleAccepts, \* Jobs that the loop of an ended run accepted
  wrongRunEnds, \* runs that ended on a failure line that their last Job did not write
  sinceStop,    \* a stop came, and the player started nothing since
  justRestarted,
  storyWriteWhileBusy, configWriteUnderRun,
  lastAccepted

story == <<failure, optFail, lastIsInput, stateWrites, late>>
place == <<gameExists, gameActive>>
counters == <<nextJob, nextRun, restarts>>
ghosts == <<staleAccepts, wrongRunEnds, sinceStop, justRestarted,
            storyWriteWhileBusy, configWriteUnderRun, lastAccepted>>

vars == <<holder, jobs, loops, cut, endedRuns, story, place, counters, ghosts>>

Running(j) == jobs[j].pc \notin {"unborn", "ended"}
LoopRunning(r) == loops[r].pc \notin {"unborn", "ended"}
EntryHolds(j) == holder.kind # "none" /\ holder.job = j
OwnHolder(r) == holder.kind = "run" /\ holder.run = r

\* "Each save method of a JobWriter checks, in one hold of the registry,
\* that the job entry of the Game's holder still holds the Job's id."
Passes(j) == gameExists /\ (~IdCheck \/ EntryHolds(j))

Late(j, what) == IF j \in cut THEN late \cup {<<j, what>>} ELSE late

\* "The status of a Game has one rule."
Status == CASE holder.kind = "run" -> "autoplay"
            [] holder.kind = "job" -> "thinking"
            [] failure.on          -> "failed"
            [] OTHER               -> "idle"

\* What a stop, a reset, a delete or a restart cuts.
CutAll == {j \in JobIds : Running(j)}
EndAllRuns == {r \in RunIds : LoopRunning(r)}

Init ==
  /\ holder = NoHolder
  /\ jobs = [j \in JobIds |-> NoJobRec]
  /\ loops = [r \in RunIds |-> NoLoopRec]
  /\ failure = NoLine
  /\ optFail = NoLine
  /\ lastIsInput = FALSE
  /\ stateWrites = [j \in JobIds |-> 0]
  /\ gameExists = TRUE
  /\ gameActive = TRUE
  /\ nextJob = 1
  /\ nextRun = 1
  /\ restarts = 0
  /\ cut = {}
  /\ endedRuns = {}
  /\ late = {}
  /\ staleAccepts = {}
  /\ wrongRunEnds = {}
  /\ sinceStop = FALSE
  /\ justRestarted = FALSE
  /\ storyWriteWhileBusy = FALSE
  /\ configWriteUnderRun = FALSE
  /\ lastAccepted = 0

---------------------------------------------------------------------------
(* The player *)

\* "A Game has at most one Job. A second request for a busy Game is refused."
\* "The accept clears the failure line in its transaction."
YouSendAnAction ==
  /\ gameExists /\ gameActive /\ holder.kind = "none" /\ nextJob <= MaxJobs
  /\ holder' = [kind |-> "job", run |-> 0, job |-> nextJob, step |-> "none", cycles |-> 0]
  /\ jobs' = [jobs EXCEPT ![nextJob] = [NoJobRec EXCEPT !.pc = "input", !.kind = "action"]]
  /\ failure' = NoLine
  /\ nextJob' = nextJob + 1
  /\ lastAccepted' = nextJob
  /\ sinceStop' = FALSE
  /\ justRestarted' = FALSE
  /\ UNCHANGED <<loops, cut, endedRuns, optFail, lastIsInput, stateWrites, late, place,
                 nextRun, restarts, staleAccepts, wrongRunEnds,
                 storyWriteWhileBusy, configWriteUnderRun>>

\* The run holds the Game from its start, before its first Job (Scenario 43.12).
YouStartAutoplay ==
  /\ gameExists /\ gameActive /\ holder.kind = "none" /\ nextRun <= MaxRuns
  /\ holder' = [kind |-> "run", run |-> nextRun, job |-> 0, step |-> "none", cycles |-> 0]
  /\ loops' = [loops EXCEPT ![nextRun] = [NoLoopRec EXCEPT !.pc = "start"]]
  /\ nextRun' = nextRun + 1
  /\ sinceStop' = FALSE
  /\ justRestarted' = FALSE
  /\ UNCHANGED <<jobs, cut, endedRuns, story, place, nextJob, restarts,
                 staleAccepts, wrongRunEnds, storyWriteWhileBusy, configWriteUnderRun,
                 lastAccepted>>

\* "A stop removes the holder, clears the failure line and clears the Options
\* failure of the dock record. The Game is idle at once."
YouPressStop ==
  /\ gameExists /\ gameActive
  /\ holder' = NoHolder
  /\ failure' = NoLine
  /\ optFail' = NoLine
  /\ cut' = CutAll
  /\ endedRuns' = EndAllRuns
  /\ sinceStop' = TRUE
  /\ UNCHANGED <<jobs, loops, lastIsInput, stateWrites, late, place, counters,
                 staleAccepts, wrongRunEnds, justRestarted,
                 storyWriteWhileBusy, configWriteUnderRun, lastAccepted>>

\* "A reset creates a new Game and removes the active Game with its holder.
\* Both writes land in one hold of the registry."
YouResetTheGame ==
  /\ gameExists /\ gameActive
  /\ gameExists' = FALSE
  /\ gameActive' = FALSE
  /\ holder' = NoHolder
  /\ cut' = CutAll
  /\ endedRuns' = EndAllRuns
  /\ UNCHANGED <<jobs, loops, story, counters, ghosts>>

\* "A stop, a reset and a delete of the Game remove the holder."
\* Only a Game that is not active can be deleted (Scenario 19.2).
YouDeleteTheGame ==
  /\ gameExists /\ ~gameActive
  /\ gameExists' = FALSE
  /\ holder' = NoHolder
  /\ cut' = CutAll
  /\ endedRuns' = EndAllRuns
  /\ UNCHANGED <<jobs, loops, story, gameActive, counters, ghosts>>

\* "A Job runs on the Game that its request named."
YouSwitchToAnotherGame ==
  /\ gameExists /\ gameActive
  /\ gameActive' = FALSE
  /\ UNCHANGED <<holder, jobs, loops, cut, endedRuns, story, gameExists, counters, ghosts>>

YouSwitchBack ==
  /\ gameExists /\ ~gameActive
  /\ gameActive' = TRUE
  /\ UNCHANGED <<holder, jobs, loops, cut, endedRuns, story, gameExists, counters, ghosts>>

\* "While an Autoplay run holds the Game, a write of the Game's config columns
\* is refused as busy. A config write passes while a plain Job runs."
YouChangeTheConfig ==
  /\ gameExists /\ holder.kind # "run"
  /\ configWriteUnderRun' = (configWriteUnderRun \/ holder.kind = "run")
  /\ UNCHANGED <<holder, jobs, loops, cut, endedRuns, story, place, counters,
                 staleAccepts, wrongRunEnds, sinceStop, justRestarted,
                 storyWriteWhileBusy, lastAccepted>>

\* "While a holder exists, the busy rule refuses every story write as busy."
\* An edit, a delete or a swipe switch can change the end of the story.
YouEditDeleteOrSwitchASwipe ==
  /\ gameExists /\ gameActive /\ holder.kind = "none"
  /\ lastIsInput' \in BOOLEAN
  /\ storyWriteWhileBusy' = (storyWriteWhileBusy \/ holder.kind # "none")
  /\ UNCHANGED <<holder, jobs, loops, cut, endedRuns, failure, optFail, stateWrites, late,
                 place, counters, staleAccepts, wrongRunEnds, sinceStop, justRestarted,
                 configWriteUnderRun, lastAccepted>>

\* "No saved record holds the holder, the step or the progress. A restart loses
\* every holder, so every Game is idle after a start."
TheEngineRestarts ==
  /\ restarts < MaxRestarts
  /\ restarts' = restarts + 1
  /\ holder' = NoHolder
  /\ jobs' = [j \in JobIds |-> IF jobs[j].pc = "unborn" THEN jobs[j] ELSE EndedJob(FALSE)]
  /\ loops' = [r \in RunIds |-> IF loops[r].pc = "unborn" THEN loops[r] ELSE EndedLoop]
  /\ stateWrites' = [j \in JobIds |-> 0]
  /\ cut' = {}
  /\ endedRuns' = {}
  /\ justRestarted' = TRUE
  /\ UNCHANGED <<failure, optFail, lastIsInput, late, place, nextJob, nextRun,
                 staleAccepts, wrongRunEnds, sinceStop,
                 storyWriteWhileBusy, configWriteUnderRun, lastAccepted>>

---------------------------------------------------------------------------
(* A Job. The parts of a generation, from "The Parts of a Generation".     *)
(* pc: input -> stepN -> callN -> text -> stepQ -> callQ -> update         *)
(*     -> stepO -> callO -> saveO | saveOF -> end -> ended                 *)
(* A failure goes to "fail"; the Options run follows when the Job saved a  *)
(* Narration.                                                              *)

\* The variables that a step of a Job never changes.
notJob == <<loops, cut, endedRuns, place, counters, ghosts>>

StepName(p) == CASE p = "stepN" -> "narrating"
                 [] p = "stepQ" -> "quantifying"
                 [] OTHER       -> "options"

CallAfter(p) == CASE p = "stepN" -> "callN"
                  [] p = "stepQ" -> "callQ"
                  [] OTHER       -> "callO"

\* The parts that check the job entry first.
NeedsCheck(j) ==
  \/ jobs[j].pc \in {"input", "stepN", "text", "stepQ", "update", "fail", "stepO", "saveO"}
  \/ jobs[j].pc = "saveOF" /\ ~jobs[j].failed

Move(j, pc) == jobs' = [jobs EXCEPT ![j].pc = pc]

TheInputIsSaved(j) ==
  /\ jobs[j].pc = "input" /\ Passes(j)
  /\ Move(j, "stepN")
  /\ lastIsInput' = TRUE
  /\ late' = Late(j, "input")
  /\ UNCHANGED <<holder, failure, optFail, stateWrites>> /\ UNCHANGED notJob

\* "Each step sets its name in the job entry of the Job before it runs."
AStepStarts(j) ==
  /\ jobs[j].pc \in {"stepN", "stepQ", "stepO"} /\ Passes(j)
  /\ jobs' = [jobs EXCEPT ![j].pc = CallAfter(jobs[j].pc), ![j].step = StepName(jobs[j].pc)]
  /\ holder' = IF holder.job # 0 THEN [holder EXCEPT !.step = StepName(jobs[j].pc)] ELSE holder
  /\ UNCHANGED story /\ UNCHANGED notJob

\* "A model call that runs when the stop comes runs to its end."
TheNarratorAnswers(j) ==
  /\ jobs[j].pc = "callN" /\ Move(j, "text")
  /\ UNCHANGED <<holder, story>> /\ UNCHANGED notJob

TheNarratorCallFails(j) ==
  /\ jobs[j].pc = "callN" /\ Move(j, "fail")
  /\ UNCHANGED <<holder, story>> /\ UNCHANGED notJob

\* A failed Quantifier call falls back, so the analysis always answers (Scenario 41.24).
TheAnalysisAnswers(j) ==
  /\ jobs[j].pc = "callQ" /\ Move(j, "update")
  /\ UNCHANGED <<holder, story>> /\ UNCHANGED notJob

TheOptionsCallAnswers(j) ==
  /\ jobs[j].pc = "callO" /\ Move(j, "saveO")
  /\ UNCHANGED <<holder, story>> /\ UNCHANGED notJob

TheOptionsCallFails(j) ==
  /\ jobs[j].pc = "callO" /\ Move(j, "saveOF")
  /\ UNCHANGED <<holder, story>> /\ UNCHANGED notJob

\* The text save writes the pre-analysis state of the new Swipe.
\* An Impersonate Job's text is an Input; the other kinds save a Narration.
TheTextIsSaved(j) ==
  /\ jobs[j].pc = "text" /\ Passes(j)
  /\ jobs' = [jobs EXCEPT ![j].pc = "stepQ", ![j].narration = (jobs[j].kind # "impersonate")]
  /\ lastIsInput' = (jobs[j].kind = "impersonate")
  /\ stateWrites' = [stateWrites EXCEPT ![j] = @ + 1]
  /\ late' = Late(j, "text")
  /\ UNCHANGED <<holder, failure, optFail>> /\ UNCHANGED notJob

\* The state update writes the analysed state of the same Swipe.
TheStateIsUpdated(j) ==
  /\ jobs[j].pc = "update" /\ Passes(j)
  /\ Move(j, IF jobs[j].narration THEN "stepO" ELSE "end")
  /\ stateWrites' = [stateWrites EXCEPT ![j] = @ + 1]
  /\ late' = Late(j, "state")
  /\ UNCHANGED <<holder, failure, optFail, lastIsInput>> /\ UNCHANGED notJob

TheStateUpdateFails(j) ==
  /\ jobs[j].pc = "update" /\ Passes(j)
  /\ Move(j, "fail")
  /\ UNCHANGED <<holder, story>> /\ UNCHANGED notJob

\* "A part that fails writes the failure line in one transaction."
\* "When the Job saved a Narration ..., the Options run follows a failure of any later part."
TheFailureLineIsSaved(j) ==
  /\ jobs[j].pc = "fail" /\ Passes(j)
  /\ failure' = [on |-> TRUE, by |-> j]
  /\ jobs' = [jobs EXCEPT ![j].pc = IF jobs[j].narration THEN "stepO" ELSE "end", ![j].failed = TRUE]
  /\ late' = Late(j, "failure line")
  /\ UNCHANGED <<holder, optFail, lastIsInput, stateWrites>> /\ UNCHANGED notJob

TheOptionsAreSaved(j) ==
  /\ jobs[j].pc = "saveO" /\ Passes(j)
  /\ Move(j, "end")
  /\ late' = Late(j, "options")
  /\ UNCHANGED <<holder, failure, optFail, lastIsInput, stateWrites>> /\ UNCHANGED notJob

\* "A failed Options run writes its Options failure" (Scenario 41.5).
TheOptionsFailureIsSaved(j) ==
  /\ jobs[j].pc = "saveOF" /\ ~jobs[j].failed /\ Passes(j)
  /\ optFail' = [on |-> TRUE, by |-> 0]
  /\ Move(j, "end")
  /\ late' = Late(j, "options failure")
  /\ UNCHANGED <<holder, failure, lastIsInput, stateWrites>> /\ UNCHANGED notJob

\* "A failure of that Options run goes to the log."
TheOptionsFailureGoesToTheLog(j) ==
  /\ jobs[j].pc = "saveOF" /\ jobs[j].failed
  /\ Move(j, "end")
  /\ UNCHANGED <<holder, story>> /\ UNCHANGED notJob

\* "When the check fails, the save is refused and the Job ends with no further write."
ASaveIsRefused(j) ==
  /\ NeedsCheck(j) /\ ~Passes(j)
  /\ Move(j, "end")
  /\ UNCHANGED <<holder, story>> /\ UNCHANGED notJob

\* The docs do not say how the end of a Job leaves the registry (gap G1).
\* Literal reading: the end removes the job entry, and for a plain Job the holder.
\* The ended record keeps only what the loop of its run reads: whether it failed.
TheJobEnds(j) ==
  /\ jobs[j].pc = "end"
  /\ jobs' = [jobs EXCEPT ![j] = EndedJob(jobs[j].run # 0 /\ jobs[j].failed)]
  /\ stateWrites' = [stateWrites EXCEPT ![j] = 0]
  /\ cut' = cut \ {j}
  /\ holder' = IF ReleaseChecksId /\ ~EntryHolds(j) THEN holder
               ELSE CASE holder.kind = "job" -> NoHolder
                      [] holder.kind = "run" -> [holder EXCEPT !.job = 0, !.step = "none"]
                      [] OTHER               -> holder
  /\ UNCHANGED <<failure, optFail, lastIsInput, late>>
  /\ UNCHANGED <<loops, endedRuns, place, counters, ghosts>>

---------------------------------------------------------------------------
(* The Autoplay loop.                                                      *)
(* "In one hold of the registry, the accept reads the end of the story,    *)
(* counts the cycle that the last Job ended, and either finishes the run   *)
(* or accepts the next Job."                                               *)

LoopReady(r) ==
  \/ loops[r].pc = "start"
  \/ loops[r].pc = "job" /\ jobs[loops[r].job].pc = "ended"

\* Gap G2. Literal reading: the busy rule refuses the accept only while
\* another holder exists ("only the accept of the run's own loop passes").
LoopPasses(r) == IF LoopNeedsOwnHolder THEN OwnHolder(r)
                 ELSE OwnHolder(r) \/ holder.kind = "none"

LastJob(r) == loops[r].job

\* Gap G3. Literal reading: the accept reads the failure line from storage.
LastFailed(r) == IF LoopReadsJobResult THEN LastJob(r) # 0 /\ jobs[LastJob(r)].failed
                 ELSE failure.on
FailedBy(r) == IF LoopReadsJobResult THEN LastJob(r) ELSE failure.by

\* "A cycle ends when a Job of the run ends without a failure line and leaves
\* the story on a Narration." The holder holds the progress.
Counted(r) == holder.cycles + (IF LastJob(r) # 0 /\ ~LastFailed(r) /\ ~lastIsInput THEN 1 ELSE 0)

\* Gap G1 for a run. Literal reading: the end of the run removes the holder.
RunRelease(r) == IF ReleaseChecksId /\ ~OwnHolder(r) THEN holder ELSE NoHolder

\* The loop no longer needs the ended record of its last Job.
ForgetLastJob(r, next) ==
  [k \in JobIds |-> IF k = LastJob(r) THEN EndedJob(FALSE) ELSE next[k]]

TheRunIsRefused(r) ==
  /\ LoopReady(r) /\ ~LoopPasses(r)
  /\ loops' = [loops EXCEPT ![r] = EndedLoop]
  /\ jobs' = ForgetLastJob(r, jobs)
  /\ endedRuns' = endedRuns \ {r}
  /\ holder' = RunRelease(r)
  /\ UNCHANGED <<cut, story, place, counters, ghosts>>

TheRunFinishes(r) ==
  /\ LoopReady(r) /\ LoopPasses(r)
  /\ LastFailed(r) \/ Counted(r) >= Budget
  /\ loops' = [loops EXCEPT ![r] = EndedLoop]
  /\ jobs' = ForgetLastJob(r, jobs)
  /\ endedRuns' = endedRuns \ {r}
  /\ holder' = RunRelease(r)
  /\ wrongRunEnds' = IF LastFailed(r) /\ FailedBy(r) # LastJob(r)
                     THEN wrongRunEnds \cup {r} ELSE wrongRunEnds
  /\ UNCHANGED <<cut, story, place, counters, staleAccepts, sinceStop, justRestarted,
                 storyWriteWhileBusy, configWriteUnderRun, lastAccepted>>

\* "The next Job of a run is a Continue when the last Message is an Input,
\* and an Impersonate in every other case."
TheRunAcceptsItsNextJob(r) ==
  /\ LoopReady(r) /\ LoopPasses(r)
  /\ ~LastFailed(r) /\ Counted(r) < Budget
  /\ nextJob <= MaxJobs
  /\ holder' = [kind |-> "run", run |-> r, job |-> nextJob, step |-> "none", cycles |-> Counted(r)]
  /\ jobs' = ForgetLastJob(r, [jobs EXCEPT ![nextJob] = [NoJobRec EXCEPT
                 !.pc = "stepN",
                 !.kind = IF lastIsInput THEN "continue" ELSE "impersonate",
                 !.run = r]])
  /\ loops' = [loops EXCEPT ![r].pc = "job", ![r].job = nextJob]
  /\ failure' = NoLine
  /\ nextJob' = nextJob + 1
  /\ lastAccepted' = nextJob
  /\ staleAccepts' = IF r \in endedRuns THEN staleAccepts \cup {nextJob} ELSE staleAccepts
  /\ UNCHANGED <<cut, endedRuns, optFail, lastIsInput, stateWrites, late, place, nextRun,
                 restarts, wrongRunEnds, sinceStop, justRestarted,
                 storyWriteWhileBusy, configWriteUnderRun>>

---------------------------------------------------------------------------

\* The end state: the Game is gone and nothing runs. A loop that has no Job id
\* left stops at the bound of the TLA+ model (MaxJobs), not at a rule of the docs.
NothingMoreHappens ==
  /\ ~gameExists
  /\ \A j \in JobIds : ~Running(j)
  /\ \A r \in RunIds : LoopRunning(r) => nextJob > MaxJobs
  /\ UNCHANGED vars

Next ==
  \/ YouSendAnAction
  \/ YouStartAutoplay
  \/ YouPressStop
  \/ YouResetTheGame
  \/ YouDeleteTheGame
  \/ YouSwitchToAnotherGame
  \/ YouSwitchBack
  \/ YouChangeTheConfig
  \/ YouEditDeleteOrSwitchASwipe
  \/ TheEngineRestarts
  \/ \E j \in JobIds :
       \/ TheInputIsSaved(j)
       \/ AStepStarts(j)
       \/ TheNarratorAnswers(j)
       \/ TheNarratorCallFails(j)
       \/ TheAnalysisAnswers(j)
       \/ TheOptionsCallAnswers(j)
       \/ TheOptionsCallFails(j)
       \/ TheTextIsSaved(j)
       \/ TheStateIsUpdated(j)
       \/ TheStateUpdateFails(j)
       \/ TheFailureLineIsSaved(j)
       \/ TheOptionsAreSaved(j)
       \/ TheOptionsFailureIsSaved(j)
       \/ TheOptionsFailureGoesToTheLog(j)
       \/ ASaveIsRefused(j)
       \/ TheJobEnds(j)
  \/ \E r \in RunIds :
       \/ TheRunIsRefused(r)
       \/ TheRunFinishes(r)
       \/ TheRunAcceptsItsNextJob(r)
  \/ NothingMoreHappens

Spec == Init /\ [][Next]_vars

---------------------------------------------------------------------------
(* The rules. Each one quotes its sentence. The ids pair with the table of *)
(* the ticket 03 asset.                                                    *)

\* R01 game_flow.md, The Writer Rule: "A stop, a reset and a delete of the
\* Game remove the holder. A late save of their Job lands nowhere."
R01_ALateSaveLandsNowhere == late = {}

\* R02 game_flow.md, Overview: "While a Job runs, its Game is busy."
\* Scenario 39.5: "the story log holds the Input and the Narration of the second Job".
R02_ARunningJobKeepsItsEntry ==
  \A j \in JobIds : Running(j) /\ j \notin cut => EntryHolds(j)

\* R03 game_flow.md, The Busy State and Its Holder: "An Autoplay run. It lasts
\* from the start of the run to its end, also between two Jobs of the run."
\* Scenario 41.17: "The end of a stopped run does not release the run that started after it".
R03_ARunHoldsTheGameUntilItEnds ==
  \A r \in RunIds : LoopRunning(r) /\ r \notin endedRuns => OwnHolder(r)

\* R04 Scenarios 41.6, 41.7, 41.15, 41.16: "no further Job of the run starts".
\* game_flow.md: "For an Autoplay run, only the accept of the run's own loop passes."
R04_AnEndedRunStartsNoJob == staleAccepts = {}

\* R05 game_flow.md, Stop and Reset: "A stop removes the holder, clears the
\* failure line ... The Game is idle at once." The stop is the last thing the player did.
R05_TheGameStaysIdleAfterAStop == sinceStop => Status = "idle"

\* R06 game_flow.md, The State of a Game: "A stop or a restart between the two
\* writes leaves the Swipe with its pre-analysis state."
R06_ACutSwipeKeepsItsPreAnalysisState == \A j \in JobIds : <<j, "state">> \notin late

\* R07 game_flow.md, Stop and Reset: "A stop ... clears the Options failure of the dock record."
R07_TheDockHoldsNoOptionsFailureAfterAStop == sinceStop => ~optFail.on

\* R08 game_flow.md, Status: "The accept clears it in storage, because a restart
\* during the new Job would otherwise show the old failure again."
R08_TheFailureLineIsOfTheLastJob == failure.on => failure.by = lastAccepted

\* R09 game_flow.md: "A plain Job. Its job entry holds the Job's id and the current
\* Step." Steps: "Each step sets its name in the job entry of the Job before it runs."
R09_TheEntryShowsTheStepOfItsOwnJob ==
  holder.job # 0 => holder.step = jobs[holder.job].step

\* R10 autoplay.md: "A run ends for one of these reasons: a stop, a reset or a
\* delete of the Game, the budget, a Job that ends with a failure line, or an
\* accept that cannot read a record of the Game."
R10_ARunEndsOnAFailureOfItsOwnJobOnly == wrongRunEnds = {}

\* R11 game_flow.md, Status: "the number of the cycle in progress and the budget".
R11_TheCycleInProgressIsInTheBudget == holder.kind = "run" => holder.cycles < Budget

\* R12 game_flow.md: "While a holder exists, the busy rule refuses every story write as busy."
R12_NoStoryWriteWhileBusy == ~storyWriteWhileBusy

\* R13 game_flow.md: "While an Autoplay run holds the Game, a write of the Game's
\* config columns ... is refused as busy."
R13_NoConfigWriteUnderARun == ~configWriteUnderRun

\* R14 game_flow.md: "Only the Job that makes a Swipe writes its Swipe state, at most twice".
R14_ASwipeStateIsWrittenAtMostTwice == \A j \in JobIds : stateWrites[j] <= 2

\* R15 game_flow.md: "A restart loses every holder, so every Game is idle after a start."
R15_EveryGameIsIdleAfterAStart == justRestarted => holder.kind = "none"

=============================================================================
