# Feature Spec: Story Log

Endpoints:
- `POST /history/delete`, with the form field `message`: the id of the Message that the page rendered the control on
- `POST /history/:id`
- `GET /fragment/story-log`

## Scenarios

#### Scenario 8.1: Delete-last between actions — deleted narration stays absent

```gherkin
Given a Game whose story log holds only its opening Narration
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the Game is no longer busy (narration A is in the story log)
And the client POST /history/delete (deletes the last message — narration A)
And the client POST /action with command="look around"
And the Game is no longer busy (narration B is in the story log)
Then the story log holds 2 Input entries
And no Narration entry's text equals narration A's text
And the story log holds narration B
```

#### Scenario 8.2: Delete mid-sequence removes the targeted narration

```gherkin
Given a Game whose story log holds only its opening Narration
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the Game is no longer busy (narration A is in the story log)
And the client POST /action with command="look around"
And the Game is no longer busy (narration B is in the story log)
And the client POST /history/delete (deletes narration B — the last message)
And the client POST /action with command="check door"
And the Game is no longer busy (narration C is in the story log)
Then the story log holds exactly 3 Input entries
And no Narration entry's text equals narration B's text
```

#### Scenario 8.3: Retry after delete of last input does not leave state generating

```gherkin
Given a Game whose story log holds only its opening Narration
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the Game is no longer busy (the Input and its Narration are in the story log)
And the client POST /history/delete (deletes the last message — the narration)
And the client POST /swipe/new (retry with no anchor)
Then the response is not 500 INTERNAL_SERVER_ERROR
And within 1 s, the Game is no longer busy
```

#### Scenario 8.4: Delete-last removes the rendered log entry

```gherkin
Given a Game whose story log holds only its opening Narration
And a narrator backend that returns a non-empty narration for any prompt
When the client POST /action with command="examine room"
And the Game is no longer busy
And the client GET /fragment/story-log (at least 2 .log-entry rendered)
And the client POST /history/delete
Then the response is 200 with an empty body
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

#### Scenario 8.7: An edit and a delete while a Job runs are refused

```gherkin
Given a Game whose Job is held at its narrator call
When the client POST /history/:id with a new text
And the client POST /history/delete
Then each response is 409 carrying the "Still thinking..." line
And the Message the client tried to edit keeps the text it had
And the story log still holds every Message of the Job
```

#### Scenario 8.8: The story log shows the text while the analysis holds the Job

```gherkin
Given a Game whose Job is held at the analysis call of a Narration
When the client GET /fragment/story-log
Then the rendered log holds the Narration's text
And the client GET /status/busy answers the status "thinking" with the label "Quantifying..."
```

#### Scenario 8.9: An edit of a Message that does not exist answers 400

```gherkin
Given a Game whose story log holds its opening Narration
When the client POST /history/999999 with a new text
Then the response is 400 Bad Request carrying "Message not found"
And the story log holds every Message it held before
```

#### Scenario 8.10: A delete on the opening alone answers 400

```gherkin
Given a Game whose story log holds only its opening Narration
When the client POST /history/delete
Then the response is 400 Bad Request carrying "The story keeps its last message"
And the story log still holds the opening Narration
```

#### Scenario 8.11: The story log of a long story ends at the last Message

```gherkin
Given a Game whose story holds 53 Messages
When the client GET /fragment/story-log
Then the response renders exactly 50 .log-entry
And the last .log-entry is the last stored Message
And the first .log-entry is the fourth stored Message
And the Delete and Retry controls sit on the last .log-entry
```

#### Scenario 8.12: Delete-last in a long story removes the entry the player sees

```gherkin
Given a Game whose story holds 53 Messages
And the client GET /fragment/story-log
When the client POST /history/delete
Then the response status is "200 OK"
And the client GET /fragment/story-log renders exactly 50 .log-entry
And the last .log-entry is the stored Message before the deleted one
And no .log-entry holds the deleted Message
And the first .log-entry is the third stored Message
```

#### Scenario 8.13: In a long story the oldest entry of the story log shows its room

```gherkin
Given a Game whose story holds 53 Messages
And every Message holds the state of the same room "Test Room"
When the client GET /fragment/story-log
Then the response renders exactly 50 .log-entry
And the first .log-entry shows the room "Test Room"
And no other .log-entry shows a room
```

#### Scenario 8.14: The story log offers Retrigger on an Impersonate Input

```gherkin
Given a Game whose last Message is an Impersonate Input
And the state of that Input's Swipe holds an event record
When the client GET /fragment/story-log
Then the last .log-entry carries the `.retrigger-btn` control
And no other .log-entry carries it
```

#### Scenario 8.15: An edit answers nothing and the story log shows the new text

```gherkin
Given a Game whose last Message is a Narration
When the client POST /history/:id with the id of that Message and a new text
Then the response is 200 with an empty body
And the client GET /fragment/story-log shows the new text on that Message
```

#### Scenario 8.16: The story log sends its requests through htmx and renders the controls of an edit

```gherkin
Given a Game whose story ends on an Impersonate Input that offers Retrigger
When the client GET /fragment/story-log
Then each .log-entry renders .edit-actions with a .save-btn, a .cancel-btn and one error slot
And each .save-btn sends the edit of its Message with the text of #edit-textarea
And the last .log-entry renders .delete-btn, .retrigger-btn and the Retry control as htmx requests
And .delete-btn asks the player to confirm before it sends
And .delete-btn, .retrigger-btn and the Retry control name the last Message in the field message
When the last Message has two Swipes
Then the Previous swipe control and the Next swipe control are htmx requests that name the switch
And no button of the log calls a script that sends a request
```

#### Scenario 8.17: A delete that names a Message that is not the last one is refused

```gherkin
Given a Game whose story holds an Input and the Narration that answers it
When the client POST /history/delete with the field message set to the id of the Input
Then the response is 409
And the first line of the body is "The last Message changed, so the Message was not deleted."
And the second line of the body is "The page showed a Message that is no longer the last one."
And the story log still holds every Message
```
