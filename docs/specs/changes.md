# Feature Spec: Change numbers

Endpoints:
- `GET /changes`

The server keeps one number for each **kind of data**. A number only goes up. It goes up once the data it counts is committed, so a client that reads a changed number then reads the new data. A read, a failed save and a rolled-back transaction raise no number.

`GET /changes` answers one JSON object with no HTML:

| Field | Meaning |
|---|---|
| `server` | The id of this server process. It is the same in every answer of one process. |
| `game` | The id of the active Game, or `null` with no active Game. |
| `story`, `options`, `busy` | The numbers of the active Game: its story, its Options, and its busy state with its failure line. The answer holds no per-Game number with no active Game. |
| `games`, `worlds`, `settings`, `presets`, `llm` | The numbers of the Games list, the Worlds, the settings, the prompt presets and the LLM messages. They count the whole engine. |

What raises each number:

| Number | Raised by |
|---|---|
| `story` | A write of a Message or a Swipe of the Game: a new Message, a new Swipe, a text, a Swipe state, or the active Swipe. |
| `options` | A write of the dock record of the Game: the offered set, the Options failure, or their removal. |
| `busy` | A change of the busy state of the Game, such as the start of a Job, a new step or the end of a Job. Also a write of the failure line of the Game. |
| `games` | A new, a removed or a renamed Game, a change of the config of a Game, a Game that becomes active, a new or a removed Message of any Game, and the start or the end of an Autoplay run. A change of the step of a run does not raise it. |
| `worlds` | A new, an edited or a removed World. |
| `settings` | A save of the settings: a Connection, a Role, the text check, or an active prompt preset. |
| `presets` | A new, an edited or a removed prompt preset. |
| `llm` | A new LLM message. |

A new or a removed Message raises `games` because the Games list is ordered by the newest Message of each Game.

The scenarios read `GET /changes` before and after one step, and name the numbers that moved. A number moved when its value differs. Each save scenario lists its save paths in a table, and each row names the numbers it moves. A number that no row names stays as it was.

## Scenarios

#### Scenario 43.1: The answer holds the server id, the active Game and a number for each kind

```gherkin
Given an active Game
When the client GET /changes
Then the response status is 200
And the response body is a JSON object
And its `server` is a non-empty string
And its `game` is the id of the active Game
And its `story`, `options`, `busy`, `games`, `worlds`, `settings`, `presets` and `llm` are whole numbers

Given no active Game
When the client GET /changes
Then the response status is 200
And its `game` is null
And the response holds no `story`, `options` or `busy`
And its `games`, `worlds`, `settings`, `presets` and `llm` are whole numbers
```

#### Scenario 43.2: The server id is the same in every answer of one server, and differs between two servers

```gherkin
Given a server
When the client GET /changes twice, with a save between the two requests
Then the `server` of both answers is the same

Given a second server
When the client GET /changes on each server
Then the two `server` values differ
```

#### Scenario 43.3: A read moves no number

```gherkin
Given an active Game with a story, a World, a prompt preset and an LLM message
When the client reads, one after the other:
  GET /changes, GET /fragment/header, GET /fragment/failure-banner, GET /fragment/story-log, GET /fragment/visual-sidebar,
  GET /fragment/options-dock, GET /fragment/games, GET /fragment/worlds, GET /fragment/settings,
  GET /fragment/prompt-presets, GET /fragment/llm-messages, GET /status/busy and GET /debug/state
Then each response status is 200
And no number of GET /changes moved after any of them
```

#### Scenario 43.4: A save of a Game moves the Games list only

```gherkin
Given two Games, the first one active
When the client saves one of these, each on a fresh server:
| Save                                              | Numbers that move |
| POST /games/:id/rename of the active Game        | games             |
| POST /games/:id/posture                           | games             |
| POST /games/:id/mode with the other mode          | games             |
| POST /games/:id/presets                           | games             |
| POST /games/:id/delete of the second Game         | games             |
Then the response status is 200
And exactly the numbers of its row moved
```

#### Scenario 43.5: A new, a switched and a reset Game change the active Game and move the Games list

```gherkin
Given two Games, the first one active
When the client saves one of these, each on a fresh server:
| Save                                       | `game`          | Numbers that move |
| POST /games                                | the new Game    | games             |
| POST /games/:id/switch to the second Game | the second Game | games             |
| POST /reset                                | the new Game    | games             |
| POST /games/:id/switch to the active Game | the first Game  |                   |
Then the response status is 200
And the `game` of the answer is as its row says
And exactly the engine-wide numbers of its row moved
```

#### Scenario 43.6: A save of the settings moves the settings only

```gherkin
Given an active Game
When the client saves one of these, each on a fresh server:
| Save                                              | Numbers that move |
| POST /connections/add                             | settings          |
| POST /connections/:id/edit                        | settings          |
| POST /connections/:id/delete of a free Connection | settings          |
| POST /connections/set-narrator                    | settings          |
| POST /connections/set-quantifier                  | settings          |
| POST /settings/text-check                         | settings          |
Then the response status is 200
And exactly the numbers of its row moved
```

#### Scenario 43.7: A save of a prompt preset moves the prompt presets, and an Activate moves the settings

```gherkin
Given an active Game and a prompt preset that is not a default
When the client saves one of these, each on a fresh server:
| Save                                 | Numbers that move |
| POST /prompt-presets                 | presets           |
| POST /prompt-presets/:id             | presets           |
| POST /prompt-presets/:id/duplicate   | presets           |
| POST /prompt-presets/:id/delete      | presets           |
| POST /prompt-presets/:id/activate    | settings          |
Then the response status is 200
And exactly the numbers of its row moved
```

#### Scenario 43.8: A save of a World moves the Worlds only

```gherkin
Given an active Game and a World that no Game uses
When the client saves one of these, each on a fresh server:
| Save                              | Numbers that move |
| POST /worlds                      | worlds            |
| POST /worlds/:key                 | worlds            |
| POST /worlds/:key/posture         | worlds            |
| POST /worlds/:key/delete          | worlds            |
Then the response status is 200
And exactly the numbers of its row moved
```

#### Scenario 43.9: A save that fails moves no number

```gherkin
Given an active Game
When the client sends one of these saves, and the write behind it fails:
| Save                              |
| POST /games/:id/rename            |
| POST /settings/text-check         |
| POST /prompt-presets              |
| POST /prompt-presets/:id/activate |
| POST /worlds                      |
| POST /worlds/:key/delete          |
Then the response status is 500
And no number of GET /changes moved
```

#### Scenario 43.10: A transaction whose later write fails moves none of its own numbers

```gherkin
Given an active Game whose story holds more than its opening Narration
And a write of the dock record that fails
When the client POST /history/delete for the last Message
Then the response answers that the save failed
And the `story`, `options` and `games` numbers did not move
And the `busy` number moved, because the failure line of the Game is saved
```

#### Scenario 43.11: A Job moves the busy state when it starts and when it ends, and its story, Options, Games list and LLM messages

```gherkin
Given an active Game, and a Job that is held at its narrator call
When the client POST /action with command="look"
Then the `busy` number moved
And the `story` and `games` numbers moved, because the Input is saved
And the `llm`, `options`, `worlds`, `settings` and `presets` numbers did not move

When the Job's model answer arrives
And the Job ends
Then the `busy` number moved again
And, since the numbers before the Action, exactly `story`, `options`, `busy`, `games` and `llm` moved
```

#### Scenario 43.12: The start and the end of an Autoplay run move the busy state and the Games list

```gherkin
Given an active Game
When the client POST /autoplay with cycles=2
And the run is between the start and its first Job
Then exactly `busy` and `games` moved

When the client POST /status/reset-busy
Then exactly `busy`, `games` and `options` moved since the start of the run
```

#### Scenario 43.13: A change of the step of a run moves the busy state and not the Games list

```gherkin
Given a run that is between the start and its first Job
When the first Job of the run starts and reaches its narrator call
Then the `busy` number moved
And the `games` number did not move
```

#### Scenario 43.14: A Message that a Job saves in a Game that is not the active Game moves the Games list

```gherkin
Given a Job that is held at its narrator call in the first Game
And the client created a second Game, which is the active Game
When the Job's model answer arrives
And the Job ends
Then exactly `games` and `llm` moved since the creation of the second Game
And the `game` of the answer is the second Game
And the `story`, `options` and `busy` numbers of the second Game did not move
```
