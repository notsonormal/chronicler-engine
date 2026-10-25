# Feature Spec: Browser Live Regions

Endpoint: browser DOM.

The dashboard shows server data in **live regions**. A live region is an element with a `data-live` attribute. The attribute lists the kinds of data the region shows. The kinds are the names of the numbers of `GET /changes` (`docs/specs/changes.md`): `story`, `options`, `busy`, `games`, `worlds`, `settings`, `presets` and `llm`.

The page asks `GET /changes` while it is visible. A region loads its own fragment when a number of one of its kinds differs from the number that the region last loaded. A region keeps what its load does not change. A region that holds an edit waits, and loads once when the edit ends. The region reports its state in `data-live-state`: `current`, `loading`, `held` or `due`.

The Game tab regions of this spec are `#header` (`games`, `settings`, `llm`), `#story-log` (`story`, `busy`, `worlds`), `.visual-sidebar` (`story`, `worlds`), `#options-dock` (`options`, `busy`) and `#status-display` (`busy`).

Each scenario asks the page once, through the page function that runs one ask and returns when every load of that ask has settled. The timer of the page does not run in these scenarios.

## Scenarios

#### Scenario 44.1: A moved number loads the regions that list its kind and no other region

```gherkin
Given the dashboard is loaded and every region is current
When the number of one kind moves, and the page asks
Then each region whose data-live lists that kind loads its fragment once
And each other region does not load
And the kind `presets` loads no region of the Game tab
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
Then every region loads its fragment once
```

#### Scenario 44.4: A new active Game loads every region

```gherkin
Given the dashboard is loaded and every region is current
When the answer carries a different `game` and the page asks
Then every region loads its fragment once
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

#### Scenario 44.13: A load keeps the engaged state of each Game tab region

```gherkin
Given each of #header, #story-log, .visual-sidebar, #options-dock and #status-display
And every interactive control of the region is in its engaged state:
  an open edit with typed text and focus, a focused Option, an open Details disclosure, a text selection
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
And each region loads its fragment exactly once
And each region reports data-live-state "current"
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
