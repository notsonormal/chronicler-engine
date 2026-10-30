# GateSpecOnly

TLC reached more states than the chart budget of 500, so the chart draws only the bug paths: 5 states and 5 arrows.

## R02_ARunningJobKeepsItsEntry breaks
The shortest path has 5 steps:
0. Idle
1. you send an Action: A Job holds the Game
2. you press Stop: Idle; a stopped Job still runs
3. you send an Action: A Job holds the Game; a stopped Job still runs
4. a save of the stopped Job is refused: A Job holds the Game; a stopped Job still runs
5. the stopped Job ends: Idle; a Job runs with no entry

## Rules not checked
R01_ALateSaveLandsNowhere, R03_ARunHoldsTheGameUntilItEnds, R04_AnEndedRunStartsNoJob, R05_TheGameStaysIdleAfterAStop, R06_ACutSwipeKeepsItsPreAnalysisState, R07_TheDockHoldsNoOptionsFailureAfterAStop, R08_TheFailureLineIsOfTheLastJob, R09_TheEntryShowsTheStepOfItsOwnJob, R10_ARunEndsOnAFailureOfItsOwnJobOnly, R11_TheCycleInProgressIsInTheBudget, R12_NoStoryWriteWhileBusy, R13_NoConfigWriteUnderARun, R14_ASwipeStateIsWrittenAtMostTwice, R15_EveryGameIsIdleAfterAStart, NoDeadlock
After the first break, each TLC run stops at 100,000 distinct states. A run reached that bound before it checked these rules in every state.
