# Ticket 03: review sketches

Review aids for ticket 06, one for each machine of `03-dispositions.md` whose disposition is not "leave it". Each sketch shows the code of the working tree (HEAD `8eb0b057`). A sketch is not a doc, and it never goes into `docs/`.

Marks on the arrows:

- `[+]` an arrow that the disposition adds, or a fix that the disposition checks.
- `[-]` an arrow that the disposition removes.
- `[?]` an arrow that a behaviour question decides. Today's code has the arrow.

## B1: the status read (TLA+ model, extend `verify-gate`)

The answer of one `GET` status read, as two steps today. The Game starts with a failure line and no holder.

```mermaid
stateDiagram-v2
  [*] --> ReadBusy: status read starts
  ReadBusy --> AnswerBusy: holder seen
  ReadBusy --> BetweenReads: no holder seen
  BetweenReads --> ReadLine: no accept between the reads
  BetweenReads --> ReadLineAfterAccept: accept lands (clears the line, inserts the holder)
  ReadLine --> AnswerFailed: line seen
  ReadLineAfterAccept --> AnswerIdle: no line seen, but the Game is busy [-]
  ReadBusy --> OneHoldRead: [+] busy value and line read in one hold
  OneHoldRead --> AnswerBusy: [+]
  OneHoldRead --> AnswerFailed: [+]
  OneHoldRead --> AnswerIdle: [+] only with no holder and no line
  AnswerBusy --> [*]
  AnswerFailed --> [*]
  AnswerIdle --> [*]
```

N2 (restart) is outside this sketch: after a restart the read answers `Idle` for a Game whose Job never ended `[?]`.

## B2: the dock record (model-based test)

States of the dock record of one Game. "Set" is a non-empty offered set. "Fail" is an Options failure.

```mermaid
stateDiagram-v2
  state "Empty, no failure" as E
  state "Set, no failure" as S
  state "Set, Fail" as SF
  state "Empty, Fail" as EF

  [*] --> E: create (gate.rs:602)
  E --> S: Options saved (gate.rs:839)
  S --> S: Options saved
  SF --> S: Options saved
  EF --> S: Options saved
  S --> SF: Options failed, keeps the set (gate.rs:844)
  E --> EF: Options failed
  S --> E: Narration text saved (gate.rs:872, :893)
  SF --> E: Narration text saved
  EF --> E: Narration text saved
  SF --> S: stop keeps the set, clears the failure (gate.rs:489-499)
  EF --> E: stop
  S --> E: swipe switch or delete (gate.rs:527, :564)
  SF --> E: swipe switch or delete
  EF --> E: swipe switch or delete
  SF --> SF: edit keeps the failure [?] Q1
  EF --> EF: edit keeps the failure [?] Q1
  SF --> SF: accept keeps the failure [?] Q13 (ticket 09)
  EF --> EF: accept keeps the failure [?] Q13 (ticket 09)
  S --> S: Input saved, its Narration fails: the old set stays [?] N1
```

The test adds no arrow. It checks each arrow above against the real code, through generated sequences. `[-]` for the dead write: `Tx::clear_dock` (`transaction.rs:321`) is an arrow "any --> E" that no code takes, and W2 removes it.

## B4: the encounters of one Character, with the B3 fire mark (model-based test)

`times_met` is `n`. "Marked" is the fire mark of one one-time Trigger.

```mermaid
stateDiagram-v2
  state "Absent, n" as A
  state "Present, n" as P

  [*] --> P: scenario start, n = 1 (swipe_state.rs:64-80)
  [*] --> A: not in the start scene
  A --> P: Entered, n + 1 (game_rules.rs:147-155)
  P --> A: Left (game_rules.rs:156-160)
  A --> P: Entered during an event generation, n + 1 [?] bug 03
  P --> P: Trigger matches, event text saved: Marked (generation.rs:118-125)
  P --> P: Trigger matches, event generation fails: not Marked, fires again [?] Q3
  P --> P: currently_meeting set (game_rules.rs:118, :153) [-] no reader
```

Bug 03 decides whether `n` counts every entry or the first only. The test adds no arrow.

## B10: Role health (explicit in the code)

The health that the banner reads for one Role.

```mermaid
stateDiagram-v2
  state "No calls yet" as N
  state "Healthy" as H
  state "Degraded or Unreachable" as D

  [*] --> N
  N --> H: call ok, row saved
  N --> D: call failed, row saved
  H --> D: call failed, row saved
  D --> H: call ok, row saved
  H --> N: 50 newer rows of other Roles prune the last row [-]
  D --> N: 50 newer rows of other Roles prune the last row [-]
  H --> H: empty Narration saved as a success, Job fails EmptyText [?] Q11
  H --> H: call failed, row not saved, only a warning [?] Q12
```

The two `[-]` arrows go when the prune keeps the newest row of each Role (W4, if Q6 keeps `dashboard.md:171`).

## B11: the per-Game change numbers (explicit in the code)

The per-Game entry of one Game id in `ChangeCounters`.

```mermaid
stateDiagram-v2
  state "No entry" as N
  state "Entry" as E
  state "Forgotten" as F

  [*] --> N
  N --> E: first raise of the Game
  E --> E: raise
  E --> F: delete or reset, forget_game (gate.rs:584, :592)
  F --> E: refused accept with a stale id writes the line on the missing row (bug 11) [-]
```

After bug 11, `Tx::set_game_failure` fails on a missing row, so the transaction raises nothing and "Forgotten" has no way out.

## B12: the preset selection against a mode switch (TLA+ model, extend `verify-gate`)

One `set_preset_selection` of a Novel-only preset and one `switch_mode` to Interactive Fiction, on an idle Game in Novel mode.

```mermaid
stateDiagram-v2
  state "Novel, bundle preset" as NB
  state "Check read Novel" as C
  state "IF, IF bundle preset" as IB
  state "IF, Novel-only preset" as IN
  state "Novel, chosen preset" as NC

  [*] --> NB
  NB --> C: selection reads the Game and checks the mode (catalogue.rs:192-215)
  C --> NC: selection writes first
  NC --> IB: then the switch writes the IF bundle
  C --> IB: the switch writes between the check and the write
  IB --> IN: the selection writes with its old check [-]
  IB --> IB: [+] the check inside the save_config hold refuses the selection
  NB --> IB: switch first
  IB --> IB: then the selection is refused by the check
```

The `[-]` arrow is the race. The `[+]` arrow is the fix that the TLA+ model checks.
