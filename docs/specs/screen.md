# Feature Spec: The screen load, action answers and the Game header

Endpoints:
- `GET /screen?parts=<names>` and `GET /screen?parts=<names>&games_page=<n>`
- every `POST` route
- the `GET` routes that open a dialog, for a record that is gone: `GET /fragment/games/:id/rename`, `GET /fragment/prompt-presets/:id/edit`, `GET /fragment/prompt-presets/:id/view`
- the Game-bound `POST` routes: `POST /action`, `POST /action/check`, `POST /action/confirm`, `POST /autoplay`, `POST /swipe/new`, `POST /retrigger`, `POST /reset`, `POST /message/:id/swipe/:index`, `POST /history/:id`, `POST /history/delete` and `POST /status/reset-busy`

The screen is the header, the failure banner and the parts of the visible tab. `GET /screen` renders the parts that the page names, in one answer. The part names are `header`, `failure-banner`, `story-log`, `visual-sidebar`, `options-dock`, `send-control`, `autoplay-controls`, `status-display`, `settings`, `prompt-presets`, `worlds`, `games` and `llm-messages`.

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
And the first line of the response body is "The Game is busy, so the Action was not sent."
And the second line of the response body is "Still thinking..."
And the response body holds no status markup
And the `server:revision` of GET /changes is the same before and after the request
```

#### Scenario 45.4: A stale Game header is refused on each Game-bound route

```gherkin
Given an active Game and a second saved Game
When the client sends each Game-bound POST route with the header X-Chronicler-Game set to the id of the second Game
Then each response status is 409
And the first line of each response body begins "The active Game changed, so "
And the second line of each response body is "The page showed a Game that is no longer active."
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

#### Scenario 45.8: A screen load answers one template for each named part, in the order asked

```gherkin
Given an engine with an active Game
When the client GET /screen?parts=status-display,header,story-log
Then the response status is 200
And the body holds one template with data-part="status-display", then one with data-part="header", then one with data-part="story-log"
And the header X-Revision has the form `<server>:<revision>`
And the header X-Game is the id of the active Game
```

#### Scenario 45.9: A part of the screen equals the answer of its own part route

```gherkin
Given an engine with an active Game
When the client GET /screen?parts=<name> for each part name
Then the content of the template of each part equals the body of its own route: GET /fragment/header, /fragment/failure-banner, /fragment/story-log, /fragment/visual-sidebar, /fragment/options-dock, /status/busy, /fragment/settings, /fragment/prompt-presets, /fragment/worlds, /fragment/games and /fragment/llm-messages
```

#### Scenario 45.10: A missing, unknown or repeated part name answers 400 and renders nothing

```gherkin
Given an engine with an active Game
When the client GET /screen with no parts query, with an empty one, with a part name that no part has, or with one name twice
Then each response status is 400
And each body holds no template
And the `server:revision` of GET /changes is the same before and after each request
```

#### Scenario 45.11: One part that fails to render fails the whole screen

```gherkin
Given an engine with an active Game
And the settings read fails
When the client GET /screen?parts=header,settings
Then the response status is 500
And the body names the failure
And the body holds no template
When the client GET /screen?parts=header
Then the response status is 200
```

#### Scenario 45.12: A screen carries the revision of the changes route, and a write moves it

```gherkin
Given an active Game and a second saved Game
When the client GET /screen?parts=header
Then the X-Revision of the answer equals the `server:revision` that GET /changes answers right after
When the client POST /games/:id/rename for the active Game with a new name
And the client GET /screen?parts=header
Then the revision of the new X-Revision is higher than the earlier one
And the template of the header holds the new name
```

#### Scenario 45.13: With no active Game the Game parts render empty and the other parts render

```gherkin
Given an engine with no active Game
When the client GET /screen?parts=header,story-log,visual-sidebar,options-dock,status-display,games
Then the response status is 200
And the header X-Game is empty
And the template of the header shows "No active game"
And the templates of story-log, visual-sidebar and options-dock are empty
And the template of status-display shows Ready
And the template of games holds the New Game control
```

#### Scenario 45.14: The Worlds part fails when the Games that it counts cannot be read

```gherkin
Given an engine with an active Game
And the read of the Games fails
When the client GET /screen?parts=worlds
Then the response status is 500
And the body names the failure
And the body holds no template
```

---

## Properties

- **S.1** `GET /screen` answers each part with the renderer of its own part route, so the screen and the part routes never differ. The two action control parts have no route of their own. The screen reads the revision before it renders, so it holds at least that revision.

These properties hold across every `POST` route and are observable through HTTP.
Drift indicates a regression even if all scenarios pass.

- **I.1** The answer carries `X-Revision: <server>:<revision>`. `<server>` and `<revision>` are the `server` and the `revision` of `GET /changes` (`docs/specs/changes.md`), read after the handler has ended. A success, a refusal and a failure all carry it. An action that wrote nothing carries the current stamp.
- **I.2** A Game that a Job or a run holds refuses a write with `409`. The first line of the body is "The Game is busy, so <the words of the route>." and the second line is "Still thinking...". The refusal writes nothing.

These properties hold across every Game-bound route (the endpoints above) and are observable through HTTP.

- **I.3** A request without the `X-Chronicler-Game` header acts on the active Game.
- **I.4** A request whose `X-Chronicler-Game` header does not name the active Game is refused with `409`. The first line of the body is "The active Game changed, so <the words of the route>." and the second line is "The page showed a Game that is no longer active.". The refusal writes nothing, and it comes before the handler runs.
- **I.5** A `GET` route does not read the `X-Chronicler-Game` header.

#### Scenario 45.15: A games_page that is not a whole number of at least 1 answers 400 and renders nothing

```gherkin
Given an engine with an active Game
When the client GET /screen?parts=games&games_page=<value> for each of "0", "-1", "two" and ""
Then each response status is "400 BAD_REQUEST"
And each body is an error fragment that names games_page
And no body holds a template
```

#### Scenario 45.16: A part whose read fails answers the failure of that read, not a configuration error

```gherkin
Given an engine with an active Game
And the read of the LLM messages fails
When the client GET /screen?parts=failure-banner, /screen?parts=settings and /screen?parts=llm-messages
Then each response status is 500
And each body names the failure of the read
And no body calls the failure a configuration error
```

#### Scenario 45.18: Each kind of failed action answers its status and one body

```gherkin
Given an engine with an active Game and a second saved Game
When the client POST /games/:id/rename with an empty name
Then the response is a 400

When the client POST /history/:id for a Message that does not exist
Then the response is a 400

When the client GET /fragment/games/:id/rename, GET /fragment/prompt-presets/:id/edit and GET /fragment/prompt-presets/:id/view for a record that does not exist
Then each response is a 404

When the client POST /reset with the header X-Chronicler-Game set to the id of the second Game
Then the response is a 409

When the client POST /worlds/:key/delete for a World that still has Games
Then the response is a 400 that holds the refusal "Cannot delete world with 1 game"

When the client POST /prompt-presets and the storage fails on save
Then the response is a 500 that holds the text of the failure

And each body is one `error-message` element
And no body starts with "Error: "
```

#### Scenario 45.17: A refusal names what was not done

```gherkin
Given an engine with an active Game and a second saved Game
When the client sends each Game-bound POST route with the header X-Chronicler-Game set to the id of the second Game
Then each response status is 409
And the first line of each body is "The active Game changed, so <the words of the route>."
And the second line of each body is "The page showed a Game that is no longer active."

Given a Job holds the Game, inside its narrator call
When the client POST /action, /action/confirm, /autoplay, /swipe/new, /retrigger, /history/:id, /history/delete and /message/:id/swipe/:index
Then each response status is 409
And the first line of each body is "The Game is busy, so <the words of the route>."
And the second line of each body is "Still thinking..."

Given an Autoplay run holds the Game
When the client POST /games/:id/mode, /games/:id/posture and /games/:id/presets for that Game
Then each response status is 409
And the first line of each body is "The Game is busy, so the change was not saved."
And the second line of each body is "Still thinking..."
```

#### Scenario 45.19: The first line of a failed answer says what was not done

```gherkin
Given an engine with an active Game and a second saved Game
When the client POST /games/:id/rename with an empty name
Then the response is a 400
And the body is one line, "The Game was not renamed: Display name cannot be empty."

When the client POST /games/:id/delete for the active Game
Then the response is a 400
And the first line of the body is "The Game "<its Display name>" was not deleted: Cannot delete the active game."

When the client POST /history/:id for a Message that does not exist
Then the first line of the body is "The edit was not saved: Message not found."
And the sentence names no item

When the client GET /fragment/games/:id/rename for a Game that does not exist
Then the response is a 404
And the first line of the body is "The form was not opened: Game not found."

When the client POST /prompt-presets with the name "Fail <b>" and the storage fails on save
Then the response is a 500
And the first line of the body is `The preset "Fail <b>" was not saved.`, with the name as escaped text
And the second line of the body holds the text of the failure

When the client POST /games/abc/rename
Then the response is a 400
And the first line of the body is "The Game was not renamed."
And the second line of the body holds the text of the rejection
```
