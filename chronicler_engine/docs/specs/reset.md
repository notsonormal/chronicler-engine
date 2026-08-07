# Feature Spec: Reset

Endpoint: `POST /reset`

Behavioural authority for the reset endpoint — what a client observes
through HTTP. Resets the current game: deletes the current game row and
creates a fresh game from the world's starting scenario. Each "When" is
an HTTP request; each "Then" is an HTTP-observable outcome asserted via
`message_service.load_messages()`. Full state restoration (movement
room, scene NPCs) is covered at the unit tier — this spec asserts only
the HTTP-observable history-clearing.

Scenario IDs are `7.x` and stay stable across edits. The pilot dedups by
ID across `docs/specs/*.md`, so IDs must stay unique.

## Scenarios

#### Scenario 7.1: Reset clears the previous game's story-log history

```gherkin
Given a fresh game state with narrative.history empty
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the pipeline returns to idle
And message_service.load_messages() contains at least one Input entry
When the client POST /reset (deletes the current game and creates a fresh one)
Then message_service.load_messages() contains zero Input entries (the previous game's input is gone)
And message_service.load_messages() contains exactly 1 Narration entry (the fresh game's scenario opening)
```

#### Scenario 7.2: Action after reset produces a fresh input

```gherkin
Given a fresh game state with narrative.history empty
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the pipeline returns to idle
And the client POST /reset
And the client POST /action with command="look around"
And the pipeline returns to idle
Then message_service.load_messages() contains exactly 1 Input entry
And the Input entry's text is "look around"
```
