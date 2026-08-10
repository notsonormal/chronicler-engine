# Feature Spec: Story Log

Endpoint: `POST /history/delete`

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
