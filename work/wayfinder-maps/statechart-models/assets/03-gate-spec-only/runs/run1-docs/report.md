# GateSpecOnly

TLC reached more states than the chart budget of 500, so the chart draws only the bug paths: 10 states and 14 arrows.

## R04_AnEndedRunStartsNoJob breaks
The shortest path has 3 steps:
0. Idle
1. you start Autoplay: A run holds the Game between Jobs
2. you press Stop: Idle; a stopped run's loop still runs
3. the run starts its next Job: A run holds the Game, its Job starts; a stopped run's loop still runs

## R05_TheGameStaysIdleAfterAStop breaks
The shortest path has 3 steps:
0. Idle
1. you start Autoplay: A run holds the Game between Jobs
2. you press Stop: Idle; a stopped run's loop still runs
3. the run starts its next Job: A run holds the Game, its Job starts; a stopped run's loop still runs

## R02_ARunningJobKeepsItsEntry breaks
The shortest path has 4 steps:
0. Idle
1. you start Autoplay: A run holds the Game between Jobs
2. you press Stop: Idle; a stopped run's loop still runs
3. you send an Action: A Job holds the Game; a stopped run's loop still runs
4. the run finds another holder and ends: Idle; a Job runs with no entry

## R03_ARunHoldsTheGameUntilItEnds breaks
The shortest path has 4 steps:
0. Idle
1. you start Autoplay: A run holds the Game between Jobs
2. you press Stop: Idle; a stopped run's loop still runs
3. you start Autoplay: A run holds the Game between Jobs; a stopped run's loop still runs
4. the run finds another holder and ends: Idle

## R10_ARunEndsOnAFailureOfItsOwnJobOnly breaks
The shortest path has 8 steps:
0. Idle
1. you send an Action: A Job holds the Game
2. the Job saves its Input: A Job holds the Game
3. the Job starts its next step: A Job holds the Game, narrating
4. the narrator call of the Job fails: A Job holds the Game, narrating
5. the Job saves a failure line: A Job holds the Game, narrating
6. the engine restarts: Idle with a failure line
7. you start Autoplay: A run holds the Game between Jobs
8. the run ends: Idle with a failure line

## Rules not checked
R01_ALateSaveLandsNowhere, R06_ACutSwipeKeepsItsPreAnalysisState, R07_TheDockHoldsNoOptionsFailureAfterAStop, R08_TheFailureLineIsOfTheLastJob, R09_TheEntryShowsTheStepOfItsOwnJob, R11_TheCycleInProgressIsInTheBudget, R12_NoStoryWriteWhileBusy, R13_NoConfigWriteUnderARun, R14_ASwipeStateIsWrittenAtMostTwice, R15_EveryGameIsIdleAfterAStart, NoDeadlock
After the first break, each TLC run stops at 100,000 distinct states. A run reached that bound before it checked these rules in every state.
