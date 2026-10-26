# Feature Spec: Browser Dashboard

Endpoint: browser DOM.

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

#### Scenario 16.9: Primary button locks and unlocks with the status after confirming a preview

```gherkin
Given the client submitted a command the text-check preview intercepted, so the preview is open
When the client confirms the preview, so a generation starts
Then the primary button is disabled and labelled as a generating indicator
And the command form is the same node as before
When the status display loads idle and returns to Ready
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
Then #screen-error shows "GET /screen answered 500" with the server's text behind its Details control
And the banner is not visible, because the engine answered
And the story log still shows the same entries
When a later screen load succeeds
Then #screen-error is hidden
```

#### Scenario 16.19: A failed action renders its message in the form's inline slot

```gherkin
Given the dashboard is loaded and the action form is visible
When the client submits a command and the server fails the action
Then the form shows an inline error naming the action as failed
And the inline error's short line carries no raw server text
And the raw server text is reachable in that error's disclosure
And the inline error is drawn as a message with a Details control, not as a second Send button
And the banner is not raised
```

#### Scenario 16.20: A dead engine renders the same inline error for a pending action

```gherkin
Given the dashboard is loaded and the engine has stopped
When the client submits a command
Then the form shows an inline error saying the engine is unreachable
And the banner is visible and reports the engine as unreachable
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

#### Scenario 16.25: A failed action on a reachable server never reports the engine unreachable

```gherkin
Given the dashboard is loaded and a panel control's save cannot be stored
When the client changes that control and the reachable engine fails the save
Then the failure reports on its own surface
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

#### Scenario 16.28: A load of the status display that leaves generating re-enables Send

```gherkin
Given the dashboard is loaded and the status display loads a generation step
Then #status-display shows a generating state
And #submit-btn is disabled
When the status display loads idle
Then #status-display returns to Ready
And #submit-btn is enabled
And pressing Enter in #command-form input[name="command"] submits the form
```

#### Scenario 16.29: A failed connection form keeps its dialog and renders the failure inline

```gherkin
Given the edit dialog is open with the connection Add form
And the form is the same node as before the submission
When the client submits the form and the server refuses it
Then the edit dialog is still open with the same form
And the form shows an inline error naming the failure
And that short line quotes the server's own explanation of the refusal
And the raw server text is reachable in that error's disclosure
```

#### Scenario 16.30: A failed preset add keeps its dialog and renders the failure inline

```gherkin
Given the edit dialog is open with a preset Add form
And the form is the same node as before the submission
When the client submits the Add form and the server refuses it
Then the edit dialog is still open with the same form and its typed text
And the Add form shows an inline error naming the failure
And the raw server text is reachable in that error's disclosure
```

#### Scenario 16.31: A failed preset edit keeps its dialog and renders the failure inline

```gherkin
Given the Prompt Presets panel shows a non-default preset card
And the client has opened that card's edit form in the edit dialog
And the edit form is the same node as before the submission
When the client submits the edit form and the server fails the save
Then the edit dialog is still open with the same form
And the form shows an inline error naming the failure
And the raw server text is reachable in that error's disclosure
```

#### Scenario 16.32: A refused preset delete keeps its card and renders the failure inline

```gherkin
Given the Prompt Presets panel shows a non-default preset card
And the card is the same node as before the submission
When the client deletes that preset and the server refuses it
Then the card is still in the document
And the card shows an inline error naming the refusal
And the raw server text is reachable in that error's disclosure
```

#### Scenario 16.33: A failed panel load reports in the screen error

```gherkin
Given the dashboard loads and the Worlds, Games, Settings and Prompt Presets panel renders fail
When the client opens each of those tabs
Then #screen-error shows a short failure message
And its Details disclosure holds the raw server text
And the panel holds no content
And the banner is not raised
```

#### Scenario 16.34: A failed row action reports in the row's own error slot

```gherkin
Given the Worlds panel shows a world row and the Games panel shows a game row
When the client confirms that world row's Delete and the server fails it
Then the world row shows its own short failure message
And its Details disclosure holds the raw server text
And the world row is still in the list
And its Delete control has keyboard focus
When the client confirms that game row's Reset and the server fails it
Then the game row shows its own short failure message
And the game row's Reset control has keyboard focus
And the banner is not raised
```

#### Scenario 16.35: A refused connection edit keeps its dialog and renders the failure inline

```gherkin
Given the edit dialog is open with a saved connection's Edit form
And the form is the same node as before the submission
When the client submits the form and the server refuses it
Then the edit dialog is still open with the same form
And the form shows an inline error naming the failure
And the raw server text is reachable in that error's disclosure
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
Then the form shows an inline error saying the engine is unreachable
And the command input still holds the typed command
```

#### Scenario 16.38: The engine's recovery clears the form's inline error and the banner

```gherkin
Given the dashboard is loaded and the engine has stopped
When the client submits a command and the form shows its inline error
And the engine answers again
Then the inline error is gone
And the banner is gone
```

#### Scenario 16.39: The active tab survives a reload

```gherkin
Given the dashboard is loaded on the Game tab
When the client activates the Worlds tab
And the page reloads
Then the Worlds tab is the active tab and its panel is the visible one
```

#### Scenario 16.40: A failed confirm reports on the command form's slot

```gherkin
Given the dashboard is loaded with text check enabled and the preview is open
And the engine has stopped
When the client confirms the preview
Then the preview closes
And the command form's own inline error slot reports the failure
And the command input still holds the typed command
```

#### Scenario 16.41: A generation error takes the error state's class and colour

```gherkin
Given the dashboard is loaded and idle
When the status display reports a generation error
Then #status-display carries the error state's class and not the ready state's
And the short line is drawn in the palette's error colour, not the Ready colour
And the whole short line is readable without opening Details
When the status returns to Ready
Then #status-display carries the ready state's class again
And Ready is drawn in its own colour
```

#### Scenario 16.42: A refused concurrent send keeps the typed command and reports in the command form's slot

```gherkin
Given the dashboard is loaded
And a generation is already in flight
When the client submits a command the engine refuses
Then the command form's own inline error slot says that the Game is busy
And its Details disclosure holds the engine's answer
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

#### Scenario 16.44: A confirm that the busy Game refuses reports on the command form's slot

```gherkin
Given the dashboard is loaded with text check enabled and the preview is open
And a generation is already in flight
When the client confirms the preview and the engine refuses the command
Then the preview closes
And the command form's own inline error slot says that the Game is busy
And the command input still holds the typed command
```

#### Scenario 16.45: The action area shows its own state until the status display shows the send

```gherkin
Given the dashboard is loaded and idle
And the status display cannot load
When the client submits a command the engine takes
Then #action-status shows "Thinking..."
And the primary button is disabled and labelled as a generating indicator
When the status display can load and the page asks
Then #action-status is empty
And #status-display shows the label of the Job's step
And the primary button is disabled
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
```

#### Scenario 16.48: A save that the engine accepts closes the dialog

```gherkin
Given the edit dialog is open with a form
When the client saves and the engine accepts the save
Then the edit dialog is closed and its body is empty
```

A save that the engine refuses keeps the dialog open: see Scenarios 16.29, 16.35 and 40.6.

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
And a refused save shows its error with the Details open
When the client presses Escape
Then the error detail is closed
And the edit dialog is still open, and the field keeps the typed value
When the client presses Escape again
Then the edit dialog is closed
```

#### Scenario 16.51: Closing the dialog focuses the control that sends the opener's request

```gherkin
Given the edit dialog is open, and a Prompt Presets card control opened it
And a load of the panel has rewritten that control in place into another control
When the client activates Cancel
Then the edit dialog is closed
And focus is on the control of the panel that sends the request of the control that opened the dialog
```

#### Scenario 16.52: The open dialog is centred in the viewport

```gherkin
Given the dashboard is loaded
When a swap into the dialog body opens the dialog
Then the dialog is centred horizontally in the viewport
And the dialog is centred vertically, with its top edge below the top of the viewport
```

#### Scenario 16.53: Each action carries the Game that the page shows

```gherkin
Given the dashboard has loaded the Game that GET /changes names
When the client submits a command
Then the request carries the header X-Chronicler-Game with the id of that Game

When the client deletes the last message of the story log and confirms
Then that request carries the header X-Chronicler-Game with the same id
```

#### Scenario 16.54: A send for a Game that is no longer active keeps the typed command and reports in the command form's slot

```gherkin
Given the dashboard shows the active Game
And the engine switched the active Game, and the page has not asked since
When the client submits a command
Then the engine refuses the command with 409 and "The active Game changed"
And the command form's own inline error slot says that the active Game changed
And the command input still holds the typed command
And no preview is open
```

#### Scenario 16.55: An edit saved for a Game that is no longer active stays open

```gherkin
Given the dashboard shows the active Game and an entry of the story log is in edit with typed text
And the engine switched the active Game, and the page has not asked since
When the client presses Save
Then the engine refuses the save with 409 and "The active Game changed"
And the story log's error slot says that the active Game changed
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

#### Scenario 16.57: A switch that this page sent clears the Game that the page names, until the next ask

```gherkin
Given the dashboard shows Game 1 and the Games panel lists a second saved Game
When the client presses Switch for the second Game and the engine accepts it
Then the page names no Game until the next load of the screen
And a send in that window carries no X-Chronicler-Game header
And after the ask the page names the second Game
```

A create and a Reset that this page sent do the same.

#### Scenario 16.58: A Stop for a Game that is no longer active reports in the command form's slot

```gherkin
Given an Autoplay run holds the Game and Stop shows
And the engine switched the active Game, and the page has not asked since
When the client presses Stop
Then the engine refuses it with 409 and "The active Game changed"
And the command form's own inline error slot says that the run was not stopped because the active Game changed
```

#### Scenario 16.59: A Reset for a Game that is no longer active reports in the game row

```gherkin
Given the Games panel shows the active Game row
And the engine switched the active Game, and the page has not asked since
When the client confirms that row's Reset
Then the engine refuses it with 409 and "The active Game changed"
And the game row's own inline error slot says that the game was not reset because the active Game changed
```

#### Scenario 16.60: A load that started before a Game change that this page sent does not name the old Game

```gherkin
Given the dashboard shows Game 1 and a screen load that names Game 1 is in flight
When the client presses Switch for the second Game, the engine accepts it, and the old load ends
Then the page names no Game, and a send in that window carries no X-Chronicler-Game header
And after the next load the page names the second Game
```

#### Scenario 16.61: A preview send clears the Action input only if it still holds the text that the preview sent

```gherkin
Given the text check shows a preview of the command in the Action input
When the client presses Send Original, and types more text in the input while the request is in flight
And the engine takes the command and the screen shows the send
Then the Action input keeps the typed text
```
