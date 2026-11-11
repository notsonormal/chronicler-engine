# The rules of the gate, from docs/specs/ only

Written 2026-11-05 20:42 UTC, before any read of `src/` in this session. Each rule quotes one spec sentence.

| Id | Rule (plain) | Quoted spec sentence | Scenario |
|---|---|---|---|
| OneJob | A Game has at most one accepted Job that is not stopped. | "one Action is accepted and the other is refused with 409" | actions.md 3.5 |
| BusyRefusesStory | No story write (a new Job, an edit, a delete, a swipe switch) lands while a Job or a run holds the Game, except the run's own next Job. | "Then each response status is 409" (Given a Job holds the Game) | screen.md 45.17; actions.md 3.4; story_log.md 8.7; swipe_switch.md 36.3; autoplay.md 41.11 |
| StopIdleAtOnce | After a stop answers, the Game is idle, before the stopped model call ends. | "Then the stop answers "reset" And the Game is idle" | dashboard.md 39.4; autoplay.md 41.6 |
| StopNoFailureLine | After a stop, the status display shows no failure line of the stopped Job. | "And the status display answers with no failure line" | dashboard.md 39.4 |
| StoppedJobWritesNothing | A stopped Job writes no record after the stop. | "the story log holds no Narration of the stopped Job" | dashboard.md 39.5, 39.4 |
| StoppedSwipePreAnalysis | A stop at the analysis call leaves the Narration with no room (its pre-analysis state). | "the Input of the Job and the Narration, with no room on the Narration and no event Narration" | dashboard.md 39.4 |
| NextJobKeepsHolder | The late end of a stopped Job does not end the next Job of the Game: the next Job saves its own text. | "the story log holds the Input and the Narration of the second Job" | dashboard.md 39.5 |
| RunHoldsBetweenJobs | A run that is not stopped holds its Game from its start to its end, also between two Jobs. | "The run holds its Game from its start to its end, also between two Jobs." | autoplay.md intro; 41.11; 41.13 |
| StoppedRunStartsNoJob | A stopped, reset or deleted run starts no further Job. | "And no further Job of the run starts" | autoplay.md 41.6, 41.7, 41.15, 41.16 |
| LateRunEndKeepsNewRun | The end of a stopped run A does not release a run B that started after it. | "Then the status display answers with the progress of run B" | autoplay.md 41.17 |
| RunRefusesConfig | A config write lands only while no run holds the Game. | "the config writes are refused and the config stays" | autoplay.md 41.12; screen.md 45.17 |
| RunEndReasons | A run ends only for a stop, a reset or a delete, the budget, a Job that ends with a failure line, or an accept that cannot read a record. A failed Options run does not end it. | "A run ends for one of these reasons: ... A failed Options run and a fallback of the Quantifier do not stop it." | autoplay.md intro; 41.5; 41.24 |
| BudgetBound | A run plays at most its budget of cycles. | "The **budget** is the number of cycles" | autoplay.md intro; 41.1 |
| RunFailureLineStays | After a run ends on a failed Job, the Game is not busy and shows the failure line of that Job. | "Then the status display answers with the failure line of that Job And the Game is not busy" | autoplay.md 41.4 |
| ResetJobWritesNothing | The Job of a reset Game writes nothing to any Game after the reset, and the new Game is idle. | "the new Game's story log holds no Input and no Narration of the Job the reset ended" | reset.md 7.3; autoplay.md 41.15 |
| JobOwnGame | Every write of a Job goes to the Game its request named. | "Game A's story log holds its own Narration and no Narration of Game B" | games.md 18.4; autoplay.md 41.14 |
| FailedStopKeepsJob | A stop that fails leaves the Job running, and its save lands. | "And the status display still answers with the step of the Job ... Then the Job saves its Narration and the Game is idle" | dashboard.md 39.8 |
| AcceptReadFailIdle | An accept that cannot read a record makes no model call and leaves the Game not busy. | "And the Game is not busy And no model call is made" | autoplay.md 41.23 |
| StopClearsOptionsFailure | A stop clears the Options failure of the dock. | "When the client POST /status/reset-busy Then the dock renders set A" | options.md 24.5, 24.18 |
| JobEnds | A Job always ends: the Game is then idle or shows a failure line. | "**I.1** A Job always ends" | actions.md I.1 |
| ConfigPassesInJob | A config write passes while a plain Job runs (a branch, not an invariant). | "When the client sets the Game's system preset to a second preset" | actions.md 6.4 |
| DeleteNotActive | A delete of the active Game is refused. | "Deleting the active game returns 400" | games.md 19.2 |
