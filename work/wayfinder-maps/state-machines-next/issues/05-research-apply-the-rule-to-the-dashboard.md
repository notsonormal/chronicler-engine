# Apply the rule to the dashboard

Type: research
Status: claimed
Blocked by: 01, 02

## Question

Apply the rule of ticket 01 to each machine of the dashboard in the browser. For each machine, what is its disposition and the evidence for it? Which product behaviour questions must the owner answer before the disposition holds?

## Machines

Catalogue rows. Ticket 02 can add rows.

- **Refresh.** G1 the action area, G2 the screen stamp and its waiters, G3 the ask and load loop, G4 what a load keeps, G5 the Game that a request names.
- **View state.** G6 the story-log edit, G7 the story-log scroll follow, G8 the edit dialog session, G9 the notice place, G10 the failure banner, G11 the Saved flag of a control, G12 the tab, the Settings sub-tab and the saved-Games page, G13 the slash menu, G14 the error detail popover, G15 the live-region announcements.

The statechart-models map named "the refresh rules of the dashboard" as a candidate for a TLA+ model. This ticket decides it.

## Behaviour questions already known (charting)

1. While the engine is unreachable, "Thinking…", "Saving…" and the edit lock stay until a load succeeds. Keep this, or end them after a time with a message?
2. "Saved" stays next to a control until a later save fails. Must it go away after some time?
3. The Test button in the edit dialog becomes the control that gets the focus when the dialog closes. Is that a defect? **Refuted by ticket 02:** the Test button inside the Connection form posts into its own `next .connection-test-slot` (`src/adapters/driving/http/settings/templates/settings.rs:199-202`), not into `#edit-dialog-body`, so it never becomes the opener. The real opener is the `connection-test-open-{id}` button of the Connection card (`settings.rs:60`). No question is left.

## Behaviour questions added by ticket 02 (scan 2)

4. G4 what a load keeps, G8 the edit dialog. A script can detach a subtree while one of its requests is in flight: the dialog close handler empties `#edit-dialog-body` (`assets/dashboard-dialog.js:87-93`), and `leaveSection` empties a part (`assets/dashboard-screen.js:400-404`). htmx 1.9.10 fires `htmx:afterRequest` on the detached element, and the event does not reach `document.body`, where `endRequest` listens (`dashboard-screen.js:101-102`). So the `requestsInFlight` entry of that node stays (`:72-82`), and the answer is never read: a save that fails after the player closed the dialog shows no report. The doc says one piece of code reads every answer (`docs/diataxis/reference/frontend/dashboard.md:16`) and that a save sent before Escape still applies (`:118`). Must a failed save after a close show a notice? (Inferred from the htmx source; not run.)
5. G15 live-region announcements. The three memories (`assets/dashboard-action-area.js:110`, `:140`, `:171`) are never reset on a Game switch. After a switch, every Narration in the story window of the other Game has an id that was not seen, and `announceNewNarration` (`:142-160`) announces all of them in one block. The Options of the other Game are announced too (`announceOptions`, `:173-187`). What must a screen reader hear after a Game switch?

## Also check

The frontend scout reported four sentences of `docs/diataxis/reference/frontend/dashboard.md` that do not agree with the code (catalogue, last section). Check each one. A doc error that needs no decision goes to `work/issues/pending/`.

Scan 2 frontend (`assets/scan2-frontend.md`, *Doc versus code*) reported five more. Ticket 02 did not verify them. Check them in the same way:

- D1. The doc says a load skips an element that holds unsaved input. The keep rules skip only an entry with `data-editing` and a node with a request in flight (`dashboard-screen.js`, `loadKeeps`). A part form with typed text that is not sent yet is not skipped.
- D2. The doc says `data-revision` is forgotten at the end of a request that is not a read. The code also forgets it in `leaveSection`, in the Previous/Next click of the Games pager and in `clearEditState`.
- D3. The doc explains Stop in the Autoplay state through `data-state`. The code shows Stop through CSS `:has(#stop-btn)` (`styles.css:673-676`). The CSS reads no `data-state` value but `generating`.
- D4. The doc list of client state leaves out `sendPending` and `checkInFlight` as variables, `previewSentText`, `dialogSession`, the three live-region memories, `screenWaiters` and `requestsInFlight`.
- D5. The doc names Edit and the Swipe controls as locked while an entry is in edit. Delete and Retrigger are not locked: CSS only hides them on the entry in edit.

## To check (from scan 2)

Scan 2 labelled these inferred or unsure. Ticket 02 did not prove them.

- G1: `sendPending` and `checkInFlight` are not reset by `screen:cleared` or by a Game switch. After a tab change the action area can read Idle until the Game tab loads again (`scan2-frontend.md`, *Action area*).
- G1: the screen parts read the busy registry one by one (`src/adapters/driving/http/screen/handlers/parts.rs`), so one load can show Send enabled and a busy status display together (`scan2-driving.md`, findings 1 and 9).
- G5: a script write to `input.value` fires no `input` event. Every such write must call `syncCommandDraft` by hand (`scan2-frontend.md`, *Command draft pin*).
- G6: a deleted entry that is also in edit can stay as a ghost, because the keep rules keep a node in edit (`scan2-frontend.md`, *Story-log edit mode*).
- G9: any new request, also a background auto-save, clears a report that the player has not read (`scan2-driving.md`, finding 8). The doc allows it (`dashboard.md:158`); check that this is the wanted reading.

## Output

- `assets/05-dispositions.md`: the disposition table in the form that ticket 01 decides (one row for each machine, with the columns of its Answer), with the evidence for each machine.
- A review sketch of each machine whose disposition is not "leave it": a Mermaid state diagram in `assets/05-sketches.md`, drawn from the code as it is today. Mark each arrow that the disposition adds or removes. Ticket 06 shows the owner these sketches, so the owner can check each machine against intent. A sketch is a review aid, not a doc: it never goes into `docs/`.
- The behaviour questions: the three above, checked again, plus any new ones. Each one names its machine and its file:line.
- Each plain defect goes to `work/issues/pending/` as a ticket.
