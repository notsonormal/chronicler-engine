---- MODULE SwipeState ----
\* The state of one new Swipe, as docs/diataxis/reference/game_flow.md describes it:
\* the text save writes the pre-analysis state, the state update writes the analysed
\* one, a Retry of a typed Input keeps the start state, and a stop or a restart
\* between the two writes leaves the pre-analysis state.
\*
\* The model follows src/domain/model/generation.rs (GenerationOrder::write) and
\* src/application/pipeline/job.rs. It leaves out the rooms, the Characters and the
\* event record: it is about how many times a Swipe's state is written, and when.
EXTENDS Naturals

VARIABLES stage, request, current, writes

vars == <<stage, request, current, writes>>

TypeOK ==
  /\ stage \in {"idle", "writing", "saved", "analysed"}
  /\ request \in {"none", "an action", "a retry of a typed input"}
  /\ current \in {"start", "pre-analysis", "analysed"}
  /\ writes \in 0..2

Init ==
  /\ stage = "idle"
  /\ request = "none"
  /\ current = "start"
  /\ writes = 0

\* The player sends an Action: the Job starts and writes the Input before the narrator.
YouSendAnAction ==
  /\ stage = "idle"
  /\ stage' = "writing"
  /\ request' = "an action"
  /\ writes' = 0
  /\ current' = "start"

\* A Retry of a typed Input writes a new Swipe that keeps the start state.
YouRetryATypedInput ==
  /\ stage = "idle"
  /\ stage' = "writing"
  /\ request' = "a retry of a typed input"
  /\ writes' = 0
  /\ current' = "start"

\* The text save: the story shows the text with the pre-analysis state.
TheTextIsSaved ==
  /\ stage = "writing"
  /\ request = "an action"
  /\ stage' = "saved"
  /\ current' = "pre-analysis"
  /\ writes' = writes + 1
  /\ UNCHANGED request

TheTextOfARetryIsSaved ==
  /\ stage = "writing"
  /\ request = "a retry of a typed input"
  /\ stage' = "idle"
  /\ current' = "start"
  /\ writes' = writes + 1
  /\ UNCHANGED request

\* The state update: the same Swipe takes the state that the game rules read.
TheStateUpdateLands ==
  /\ stage = "saved"
  /\ stage' = "analysed"
  /\ current' = "analysed"
  /\ writes' = writes + 1
  /\ UNCHANGED request

TheJobEnds ==
  /\ stage = "analysed"
  /\ stage' = "idle"
  /\ UNCHANGED <<request, current, writes>>

\* The player presses Stop: the holder goes, and the Job writes no further state.
YouPressStop ==
  /\ stage # "idle"
  /\ stage' = "idle"
  /\ UNCHANGED <<request, current, writes>>

\* A restart loses every holder, so a Job between its two writes writes nothing more.
TheServerRestarts ==
  /\ stage # "idle"
  /\ stage' = "idle"
  /\ UNCHANGED <<request, current, writes>>

Next ==
  \/ YouSendAnAction
  \/ YouRetryATypedInput
  \/ TheTextIsSaved
  \/ TheTextOfARetryIsSaved
  \/ TheStateUpdateLands
  \/ TheJobEnds
  \/ YouPressStop
  \/ TheServerRestarts

Spec == Init /\ [][Next]_vars

\* No Game rests with a text whose state the game rules have not read.
\* The path that breaks this is the stop and the restart between the two writes.
ANewTextIsShownWithACheckedState == stage = "idle" => current # "pre-analysis"

\* A Swipe's state is written at most twice: the pre-analysis one, then the analysed one.
ASwipeStateIsWrittenAtMostTwice == writes <= 2

====
