# Feature Spec: Browser Dashboard

Endpoint: browser DOM.

The action controls are two parts of the screen. `#send-control` holds Send, and `#autoplay-controls` holds Autoplay and Stop. The server renders their `disabled` state from the busy state of the Game, and no script writes `disabled`. The DOM order is the visual order: the Action input, Send, the cycles field, Autoplay.

The disabled state follows the busy state of the Game and the request in flight, not the view states of the action area. Two differences from the script that wrote `disabled` before stage 7 are kept:

- Autoplay takes clicks while the state of the action area is Checking or Preview, because the Game is idle then. A run can start while the preview stays open.
- The cycles field is enabled while a Job runs, because only a request of its own form disables it. Enter in the field does not submit the form during a Job, because its default button, Autoplay, is disabled.

## Scenarios

#### Scenario 16.5: Command form stays static after submission

```gherkin
Given the command form #command-form is rendered
When the client submits the form (htmx POST to /action/check)
And the story log updates with new entries
Then the same #command-form element is still in the document (form is a static shell, not re-rendered)
```

#### Scenario 16.6: Status display updates during generation

```gherkin
Given the page is loaded and idle
And the Narrator has not answered
When the client submits the command "wait"
Then #status-display text is not "Ready" within 500ms
And #status-display text contains the label of the Job's step, "Generating narration..."
```

#### Scenario 16.9: Primary button locks and unlocks with the screen after confirming a preview

```gherkin
Given the client submitted a command the text-check preview intercepted, so the preview is open
When the client confirms the preview, so a generation starts
Then the primary button is disabled and labelled as a generating indicator
And the command form is the same node as before
When the screen loads idle and the status display returns to Ready
Then the primary button is enabled again with a "Send" label
```

#### Scenario 16.10: A status error after confirming a preview shows in the status display

```gherkin
Given the client submitted a command the text-check preview intercepted, so the preview is open
When the client confirms the preview
And the status display loads an error fragment
Then the status display shows the error's short line
And the command form is still in the document
And the raw error text is not visible until the client opens the Details disclosure
And the banner is not raised
```

#### Scenario 16.12: Confirming a preview leaves the command form usable and Ready

```gherkin
Given the dashboard is loaded with text check enabled
When the client submits a misspelled command and the preview opens
And the client confirms the preview
Then the Action runs and the status returns to Ready
And the command form is still in the document
And the command input accepts and submits a new command with no reload
```

#### Scenario 16.13: The send preview opens beside the status display

```gherkin
Given a generation is running and the status display shows its step
When the client submits a command the preview intercepts
Then the preview is open
And the status display is still in the document and still shows the step
```

#### Scenario 16.15: Focus moves into the preview and back to the command input

```gherkin
Given the dashboard is loaded with the command form
When the client submits a command the preview intercepts
Then the preview opens with focus in the correction textarea
When the client cancels the preview
Then the preview closes and focus returns to the command input
```

#### Scenario 16.16: The tab bar exposes a tablist and its panels

```gherkin
Given the dashboard is loaded
Then the tab bar exposes a tablist
And each tab exposes a tab role and names the panel it controls
And the Game tab reports itself selected
When the client activates the Settings tab
Then the Settings tab reports itself selected and the Game tab reports itself unselected
And the Settings panel is the visible one
```

#### Scenario 16.17: The page exposes a main landmark and a labelled command input

```gherkin
Given the dashboard is loaded
Then a skip link targets the main landmark
And a label names the command input "Action"
```

#### Scenario 16.18: A failed screen load leaves the parts unchanged and shows the screen error

```gherkin
Given the dashboard is loaded and the story log shows its entries
When a screen load fails with a non-2xx answer
Then #notice shows "GET /screen answered 500" with the server's text behind its Details control
And the banner is not visible, because the engine answered
And the story log still shows the same entries
When a later screen load succeeds
Then #notice is hidden
```

#### Scenario 16.19: A failed Action reports in the notice with the sentence of the server

```gherkin
Given the dashboard is loaded and the Action form is visible
When the client submits an Action and the server fails it
Then #notice shows the first line of the server's answer, "The Action was not sent."
And the short line carries no raw server text and no "That action failed." lead
And the raw server text is reachable in that notice's disclosure
And the notice is drawn as a message with a Details control, not as a second Send button
And the form's inline slot is empty
And the banner is not raised
```

#### Scenario 16.20: A dead engine raises the banner for a pending action and nothing else

```gherkin
Given the dashboard is loaded and the engine has stopped
When the client submits a command
Then the banner is visible and reports the engine as unreachable
And no notice and no inline slot holds a report
```

#### Scenario 16.21: A generation error clamps to one line with a popover

```gherkin
Given the dashboard is loaded and idle
And the action area and command input are measured
When the status display reports a generation error
Then the status display shows one short line and the raw text is not visible
And the action area's height and the command input's width are unchanged
And the error's Details control opens its disclosure, which shows the raw text
When the client presses Escape
Then the disclosure is closed
When the status returns to Ready
Then the error is gone
```

#### Scenario 16.22: A degraded role raises a status banner with its Details disclosure

```gherkin
Given a role is degraded
When the failure banner loads the degraded-role banner
Then the banner is visible and reports itself as status
And the banner names the role, its failure effect, and the banner's engine-wide scope
And its Details control opens the anchored disclosure
And that disclosure names each role with its backend and model
```

#### Scenario 16.23: Status-display changes are announced to assistive technology

```gherkin
Given the dashboard is loaded and idle
When the status display shows a generation step
Then a polite region announces that step
And the step is not announced again while it is unchanged
When the generation fails
Then an assertive region announces the error's short line but not its raw text
When the generation finishes
Then the polite region announces Ready
```

#### Scenario 16.24: A new narration and a changed option set are announced once

```gherkin
Given the dashboard is loaded showing the story log and a set of options
When a new narration arrives and the option set changes
Then the new narration is announced once and not the whole log
And the new options are announced once
And neither is announced again while its content is unchanged
```

#### Scenario 16.25: A failed request on a reachable server never reports the engine unreachable

```gherkin
Given the dashboard is loaded and a panel control's save cannot be stored
When the client changes that control and the reachable engine fails the save
Then the failure reports in the notice
And the panel keeps the content it had
And the banner is not raised
```

#### Scenario 16.26: The status display's open Details disclosure survives a load

```gherkin
Given the status display shows a generation error
And the error's Details disclosure is open with focus in its Details control
When the status display loads again
Then the disclosure is still open
And focus is still in its Details control
When the client presses Escape
Then the disclosure is closed
```

#### Scenario 16.27: The banner's open Details disclosure survives a load

```gherkin
Given the banner reports a degraded role
And the disclosure's Details control has been activated, so it is open with focus
When the failure banner loads again
Then the disclosure is still open, on the same element (the popover lists its open state in data-keep)
And focus is still in its Details control
And the banner is still up
```

#### Scenario 16.28: A screen load that leaves generating re-enables Send

```gherkin
Given the dashboard is loaded and the screen loads a generation step
Then #status-display shows a generating state
And #submit-btn is disabled
When the screen loads idle
Then #status-display returns to Ready
And #submit-btn is enabled
And pressing Enter in #command-form input[name="command"] submits the form
```

#### Scenario 16.29: A failed connection form keeps its dialog and renders the failure inline

```gherkin
Given the edit dialog is open with the connection Add form
And the form is the same node as before the submission
When the client submits the form and the server answers 400
Then the edit dialog is still open with the same form
And the form shows an inline error with the sentence of the server, "The Connection was not saved: <the engine's explanation>."
And that error has no Details control
```

#### Scenario 16.30: A failed preset add keeps its dialog and renders the failure inline

```gherkin
Given the edit dialog is open with a preset Add form
And the form is the same node as before the submission
When the client submits the Add form and the server answers 400
Then the edit dialog is still open with the same form and its typed text
And the Add form shows an inline error naming the failure
And that error has no Details control
```

#### Scenario 16.31: A failed preset edit keeps its dialog and reports in the notice

```gherkin
Given the Prompt Presets panel shows a non-default preset card
And the client has opened that card's edit form in the edit dialog
And the edit form is the same node as before the submission
When the client submits the edit form and the server fails the save
Then the edit dialog is still open with the same form
And #notice shows "The preset was not saved." and the form's inline slot is empty
And the raw server text is reachable in the notice's disclosure
```

#### Scenario 16.32: An invalid preset delete keeps its card and reports in the notice

```gherkin
Given the Prompt Presets panel shows a non-default preset card
And the card is the same node as before the submission
When the client deletes that preset and the server answers 400
Then the card is still in the document
And #notice names what was not done and why
And the report has no Details control
```

#### Scenario 16.33: A failed panel load reports in the screen error

```gherkin
Given the dashboard loads and the Worlds, Games, Settings and Prompt Presets panel renders fail
When the client opens each of those tabs
Then #notice shows a short failure message
And its Details disclosure holds the raw server text
And the panel holds no content
And the banner is not raised
```

#### Scenario 16.34: A failed row button reports in the notice

```gherkin
Given the Worlds panel shows a world row and the Games panel shows a game row
When the client confirms that world row's Delete and the server fails it
Then #notice shows `The World "Redmist Estate" was not deleted.`
And its Details disclosure holds the raw server text
And the world row is still in the list and holds no report
And its Delete control has keyboard focus
When the client confirms that game row's Reset and the server fails it
Then #notice shows "The Game was not reset."
And the game row's Reset control has keyboard focus
And the banner is not raised
```

#### Scenario 16.35: An invalid connection edit keeps its dialog and renders the failure inline

```gherkin
Given the edit dialog is open with a saved connection's Edit form
And the form is the same node as before the submission
When the client submits the form and the server answers 400
Then the edit dialog is still open with the same form
And the form shows an inline error with the sentence of the server, "The Connection was not saved: <the engine's explanation>."
And that error has no Details control
```

#### Scenario 16.36: A confirmed preview consumes the command text

```gherkin
Given the dashboard is loaded with text check enabled
When the client submits a misspelled command and the preview opens
Then the command input still holds the submitted text
When the client confirms the preview with edits
Then the preview closes and the command input is empty
When the client submits the command again and sends the original
Then the preview closes and the command input is empty
When the client submits the command again and cancels the preview
Then the command input still holds the submitted text
```

#### Scenario 16.37: A send that cannot reach the engine keeps the typed command

```gherkin
Given the dashboard is loaded and the engine has stopped
When the client submits the typed command
Then the banner reports the engine as unreachable
And the command input still holds the typed command
```

#### Scenario 16.38: The engine's recovery clears the banner

```gherkin
Given the dashboard is loaded and the engine has stopped
When the client submits a command and the banner reports the engine as unreachable
And the engine answers again
Then the banner is gone
```

#### Scenario 16.39: The active tab survives a reload

```gherkin
Given the dashboard is loaded on the Game tab
When the client activates the Worlds tab
And the page reloads
Then the Worlds tab is the active tab and its panel is the visible one
```

#### Scenario 16.40: A confirm that the engine fails or does not answer keeps the preview open

```gherkin
Given the dashboard is loaded with text check enabled and the preview is open
And the engine has stopped
When the client confirms the preview
Then the banner reports the engine as unreachable
And the preview stays open with the corrected text
And the command input still holds the typed command

Given the dashboard is loaded with text check enabled and the preview is open
When the client confirms the preview and the server fails the Action
Then #notice shows "The Action was not sent." with the server's text behind Details
And the preview stays open with the corrected text
And the command input still holds the typed command
```

#### Scenario 16.41: A generation error is drawn in the error colour

```gherkin
Given the dashboard is loaded and idle
When the status display reports a generation error
Then the short line is drawn in the palette's error colour, not the Ready colour
And the whole short line is readable without opening Details
When the status returns to Ready
Then Ready is drawn in its own colour
```

#### Scenario 16.42: A refused concurrent send keeps the typed command and reports in the notice

```gherkin
Given the dashboard is loaded
And a generation is already in flight
When the client submits a command the engine refuses
Then #notice says "The Game is busy, so the Action was not sent."
And its Details disclosure holds both lines of the answer
And the status display keeps the state it had
And the command input still holds the typed command
And no preview is open
```

#### Scenario 16.43: Every failure sentence reads in full in the status column

```gherkin
Given the dashboard is loaded at a viewport width of 1280
When the status display reports a failed generation of any kind
Then the whole sentence for that kind is drawn inside #status-display
And no part of the sentence is clipped or ellipsized
And the command input and the action area keep their sizes
When the viewport width is 1024
Then the whole sentence for every kind still reads in full
```

#### Scenario 16.44: A confirm that the busy Game refuses reports in the notice and keeps the preview

```gherkin
Given the dashboard is loaded with text check enabled and the preview is open
And a generation is already in flight
When the client confirms the preview and the engine refuses the command
Then the preview stays open with the corrected text
And #notice says that the Game is busy
And the command input still holds the typed command
```

#### Scenario 16.45: The action area shows its own state until the status display shows the send

```gherkin
Given the dashboard is loaded and idle
And the status display cannot load
When the client submits a command the engine takes
Then #action-status shows "Thinking..."
And #action-area carries the state "generating"
And the primary button takes no click
When the status display can load and the page asks
Then #action-status is empty
And #status-display shows the label of the Job's step
And the primary button is disabled and labelled as a generating indicator
```

#### Scenario 16.46: A swap into the dialog body opens the modal dialog

```gherkin
Given the dashboard is loaded and the edit dialog is closed
When the client activates a control whose answer htmx swaps into #edit-dialog-body
Then #edit-dialog is open as a modal dialog
And the dialog is named by the heading of the form that it holds
And the first field of the form has keyboard focus
```

#### Scenario 16.47: Cancel and Escape close the dialog and return focus to the opener

```gherkin
Given the edit dialog is open, and a control of the page opened it
When the client presses Escape
Then the edit dialog is closed and its body is empty
And the control that opened it has keyboard focus
When the client opens the dialog again
And clicks the backdrop outside the dialog
Then the edit dialog is still open
When the client activates Cancel
Then the edit dialog is closed and its body is empty
And the control that opened it has keyboard focus
And closing the dialog sent no request
When a click that does not focus its control opens the dialog, and the client activates Cancel
Then the control that opened it has keyboard focus
```

#### Scenario 16.48: A save that the engine accepts closes the dialog

```gherkin
Given the edit dialog is open with a form
When the client saves and the engine accepts the save
Then the edit dialog is closed and its body is empty
```

A save that the engine answers with 400 keeps the dialog open: see Scenarios 16.29 and 16.35.

#### Scenario 16.49: An answer that comes from outside never replaces the form of the open dialog

```gherkin
Given the client activated a connection's Test control, and its request is in flight
And the client then opened the Add Connection form in the dialog, and typed a name
When the answer of the Test arrives
Then the edit dialog is still open with the Add Connection form
And the typed name is still in the form
And the result of the Test is not shown
```

#### Scenario 16.50: Escape closes an open error detail before it closes the dialog

```gherkin
Given the edit dialog is open with a form, and a typed value in a field
And a failed Test of the form shows its error with the Details open
When the client presses Escape
Then the error detail is closed
And the edit dialog is still open, and the field keeps the typed value
When the client presses Escape again
Then the edit dialog is closed
```

#### Scenario 16.51: Closing the dialog focuses the opener by its id

```gherkin
Given the edit dialog is open, and a Prompt Presets card control opened it
And a load of the panel has rewritten the card of that control
When the client activates Cancel
Then the edit dialog is closed
And focus is on the control that has the id of the control that opened the dialog
```

#### Scenario 16.52: The open dialog is centred in the viewport

```gherkin
Given the dashboard is loaded
When a swap into the dialog body opens the dialog
Then the dialog is centred horizontally in the viewport
And the dialog is centred vertically, with its top edge below the top of the viewport
```

#### Scenario 16.53: Each request carries the Game that the page shows

```gherkin
Given the dashboard has loaded the Game that GET /changes names
When the client submits a command
Then the request carries the header X-Chronicler-Game with the id of that Game

When the client deletes the last message of the story log and confirms
Then that request carries the header X-Chronicler-Game with the same id
```

#### Scenario 16.54: A send for a Game that is no longer active keeps the typed command and reports in the notice

```gherkin
Given the dashboard shows the active Game
And the engine switched the active Game, and the page has not asked since
When the client submits a command
Then the engine refuses the command with 409 and "The active Game changed, so the Action was not sent."
And #notice says that the active Game changed
And the command input still holds the typed command
And no preview is open
```

#### Scenario 16.55: An edit saved for a Game that is no longer active stays open

```gherkin
Given the dashboard shows the active Game and an entry of the story log is in edit with typed text
And the engine switched the active Game, and the page has not asked since
When the client presses Save
Then the engine refuses the save with 409 and "The active Game changed"
And #notice says that the active Game changed
And the editor is still open and holds the typed text
```

#### Scenario 16.56: The page names a Game only when the screen shows it

```gherkin
Given the dashboard shows Game 1
And the engine switched the active Game to Game 2
And the screen load that shows Game 2 is still in flight
Then the page still names Game 1
When the screen load ends
Then the page names Game 2
```

#### Scenario 16.57: A send after a switch that this page sent gets the refusal of a changed Game and keeps its text

```gherkin
Given the dashboard shows Game 1 and the Games panel lists a second saved Game
And the client typed a command in the Action input
When the client presses Switch for the second Game and the engine accepts it
Then the page still names Game 1 until the next load shows Game 2
When the client sends the command before that load ends
Then the engine refuses it with 409 and "The active Game changed"
And #notice says that the active Game changed
And the Action input still holds the command
When the load ends
Then the page names the second Game
```

A create and a Reset that this page sent do the same.

#### Scenario 16.58: A Stop for a Game that is no longer active reports in the notice

```gherkin
Given an Autoplay run holds the Game and Stop shows
And the engine switched the active Game, and the page has not asked since
When the client presses Stop
Then the engine refuses it with 409 and "The active Game changed"
And #notice says "The active Game changed, so the Autoplay run was not stopped."
```

#### Scenario 16.59: A Reset for a Game that is no longer active reports in the notice

```gherkin
Given the Games panel shows the active Game row
And the engine switched the active Game, and the page has not asked since
When the client confirms that row's Reset
Then the engine refuses it with 409 and "The active Game changed"
And #notice says "The active Game changed, so the Game was not reset."
```

#### Scenario 16.61: A preview send clears the Action input only if it still holds the text that the preview sent

```gherkin
Given the text check shows a preview of the command in the Action input
When the client presses Send Original, and types more text in the input while the request is in flight
And the engine takes the command and the screen shows the send
Then the Action input keeps the typed text
```

#### Scenario 16.62: A check request disables the Action input and shows Send as busy until it ends

```gherkin
Given the dashboard is loaded and idle, and the engine holds the request that checks a command
When the client submits a command
Then the Action input is disabled
And #submit-btn is not disabled, and it takes no click
And #action-area carries the state "checking"
When the engine answers the request
Then the Action input is enabled and #submit-btn takes clicks
And #action-area no longer carries the state "checking"
```

#### Scenario 16.63: A Send while a check request runs is not sent twice

```gherkin
Given the dashboard is loaded and idle, and the engine holds the request that checks a command
When the client submits a command
And the client clicks Send, submits the command form, and presses Enter in the Action input
Then the page sends no second request that checks a command
When the engine answers the request
Then the engine has received one request that checks a command
```

#### Scenario 16.64: A load that lands during a check request renders Send as the server does

```gherkin
Given the dashboard is loaded and idle, and the engine holds the request that checks a command
When the client submits a command
And the Game starts a Job, so the page loads the screen while the request runs
Then #submit-btn is disabled and labelled as a generating indicator, as the server renders it
When the engine answers the request without a screen change
Then #submit-btn is still disabled and labelled as a generating indicator
```

#### Scenario 16.65: A check request writes no `disabled` on Send

```gherkin
Given the dashboard shows a Job that runs, so #submit-btn is disabled
When the client submits the command form and the engine answers the check
Then #submit-btn is disabled and labelled as a generating indicator
And its `disabled` attribute was not written at the start or at the end of the request
```

#### Scenario 16.66: #action-area carries the state "preview" while the text-check preview is open

```gherkin
Given the text check returns a preview for a command
When the client submits the command
Then #action-area carries the state "preview"
When the client sends the original command
Then #action-area no longer carries the state "preview"
```


#### Scenario 16.67: Tab follows the visual order of the action controls

```gherkin
Given the dashboard is loaded and idle
Then, from left to right, the controls are the Action input, Send, the cycles field, and Autoplay
When the client focuses the Action input and presses Tab three times
Then focus is on Send, then on the cycles field, then on Autoplay
```

#### Scenario 16.68: A check answer whose swap throws leaves the command form dead and the screen loading

```gherkin
Given the dashboard is loaded and idle
And the swap of the answer to a check request throws
When the client submits a command
Then htmx reports the load error
And the screen loads again, though no change moved the revision
And #submit-btn is enabled as the server renders it, and #action-area does not carry the state "checking"
But the Action input stays disabled, #submit-btn still takes no click, and the command form sends nothing when it is submitted again
```

#### Scenario 16.69: A pending state ends when the screen shows the answer of its request

```gherkin
Given the dashboard is loaded and idle
When the client sends a command, sends a previewed command, starts Autoplay, retries, retriggers, or asks for new options, and the engine takes the request
Then #action-status shows "Thinking..." while the screen does not show the answer yet
When the screen shows the answer
Then #action-status is empty
```

#### Scenario 16.70: A pending state ends at once when the engine does not take its request

```gherkin
Given the dashboard is loaded and idle
When the client retries, retriggers, or asks for new options, and the engine refuses the request or it fails
Then the report shows in #notice
And #action-status is empty
```

#### Scenario 16.71: A 409 on the Options regen form shows exactly one refusal report in the notice

```gherkin
Given the dashboard is loaded and the options dock shows options
When the client asks for new options and the engine refuses the request with 409
Then #notice shows one refusal report
And the report says "the Action was not sent"
And the options dock holds no report
```

#### Scenario 16.72: The Close control of the notice hides it

```gherkin
Given the Worlds panel shows a world row and the server fails its Delete
When the client confirms the Delete
Then #notice shows the report, and its Close control has the accessible name "Close the notice"
When the client presses Close
Then #notice is hidden and its report is empty
And focus is on #main-content
```

Where each kind of control reports a refusal or a failure is pinned by the scenarios of that control: 16.19 (the Action input), 16.25 and 44.18 (a control that saves on change), 16.29 (an invalid dialog form), 16.32 and 16.34 (row buttons), 16.40 (a preview confirm), 16.58 and 16.59 (a Game that is no longer active), 44.17 (a dialog form that fails), and 16.42 and 42.6 (the busy refusal, with both lines in Details). A failure with no lead and a refusal with both lines are pinned there too.

#### Scenario 16.73: A refused dialog form reports in the notice, and an invalid answer of a form with typed input reports in that form

```gherkin
Given the Rename dialog of a Game is open with a typed name
When the client saves the form and the engine refuses it with 409
Then #notice shows "The Game is busy, so the Game was not renamed." with both lines in Details
And that is the only report on the page, and the banner is not raised
And the dialog stays open with its draft, and its status line repeats the sentence

Given the dashboard is loaded
When the engine answers the Action input with 400
Then #command-form's slot shows "The Action was not sent: <the engine's message>."
And #notice stays empty, and the typed command stays
When the engine answers the Autoplay cycles with 400
Then #autoplay-form's slot shows "The Autoplay run did not start: <the engine's message>."
And #notice stays empty, and the typed cycles stay
```

No engine route answers a dialog form with 409 today. The first block guards the case for a future route.

#### Scenario 16.74: A notice stays until the next request, until a failing screen load works again, or until Close

```gherkin
Given the dashboard shows a notice for a failed request
When the revision moves and a screen load works
Then the notice stays
When the client changes the tab
Then the notice stays
When the client presses a dialog opener
Then the notice clears
When the client sends the next request
Then the notice clears

Given a screen load fails and #notice shows its report
When a later screen load works
Then #notice is hidden

Given an invalid answer shows in the slot of a form
When the client sends that form again
Then the slot is empty when the request starts
```

#### Scenario 16.76: No answer raises the Unreachable banner and nothing else

```gherkin
Given the engine has stopped
When the client presses a row button, changes a control that saves on change, saves a dialog form, or saves a story-log edit
Then the banner reports the engine as unreachable
And no slot and no notice holds a report
And a story-log edit that gets no answer is reverted
And a dialog stays open and repeats the Unreachable text in its status line
When the engine answers again
Then the banner is gone, and the dialog status line no longer says the engine is unreachable
```

A preview confirm with no answer is pinned by 16.40.

#### Scenario 16.77: A load error of the new-Options form does not end a check that is in flight

```gherkin
Given a check request of the command form is in flight
When htmx reports a load error for the new-Options form
Then the action area is still checking
When htmx reports a load error for #command-form
Then the action area is no longer checking
```

#### Scenario 16.78: An opener keeps its id through a load, and an opener that is gone takes no focus

```gherkin
Given the edit dialog is open, and a control of a saved Game in the Games panel opened it
And a load of the panel has removed the row above that control
When the client activates Cancel
Then focus is on the control that has the id of the control that opened the dialog
When the client opens the dialog again, and a load of the panel removes the row of the control that opened it
And the client activates Cancel
Then the dialog gives focus to no control, and focus is on the body
```

#### Scenario 16.79: A report whose sentence is the whole text has no Details

```gherkin
Given the dashboard is loaded
When the engine answers the Action input with 400, and the body is one line
Then #command-form's slot shows that line
And the slot holds no Details control and no text that repeats the line
```

#### Scenario 16.80: A long sentence without Details reads in full

```gherkin
Given the Add form of a Connection is open in the edit dialog
And a report whose sentence is the whole text and is longer than its place is wide
When the report shows in the slot of the form and in the notice, at 1280 and at 1024 pixels
Then each place draws the whole sentence on more than one line
And no place cuts the sentence or ends it with an ellipsis
```
