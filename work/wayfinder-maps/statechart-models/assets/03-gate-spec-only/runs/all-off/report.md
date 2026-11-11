# GateSpecOnly

TLC reached 2,271,206 states, more than the chart budget of 500, so the command wrote no chart.mmd.

## Rules that hold
R11_TheCycleInProgressIsInTheBudget, R12_NoStoryWriteWhileBusy, R13_NoConfigWriteUnderARun, R14_ASwipeStateIsWrittenAtMostTwice, R15_EveryGameIsIdleAfterAStart, NoDeadlock
