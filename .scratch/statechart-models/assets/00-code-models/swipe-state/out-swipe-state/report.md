# SwipeState

TLC reached 10 states and 24 steps. The chart draws 7 states and 12 arrows.

## ANewTextIsShownWithACheckedState breaks
The shortest path has 3 steps:
0. Idle, the text keeps its start state
1. you send an action: The narrator writes
2. the text is saved with the pre-analysis state: The text shows, its state is not read yet
3. the server restarts: Idle, the text keeps a state the rules never read

## Rules that hold
TypeOK, ASwipeStateIsWrittenAtMostTwice, NoDeadlock
