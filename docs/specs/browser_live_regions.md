# Feature Spec: Browser Live Regions

Endpoint: browser DOM.

The dashboard shows server data in **live regions**. A live region is an element with a `data-live` attribute. The attribute lists the kinds of data the region shows. The kinds are the names of the numbers of `GET /changes` (`docs/specs/changes.md`): `story`, `options`, `busy`, `games`, `worlds`, `settings`, `presets` and `llm`.

The page asks `GET /changes` while it is visible. A region loads its own fragment when a number of one of its kinds differs from the number that the region last loaded. A region keeps what its load does not change. A region that holds an edit waits, and loads once when the edit ends. A disclosure that closes without a save discards its edit, so it ends the edit. A region in a hidden dashboard section does not load: it waits, and loads once when the player opens the section. The region reports its state in `data-live-state`: `current`, `loading`, `held`, `stale` or `due`.

The header is outside every section, so it is always visible. The Game tab is a section too, and it is hidden while the player is on another section.

The regions of this spec:

| Region | Section | Kinds in `data-live` |
|---|---|---|
| `#header` | none | `games`, `settings`, `llm` |
| `#story-log` | Game | `story`, `busy`, `worlds` |
| `.visual-sidebar` | Game | `story`, `worlds` |
| `#options-dock` | Game | `options`, `busy` |
| `#status-display` | Game | `busy` |
| `.games-panel` | Games | `games`, `worlds`, `presets` |
| `.llm-messages-panel` | LLM Messages | `llm` |

The scenarios start on the Game tab. A scenario that does not open another section sees the regions of the Game tab and the header load, and the regions of the other sections stay `stale`.

Each scenario asks the page once, through the page function that runs one ask and returns when every load of that ask has settled. The timer of the page does not run in these scenarios.

## Scenarios

#### Scenario 44.1: A moved number loads the regions that list its kind and no other region

```gherkin
Given the dashboard is loaded and every region is current
When the number of one kind moves, and the page asks
Then each region whose data-live lists that kind, and that is not in a hidden section, loads its fragment once
And each other region does not load
And the kind `presets` loads no region of the Game tab and no region of the header
```

#### Scenario 44.2: An unchanged answer loads nothing

```gherkin
Given the dashboard is loaded and every region is current
When the page asks and no number moved
Then no region loads
And every region reports data-live-state "current"
```

#### Scenario 44.3: A new server id loads every region

```gherkin
Given the dashboard is loaded and every region is current
When the answer carries a different `server` and the page asks
Then every region that is not in a hidden section loads its fragment once
And each region in a hidden section reports data-live-state "stale"
```

#### Scenario 44.4: A new active Game loads every region

```gherkin
Given the dashboard is loaded and every region is current
When the answer carries a different `game` and the page asks
Then every region that is not in a hidden section loads its fragment once
And each region in a hidden section reports data-live-state "stale"
```

#### Scenario 44.5: A failed ask shows the Unreachable banner, and a good ask clears it

```gherkin
Given the dashboard is loaded and every region is current
When GET /changes fails and the page asks
Then #failure-banner is visible and says that the engine is unreachable
And no region loads
When GET /changes answers again and the page asks
Then #failure-banner is hidden
```

#### Scenario 44.6: A failed region load is tried again on the next ask

```gherkin
Given the dashboard is loaded and every region is current
When the number of `story` moves, and the load of #story-log fails, and the page asks
Then #story-log keeps its content and reports data-live-state "due"
When the load of #story-log works and the page asks, with no further number moved
Then #story-log loads once more and shows the new content
And #story-log reports data-live-state "current"
```

#### Scenario 44.7: A hidden page does not ask

```gherkin
Given the dashboard is loaded and the browser reports the page hidden
When the timer of the page ticks
Then the page does not send GET /changes
```

#### Scenario 44.8: A page that becomes visible asks at once

```gherkin
Given the dashboard is loaded, the browser reports the page hidden, and a number moved
When the browser reports the page visible
Then the page sends GET /changes without waiting for the timer
And each region that lists the moved kind loads its fragment
```

#### Scenario 44.9: An edit holds its region, which loads once when the edit ends

```gherkin
Given an edit is open in #story-log with typed text, and the textarea has focus
When the number of `story` moves three times, and the page asks after each move
Then #story-log does not load
And #story-log reports data-live-state "held"
And the textarea is the same element, with the typed text and focus
When the edit ends
Then #story-log loads exactly once
And #story-log reports data-live-state "current"
```

#### Scenario 44.10: A load that was in flight when an edit began does not replace the edit

```gherkin
Given a load of #story-log is in flight
When an edit opens in #story-log before the answer arrives
And the answer arrives
Then the answer is not swapped in
And the textarea is the same element, with its text and focus
And #story-log reports data-live-state "held"
```

#### Scenario 44.11: A disclosure that lists its open state in data-keep stays open through a load

```gherkin
Given #options-dock shows an Options failure with its Details disclosure open
When the number of `options` moves, and the page asks
Then #options-dock loads its fragment
And the Details disclosure is the same element
And it is still open
```

#### Scenario 44.12: Focus moves to a surviving row when a load removes the focused node

```gherkin
Given the focus is on a control in the newest .log-entry of #story-log
When a load of #story-log no longer holds that entry
Then the focus is on a control in the entry that is now the newest
```

#### Scenario 44.13: A load keeps the engaged state of each region

```gherkin
Given each region of the table above, with its section open
And every interactive control of the region is in its engaged state:
  an open edit with typed text and focus, a focused Option, an open Details disclosure, a text selection,
  an open rename form with typed text, a chosen option of a select, an expanded LLM message,
  an open error Details popover in the failure banner, the chosen Settings sub-tab,
  an open connection editor, a preset card in edit, an open World editor, each with typed text and focus
When each kind that the region lists moves, one kind at a time, and the page asks
Then each node, value, open state, selection and focus of the engaged state is as it was
When every edit of the region ends
Then the region loads exactly once, and it reports data-live-state "current"
```

#### Scenario 44.14: At page start each region loads exactly once

```gherkin
Given the dashboard shell is served and no number moves
When the page starts
Then the page asks once without waiting for the timer
And each region of the header (the header and the failure banner) and the Game tab loads its fragment exactly once
And each of those regions reports data-live-state "current"
And each region of another section does not load, and reports data-live-state "stale"
```

#### Scenario 44.15: An action asks at once when its request ends

```gherkin
Given the dashboard is loaded, every region is current, and the timer of the page does not run
When an htmx request that is not the load of a region ends, for example the Stop control
Then the page sends GET /changes at once
And each region that lists a kind the action moved loads its fragment

When a script action ends, for example the Delete control of the story log
Then the page sends GET /changes at once
And the story log loads its fragment and no longer shows the deleted entry
```

#### Scenario 44.16: A region in a hidden section does not load, and the section loads it once when it opens

```gherkin
Given the dashboard is loaded on the Game tab and every region of the Game tab is current
And .games-panel and .llm-messages-panel have not loaded
When the number of `games` moves, and the page asks
Then #header loads its fragment once
And .games-panel does not load
And .games-panel reports data-live-state "stale"
When the number of `llm` moves, and the page asks
Then .llm-messages-panel does not load, and it reports data-live-state "stale"
When the client opens the Games tab
Then .games-panel loads its fragment exactly once
And .games-panel reports data-live-state "current"
And each region of the Game tab reports data-live-state "current"
When the number of `story` moves, and the page asks
Then #story-log does not load, and it reports data-live-state "stale"
When the client opens the Game tab
Then #story-log loads its fragment exactly once
```

#### Scenario 44.17: A successful save ends the edit of its form, and a disclosure marked data-close-on-save closes

```gherkin
Given .games-panel is open and current
And the rename form of a saved Game is open inside its `details` marked `data-close-on-save`, with a changed name
Then .games-panel reports data-live-state "held"
When the client presses Save and the engine answers 200
Then the name field is no longer a changed field: its default value is its current value
And the `details` is closed
And the name field still shows the saved name, because the close that follows a save discards nothing
And .games-panel loads its fragment exactly once, and it reports data-live-state "current"
```

#### Scenario 44.18: A save that fails keeps the edit open, and a control that saves on change goes back to its rendered value

```gherkin
Given .games-panel is open and current
And the rename form of a saved Game is open, with a changed name
When the client presses Save and the engine answers 500
Then the form's own error slot shows the failure
And the name field keeps the changed name
And the `details` is still open
And .games-panel reports data-live-state "held"

Given the Tense select of #game-posture-controls shows "past"
When the client changes it to "present", and the engine answers 500
Then the Tense select shows "past" again
And the error slot of #game-posture-controls shows the failure
And .games-panel does not report data-live-state "held"
```

#### Scenario 44.19: A load keeps an inline error that is showing

```gherkin
Given .games-panel is open and current
And a failed action shows an error in an inline error slot inside .games-panel
When the number of `games` moves, and the page asks
Then .games-panel loads its fragment
And the inline error slot still shows the same error
When the same action succeeds
Then the inline error slot is empty and hidden
```

#### Scenario 44.20: A preset card in edit survives the Activate of another card

```gherkin
Given .prompt-presets-panel is open and current
And a preset card is in edit, with a changed name and focus in the name field
When the client presses Set Active on another preset card, and the engine answers 200
Then the page asks at once
And the card in edit is the same element, with its changed name
And .prompt-presets-panel reports data-live-state "held" and has not loaded
When the client cancels the edit
Then .prompt-presets-panel loads exactly once, and it reports data-live-state "current"
And the other card shows its Active badge
```

#### Scenario 44.21: A World editor survives a Game rename that raises games

```gherkin
Given .worlds-panel is open and current
And the World editor is open in #world-editor, with a changed name and focus
When a Game is renamed, so the number of `games` moves, and the page asks
Then the editor is the same element, with its changed name
And .worlds-panel reports data-live-state "held" and has not loaded
When the client closes the editor with Cancel
Then .worlds-panel loads exactly once, and it reports data-live-state "current"
And #world-editor is empty
```

#### Scenario 44.22: A connection editor keeps the chosen sub-tab, and Cancel sends no request

```gherkin
Given .settings-panel is open and current, with the Text Check sub-tab chosen
And the connection editor is open in #connection-editor
When the number of `settings` moves, and the page asks
Then .settings-panel reports data-live-state "held" and has not loaded
When the client cancels the editor
Then Cancel sent no request, and .settings-panel loads exactly once
And the Text Check sub-tab is still the chosen one
```

#### Scenario 44.23: A disclosure closed without a save discards its edit, and its region loads at once

```gherkin
Given .games-panel is open and current
And the rename form of a saved Game is open, with a changed name
And the number of `worlds` moved, and the page asked
And .games-panel reports data-live-state "held"
When the client closes the `details` without pressing Save
Then the name field shows the rendered name again
And the page asks at once, without waiting for its timer
And .games-panel loads its fragment exactly once, and it reports data-live-state "current"

Given .prompt-presets-panel is open and current
And an Add form of a preset type is open, with a typed name
And the number of `presets` moved, and the page asked
And .prompt-presets-panel reports data-live-state "held"
When the client closes the Add form without pressing Add Preset
Then the name field is empty again
And .prompt-presets-panel loads its fragment exactly once, and it reports data-live-state "current"
```
