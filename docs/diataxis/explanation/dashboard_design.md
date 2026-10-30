---
diataxis: explanation
title: Dashboard Design
---

## The SillyTavern lineage

The dashboard takes its interaction model from SillyTavern, a chat front end for LLM roleplay. A player who knows SillyTavern finds the chat layout, the Swipes of the last Message and the inline edit of a Message where they expect them.

## How the dashboard keeps the screen current

The browser keeps no copy of engine data. The server renders all of it, and the page shows what the server rendered. Each section states one rule of this model, its reason and its cost. The Dashboard reference states the mechanism.

### One revision, one screen load

The engine has one revision. The page asks `GET /changes`, and when the revision moves, one `GET /screen` load writes the parts of the visible tab. The page needs no table from a kind of data to the parts that show it, so a new part needs only a renderer and a place in the shell. The page converges after any change, because the load does not depend on what changed. The page polls and holds no stream, so a lost answer costs one ask.

The costs:

- A move of the revision loads the whole visible tab, also when no visible part shows the change. The morph keeps each unchanged node, so the player sees nothing.
- The server reads the parts one after the other. A load shows at least the revision that it names, and one Game. A story log from before a Narration can show beside a status display from after it, until the next ask. "No part lags" holds for each load as a lower bound, not as a snapshot.

### A request answers only its outcome

A request answers a status and the revision after its commit. Only a screen load writes a part, so each part has one writer and one shape. The page shows "Saved", closes a dialog or clears the Action input only when the screen shows the revision of the answer. The confirmation and the screen then agree. The cost is the ask and the load after each request.

Three `POST` routes answer content: `/action/check` (the preview), `/connections/:id/test` and `/connections/test` (the Test result). The engine stores no result of these requests, so no load can show it. Each answers into a place outside every part.

### Five outcomes

Every request ends in Done, Refused, Invalid, Failed or No answer. Each outcome asks the player for a different next step:

- **Done.** The player goes on.
- **Refused.** A `409` only: a busy Game, a Game that is no longer active, or a Message that is no longer the last one. The player waits, or looks at what changed.
- **Invalid.** The player fixes the input, or reads why.
- **Failed.** A fault in the engine stopped the request. The player tries again later, or reads the engine lines.
- **No answer.** The player waits for the engine.

A request with no answer within its time limit is No answer, so one hung ask cannot stop the screen. The limit covers the ask and the load. A request that a control sends has no limit, and its control stays busy while it waits.

The server writes the sentence of every failed answer, because only the engine knows what it did not do and which item it read. The sentence names the item, so it reads correctly away from its control. Details hold only the engine lines after the sentence. The Unreachable text, the summary of a failed screen load and the line for an answer with no body stay in the page, because the engine cannot write them.

### A request names what its control showed

A control that acts on the last Message names that Message. A request that acts on a Game names a Game, and a draft names the Game that it was written for. The reason is one rule: a stale page must not act on something that the player does not see. The engine refuses such a request, and the typed text stays. After a refused send, the Action input takes the Game that the screen shows, because the player sees that Game now.

The cost is a refusal that the player did not cause. Another browser tab, an Autoplay run or a finished Job can change the last Message or the active Game after the page rendered its controls.

### A report shows where the player can act on it

An invalid answer to a form with typed input shows in that form, because the fix is in that text. Every other report shows in the one notice place under the header, because its cause is the state of the engine. An open dialog repeats the summary, because the modal hides the page. The latest report wins, and a one-line report wraps.

The costs: a refusal shows away from the control that the player pressed, so its sentence names what was not done. A failing screen load repeats each second and replaces the notice of a request, so the player can lose a refusal report after about one second.

### One reader reports every answer

One piece of browser code reads every request answer and shows every report. A new request needs no reader of its own, and its words and its place follow from its outcome.

The server renders every control that sends a request and every place for a report, except in the shell. The shell holds `#notice`, the command form and the Autoplay form, because they must survive a load and a tab change. The shell is a second source of markup, so a sweep test checks the report places of both sources.

### Cannot act has more than one source

The server renders `disabled` for what the engine knows: a busy Game, a view-only Game. The Games pager uses `aria-disabled`, so it keeps focus through a load. CSS shows a request in flight as busy. The story-log edit lock, with its click guard, stops Edit and the swipe controls, because only the browser knows the edit. No script writes `disabled`, so each load corrects every control and no failure leaves one disabled. The Action input and the cycle budget stay enabled during their request, so focus stays in the input.

The cost is that the busy state of a request is CSS only. Assistive technology gets no signal. The page drops a second submit of the same form, and the engine refuses a second send.

### A draft lives outside the screen

A form that waits for a Save opens in one modal dialog outside the screen, so no load meets a draft and the page needs no check of what counts as a draft. The Action input and the Autoplay form live in the shell for the same reason. The cost is that the dialog takes the player out of the list, and it shows the data as it was when it opened. The last save wins.

The inline Message edit is the one draft inside a part. The edit stays in the story, next to the text that it changes. Its cost is a list of rules that no other draft needs:

- A load skips the entry in edit.
- The edit lock and its click guard stop the controls that would change the story under the draft.
- Its report slot is in `.edit-actions`, the one slot that is not in a form.
- The Save hint reads the status display of another part.

### View state and tabs

The player owns view state, such as an open Details, so a load keeps it. The active tab, the Settings sub-tab and the Games page also survive a tab change and a reload, in `sessionStorage`. The cost is that each browser tab holds its own view state.

A tab's parts hold content only while the tab is open. A closed tab cannot be stale, and no rule decides when a hidden part loads. The cost is that leaving the Game tab loses the scroll position of the story and an open Message edit.

### A list sends short rows

A list sends the header of each row, and the full item opens in the dialog. The full texts of the LLM messages are the largest data of the dashboard, and they never change after the engine records them. The cost is one more request when the player opens an item.

### The convergence check

Every browser check ends with a comparison of each visible part with a fresh render of the same part. One shared check finds a stale part in any part, so no part needs a protection of its own. `tests/STRATEGY.md` states the mechanism.

## Document References

- [`../reference/frontend/dashboard.md`](../reference/frontend/dashboard.md) — the dashboard's parts and what owns what.
