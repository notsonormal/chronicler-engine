---- MODULE VerifyGate ----
\* The gate of the Chronicler Engine, made from the code: the busy registry, the
\* Job id check, the Autoplay holder, stop, reset, delete, the story writes and the
\* config write.
\*
\* Each action names the code function it mirrors. One action is one hold of
\* `BusyRegistry::hold` (src/application/generation/gate.rs), or one step outside
\* every hold (a model call, a read). SKILL.md lists the actions, the rules and the
\* places where one action covers more than one hold.
\*
\* The rules (the INVARIANTS and the PROPERTY at the end) come from docs/specs/.
\* Each one quotes its scenario.
EXTENDS Integers, FiniteSets

CONSTANTS
  Games0,       \* the Games that exist at the start
  NewGames,     \* the ids that a reset can give to the Game it creates
  Active0,      \* the active Game at the start
  MaxJobs,      \* the number of Jobs that can be accepted (the job ids 1..MaxJobs)
  MaxRuns,      \* the number of Autoplay runs that can start (the run ids 1..MaxRuns)
  Budget,       \* the budget of each run, in cycles
  PlainKinds,   \* the kinds of plain Job a request can start: "action", "options"
  Env           \* the requests that this config allows

AllGames == Games0 \cup NewGames
JobIds == 1..MaxJobs
RunIds == 1..MaxRuns
NoJob == 0

\* The values of the fields below. `fail` is the failure that the Job carries to its end.
JobPcs == {"idle", "start", "call", "save", "analyse", "quant", "update",
           "afterGens", "optstart", "optcall", "optsave", "endfail", "drop", "done"}
JobKinds == {"none", "action", "options", "impersonate", "continue"}
RunPcs == {"idle", "accept", "job", "end", "over"}
RunEnds == {"none", "stopped", "budget", "jobFailed", "notAccepted"}

VARIABLES
  exists,       \* the Games whose rows exist
  fresh,        \* the ids of NewGames that no reset has used yet (a Game id is never used twice)
  active,       \* ActiveGame: the active Game
  holder,       \* BusyRegistry: the holder of each Game ("none" = idle)
  failLine,     \* the failure line of each Game: 0 none, -1 an accept's line, j the line of Job j
  optFail,      \* the Options failure of the dock of each Game: 0 none, j the failure of Job j
  last,         \* the type of the last Message of each Game: "narr" or "input"
  nextJob,      \* the next job id
  nextRun,      \* the next run id
  job,          \* the state of each Job
  run,          \* the state of each Autoplay run
  pending,      \* the two-part requests in flight: a delete past its check, a reset before its pointer move
  \* Ghost variables: they record what the rules read, and no code reads them.
  cut,          \* the Jobs whose entry a stop, a reset or a delete removed
  cutRuns,      \* the runs whose holder a stop, a reset or a delete removed
  late,         \* a write of a cut Job landed
  storyBusy,    \* a story write or a Job start landed while the Game had a holder
  cfgRun,       \* a config write landed while a run held the Game
  badAccept,    \* a cut run accepted a Job
  deletedActive \* a delete removed the active Game

vars == <<exists, fresh, active, holder, failLine, optFail, last, nextJob, nextRun, job, run,
          pending, cut, cutRuns, late, storyBusy, cfgRun, badAccept, deletedActive>>

Idle == [kind |-> "none", job |-> NoJob, run |-> 0, done |-> 0]
NoJobState == [g |-> 0, run |-> 0, kind |-> "none", pc |-> "idle", gen |-> "first",
               narr |-> FALSE, fail |-> "none", lineBy |-> "none", opt |-> "none",
               optRes |-> "none"]
NoRunState == [g |-> 0, pc |-> "idle", done |-> 0, after |-> FALSE, job |-> NoJob,
               end |-> "none", endLine |-> 0, endIdle |-> FALSE]

TypeOK ==
  /\ exists \subseteq AllGames
  /\ active \in AllGames
  /\ \A g \in AllGames : holder[g].kind \in {"none", "job", "run"}
  /\ \A j \in JobIds : job[j].pc \in JobPcs /\ job[j].kind \in JobKinds
  /\ \A r \in RunIds : run[r].pc \in RunPcs /\ run[r].end \in RunEnds
  /\ cut \subseteq JobIds /\ cutRuns \subseteq RunIds

Init ==
  /\ exists = Games0
  /\ fresh = NewGames
  /\ active = Active0
  /\ holder = [g \in AllGames |-> Idle]
  /\ failLine = [g \in AllGames |-> 0]
  /\ optFail = [g \in AllGames |-> 0]
  /\ last = [g \in AllGames |-> "narr"]
  /\ nextJob = 1
  /\ nextRun = 1
  /\ job = [j \in JobIds |-> NoJobState]
  /\ run = [r \in RunIds |-> NoRunState]
  /\ pending = [delete |-> {}, reset |-> 0]
  /\ cut = {}
  /\ cutRuns = {}
  /\ late = FALSE
  /\ storyBusy = FALSE
  /\ cfgRun = FALSE
  /\ badAccept = FALSE
  /\ deletedActive = FALSE

-----------------------------------------------------------------------------
\* Helpers

\* `JobWriter::hold`: the job entry of the Game's holder still holds this Job's id.
Own(j) == holder[job[j].g].job = j

\* A holder removal by a stop, a reset or a delete: its Job and its run are cut.
CutOf(g) ==
  /\ cut' = IF holder[g].job # NoJob THEN cut \cup {holder[g].job} ELSE cut
  /\ cutRuns' = IF holder[g].kind = "run" THEN cutRuns \cup {holder[g].run} ELSE cutRuns

\* A record write of Job j: it lands, and the ghost notes a write of a cut Job or a
\* write to a Game whose rows are gone.
Wrote(j) == late' = (late \/ j \in cut \/ job[j].g \notin exists)

\* The record of an ended Job keeps only what the run loop and the rules read, so
\* that two ended Jobs with the same outcome are one state.
Done(rec, fail) == [NoJobState EXCEPT !.g = rec.g, !.run = rec.run, !.pc = "done",
                                      !.fail = fail, !.lineBy = rec.lineBy]

\* The Job ends with no further write: `JobFailure::EntryMoved`. The drop of the
\* `JobWriter` checks its id again, finds another entry, and does nothing.
Moved(j) == job' = [job EXCEPT ![j] = Done(@, "moved")]

GhostsSame == UNCHANGED <<cut, cutRuns, late, storyBusy, cfgRun, badAccept, deletedActive>>
JobSide == UNCHANGED <<exists, fresh, active, pending, nextRun, run>>

-----------------------------------------------------------------------------
\* Requests (the HTTP driving side)

\* `GameWriter::accept` (job.rs `Job::start`): in one hold, the busy check, the story
\* read, the plan, `accept_in_tx` (it clears the failure line), and the job entry.
YouStartAJob(g, k) ==
  /\ "plain" \in Env
  /\ g \in exists
  /\ k \in PlainKinds
  /\ nextJob \in JobIds
  /\ holder[g].kind = "none"                         \* `require_idle`
  /\ storyBusy' = (storyBusy \/ holder[g].kind # "none")
  /\ holder' = [holder EXCEPT ![g] = [Idle EXCEPT !.kind = "job", !.job = nextJob]]
  /\ failLine' = [failLine EXCEPT ![g] = 0]
  /\ job' = [job EXCEPT ![nextJob] =
               [NoJobState EXCEPT !.g = g, !.kind = k,
                  !.pc = IF k = "options" THEN "optstart" ELSE "start",
                  !.opt = IF k = "options" THEN "ondemand" ELSE "after"]]
  /\ nextJob' = nextJob + 1
  /\ UNCHANGED <<exists, fresh, active, optFail, last, nextRun, run, pending,
                 cut, cutRuns, late, cfgRun, badAccept, deletedActive>>

\* `GameWriter::accept`, the branch where the story read or a record check fails:
\* `not_accepted_with_failure` writes the failure line, and no entry is made.
TheAcceptCannotRead(g) ==
  /\ "plain" \in Env
  /\ g \in exists
  /\ holder[g].kind = "none"
  /\ failLine' = [failLine EXCEPT ![g] = -1]
  /\ UNCHANGED <<exists, fresh, active, holder, optFail, last, nextJob, nextRun, job, run,
                 pending>>
  /\ GhostsSame

\* `GameWriter::reserve_for_autoplay` (driver.rs `AutoplayRun::start`).
YouStartAutoplay(g) ==
  /\ "run" \in Env
  /\ g \in exists
  /\ nextRun \in RunIds
  /\ holder[g].kind = "none"                         \* `require_idle`
  /\ storyBusy' = (storyBusy \/ holder[g].kind # "none")
  /\ holder' = [holder EXCEPT ![g] = [Idle EXCEPT !.kind = "run", !.run = nextRun]]
  /\ run' = [run EXCEPT ![nextRun] = [NoRunState EXCEPT !.g = g, !.pc = "accept"]]
  /\ nextRun' = nextRun + 1
  /\ UNCHANGED <<exists, fresh, active, failLine, optFail, last, nextJob, job, pending,
                 cut, cutRuns, late, cfgRun, badAccept, deletedActive>>

\* `GameWriter::stop`: the dock read goes first, then the holder goes, and one
\* transaction clears the failure line and the Options failure.
\* The branch where the dock read fails changes nothing (dashboard.md 39.8).
YouPressStop(g) ==
  /\ "stop" \in Env
  /\ g \in exists
  /\ holder' = [holder EXCEPT ![g] = Idle]
  /\ failLine' = [failLine EXCEPT ![g] = 0]
  /\ optFail' = [optFail EXCEPT ![g] = 0]
  /\ CutOf(g)
  /\ UNCHANGED <<exists, fresh, active, last, nextJob, nextRun, job, run, pending,
                 late, storyBusy, cfgRun, badAccept, deletedActive>>

\* `GameCatalogue::reset` then `GameWriter::replace`: one hold creates the new Game,
\* removes the old holder and deletes the old Game. The pointer moves after the hold.
YouResetTheGame(n) ==
  /\ "reset" \in Env
  /\ active \in exists
  /\ pending.reset = 0
  /\ n \in fresh
  /\ fresh' = fresh \ {n}
  /\ LET g == active IN
       /\ exists' = (exists \cup {n}) \ {g}
       /\ holder' = [holder EXCEPT ![g] = Idle]
       /\ CutOf(g)
  /\ pending' = [pending EXCEPT !.reset = n]
  /\ UNCHANGED <<active, failLine, optFail, last, nextJob, nextRun, job, run,
                 late, storyBusy, cfgRun, badAccept, deletedActive>>

\* `GameCatalogue::reset`, its last line: `ActiveGame::set_new`.
TheResetMovesThePointer ==
  /\ pending.reset # 0
  /\ active' = pending.reset
  /\ pending' = [pending EXCEPT !.reset = 0]
  /\ UNCHANGED <<exists, fresh, holder, failLine, optFail, last, nextJob, nextRun, job, run>>
  /\ GhostsSame

\* `GameCatalogue::delete_game`, its first line: the check against the active Game.
\* It reads `ActiveGame`, outside the hold of the gate.
YouDeleteAGame(g) ==
  /\ "delete" \in Env
  /\ g \in exists
  /\ g \notin pending.delete
  /\ active # g
  /\ pending' = [pending EXCEPT !.delete = @ \cup {g}]
  /\ UNCHANGED <<exists, fresh, active, holder, failLine, optFail, last, nextJob, nextRun,
                 job, run>>
  /\ GhostsSame

\* `GameWriter::delete`: one hold removes the holder and the rows of the Game.
TheDeleteRemovesTheGame(g) ==
  /\ g \in pending.delete
  /\ exists' = exists \ {g}
  /\ holder' = [holder EXCEPT ![g] = Idle]
  /\ CutOf(g)
  /\ deletedActive' = (deletedActive \/ (active = g /\ g \in exists))
  /\ pending' = [pending EXCEPT !.delete = @ \ {g}]
  /\ UNCHANGED <<fresh, active, failLine, optFail, last, nextJob, nextRun, job, run,
                 late, storyBusy, cfgRun, badAccept>>

\* `GameCatalogue::switch_game`: the Game read and `ActiveGame::set`.
YouSwitchTheGame(g) ==
  /\ "switch" \in Env
  /\ g \in exists
  /\ active # g
  /\ active' = g
  /\ UNCHANGED <<exists, fresh, holder, failLine, optFail, last, nextJob, nextRun, job, run,
                 pending>>
  /\ GhostsSame

\* `GameWriter::edit`: the busy check, then the text write. No failure line change.
YouEditAMessage(g) ==
  /\ "story" \in Env
  /\ g \in exists
  /\ holder[g].kind = "none"                         \* `require_idle`
  /\ storyBusy' = (storyBusy \/ holder[g].kind # "none")
  /\ UNCHANGED <<exists, fresh, active, holder, failLine, optFail, last, nextJob, nextRun,
                 job, run, pending, cut, cutRuns, late, cfgRun, badAccept, deletedActive>>

\* `GameWriter::switch_swipe` and `GameWriter::delete_last`: the busy check, the
\* write, and `clear_failure_and_put_dock` (the failure line and the Options failure go).
YouSwitchOrDeleteASwipe(g) ==
  /\ "story" \in Env
  /\ g \in exists
  /\ holder[g].kind = "none"                         \* `require_idle`
  /\ storyBusy' = (storyBusy \/ holder[g].kind # "none")
  /\ failLine' = [failLine EXCEPT ![g] = 0]
  /\ optFail' = [optFail EXCEPT ![g] = 0]
  /\ UNCHANGED <<exists, fresh, active, holder, last, nextJob, nextRun, job, run, pending,
                 cut, cutRuns, late, cfgRun, badAccept, deletedActive>>

\* `GameWriter::save_config` (catalogue.rs `set_posture`, `switch_mode`,
\* `set_preset_selection`): refused while an Autoplay run holds the Game.
YouChangeTheConfig(g) ==
  /\ "config" \in Env
  /\ g \in exists
  /\ holder[g].kind # "run"
  /\ cfgRun' = (cfgRun \/ holder[g].kind = "run")
  /\ UNCHANGED <<exists, fresh, active, holder, failLine, optFail, last, nextJob, nextRun,
                 job, run, pending, cut, cutRuns, late, storyBusy, badAccept, deletedActive>>

-----------------------------------------------------------------------------
\* One Job (job.rs `Job::run` and `Job::generate`, through `JobWriter`)

\* `run_step` and its `JobWriter::step` hold before the narrator call. For the first
\* generation of an Action, this action also covers the room check (a read) and the
\* hold of `JobWriter::save_input`.
AJobStartsAGeneration(j) ==
  /\ job[j].pc = "start"
  /\ IF Own(j)
       THEN /\ IF job[j].kind = "action" /\ job[j].gen = "first"
                 THEN /\ last' = [last EXCEPT ![job[j].g] = "input"]
                      /\ Wrote(j)
                 ELSE UNCHANGED <<last, late>>
            /\ job' = [job EXCEPT ![j].pc = "call"]
       ELSE /\ Moved(j)
            /\ UNCHANGED <<last, late>>
  /\ UNCHANGED <<holder, failLine, optFail, nextJob>>
  /\ JobSide
  /\ UNCHANGED <<cut, cutRuns, storyBusy, cfgRun, badAccept, deletedActive>>

\* `Job::narrate`: the model call, outside every hold.
TheNarratorAnswers(j) ==
  /\ job[j].pc = "call"
  /\ job' = [job EXCEPT ![j].pc = "save"]
  /\ UNCHANGED <<holder, failLine, optFail, last, nextJob>>
  /\ JobSide /\ GhostsSame

\* `Job::narrate` fails: the generation ends with a failure line (`JobFailure::Line`).
TheNarratorFails(j) ==
  /\ job[j].pc = "call"
  /\ job' = [job EXCEPT ![j].pc = "afterGens", ![j].fail = "line", ![j].lineBy = "narrator"]
  /\ UNCHANGED <<holder, failLine, optFail, last, nextJob>>
  /\ JobSide /\ GhostsSame

\* `JobWriter::save_text`, with its `run_step` hold. A Narration and an event
\* Narration empty the dock (`empties_the_dock`), so the Options failure goes too.
TheTextIsSaved(j) ==
  /\ job[j].pc = "save"
  /\ LET g == job[j].g
         savesNarr == job[j].gen = "event" \/ job[j].kind \in {"action", "continue"}
     IN IF Own(j)
          THEN /\ last' = [last EXCEPT ![g] = IF savesNarr THEN "narr" ELSE "input"]
               /\ optFail' = IF savesNarr THEN [optFail EXCEPT ![g] = 0] ELSE optFail
               /\ Wrote(j)
               /\ job' = [job EXCEPT ![j].pc = "analyse", ![j].narr = (@ \/ savesNarr)]
          ELSE /\ Moved(j)
               /\ UNCHANGED <<last, optFail, late>>
  /\ UNCHANGED <<holder, failLine, nextJob>>
  /\ JobSide
  /\ UNCHANGED <<cut, cutRuns, storyBusy, cfgRun, badAccept, deletedActive>>

\* `run_step(Step::Quantifying)`: the `JobWriter::step` hold before the analysis call.
TheAnalysisStarts(j) ==
  /\ job[j].pc = "analyse"
  /\ IF Own(j) THEN job' = [job EXCEPT ![j].pc = "quant"] ELSE Moved(j)
  /\ UNCHANGED <<holder, failLine, optFail, last, nextJob>>
  /\ JobSide /\ GhostsSame

\* `Job::analyse`: the Quantifier call. It does not fail: a failed call gives the fallback.
TheAnalysisAnswers(j) ==
  /\ job[j].pc = "quant"
  /\ job' = [job EXCEPT ![j].pc = "update"]
  /\ UNCHANGED <<holder, failLine, optFail, last, nextJob>>
  /\ JobSide /\ GhostsSame

\* `Job::update_state` and `JobWriter::save_state_update`, with the `run_step` hold.
\* The game rules fire a Trigger or not; a fired Trigger orders the event generation.
TheStateIsSaved(j, fires) ==
  /\ job[j].pc = "update"
  /\ fires \in BOOLEAN
  /\ fires => job[j].gen = "first"
  /\ IF Own(j)
       THEN /\ Wrote(j)
            /\ job' = [job EXCEPT ![j].pc = IF fires THEN "start" ELSE "afterGens",
                                  ![j].gen = IF fires THEN "event" ELSE @]
       ELSE /\ Moved(j)
            /\ UNCHANGED late
  /\ UNCHANGED <<holder, failLine, optFail, last, nextJob>>
  /\ JobSide
  /\ UNCHANGED <<cut, cutRuns, storyBusy, cfgRun, badAccept, deletedActive>>

\* `Job::run` after its generations: `OptionsRun::follows`, and the toggle that
\* `options_enabled` reads. A Job with no Options run goes to its end.
EndOfJob(j) ==
  IF job[j].fail = "line" THEN "endfail" ELSE "drop"

TheOptionsRunFollows(j) ==
  /\ job[j].pc = "afterGens"
  /\ job[j].opt = "after" /\ job[j].narr
  /\ job' = [job EXCEPT ![j].pc = "optstart"]
  /\ UNCHANGED <<holder, failLine, optFail, last, nextJob>>
  /\ JobSide /\ GhostsSame

TheJobSkipsTheOptionsRun(j) ==
  /\ job[j].pc = "afterGens"
  /\ job' = [job EXCEPT ![j].pc = EndOfJob(j)]
  /\ UNCHANGED <<holder, failLine, optFail, last, nextJob>>
  /\ JobSide /\ GhostsSame

\* `Job::options_step`: the `run_step(Step::Options)` hold before the Options call.
TheOptionsRunStarts(j) ==
  /\ job[j].pc = "optstart"
  /\ IF Own(j) THEN job' = [job EXCEPT ![j].pc = "optcall"] ELSE Moved(j)
  /\ UNCHANGED <<holder, failLine, optFail, last, nextJob>>
  /\ JobSide /\ GhostsSame

\* `AgentRunner::end_of_job`: the Options call. A failure gives `OptionsResult::Unavailable`.
TheOptionsCallEnds(j, ok) ==
  /\ job[j].pc = "optcall"
  /\ ok \in BOOLEAN
  /\ job' = [job EXCEPT ![j].pc = "optsave", ![j].optRes = IF ok THEN "offered" ELSE "unavailable"]
  /\ UNCHANGED <<holder, failLine, optFail, last, nextJob>>
  /\ JobSide /\ GhostsSame

\* `JobWriter::save_options`: a set clears the Options failure, and a failed run writes it.
TheOptionsAreSaved(j) ==
  /\ job[j].pc = "optsave"
  /\ IF Own(j)
       THEN /\ optFail' = [optFail EXCEPT ![job[j].g] =
                             IF job[j].optRes = "unavailable" THEN j ELSE 0]
            /\ Wrote(j)
            /\ job' = [job EXCEPT ![j].pc = EndOfJob(j), ![j].optRes = "none"]
       ELSE /\ Moved(j)
            /\ UNCHANGED <<optFail, late>>
  /\ UNCHANGED <<holder, failLine, last, nextJob>>
  /\ JobSide
  /\ UNCHANGED <<cut, cutRuns, storyBusy, cfgRun, badAccept, deletedActive>>

\* `Job::end` and `JobWriter::save_failure`: the failure line of the Job.
TheFailureLineIsSaved(j) ==
  /\ job[j].pc = "endfail"
  /\ IF Own(j)
       THEN /\ failLine' = [failLine EXCEPT ![job[j].g] = j]
            /\ Wrote(j)
            /\ job' = [job EXCEPT ![j].pc = "drop"]
       ELSE /\ Moved(j)
            /\ UNCHANGED <<failLine, late>>
  /\ UNCHANGED <<holder, optFail, last, nextJob>>
  /\ JobSide
  /\ UNCHANGED <<cut, cutRuns, storyBusy, cfgRun, badAccept, deletedActive>>

\* `impl Drop for JobWriter`: the entry goes only while it holds this Job's id. A plain
\* Job's holder goes with it; a run's holder stays, with no Job.
TheJobEnds(j) ==
  /\ job[j].pc = "drop"
  /\ LET g == job[j].g IN
       holder' = IF holder[g].job = j
                   THEN [holder EXCEPT ![g] =
                           IF @.kind = "run" THEN [@ EXCEPT !.job = NoJob] ELSE Idle]
                   ELSE holder
  /\ job' = [job EXCEPT ![j] = Done(@, @.fail)]
  /\ UNCHANGED <<failLine, optFail, last, nextJob>>
  /\ JobSide /\ GhostsSame

-----------------------------------------------------------------------------
\* One Autoplay run (driver.rs `AutoplayRun::run_jobs`)

\* `GameWriter::accept_for_autoplay`, in one hold: the run check, the story read,
\* `AutoplayRun::decide` (the cycle count and the budget), and `plan_and_start`.
\* `readOk` FALSE is the branch where the story read or a record check fails.
TheRunAccepts(r, readOk) ==
  /\ run[r].pc = "accept"
  /\ readOk \in BOOLEAN
  /\ LET g == run[r].g
         ours == holder[g].kind = "run" /\ holder[g].run = r
         d == run[r].done + (IF run[r].after /\ last[g] = "narr" THEN 1 ELSE 0)
         j == nextJob
     IN
     CASE ~ours ->                                    \* `AutoplayAccept::Ended`
            /\ readOk
            /\ run' = [run EXCEPT ![r].pc = "end", ![r].end = "stopped"]
            /\ UNCHANGED <<holder, failLine, job, nextJob, badAccept>>
       [] ours /\ holder[g].job # NoJob ->            \* its own Job in flight: unreachable
            /\ readOk
            /\ run' = [run EXCEPT ![r].pc = "end", ![r].end = "notAccepted"]
            /\ UNCHANGED <<holder, failLine, job, nextJob, badAccept>>
       [] ours /\ ~readOk ->                          \* `AutoplayAccept::NotAccepted`
            /\ failLine' = [failLine EXCEPT ![g] = -1]
            /\ holder' = [holder EXCEPT ![g] = Idle]
            /\ run' = [run EXCEPT ![r].pc = "end", ![r].end = "notAccepted",
                                  ![r].endLine = failLine'[g], ![r].endIdle = TRUE]
            /\ UNCHANGED <<job, nextJob, badAccept>>
       [] ours /\ readOk /\ d >= Budget ->            \* `AutoplayAccept::Finished`
            /\ holder' = [holder EXCEPT ![g] = Idle]
            /\ run' = [run EXCEPT ![r].pc = "end", ![r].end = "budget", ![r].done = d]
            /\ UNCHANGED <<failLine, job, nextJob, badAccept>>
       [] ours /\ readOk /\ d < Budget ->             \* `AutoplayAccept::Accepted`
            /\ j \in JobIds
            /\ failLine' = [failLine EXCEPT ![g] = 0]
            /\ holder' = [holder EXCEPT ![g] = [@ EXCEPT !.job = j, !.done = d]]
            /\ job' = [job EXCEPT ![j] =
                         [NoJobState EXCEPT !.g = g, !.run = r, !.pc = "start", !.opt = "after",
                            !.kind = IF last[g] = "input" THEN "continue" ELSE "impersonate"]]
            /\ nextJob' = nextJob + 1
            /\ run' = [run EXCEPT ![r].pc = "job", ![r].done = d, ![r].job = j]
            /\ badAccept' = (badAccept \/ r \in cutRuns)
  /\ UNCHANGED <<exists, fresh, active, optFail, last, nextRun, pending,
                 cut, cutRuns, late, storyBusy, cfgRun, deletedActive>>

\* `Job::run_accepted` returns its `JobEnd` to the loop.
TheRunSeesItsJobEnd(r) ==
  /\ run[r].pc = "job"
  /\ job[run[r].job].pc = "done"
  /\ LET f == job[run[r].job].fail IN
       run' = [run EXCEPT
                 ![r].pc = IF f = "none" THEN "accept" ELSE "end",
                 ![r].after = (f = "none"),
                 ![r].end = CASE f = "none" -> "none"
                              [] f = "line" -> "jobFailed"
                              [] OTHER -> "stopped"]
  /\ UNCHANGED <<exists, fresh, active, holder, failLine, optFail, last, nextJob, nextRun,
                 job, pending>>
  /\ GhostsSame

\* `impl Drop for AutoplayRun` and `GameWriter::end_autoplay`: the holder goes only
\* while it names this run. The ghost fields note the failure line at the moment the
\* run lets go of the Game (for `NotAccepted`, that is the accept itself), and
\* whether the run still holds the Game (a request that came after the run let go of
\* the Game can hold it again, and that is not a break).
TheRunEnds(r) ==
  /\ run[r].pc = "end"
  /\ LET g == run[r].g
         ours == holder[g].kind = "run" /\ holder[g].run = r
         h2 == IF ours THEN [holder EXCEPT ![g] = Idle] ELSE holder
     IN /\ holder' = h2
        /\ run' = IF run[r].end = "notAccepted" /\ run[r].endIdle
                     THEN [run EXCEPT ![r].pc = "over"]
                     ELSE [run EXCEPT ![r].pc = "over", ![r].endLine = failLine[g],
                                      ![r].endIdle = ~(h2[g].kind = "run" /\ h2[g].run = r)]
  /\ UNCHANGED <<exists, fresh, active, failLine, optFail, last, nextJob, nextRun, job, pending>>
  /\ GhostsSame

\* Every process has ended and no request can start more work. TLC sees no deadlock here.
Rest ==
  /\ \A j \in JobIds : job[j].pc \in {"idle", "done"}
  /\ \A r \in RunIds : run[r].pc \in {"idle", "over"}
  /\ pending.delete = {} /\ pending.reset = 0
  /\ UNCHANGED vars

Next ==
  \/ \E g \in AllGames, k \in PlainKinds : YouStartAJob(g, k)
  \/ \E g \in AllGames : TheAcceptCannotRead(g)
  \/ \E g \in AllGames : YouStartAutoplay(g)
  \/ \E g \in AllGames : YouPressStop(g)
  \/ \E n \in NewGames : YouResetTheGame(n)
  \/ TheResetMovesThePointer
  \/ \E g \in AllGames : YouDeleteAGame(g)
  \/ \E g \in AllGames : TheDeleteRemovesTheGame(g)
  \/ \E g \in AllGames : YouSwitchTheGame(g)
  \/ \E g \in AllGames : YouEditAMessage(g)
  \/ \E g \in AllGames : YouSwitchOrDeleteASwipe(g)
  \/ \E g \in AllGames : YouChangeTheConfig(g)
  \/ \E j \in JobIds :
       \/ AJobStartsAGeneration(j)
       \/ TheNarratorAnswers(j)
       \/ TheNarratorFails(j)
       \/ TheTextIsSaved(j)
       \/ TheAnalysisStarts(j)
       \/ TheAnalysisAnswers(j)
       \/ \E fires \in BOOLEAN : TheStateIsSaved(j, fires)
       \/ TheOptionsRunFollows(j)
       \/ TheJobSkipsTheOptionsRun(j)
       \/ TheOptionsRunStarts(j)
       \/ \E ok \in BOOLEAN : TheOptionsCallEnds(j, ok)
       \/ TheOptionsAreSaved(j)
       \/ TheFailureLineIsSaved(j)
       \/ TheJobEnds(j)
  \/ \E r \in RunIds :
       \/ \E ok \in BOOLEAN : TheRunAccepts(r, ok)
       \/ TheRunSeesItsJobEnd(r)
       \/ TheRunEnds(r)
  \/ Rest

Spec == Init /\ [][Next]_vars

-----------------------------------------------------------------------------
\* The rules. Each one quotes the spec scenario it checks; SKILL.md has the list.

Live(j) == job[j].pc \notin {"idle", "done"}

\* actions.md 3.5: "one Action is accepted and the other is refused with 409".
OneJob ==
  \A g \in AllGames : Cardinality({j \in JobIds : Live(j) /\ job[j].g = g /\ j \notin cut}) <= 1

\* screen.md 45.17: "Given a Job holds the Game ... Then each response status is 409".
BusyRefusesStory == ~storyBusy

\* dashboard.md 39.5: "the story log holds no Narration of the stopped Job";
\* reset.md 7.3: "no Input and no Narration of the Job the reset ended";
\* dashboard.md 39.4: "the Narration, with no room on the Narration".
StoppedJobWritesNothing == ~late

\* dashboard.md 39.4: "And the status display answers with no failure line".
StopNoFailureLine == \A g \in exists : failLine[g] \notin cut

\* options.md 24.5: "When the client POST /status/reset-busy Then the dock renders set A".
StopClearsOptionsFailure == \A g \in exists : optFail[g] \notin cut

\* dashboard.md 39.5: "the story log holds the Input and the Narration of the second Job".
NextJobKeepsHolder ==
  \A j \in JobIds : Live(j) /\ j \notin cut => holder[job[j].g].job = j

\* autoplay.md intro: "The run holds its Game from its start to its end, also between two Jobs."
\* autoplay.md 41.17: "the status display answers with the progress of run B".
RunHoldsItsGame ==
  \A r \in RunIds :
    run[r].pc \in {"accept", "job"} /\ r \notin cutRuns =>
      holder[run[r].g].kind = "run" /\ holder[run[r].g].run = r

\* autoplay.md 41.6 and 41.7: "And no further Job of the run starts".
StoppedRunStartsNoJob == ~badAccept

\* autoplay.md 41.12: "the config writes are refused and the config stays".
RunRefusesConfig == ~cfgRun

\* autoplay.md intro: "A run ends for one of these reasons: ... A failed Options run
\* and a fallback of the Quantifier do not stop it."
RunEndReasons ==
  \A r \in RunIds : run[r].pc = "over" =>
    /\ run[r].end = "stopped" => r \in cutRuns
    /\ run[r].end = "jobFailed" => job[run[r].job].lineBy = "narrator"

\* autoplay.md intro: "The **budget** is the number of cycles".
BudgetBound == \A r \in RunIds : run[r].done <= Budget

\* autoplay.md 41.4: "the status display answers with the failure line of that Job
\* And the Game is not busy"; 41.23: "the status display answers with the failure line
\* ... And the Game is not busy".
RunFailureLineStays ==
  \A r \in RunIds :
    run[r].pc = "over" /\ r \notin cutRuns =>
      /\ run[r].end = "jobFailed" => run[r].endLine = run[r].job
      /\ run[r].end = "notAccepted" => run[r].endLine = -1

\* autoplay.md 41.1 "And the Game is idle"; 41.23 "And the Game is not busy".
RunEndsIdle ==
  \A r \in RunIds : run[r].pc = "over" /\ r \notin cutRuns => run[r].endIdle

\* games.md 19.2: "Deleting the active game returns 400".
DeleteNotActive == ~deletedActive

\* dashboard.md 39.4 and autoplay.md 41.6: "Then the stop answers "reset" And the Game is idle".
StopLeavesIdle ==
  [][\A g \in AllGames : YouPressStop(g) => holder'[g].kind = "none"]_vars

====
