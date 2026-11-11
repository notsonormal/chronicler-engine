# GateSpecOnly

TLC reached more states than the chart budget of 500, so the chart draws only the bug paths: 6 states and 8 arrows.

## R01_ALateSaveLandsNowhere breaks
The shortest path has 3 steps:
0. Idle
1. you send an Action: A Job holds the Game
2. you press Stop: Idle; a stopped Job still runs
3. the stopped Job saves its Input: Idle; a stopped Job still runs

## R09_TheEntryShowsTheStepOfItsOwnJob breaks
The shortest path has 5 steps:
0. Idle
1. you send an Action: A Job holds the Game
2. you press Stop: Idle; a stopped Job still runs
3. you send an Action: A Job holds the Game; a stopped Job still runs
4. the stopped Job saves its Input: A Job holds the Game; a stopped Job still runs
5. the stopped Job starts its next step: A Job holds the Game, narrating; a stopped Job still runs

## R05_TheGameStaysIdleAfterAStop breaks
The shortest path has 6 steps:
0. Idle
1. you send an Action: A Job holds the Game
2. you press Stop: Idle; a stopped Job still runs
3. the stopped Job saves its Input: Idle; a stopped Job still runs
4. the stopped Job starts its next step: Idle; a stopped Job still runs
5. the narrator call of the stopped Job fails: Idle; a stopped Job still runs
6. the stopped Job saves a failure line: Idle with a failure line; a stopped Job still runs

## R08_TheFailureLineIsOfTheLastJob breaks
The shortest path has 7 steps:
0. Idle
1. you send an Action: A Job holds the Game
2. you press Stop: Idle; a stopped Job still runs
3. you send an Action: A Job holds the Game; a stopped Job still runs
4. the stopped Job saves its Input: A Job holds the Game; a stopped Job still runs
5. the stopped Job starts its next step: A Job holds the Game, narrating; a stopped Job still runs
6. the narrator call of the stopped Job fails: A Job holds the Game, narrating; a stopped Job still runs
7. the stopped Job saves a failure line: A Job holds the Game, narrating; a stopped Job still runs

## R06_ACutSwipeKeepsItsPreAnalysisState breaks
The shortest path has 9 steps:
0. Idle
1. you send an Action: A Job holds the Game
2. you press Stop: Idle; a stopped Job still runs
3. the stopped Job saves its Input: Idle; a stopped Job still runs
4. the stopped Job starts its next step: Idle; a stopped Job still runs
5. the narrator answers the stopped Job: Idle; a stopped Job still runs
6. the stopped Job saves its text: Idle; a stopped Job still runs
7. the stopped Job starts its next step: Idle; a stopped Job still runs
8. the analysis answers the stopped Job: Idle; a stopped Job still runs
9. the stopped Job saves the new state: Idle; a stopped Job still runs

## R07_TheDockHoldsNoOptionsFailureAfterAStop breaks
The shortest path has 12 steps:
0. Idle
1. you send an Action: A Job holds the Game
2. you press Stop: Idle; a stopped Job still runs
3. the stopped Job saves its Input: Idle; a stopped Job still runs
4. the stopped Job starts its next step: Idle; a stopped Job still runs
5. the narrator answers the stopped Job: Idle; a stopped Job still runs
6. the stopped Job saves its text: Idle; a stopped Job still runs
7. the stopped Job starts its next step: Idle; a stopped Job still runs
8. the analysis answers the stopped Job: Idle; a stopped Job still runs
9. the stopped Job saves the new state: Idle; a stopped Job still runs
10. the stopped Job starts its next step: Idle; a stopped Job still runs
11. the Options call of the stopped Job fails: Idle; a stopped Job still runs
12. the stopped Job saves an Options failure: Idle; a stopped Job still runs

## Rules not checked
R02_ARunningJobKeepsItsEntry, R03_ARunHoldsTheGameUntilItEnds, R04_AnEndedRunStartsNoJob, R10_ARunEndsOnAFailureOfItsOwnJobOnly, R11_TheCycleInProgressIsInTheBudget, R12_NoStoryWriteWhileBusy, R13_NoConfigWriteUnderARun, R14_ASwipeStateIsWrittenAtMostTwice, R15_EveryGameIsIdleAfterAStart, NoDeadlock
After the first break, each TLC run stops at 100,000 distinct states. A run reached that bound before it checked these rules in every state.
