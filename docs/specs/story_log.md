# Feature Spec: Story Log

Endpoints:
- `POST /history/delete`
- `GET /fragment/story-log`

## Scenarios

#### Scenario 8.1: Delete-last between actions — deleted narration stays absent

```gherkin
Given a fresh game state with narrative.history empty
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the pipeline returns to idle (narration A persisted)
And the client POST /history/delete (deletes the last message — narration A)
And the client POST /action with command="look around"
And the pipeline returns to idle (narration B persisted)
Then message_service.load_messages() contains 2 Input entries
And no Narration entry's text equals narration A's text
And message_service.load_messages() contains narration B
```

#### Scenario 8.2: Delete mid-sequence removes the targeted narration

```gherkin
Given a fresh game state with narrative.history empty
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the pipeline returns to idle (narration A)
And the client POST /action with command="look around"
And the pipeline returns to idle (narration B)
And the client POST /history/delete (deletes narration B — the last message)
And the client POST /action with command="check door"
And the pipeline returns to idle (narration C)
Then message_service.load_messages() contains exactly 3 Input entries
And no Narration entry's text equals narration B's text
```

#### Scenario 8.3: Retry after delete of last input does not leave state generating

```gherkin
Given a fresh game state with narrative.history empty
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the pipeline returns to idle (input + narration persisted)
And the client POST /history/delete (deletes the last message — the narration)
And the client POST /swipe/new (retry with no anchor)
Then the response is not 500 INTERNAL_SERVER_ERROR
And within 1 s, message_service.load_or_fresh().narrative.input_buffer.status.is_generating() is false
```

#### Scenario 8.4: Delete-last removes the rendered log entry

```gherkin
Given a fresh game state with narrative.history empty
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the pipeline returns to idle
And the client GET /fragment/story-log (at least 2 .log-entry rendered)
And the client POST /history/delete
Then the response status is "200 OK"
And the client GET /fragment/story-log renders exactly one fewer .log-entry
```

#### Scenario 8.5: Story-log fragment declares no log container

```gherkin
Given a game state with at least one message entry
When the client GET /fragment/story-log
Then the response body contains .log-entry markup
And the response body contains no id="story-log" declaration
And the response body contains no class="story-log" declaration
```

#### Scenario 8.6: Every icon-only control in the story log has an accessible name

```gherkin
Given a game with at least 2 Messages and a stored trigger
And the last Message is a Narration with one Swipe
When the client GET /fragment/story-log
Then the response shows the Edit, Delete, Retrigger and Swipe controls
And every button that shows no text has an accessible name
And a button that has a tooltip is named by the same words
And every icon in the response is hidden from assistive technology
```
