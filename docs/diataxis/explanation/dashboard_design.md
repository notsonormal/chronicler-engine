---
diataxis: explanation
title: Dashboard Design
---

## The SillyTavern lineage

The Chronicler Engine's dashboard inherits its interaction model from SillyTavern, the chat front end for LLM roleplay that set the conventions for this kind of interface. The borrowed pieces:

- **Story log, visual sidebar and action area.** The Game tab mirrors SillyTavern's chat layout: a centre column for the chat history, a side column for character portraits and scene imagery, and a bottom strip for input. The Chronicler Engine pins the action area to the bottom of the page rather than letting it float.
- **A/B swipes on the last message.** SillyTavern's swipe model, with several alternative generations on the last message and arrow buttons to move between them, maps directly to the `Swipe` aggregate.
- **Inline edit on any message.** The player opens an entry's edit control, changes the text, and saves. The Chronicler Engine applies the pattern to player inputs (re-edit what you said) and to AI responses (replace the LLM's generation with a hand edit).
- **Continue on an empty input.** SillyTavern's Continue extends the last narration from a separate button next to Send. The Chronicler Engine puts Continue on the Send button: the player sends an empty input.
- **Confirmation before destructive actions.** Deleting a message, resetting the game and deleting a game each ask the player to confirm first.

## How the dashboard shows server data

The server renders every part of the screen, so domain content has one source. The htmx runtime ships with the app's static assets, so a dashboard load reaches only the engine's own server.

The design came from one rule: a draft, a focus or a scroll position must survive every load, and no part may stay old while another part shows new data. An earlier design gave each region its own poll, its own number and its own protection from loads. Most of its bugs were a swap that destroyed client state, or a region that was out of step with another region. Each fix was local to one region, so the fixes added mechanisms and faults of their own.

Two designs of the interface were then written from the requirements alone, by two models that had no access to the code. Both models belong to one family, so their agreement is weaker evidence than two families would give. Both kept the server half: the server renders, an action answers no screen content, and a change marks the screen old before the screen fetches and morphs. Both rejected a browser table that maps each kind of data to its regions. The design below follows what both required, and takes the one point that they left free, the dialog, because it removes a dirty check.

### One screen, loaded in one request

The screen is the header, the failure banner and the parts of the visible tab. Any change moves one engine-wide revision, and the page loads the whole visible screen in one request. All parts of one load come from one revision, so no part can lag behind another. The browser has no table of kinds and regions, so a new part needs no mapping and cannot be forgotten in one.

The cost is that a change in data that no visible part shows still loads the screen. The morph keeps every node that did not change, so the player sees nothing. A local server makes the extra load cheap.

The browser names the parts that it wants, and the server holds one renderer for each part name. The shell is then the one source of the layout, and the server knows renderers, not tabs. A part that has no renderer fails at once, in every state.

### One ask on a revision

The page asks one small endpoint, `GET /changes`, once a second while the page is visible. The answer holds only numbers, so it is cheap. Polling is simpler and more stable than the htmx WebSocket extension, which was unreliable in production. A push transport can carry the same revision later.

The server still keeps one number for each kind of data. The numbers record which kind each write raises, and the browser compares only the revision. A number rises after its data is committed, and every save path raises its kinds through the storage layer. A save path that names no kinds does not compile, so the dashboard needs no safety refresh for a forgotten path. The numbers are counters, not dates, so they do not depend on the system clock.

### An action answers one of five outcomes

An action saves data and answers a status code and the revision after its commit. The data reaches the screen only through a screen load, so two answers cannot write the same part in different shapes.

Every answer is one of five outcomes, and each outcome asks the player for a different next step:

- **Done**: a `2xx` with the revision. The player goes on.
- **Refused**: a `409`. The Game is busy, another Game is active now, or the last Message is no longer the one that the page showed. The player waits, or looks at what changed.
- **Invalid**: a `400`, or a `404` for a dialog that opens a record that is gone. The input is bad, or the data does not allow the change. The player fixes the input, or reads why.
- **Failed**: a `500`. A fault in the engine stopped the action.
- **No answer**: the engine is unreachable. The player waits for it.

The status chooses the outcome, and the text tells the player what happened. The server writes the sentence of every action answer. A sentence says what was not done, and it names the item where one exists, so it reads correctly away from its control. The engine's full text is behind Details. The engine knows what it refused and which item it read, so the words have one source, and a new action brings its sentence with it.

The page shows "Saved", closes the dialog and clears the Action input only when the screen shows the revision of the answer. The confirmation and the screen then agree. The cost is one extra round trip after each action: the ask that follows it.

Each action carries the Game of the screen. The engine refuses an action from a page whose Game is no longer active, so a tab cannot send to a Game that another tab switched to a moment before. The refused text stays in the input.

Delete, Retry and Retrigger act on the last Message, so each also carries the Message that its page rendered it on. The engine refuses the request when that Message is no longer the last one. An entry stops being last when, for example, another browser tab added or deleted a Message, an Autoplay run added one, or the page was last loaded before a Job ended. The engine makes the check in the same step as the busy check, so no write lands between the check and the act.

### A report shows where the player can act on it

An invalid answer to a form with typed input shows inside that form, and the typed text stays. The forms with typed input are the dialog forms, the Action input, the Autoplay cycles and the message edit. The fix is in that text, so the report is next to it.

Every other report shows in the notice place of the shell, under the header. The cause of a refusal, a failure or a failed screen load is the state of the engine, so one fixed place on every tab holds all of them, and the player finds each report in the same place. An open dialog repeats the summary in its status line, because the modal hides the page. A refused or failed dialog stays open with its draft. No answer shows in the Unreachable banner.

The tradeoff is distance: a refusal shows away from the control that the player pressed. The sentence names what was not done, so the report still makes sense there. A refusal is rare, because the server renders Send and Autoplay disabled while the Game is busy, and the `409` catches two actions that race.

A form report stays until that form sends again or closes. A notice stays until the player sends the next action, until a failing screen load works again, or until the player closes it. The notice place is in the shell, so a tab change keeps it.

### One reader for every answer

One piece of browser code reads every action answer and shows every report. A control adds only its own work after a done answer: it closes the dialog, clears the input or moves focus. A control that saves on change also returns to its rendered value when its save does not go through. An earlier design read the answer at each control, and each control chose its own words and its own place, so a new action had to know which reader applied to it.

The server renders every control that sends a request and every place that holds a report. The server is then the one source of the markup, so one check covers every control.

### A tab exists while it is open

A tab's parts hold content only while the tab is open. Leaving a tab empties its parts, and opening a tab loads them. A closed tab cannot be stale, and no rule must decide when a hidden part loads. View state of the open tab, such as the page of the saved Games, ends with the tab.

The cost is that leaving the Game tab loses the scroll position of the story and an open message edit. The Action input keeps its text, because it is in the shell.

### View state survives a load, and a draft stays out of the screen

A change can arrive while the player works. A load must neither destroy what the player does nor wait for it.

- A load morphs each part, so a node that the new render keeps stays the same node, with its focus, selection and scroll position.
- View state, such as an open Details disclosure or the active Settings sub-tab, lives in attributes that the server never sets. An element lists them in `data-keep`, and the load keeps them.
- A form that waits for a Save opens in one modal dialog outside the screen, so no load meets a draft. A load never writes into a draft: both designs required it. The single dialog was free, and it was chosen because it needs no check of what counts as a draft. Closing the dialog discards the form, and focus returns to the control that opened it. The server renders an id on each such control, and the dialog finds the control by that id, so a load that rewrote the list does not lose it.
- Two elements still hold unsaved input inside a part: a story-log entry in edit, and a control whose request is in flight. A load skips each one and shows the rest of the part. The player can edit a message while a Job runs, and the new narration arrives around the draft.

The tradeoff is the dialog. It takes the player out of the list for the time of the edit, and it shows the data as it was when the form opened. The last save wins.

### The server renders every control state

Whether a control is disabled is data, so the server renders it from the busy state of the Game. No script writes `disabled`, and each load corrects every control. A control that a script had disabled and no script re-enabled could stay disabled after a failure. The guardrails ban such a write.

The cost is that a request shows its own busy state through CSS only. Assistive technology gets no signal in that time. A second submit of the form is dropped, and the engine refuses a second send with `409`.

### A list sends short rows

A list sends the header of each row. The full item loads when the player opens it, in the dialog. The LLM Messages record is the case: a record never changes, and the full texts of 50 records were the largest part of each load.

A list with no natural limit has pages. The saved Games show 50 for each page. The page number is view state of the browser, not of the server, so two tabs can show different pages, and a tab change resets it.

### The check that the screen converges

Every browser check ends with a comparison of each visible part with a fresh render of the same part. A part that stayed old fails the check, so a stale part is found by one test and not fixed one part at a time.

## Palette

The dashboard's palette is built for long prose. Quoted dialogue differs from narration by hue, and both stay at body weight.

## Document References

- [`../reference/frontend/dashboard.md`](../reference/frontend/dashboard.md) — the dashboard's parts and what owns what.
- [`../reference/storage.md`](../reference/storage.md) — the revision and the change counters.
