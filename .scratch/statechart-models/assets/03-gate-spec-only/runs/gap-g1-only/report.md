# GateSpecOnly

TLC reached more states than the chart budget of 500, so the chart draws only the bug paths: 6 states and 6 arrows.

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

## Rules not checked
R01_ALateSaveLandsNowhere, R04_AnEndedRunStartsNoJob, R05_TheGameStaysIdleAfterAStop, R06_ACutSwipeKeepsItsPreAnalysisState, R07_TheDockHoldsNoOptionsFailureAfterAStop, R08_TheFailureLineIsOfTheLastJob, R09_TheEntryShowsTheStepOfItsOwnJob, R10_ARunEndsOnAFailureOfItsOwnJobOnly, R11_TheCycleInProgressIsInTheBudget, R12_NoStoryWriteWhileBusy, R13_NoConfigWriteUnderARun, R14_ASwipeStateIsWrittenAtMostTwice, R15_EveryGameIsIdleAfterAStart, NoDeadlock
After the first break, each TLC run stops at 100,000 distinct states. A run reached that bound before it checked these rules in every state.
