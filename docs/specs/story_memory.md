# Feature Spec: Story Memory

Endpoints:
- `POST /action`
- `POST /swipe/new`
- `POST /message/:id/swipe/:index`
- `POST /history/delete`
- `POST /connections/:id/edit`
- `GET /screen?parts=failure-banner`

The story memory is a summary of the Messages that left the history space of the narrator call at a history cut. The narrator sees it, and the story log does not change.

The Swipe state of each new Swipe keeps the story memory, and no response shows it. So a scenario sees the story memory of the current state through the next Action. "The next Action shows X as the story memory" means that the next writer call holds X as the previous story memory. When that Action cuts no new Message, it means that the narrator's prompt holds X in its `<StoryMemory>` block.

A stop at the story memory writer call is a row of the Examples table of Scenario 39.4 in `dashboard.md`.

## Scenarios

### The cut writes the story memory

#### Scenario 46.1: A cut writes the story memory before the narrator call, and the narrator call shows it before the history

```gherkin
Given a Game whose story is longer than the history space of its Connection
And a model that answers the story memory writer with a summary
When the client sends POST /action with command="look"
Then the story memory writer is called before the narrator
And its prompt holds the oldest Messages that leave the history
And the narrator's prompt holds the summary in a <StoryMemory> block before the first history entry
And the narrator's prompt holds none of the Messages that left
And the story log still holds every Message
When the client sends POST /action with command="wait"
Then the writer's prompt holds the summary as the previous story memory
And the first Message in the writer's prompt is the first Message after the ones that the first writer call held
```

#### Scenario 46.2: An Action whose history space leaves out no uncovered Message shows the story memory again with no writer call

```gherkin
Given a Game whose last Narration keeps a story memory from a cut
And its Connection's context window widened so that every Message after the covered ones stays in the history
When the client sends POST /action with command="wait"
Then the model receives no story memory writer call
And the narrator's prompt holds the same <StoryMemory> block
When the client sends POST /action with command="listen"
Then the narrator's prompt holds the same <StoryMemory> block again
```

#### Scenario 46.3: A later cut gives the writer the previous story memory and only the Messages it does not cover

```gherkin
Given a Game whose last Narration keeps a story memory from a cut
And enough new Messages after it that the history is cut again
When the client sends POST /action with command="look"
Then the writer's prompt holds the previous story memory
And it holds only Messages newer than the ones the story memory covers
And the next Action shows the new summary as the story memory
```

#### Scenario 46.11: The story memory writer call asks for no more than the ceiling of the Connection

```gherkin
Given a Game whose story is longer than the history space of its Connection
And a Narration Connection whose ceiling is 700 tokens, under the token size of the story memory
When the client sends POST /action with command="look"
Then the story memory writer call carries a response length of 700 tokens
And the writer's prompt asks for fewer words than it asks for under a Connection with no ceiling
```

#### Scenario 46.13: One writer call holds at most one history space of the Messages that left

```gherkin
Given a Game saved before the story memory existed, whose story passes the history space of its Connection many times over
When the client sends POST /action with command="look"
Then the writer's prompt holds the oldest Messages of the story, up to one history space
And it holds none of the newer Messages that left the history
And the writer's prompt fits the context window of the Connection
When the client sends POST /action with command="wait"
Then the first Message in the writer's prompt is the first Message after the ones the story memory covers
```

### A history that fits

#### Scenario 46.4: A history inside its space makes no writer call and shows no story memory

```gherkin
Given a Game whose story fits the history space of its Connection
When the client sends POST /action with command="look"
Then the model receives no story memory writer call
And the narrator's prompt holds no <StoryMemory> block
And the next Action shows no story memory
```

### Failure

#### Scenario 46.5: A failed writer keeps the previous story memory, and the Messages it missed go to the next cut

```gherkin
Given a Game whose last Narration keeps a story memory from a cut
And enough new Messages after it that the history is cut again
And a model whose story memory writer call fails
When the client sends POST /action with command="look"
Then the Narration is saved
And the narrator's prompt holds the previous story memory
And the failure banner names the story memory and says the previous story memory is kept
When the writer answers again and the client sends POST /action with command="wait"
Then the writer's prompt holds the previous story memory
And it holds the Messages that the failed call missed
```

#### Scenario 46.6: A writer answer that the response length cut off keeps the previous story memory

```gherkin
Given a Game whose last Narration keeps a story memory from a cut
And enough new Messages after it that the history is cut again
And a model that stops its story memory writer answer at the response length
When the client sends POST /action with command="look"
Then the Narration is saved
And the narrator's prompt holds the previous story memory
And the failure banner names the story memory and says the previous story memory is kept
And the next Action shows the previous story memory as the story memory
```

### The calls and the story moves that read the story memory

#### Scenario 46.7: The Impersonate call shows the story memory

```gherkin
Given a Game whose last Narration keeps a story memory from a cut
When the client sends POST /action with command="/impersonate I wave"
Then the Impersonate call's prompt holds the story memory in a <StoryMemory> block
```

#### Scenario 46.8: A Retry starts from the story memory of the state before the last Message

```gherkin
Given a Game whose last Narration keeps a story memory from a cut
And a second Action whose cut wrote a newer story memory
When the client retries the last Narration with POST /swipe/new
Then the writer's prompt holds the story memory of the first cut as the previous story memory
And the narrator's prompt holds the writer's new answer in its <StoryMemory> block
```

#### Scenario 46.9: A swipe switch gives the story memory of the chosen Swipe back

```gherkin
Given a Game whose last Narration has two Swipes, each with its own story memory
When the client switches the last Narration back to its first Swipe
Then the next Action shows the story memory of the first Swipe as the story memory
```

#### Scenario 46.10: A delete of the last Message gives the older story memory back

```gherkin
Given a Game whose last Narration keeps a newer story memory than the Message before it
When the client deletes the last Message with POST /history/delete
Then the next Action shows the older story memory as the story memory
```

#### Scenario 46.12: The event Narration call shows the story memory

```gherkin
Given a Game whose story is longer than the history space of its Connection
And a Character whose Trigger the Narration fires
When the client sends POST /action with command="look"
Then the Narration's cut writes a story memory
And the event Narration call's prompt holds the newest story memory of the Job in a <StoryMemory> block
```
