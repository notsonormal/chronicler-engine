# GateSpecOnly

TLC reached more states than the chart budget of 500, so the chart draws only the bug paths: 4 states and 3 arrows.

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

## Rules not checked
R01_ALateSaveLandsNowhere, R02_ARunningJobKeepsItsEntry, R03_ARunHoldsTheGameUntilItEnds, R06_ACutSwipeKeepsItsPreAnalysisState, R07_TheDockHoldsNoOptionsFailureAfterAStop, R08_TheFailureLineIsOfTheLastJob, R09_TheEntryShowsTheStepOfItsOwnJob, R10_ARunEndsOnAFailureOfItsOwnJobOnly, R11_TheCycleInProgressIsInTheBudget, R12_NoStoryWriteWhileBusy, R13_NoConfigWriteUnderARun, R14_ASwipeStateIsWrittenAtMostTwice, R15_EveryGameIsIdleAfterAStart, NoDeadlock
After the first break, each TLC run stops at 100,000 distinct states. A run reached that bound before it checked these rules in every state.
