# Feature Spec: Action answers and the Game header

Endpoints:
- every `POST` route
- the Game-bound `POST` routes: `POST /action`, `POST /action/check`, `POST /action/confirm`, `POST /autoplay`, `POST /swipe/new`, `POST /retrigger`, `POST /reset`, `POST /message/:id/swipe/:index`, `POST /history/:id`, `POST /history/delete` and `POST /status/reset-busy`

An action is a `POST` request. An action answers its outcome: a status code and the headers below. The client reads the data of an action from the screen, not from the answer.

## Scenarios

#### Scenario 45.1: Every action route answers with the revision stamp

```gherkin
Given an engine with an active Game and a second saved Game
When the client sends each POST route of the router, each with the body of its form
Then each answer carries X-Revision
And the stamp of the answer has the form `<server>:<revision>`

Given a Game whose Job is held at its narrator call
When the client sends each write that starts a Job
Then the first answer, the accepted Action, carries X-Revision
And each later answer, a refusal with 409, carries X-Revision
```

#### Scenario 45.2: The stamp of an answer is the stamp after its write

```gherkin
Given an active Game and a second saved Game
When the client POST /games/:id/rename for the active Game with a new name
Then the X-Revision of the answer equals the `server:revision` that GET /changes answers right after

When the client POST /games/:id/switch for the active Game
Then the response status is 200
And the X-Revision of the answer equals the `server:revision` that GET /changes answered before the request

When the client POST /games/:id/rename with an empty name
Then the response is a refusal
And the X-Revision of the answer equals the `server:revision` that GET /changes answers right after
```

#### Scenario 45.3: A busy refusal answers 409 with the wait line and writes nothing

```gherkin
Given a Game whose Job is running, held inside its narrator call
When the client POST /action with command="listen"
Then the response status is 409
And the response body carries "Still thinking..."
And the response body holds no status markup
And the header X-Refusal is "busy"
And the `server:revision` of GET /changes is the same before and after the request
```

#### Scenario 45.4: A stale Game header is refused on each Game-bound route

```gherkin
Given an active Game and a second saved Game
When the client sends each Game-bound POST route with the header X-Chronicler-Game set to the id of the second Game
Then each response status is 409
And each response body carries "The active Game changed"
And the header X-Refusal of each answer is "game-changed"
And each answer carries X-Revision
And the `server:revision` of GET /changes is the same before and after each request
And the story log holds the Messages it held before
```

#### Scenario 45.5: A stale Stop does not stop the Job of the active Game

```gherkin
Given a Game whose Job is running, held inside its narrator call
When the client POST /status/reset-busy with the header X-Chronicler-Game set to the id of another Game
Then the response status is 409
When the model answer arrives
Then the Job ends with its Narration
And the story log holds the opening Narration and the Narration of the Action
```

#### Scenario 45.6: A request without the header, or with the active Game, acts on the active Game

```gherkin
Given an active Game and a second saved Game
When the client POST /history/:id with a new text and no X-Chronicler-Game header
Then the response status is 200
And the Message holds the new text
When the client POST /history/:id with another new text and the header X-Chronicler-Game set to the id of the active Game
Then the response status is 200
And the Message holds the other new text
```

#### Scenario 45.7: A header that names no Game is refused, and a read ignores the header

```gherkin
Given an active Game
When the client POST /history/:id with the header X-Chronicler-Game set to "not-a-game"
Then the response status is 409
And the `server:revision` of GET /changes did not move

When the client GET /fragment/story-log with the header X-Chronicler-Game set to an id that is no Game
Then the response status is 200

Given no active Game
When the client POST /history/delete with the header X-Chronicler-Game set to "1"
Then the response status is 409
```

---

## Properties

These properties hold across every `POST` route and are observable through HTTP.
Drift indicates a regression even if all scenarios pass.

- **I.1** The answer carries `X-Revision: <server>:<revision>`. `<server>` and `<revision>` are the `server` and the `revision` of `GET /changes` (`docs/specs/changes.md`), read after the handler has ended. A success, a refusal and a failure all carry it. An action that wrote nothing carries the current stamp.
- **I.2** A Game that a Job or a run holds refuses a write with `409` and the line "Still thinking...". The answer carries `X-Refusal: busy`. The refusal writes nothing.

These properties hold across every Game-bound route (the endpoints above) and are observable through HTTP.

- **I.3** A request without the `X-Chronicler-Game` header acts on the active Game.
- **I.4** A request whose `X-Chronicler-Game` header does not name the active Game is refused with `409` and the line "The active Game changed". The answer carries `X-Refusal: game-changed`. The refusal writes nothing, and it comes before the handler runs.
- **I.5** A `GET` route does not read the `X-Chronicler-Game` header.
