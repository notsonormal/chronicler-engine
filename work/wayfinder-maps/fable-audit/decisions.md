# Audit of 2026-11-10: decisions of the owner

This file records each answer that the owner gave in the grilling session on `assets/questions-for-the-owner.md`. A row is added when the answer is given. The grilling session ended on 2026-11-10 with D18, and D19 to D21 followed on the same day: the accepted work is five task tickets in `issues/`, and [the map](map.md) lists them.

- **Reports:** `assets/`. Start with `assets/questions-for-the-owner.md`.
- **Row numbers** in the column "Rows" are the rows of that file.
- **"Recommended"** is what the agent recommended. It is here so that a reader sees where the owner chose differently.

## Decisions

| # | Date | Question | Answer of the owner | Recommended | Rows |
|---|---|---|---|---|---|
| D1 | 2026-11-10 | What is a place that the map of the World does not have? | **No change.** The engine keeps the behaviour of today: the Quantifier records the nearest Room of the map, or adds a Room | An added Room with a name, found again by name | 12 |
| D2 | 2026-11-10 | What is the first release? | **No release is planned yet.** The release rows wait | A local install for one person | 22 |
| D3 | 2026-11-10 | When the status line says "Ready", what does it promise? | **No change.** The failure line of the Game, the notice of the dock and Role health stay as they are | "Ready" means that the last Job did all its work | 18 |
| D4 | 2026-11-10 | May the audit spend model credit? | **No credit.** The rows that need a trial stay as proposals | Three runs of the reference setup, to measure the noise | 27 |
| D5 | 2026-11-10 | Where do the decisions of this session go? | **This file, and tickets at the end.** The reports moved from `tmp/fable-audit/` to `assets/`. A resolved term goes to `CONTEXT.md` at once | The same | 30, 31 |
| D6 | 2026-11-10 | The API key: is the leak a defect of today? | **Close the leak, keep the listen address.** One route layer refuses a request from another site. The key of the environment is used only with the default address of its provider. The listen address stays `0.0.0.0` | The same | 4 |
| D7 | 2026-11-10 | Five fixes with no design choice | **All five accepted.** (a) A part of the screen that fails renders its own error, and the other parts render. (b) A form answers 400 for a value that it cannot read. (c) Story text is escaped where the history enters a prompt. (d) `maker` and `currently_meeting` go. (e) The four one-line fixes of the invariant audit (world-04, llm-07, llm-08, llm-04) go ahead outside the ticket order | The same | 1, 2, 3, 8, 9 |
| D8 | 2026-11-10 | Three cases where the player loses text and the screen says nothing | **(a) Yes:** a Narration that the model cut at its token limit gets a small line that says so. **(b) No:** a Narration that a hard stop lost gets no message after the restart. **(c) No:** the Games tab gets no mark for a Game whose Job still runs | Yes for (a) and (b), no for (c) | 6, 7, 10 |
| D9 | 2026-11-10 | Whose text is the opening Narration? | **A Narration like any other, with one guard.** Retry stays on the opening. The model gets the authored opening as its direction. The engine does not analyse the result, so the new Swipe keeps the starting state of the Scenario | No Retry on the opening | 5 |
| D10 | 2026-11-10 | What does "repeats" mean for a Trigger? | **Left open until a World ships a repeating Trigger.** `CONTEXT.md` gets no new sentence | Define it now | 17 |
| D11 | 2026-11-10 | The author edits a Character file after the first start. What must happen? | **No change now.** The owner's words: "Eventually we'll be able to change characters via the UI." A delete of the database is a stopgap that the owner expects nobody to use. So the boot does not load a changed data file, and a Character form in the dashboard is a later effort | The file wins at every boot | 19 |
| D12 | 2026-11-10 | Is the Redmist World meant to give the story a plot, or to be an open stage? | **Not decided.** The content rows wait | A plot | 14 |
| D13 | 2026-11-10 | Parts of the engine with no user | **Delete the fields that nothing reads:** `items` of a Room, `inventory` of a Character and of a Persona. `exits` of a Room and the Trigger operators stay. No second World now | The same | 26 |
| D14 | 2026-11-10 | Three small additions | **(b) Yes:** story counts in `scripts/trial.py compare`, from `assets/local-only/evidence/trial_truth_counts.py`. **(c) Yes:** one test that loads each shipped data file through its JSON schema and through its Rust type. **(a) No:** the Room name is not stored with the story entry | The same | 20, 28, 29 |
| D15 | 2026-11-10 | A clean of the build output deletes the database. Is that acceptable? | **Acceptable.** The database is a development file. It stays in the folder of the binary. No export of an edited prompt preset | A `--data-dir` flag | 25 |
| D16 | 2026-11-10 | Will the owner play one session by hand for the audit? | **No.** The reports keep the limit "measured in Autoplay only" | Yes, when the owner next plays | 32 |
| D17 | 2026-11-10 | The defects that needed no decision | **All go ahead.** Engine: Ctrl-C stops an Autoplay run. The story log renders no `javascript:` link and no remote picture from model text. The story log scrolls to a new Narration after 50 Messages. An edit of the last Narration clears the offered Options. The Autoplay status names the Impersonate step. `README.md` names the API key. Environment: the nine rows of section 3 of `assets/development-environment.md` | The same | — |
| D18 | 2026-11-10 | What form does the accepted work get? | **Five grouped task tickets** in `issues/`: security, engine fixes, story entries, checks, text. Each change has its failing test first | The same | 31 |
| D19 | 2026-11-10 | The findings that no question asked (N1 to N14 of `assets/findings-not-carried.md`) | **All go ahead**, as in D17. The owner accepted the group and did not read each row. Ticket 06 holds N1 to N12 and N14, and ticket 05 holds N13. Four parts are not in a ticket, because a closer look showed that each is not a local fix: the provider `mock` in the Connection form, a JSON mode for the Quantifier call, two map checks that belong to the invariant audit, and a map edit that removes the Room where a Game stands (decided later as D21). N15 to N18 stay in the list | The same | — |
| D20 | 2026-11-10 | A Quantifier answer that the parser cannot read: banner or no banner? | **The banner shows.** A result that keeps the Room and the Characters present, because no attempt gave a usable reading, counts as a failure of the Quantifier role. The status line still reads "Ready", so D3 stays. This puts the owner's earlier decision in place (prompt-flexibility ticket 10: "the banner shows"). Ticket 06, change 15 | The same | 18 |
| D21 | 2026-11-10 | An author saves a map without the Room where a Game stands. What must happen? | **Refuse the edit.** The answer names the Games that stand in that Room. Ticket 06, change 16 | The same | — |

## What follows from the decisions

| From | Row | State |
|---|---|---|
| D2 | 23 (licence), 24 (streaming) | Wait for a release |
| D4 | 11 (`example_dialogue` in the prompt), 15 (analysis of an Impersonate Input), 16 (present day) | Proposals. Each is a change to a prompt or to a call, and the owner's method needs a trial to judge it |
| D4 | 28, the part "one setup runs three times" | Proposal |
| D4 | C1 of the catalogue (the value of each mechanism) | Not done |
| D1 | 13 (Characters that the model invents) | No change. The agent stated this reading in round 2, and the owner did not object |
| D10 | The Trigger faults of finding 5 in `assets/story-damage-and-false-screen.md` | Open. The random order among Characters is fixed by D7 (e), world-04. The other faults wait with D10 |
| Earlier decision of the owner (2026-10-31) | 21 (LLM messages stay a debug record of 50 rows) | No change. The agent recommended a change only when the link from a story entry to its call is built. Stated in round 3 |
| D11 | A later Character form in the dashboard | Note for that effort: a Swipe state stores whole Character cards (`scene_state.rs:11`), so a Retry would write with the old card. Store the ids first. The long-term-memory map lists "authoring cards in the dashboard" as out of scope for that effort only |
| D19 | N15 (the mutation run), N16 (optional rows of the environment report), N17 (points that a ticket excludes), N18 (preset changes that need a trial) | Not decided. They are in `assets/findings-not-carried.md` |
| D19 | An exit to a missing Room, and a duplicate Room id, in the World form | Routed to ticket 02 of the invariant audit, which decides the one World validation |

## Changes to `CONTEXT.md`

None so far.
