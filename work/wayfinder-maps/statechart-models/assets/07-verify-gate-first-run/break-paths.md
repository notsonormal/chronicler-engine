# Break paths of the verify-gate model (shortest TLC counterexample per mutant)

Model: .agents/skills/verify-gate/model/VerifyGate.tla. Action names map to code in .agents/skills/verify-gate/SKILL.md.
Each line: mutant (the code break it mirrors) -> rule that breaks -> the shortest order of steps.

| Rule | Mutant (code break) | Shortest break path |
|---|---|---|
| OneJob | accept-without-busy-check (GameWriter::accept with no require_idle) | YouStartAJob, YouStartAJob |
| BusyRefusesStory | edit-without-busy-check (GameWriter::edit with no require_idle) | YouStartAJob, YouEditAMessage |
| BusyRefusesStory | swipe-switch-without-busy-check (switch_swipe with no require_idle) | YouStartAutoplay, YouSwitchOrDeleteASwipe |
| StoppedJobWritesNothing | job-writer-without-id-check (JobWriter::hold checks only busy, not job id) | YouStartAJob, YouPressStop, AJobStartsAGeneration (the stopped Job saves its Input) |
| StoppedJobWritesNothing | reset-keeps-the-holder (GameWriter::replace does not remove the holder) | YouStartAJob, YouResetTheGame, AJobStartsAGeneration (Job writes to deleted Game) |
| StopNoFailureLine | stop-keeps-failure-line | YouStartAJob, AJobStartsAGeneration, TheNarratorFails, TheJobSkipsTheOptionsRun, TheFailureLineIsSaved, YouPressStop (stop between the failure-line save and the JobWriter drop) |
| StopClearsOptionsFailure | stop-keeps-options-failure | YouStartAJob(options), TheOptionsRunStarts, TheOptionsCallEnds(fail), TheOptionsAreSaved, YouPressStop (stop between the Options failure save and the drop) |
| StopLeavesIdle | stop-keeps-the-run (stop keeps a run holder) | YouStartAutoplay, YouPressStop |
| NextJobKeepsHolder | job-drop-without-id-check (Drop for JobWriter removes the holder with no id check) | YouStartAJob(options), TheOptionsRunStarts, TheOptionsCallEnds, TheOptionsAreSaved, YouPressStop, YouStartAJob, TheJobEnds (old Job's drop frees the new Job) |
| RunHoldsItsGame | job-drop-removes-the-run (Drop for JobWriter removes a run holder) | YouStartAutoplay, TheRunAccepts, AJobStartsAGeneration, TheNarratorFails, TheJobSkipsTheOptionsRun, TheFailureLineIsSaved, TheJobEnds |
| RunHoldsItsGame | run-end-without-run-check (end_autoplay with no run id check) | YouStartAutoplay, TheRunAccepts (not accepted: read fails, holder goes), YouStartAutoplay (run 2), TheRunEnds (run 1's end frees run 2) |
| StoppedRunStartsNoJob | accept-without-run-check (accept_for_autoplay checks only "a run holds it") | YouStartAutoplay, YouPressStop, YouStartAutoplay, TheRunAccepts (stopped run 1 accepts a Job on run 2's holder) |
| RunRefusesConfig | config-write-during-a-run | YouStartAutoplay, YouChangeTheConfig |
| RunEndReasons | moved-job-ends-as-failure (the loop maps JobEnd::EntryMoved to JobFailed) | YouStartAutoplay, YouPressStop, YouStartAutoplay, TheRunAccepts, YouPressStop, AJobStartsAGeneration, TheRunSeesItsJobEnd, TheRunEnds |
| BudgetBound | budget-checked-too-late | a run of 2 cycles that accepts a fifth Job |
| RunFailureLineStays | not-accepted-without-failure-line | YouStartAutoplay, TheRunAccepts (story read fails), TheRunEnds |
| RunEndsIdle | run-end-keeps-the-holder (Drop for AutoplayRun does not call end_autoplay) | YouStartAutoplay, TheRunAccepts, AJobStartsAGeneration, TheNarratorFails, TheJobSkipsTheOptionsRun, TheFailureLineIsSaved, TheJobEnds, TheRunSeesItsJobEnd, TheRunEnds |
| DeleteNotActive | none: breaks on the current code | YouResetTheGame (Game 1 -> new Game 3, pointer not moved yet) or plain start; YouDeleteAGame(2) passes its check while Game 1 is active; YouSwitchTheGame(2); TheDeleteRemovesTheGame(2) deletes the active Game |
