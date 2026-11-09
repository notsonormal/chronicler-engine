# Ticket 05: review sketches of the dashboard machines

These sketches are review aids for the owner (ticket 06). They are drawn from the code as it is today. They are not docs and never go into `docs/`.

How to read them:

- A plain arrow exists in the code today. The label names the cause and the file:line.
- An arrow with the label `ADDED` is one that a disposition adds. An arrow with the label `REMOVED` is one that a disposition removes.
- A red state is a state that is not valid and that a known path reaches. Only G1 and G4 have one.
- The dispositions of each machine are in `05-dispositions.md`. All 15 machines earn at least one, so every machine has a sketch.
- Where the machine has derived states, the sketch shows the derived state and not the variables.

Each state name is a short word. Each label ends with the file name and line of the code that makes the move. `aa` is `dashboard-action-area.js`, `sc` is `dashboard-screen.js`, `sl` is `dashboard-story-log.js`, `dl` is `dashboard-dialog.js`, `er` is `dashboard-errors.js`, `sv` is `dashboard-saves.js`, `pn` is `dashboard-panels.js`, `sm` is `dashboard-slash-menu.js`.

## G1 action area

The state is derived in this order: checking, autoplay, generating, preview, idle (`aa:64-70`).

```mermaid
stateDiagram-v2
    classDef invalid fill:#fcc,stroke:#c00,color:#900
    [*] --> Idle
    Idle --> Checking: command form before-request, aa 209
    Checking --> Idle: after-request or load error, aa 266 and aa 271
    Checking --> Preview: check answer opens a preview, aa 274
    Preview --> Idle: preview closed, aa 312
    Preview --> Generating: preview send accepted, aa 295
    Idle --> Generating: send done, aa 276
    Idle --> Generating: Retry Retrigger New options before-request, aa 196
    Idle --> Generating: Autoplay start done, aa 302
    Generating --> Idle: screen shows the stamp of the answer, aa 25
    Generating --> Idle: answer of any request is not done, aa 309
    Generating --> Autoplay: status display shows autoplay, aa 66
    Autoplay --> Idle: status display no longer shows autoplay
    Generating --> IdleWhileWaiting: second request refused with 409, aa 309
    state "Idle while the first send waits" as IdleWhileWaiting
    IdleWhileWaiting --> Idle: screen shows the first stamp, aa 32
    class IdleWhileWaiting invalid
    Generating --> Generating: ADDED, refused answer of a second request ends only its own pending count
    Generating --> IdleWhileWaiting: REMOVED, refused answer ends the pending state of the first send
```

## G2 screen stamp and waiters

```mermaid
stateDiagram-v2
    [*] --> Unknown
    Unknown --> Shown: a load shows a stamp, sc 303
    Shown --> Unknown: end of a request that is not a read, sc 97
    Shown --> Unknown: tab change, sc 407
    Shown --> Unknown: pager click, sc 396
    Shown --> Unknown: edit ends, sl 109
    Shown --> Shown: a later load shows a newer stamp, sc 303
    state "A waiter waits for stamp S" as Waiting
    [*] --> Waiting: afterScreenShows S, sc 193
    Waiting --> Done: shown server equals S server and revision at least S, sc 209
    Waiting --> Done: load started after the wait shows another server, sc 205-209
    Waiting --> Waiting: any other load, sc 210
```

## G3 ask and load loop

```mermaid
stateDiagram-v2
    [*] --> Quiet
    Quiet --> Asking: askNow, sc 351-357
    Asking --> Loading: engine stamp differs from the page stamp, sc 333-337
    Asking --> Quiet: stamp equal or ask failed, sc 333-335
    Loading --> Quiet: load shown or dropped, sc 274-279
    Asking --> AskQueued: askNow while an ask runs, sc 352-368
    state "One ask waits in the chain" as AskQueued
    AskQueued --> Asking: the running ask ends, sc 356-358
    Quiet --> Asking: poll tick every data-poll-ms, sc 373
    Quiet --> Asking: page becomes visible, sc 386
```

## G4 what a load keeps

```mermaid
stateDiagram-v2
    classDef invalid fill:#fcc,stroke:#c00,color:#900
    [*] --> Free
    Free --> Held: htmx beforeRequest on the node, sc 84-87
    Held --> Held: a second request of the same node, sc 75
    Held --> Free: htmx afterRequest or onLoadError reaches the body, sc 93-102
    Held --> HeldDetached: script detaches the node, dl 89 and sc 404
    state "Held, node detached, answer lost" as HeldDetached
    class HeldDetached invalid
    HeldDetached --> HeldDetached: afterRequest does not reach the body, known by run
    HeldDetached --> Free: ADDED, release by the xhr of the request
    Held --> Kept: a load meets the node, sc 114-124
    state "The load skips the node" as Kept
    Kept --> Held: the load ends
    Free --> Kept: entry carries data-editing, sc 110
```

## G5 the Game a request names

```mermaid
stateDiagram-v2
    [*] --> NoDraft
    NoDraft --> Pinned: first keystroke, aa 234-236
    Pinned --> NoDraft: input empty, aa 234
    Pinned --> NoDraft: send done and the screen shows it, aa 225
    Pinned --> Repinned: 409 refusal and the screen shows the answer, aa 250-256
    Repinned --> Pinned: pin takes the Game of the screen
    NoDraft --> Pinned: Option or Edit option starts a new draft, aa 241-245
    state "Entry in edit pinned" as EditPinned
    NoDraft --> EditPinned: showEditForm, sl 95
    EditPinned --> NoDraft: clearEditState, sl 106
    Pinned --> RefusedAtServer: engine switched, send names the old Game, request_layers.rs line 39-56
    state "Request refused 409" as RefusedAtServer
    RefusedAtServer --> Repinned
```

## G6 story-log edit

```mermaid
stateDiagram-v2
    [*] --> Closed
    Closed --> Editing: Edit click, sl 59-99 sets editState, data-editing, lock
    Editing --> Closed: Cancel or Escape, sl 124-131
    Editing --> Closed: save failed other than 409 and 4xx, sl 139-141
    Editing --> Editing: save refused 409 or invalid, sl 138
    Editing --> SavedLocked: save done, sl 143
    state "Saved, lock stays until the screen shows the entry" as SavedLocked
    SavedLocked --> Closed: screen shows the stamp, sl 144
    Editing --> Closed: tab change, screen cleared, sl 243-246
    SavedLocked --> Closed: tab change, screen cleared, sl 243-246
```

## G7 story-log scroll follow

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Armed: screen beforemorph on story-log, sl 223-230
    Armed --> Idle: screen loaded and edit is open, sl 237
    Armed --> Idle: log was not at the bottom, sl 238
    Armed --> Idle: no new entry, sl 239
    Armed --> Following: new entry and log was at the bottom, sl 240
    Following --> Idle: log can scroll, scrolled to the bottom, sl 214-216
    Following --> Idle: edit opens, sl 213
    Following --> Idle: one second passes, sl 218
```

## G8 edit dialog session

```mermaid
stateDiagram-v2
    [*] --> Closed
    Closed --> Open: swap into the dialog body, dl 31-38
    Open --> Closed: Cancel or Escape, dl 95-100 and the close event dl 87-93
    Open --> Saving: form in the dialog answers done, dl 115-122
    Saving --> Closed: screen shows the stamp and the session is the same, dl 123-125
    Saving --> Closed: Cancel or Escape, session changes, dl 91
    Saving --> Saving: second submit blocked, dl 107-113
    Open --> Open: save refused or failed, the form stays, dl 119
    Open --> Open: swap from a control outside the dialog is dropped, dl 43-48
    Open --> LostAnswer: Escape while a save is in flight, dl 87-93
    state "Closed, answer of the save never read" as LostAnswer
    LostAnswer --> Closed: ADDED, the answer reaches one reader (question 4)
```

## G9 notice place

```mermaid
stateDiagram-v2
    [*] --> None
    None --> RequestReport: failed request, er 166
    None --> ScreenReport: failed ask or load, sc 215
    RequestReport --> None: any request starts, er 171-172
    RequestReport --> None: Close, er 176-182
    RequestReport --> ScreenReport: screen failure replaces it, er 131-139
    ScreenReport --> RequestReport: request failure replaces it, er 166
    ScreenReport --> None: good load, sc 242-245
    ScreenReport --> None: any request starts or Close, er 171 and er 176
    RequestReport --> RequestReport: the same report arrives, er 117-134
```

## G10 failure banner

```mermaid
stateDiagram-v2
    [*] --> Hidden
    Hidden --> Degraded: part carries a degraded line, screen loaded, er 184
    Hidden --> Unreachable: no answer or timeout, er 64-76
    Degraded --> Both: no answer or timeout, er 64-76
    Unreachable --> Both: part carries a degraded line, er 184
    Unreachable --> Hidden: any answer, er 78-83
    Both --> Degraded: any answer, er 78-83
    Degraded --> Hidden: the degraded line clears, er 184
```

## G11 Saved flag

```mermaid
stateDiagram-v2
    [*] --> NoFlag
    NoFlag --> Waiting: save done, fields settle, sv 61-62
    Waiting --> Saved: screen shows the stamp, sv 63
    Saved --> NoFlag: a later save fails, sv 66
    NoFlag --> NoFlag: save fails, field returns to the rendered value, sv 66-68
    Saved --> NoFlag: Update World request starts, sv 75-78
    Saved --> Waiting: the next save is done
```

## G12 tab, Settings sub-tab, saved-Games page

```mermaid
stateDiagram-v2
    [*] --> GameTab
    GameTab --> OtherTab: tab click, pn 9-25 leaves the section, sc 400-409
    OtherTab --> GameTab: tab click
    OtherTab --> OtherTab: tab click, the old load answer is dropped, sc 274
    state "Settings sub-tab" as Sub
    OtherTab --> Sub: click on a sub-tab, pn 98-105
    Sub --> Sub: load restores the remembered sub-tab, pn 47-63
    state "Games page N" as Page
    OtherTab --> Page: Previous or Next, sc 391-398
    Page --> Page: load asks for page N, engine clamps it, sc 251-259
```

## G13 slash menu

```mermaid
stateDiagram-v2
    [*] --> Closed
    Closed --> Open: input starts with a slash and matches, sm 113-118
    Open --> Open: ArrowDown or ArrowUp moves the active item, sm 124-133
    Open --> Open: input changes, matches are computed again, sm 87-91
    Open --> Closed: no match, sm 44-47
    Open --> Closed: Enter on an item or a click selects it, sm 93-99
    Open --> Closed: Escape, Enter with no item, submit, sm 139-143 and sm 175
    Open --> Closed: click elsewhere, scroll, resize, sm 158 and sm 178-179
    Open --> Closed: focus leaves the input for 150 ms, sm 164-173
```

## G14 error detail popover

```mermaid
stateDiagram-v2
    [*] --> Hidden
    Hidden --> Shown: Details click, positioned, er 219-228
    Shown --> Hidden: Details click, er 229
    Shown --> Hidden: click outside, er 234
    Shown --> Hidden: Escape, er 239-244
    Shown --> Hidden: resize or scroll, er 246-247
    Shown --> Shown: a load keeps hidden and style by data-keep, sc 61-67
```

## G15 live-region announcements

Three memories. Each is shown in one block.

```mermaid
stateDiagram-v2
    [*] --> Unprimed
    Unprimed --> Primed: first load of the part records the memory, aa 147-149 and aa 179-181
    Primed --> Primed: new Narration or new Options are announced, aa 159 and aa 189
    Primed --> Primed: label changes and no failure line shows, aa 117-121
    Primed --> Unprimed: ADDED if question 5 says so, a Game switch resets the memories
    Primed --> Flood: Game switch, every Narration of the other Game is new, aa 151-168
    state "Announces the whole other story in one block" as Flood
    Flood --> Primed
    Primed --> Silent: failure line clears and the label returns to Ready, aa 114-119
    state "No announcement of Ready" as Silent
    Silent --> Primed
```
