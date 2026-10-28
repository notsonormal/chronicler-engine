# Feature Spec: Browser Screen

Endpoint: browser DOM.

The dashboard shows one **screen**: the header, the failure banner and the visible tab. The screen is made of **parts**. A part is an element with a `data-part` attribute that names it. The shell names the parts of the header and the failure banner, and the parts of each tab:

| Section | Parts |
|---|---|
| none | `header`, `failure-banner` |
| Game | `story-log`, `visual-sidebar`, `options-dock`, `send-control`, `autoplay-controls`, `status-display` |
| Settings | `settings` |
| Prompt Presets | `prompt-presets` |
| Worlds | `worlds` |
| Games | `games` |
| LLM Messages | `llm-messages` |

The page asks `GET /changes` (`docs/specs/changes.md`) while it is visible. The answer holds the `server` and the `revision` of the engine. The page shows the **stamp** `<server>:<revision>` of its last good load in `<html data-revision>`, and the id of the Game that the load showed in `<html data-game>`. When the stamp of the answer differs from the stamp of the page, one `GET /screen?parts=<names>` (`docs/specs/screen.md`) loads the header, the failure banner and the parts of the visible tab, and a direct morph writes each part. The interval of the ask is `<html data-poll-ms>`. While an ask or a load is in flight, `<html>` has the attribute `data-screen-busy`.

A tab's parts exist only while the tab is open. A tab change empties each part of the tab that closes and loads the tab that opens. No part holds a draft: a form that waits for a Save is in the edit dialog, outside every part. A load keeps what it does not change, and it skips the one element that holds unsaved input: the story-log entry in edit (`data-editing`), and a control whose request is in flight. A load carries the tab that was visible and the page of the saved Games that was asked for (`<html data-games-page>`, `docs/specs/browser_games.md`) when it started; an answer for another tab or page is dropped. A load always records the stamp of what it rendered. The end of every action request forgets the stamp, so the next ask loads the screen once.

A non-2xx answer of `GET /changes` or `GET /screen` shows in `#screen-error`, and every part keeps its content. Only a network error shows the Unreachable banner.

The scenarios start on the Game tab, with the page quiet: the stamp of the page equals the stamp of the engine and no ask or load is in flight.

## Scenarios

#### Scenario 44.1: A change loads the visible screen with one request

```gherkin
Given the dashboard is loaded on the Game tab and the page is quiet
When the revision moves, whichever kind of data changed, and the page asks
Then one GET /screen names the header, the failure banner and the parts of the Game tab
And each of those parts loads once
And no part of another section loads
```

#### Scenario 44.2: An unchanged answer loads nothing

```gherkin
Given the dashboard is loaded and the page is quiet
When the page asks and the revision did not move
Then no GET /screen is sent
```

#### Scenario 44.3: A new server id loads the screen

```gherkin
Given the dashboard is loaded and the page is quiet
When the answer carries a different `server` and the page asks
Then the visible screen loads once
And the stamp of the page names the new server
```

#### Scenario 44.4: A new active Game loads the screen and the page names it

```gherkin
Given the dashboard is loaded and the page is quiet, and the screen names the active Game
When the active Game changes, so the revision moves, and the page asks
Then the visible screen loads once
And `<html data-game>` is the id of the new active Game
```

#### Scenario 44.5: A network error shows the Unreachable banner, and any answer clears it

```gherkin
Given the dashboard is loaded and the page is quiet
When the engine stops and the page asks
Then #failure-banner is visible with the role alert, and says that the engine is unreachable
When the engine answers again and the page asks
Then #failure-banner is hidden
```

#### Scenario 44.6: A non-2xx answer of the ask shows in the screen error, and a good ask clears it

```gherkin
Given the dashboard is loaded and the page is quiet
When GET /changes answers 500 and the page asks
Then #screen-error shows "GET /changes answered 500"
And #failure-banner is hidden, because the engine answered
When GET /changes answers again and the page asks
Then #screen-error is hidden
```

#### Scenario 44.7: A hidden page does not ask

```gherkin
Given the dashboard is loaded and the browser reports the page hidden
When the timer of the page ticks several times
Then the page does not send GET /changes
When the browser reports the page visible
Then the page asks on its next tick
```

#### Scenario 44.8: A page that becomes visible asks at once

```gherkin
Given the dashboard is loaded, the browser reports the page hidden, and the revision moved
When the browser reports the page visible
Then the page sends GET /changes without waiting for the timer
And the visible screen loads once
```

#### Scenario 44.9: A load skips the entry in edit and shows the rest of the log

```gherkin
Given an edit is open in #story-log with typed text, and the textarea has focus
And the log has a new narration entry
When the revision moves three times, and the page asks after each move
Then #story-log loads once for each move
And the textarea is the same element, with the typed text and focus
And the new narration entry shows in the log
When the edit ends
Then the textarea is gone
```

#### Scenario 44.10: A load that was in flight when an edit began skips the entry in edit

```gherkin
Given a load of the screen is in flight, and its answer holds a new narration entry for #story-log
When an edit opens in #story-log before the answer arrives
And the answer arrives
Then the answer is morphed in around the entry in edit
And the textarea is the same element, with its text and focus
```

#### Scenario 44.11: A disclosure that lists its open state in data-keep stays open through a load

```gherkin
Given #options-dock shows an Options failure with its Details disclosure open
When the revision moves, and the page asks
Then #options-dock loads
And the Details disclosure is the same element
And it is still open
```

#### Scenario 44.12: Focus moves to a surviving row when a load removes the focused node

```gherkin
Given the focus is on a control in the newest .log-entry of #story-log
When a load of #story-log no longer holds that entry
Then the focus is on a control in the entry that is now the newest
```

#### Scenario 44.13: A load keeps the engaged state of each part

```gherkin
Given each part of the screen, with its section open
And every interactive control of the part is in its engaged state:
  an open edit with typed text and focus, a focused Option, an open Details disclosure, a text selection,
  a focused Open control of an LLM message, an open error Details popover in the failure banner,
  the chosen Settings sub-tab, each with typed text and focus where it has any
When the revision moves, once for each kind of data that the part shows, and the page asks after each move
Then the part loads once for each move
And each node, value, open state, selection and focus of the engaged state is as it was
```

#### Scenario 44.14: At page start the screen of the Game tab loads exactly once

```gherkin
Given the dashboard shell is served and the revision does not move
When the page starts
Then the page asks once without waiting for the timer
And one GET /screen names the header, the failure banner and the parts of the Game tab
And each part of another section holds no content
```

#### Scenario 44.15: An action asks at once when its request ends

```gherkin
Given the dashboard is loaded, the page is quiet, and the timer of the page does not run
When an htmx request ends, for example the Stop control
Then the page sends GET /changes at once
And the screen loads and shows what the action changed

When the Delete control of the story log ends
Then the page sends GET /changes at once
And the story log no longer shows the deleted entry

When an action ends that wrote nothing, for example a Retrigger that fails
Then the page sends GET /changes at once
And the screen loads once
```

#### Scenario 44.16: A tab change empties the closing tab and loads the opening tab once

```gherkin
Given the dashboard is loaded on the Game tab and the page is quiet
When the client opens the Games tab
Then GET /screen names the header, the failure banner and the `games` part, and the `games` part loads exactly once
And each part of the Game tab holds no content
When the revision moves while the Games tab is open
Then no part of the Game tab loads
When the client opens the Game tab
Then the story log loads exactly once
And the `games` part holds no content
```

#### Scenario 44.17: A save that fails keeps the dialog open with its typed values

```gherkin
Given the rename form of a saved Game is open in the edit dialog, with a changed name
When the client presses Save and the engine answers 500
Then #screen-error shows "The Game was not renamed."
And the dialog status line repeats that sentence
And the dialog is still open with the same form, the changed name and focus
```

#### Scenario 44.18: A control that saves on change goes back to its rendered value when the save fails

```gherkin
Given the Tense select of #game-posture-controls shows "past"
When the client changes it to "present", and the engine answers 500
Then the Tense select shows "past" again
And #screen-error shows "The change was not saved."
```

#### Scenario 44.19: A load keeps the notice of an action

```gherkin
Given the `games` part is open
And a failed action shows its report in #screen-error
When the revision moves, and the page asks
Then the `games` part loads
And #screen-error still shows the same report
When the client sends the next action
Then #screen-error is hidden
```

#### Scenario 44.20: A preset form in the dialog survives a Prompt Presets load, and Cancel sends no request

```gherkin
Given the `prompt-presets` part is open
And the edit form of a preset is open in the edit dialog, with a changed name and focus in the name field
When the revision moves, and the page asks
Then the `prompt-presets` part loads exactly once
And the form is the same element, with its changed name and focus
When the client cancels the dialog
Then Cancel sent no request
```

#### Scenario 44.21: A World form in the dialog survives a Game rename

```gherkin
Given the `worlds` part is open
And the World edit form is open in the edit dialog, with a changed name and focus
When a Game is renamed, so the revision moves, and the page asks
Then the form is the same element, with its changed name and focus
And the `worlds` part loads exactly once
When the client cancels the dialog
Then the dialog is closed
```

#### Scenario 44.22: A connection form in the dialog survives a Settings load, and Cancel sends no request

```gherkin
Given the `settings` part is open, with the Text Check sub-tab chosen
And the connection form is open in the edit dialog, with a typed name and focus
When the revision moves, and the page asks
Then the `settings` part loads exactly once
And the form is the same element, with its typed name and focus
When the client cancels the dialog
Then Cancel sent no request
And the Text Check sub-tab is still the chosen one
```

#### Scenario 44.24: A load skips a select whose request is in flight

```gherkin
Given the `settings` part is open
And the client chose another connection in #role-select-quantifier, and the request of the save is in flight
And the engine still renders the old connection
When the revision moves, and the `settings` part loads
Then #role-select-quantifier still shows the client's choice
And its request is still in flight
And the stamp of the page is the stamp of the engine
```

#### Scenario 44.25: After the save, the next load sets the select to the server value

```gherkin
Given the client chose another connection in #role-select-quantifier, and the request of the save is in flight
And the engine answers the save but keeps the connection that it had
When the request ends and the revision moves
Then the `settings` part loads
And #role-select-quantifier no longer reports htmx-request
And #role-select-quantifier shows the connection that the engine renders, not the client's choice
```

#### Scenario 44.26: No part of any tab holds a draft control

```gherkin
Given the dashboard is loaded
When the client opens each tab in turn
Then no part of the tab holds a form that waits for a Save
And a control that saves on change is not such a form
```

#### Scenario 44.27: A load that skipped a control in flight records the stamp, and the end of the request loads again

```gherkin
Given the `settings` part is open
And the client chose another connection in #role-select-quantifier
And the engine committed the save, and kept the connection that it had, but its answer has not come yet
When the screen loads
Then #role-select-quantifier still shows the client's choice
And the stamp of the page is the stamp of the engine
When the answer of the save comes
Then the screen loads again
And #role-select-quantifier shows the connection that the engine renders
And the stamp of the page is the stamp of the engine
```

#### Scenario 44.28: A tab change during a load drops the old answer

```gherkin
Given a load of the Game tab is in flight
When the client opens the Games tab, and the old answer arrives
Then the old answer is dropped, and the parts of the Game tab hold no content
And the visible tab loads, and the `games` part shows its content
```

#### Scenario 44.29: Leaving the Game tab discards the story-log edit and its lock

```gherkin
Given an edit is open in #story-log, and the Edit buttons of the other entries are locked
When the client opens another tab and returns to the Game tab
Then the edit is gone
And #story-log is not locked
And a new edit opens
```

#### Scenario 44.30: A load makes a new row's post work and a changed post use its new value

```gherkin
Given the header part holds a button with hx-post="/old"
When a screen load answers the header with the same button changed to hx-post="/changed" and a new button with hx-post="/added"
Then the new button posts to "/added"
And the kept button, the same node, posts to "/changed"
```

#### Scenario 44.31: A direct morph applies the keep rules

```gherkin
Given a container holds an element with data-keep naming data-view, and a story-log entry in edit
When the screen script morphs the answer of the server into it
Then the kept attribute keeps its value
And the entry in edit keeps its draft
```

#### Scenario 44.32: A direct morph skips a control in flight

```gherkin
Given a button of a container sends a request that has not ended
When the screen script morphs a new text into it
Then the button keeps its text
When the request has ended and the screen script morphs the text again
Then the button shows the new text
```

#### Scenario 44.34: The dialog says Saving until the screen shows the save

```gherkin
Given the rename form of a saved Game is open in the edit dialog
And a screen load is held
When the client presses Save and the engine accepts the save
Then the dialog says "Saving…" and stays open
When the screen loads and shows the save
Then the dialog closes and the Games part shows the new name
```

#### Scenario 44.35: The action input clears when the screen shows the send, and only if it still holds the sent text

```gherkin
Given a screen load is held
When the client sends a command and the engine takes it
Then the Action input keeps its text
When the client types more text and the screen then shows the send
Then the Action input keeps the new text
When the client sends a command, leaves the input unchanged, and the screen shows the send
Then the Action input is empty
```

#### Scenario 44.36: A reload restores the active tab and the Settings sub-tab

```gherkin
Given the client opened the Settings tab and chose the Text Check sub-tab
When the page reloads
Then the Settings tab is the active tab, and the Text Check sub-tab is the chosen one
When the client opens the Game tab and then the Settings tab again
Then the Text Check sub-tab is the chosen one, although the `settings` part was empty while its tab was closed
```

#### Scenario 44.37: A save that the screen shows late does not close another dialog

```gherkin
Given the rename form is open in the edit dialog, and a screen load is held
When the client presses Save, and the dialog says "Saving…"
And the client presses Escape, opens the New Game form, and chooses a World and a Persona
And the held load ends, and the screen shows the save
Then the New Game dialog is still open, with its form and its choice
```

#### Scenario 44.38: A second Save while the dialog says Saving sends no request

```gherkin
Given the rename form is open in the edit dialog, and a screen load is held
When the client presses Save, and the dialog says "Saving…"
And the client submits the form again, by the Save control and by the keyboard
Then the engine received the save once
When the held load ends
Then the dialog closes
```

#### Scenario 44.39: Each move of the revision loads the screen once while a request of a part runs

```gherkin
Given the `settings` part is open
And the client pressed Test for a saved connection, and the request has not ended
When the revision moves, and the page asks
Then the screen loads once, and the Test control keeps its state
When the revision moves twice more, and the page asks after each move
Then the screen loads once after each move
When the request ends
Then the screen loads once more, and the stamp of the page is the stamp of the engine
```

#### Scenario 44.40: A wait for a stamp of another server ends only at a load that started after the wait

```gherkin
Given a screen load is in flight and the page shows a stamp of server A
And an action answer carries a stamp of server B
When the page starts to wait for the stamp of server B, and the load that was in flight ends and shows server A
Then the wait is not over
When the engine answers as server B and the screen loads
Then the wait is over
```

#### Scenario 44.41: An answer that the page cannot read or show gives the screen error

```gherkin
Given the dashboard is loaded and the page is quiet
When the answer of GET /changes is not JSON, and the page asks
Then #screen-error names GET /changes
When the answer of GET /changes is good again, and the page asks
Then #screen-error is empty
When the morph of a screen load throws, and the page asks
Then #screen-error names GET /screen
When the morph works again, and the page asks
Then #screen-error is empty
```

#### Scenario 44.42: A restart of the engine before the load of a save closes the dialog

```gherkin
Given the rename form is open in the edit dialog, and a screen load is held
When the client presses Save, and the dialog says "Saving…"
And the engine restarts with another server id, and the held load ends
Then the stamp of the page names the new server
And the dialog closes
```

#### Scenario 44.43: A save followed by a failed screen load keeps the dialog at Saving until the next good load

```gherkin
Given the rename form is open in the edit dialog, and a screen load is held
When the client presses Save, and the held load answers with a failure
Then #screen-error names GET /screen
And the dialog says "Saving…" and names the failure, and stays open
When the next load is good
Then the dialog closes, and #screen-error is empty
```

#### Scenario 44.44: A screen failure that lasts keeps its open Details and its node

```gherkin
Given the dashboard is loaded and the page is quiet
When each load of the screen fails with the same answer
And #screen-error shows the failure and the client opens its Details
And the page makes more loads that fail with the same answer
Then the Details of #screen-error stays open
And the disclosure node in #screen-error is the node that showed before
```

#### Scenario 44.45: Each form with typed input renders one slot, and no other element holds a slot

```gherkin
Given the stub renders each tab and each dialog opener
When the client loads each tab and opens each dialog opener
Then every form of the edit dialog, #command-form and #autoplay-form holds one error slot as a direct child
And every error slot is a direct child of such a form, or the report container of #screen-error
And every `.edit-actions` group of a story-log entry holds one error slot as a direct child, also while an entry is in edit
And no other element holds an error slot
```
