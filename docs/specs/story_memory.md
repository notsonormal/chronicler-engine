# Feature Spec: Story Memory

Endpoints:
- `POST /action`
- `POST /swipe/new`
- `POST /message/:id/swipe/:index`
- `POST /history/delete`
- `POST /connections/:id/edit`
- `GET /screen?parts=failure-banner`

The story memory is the first Message of the story and a list of memory entries. A memory entry records one scene of the Messages that left the history space of the narrator call at a history cut. An entry holds the first and the last Message of its scene. It also holds the story time, the place, who took part, and what happened. The engine writes an entry once, and the entry never changes. The story log does not change.

Each call with a history shows the story memory before the history in a `<StoryMemory>` block. The block holds the first Message of the story word for word in `<Opening>`, and then the memory entries, oldest first, in `<MemoryEntries>`. A story with no cut yet has no block.

A memory entry belongs to the Swipe whose Job wrote it. A call shows the entries of the active Swipes of the Messages before its place. For a Retry, that is the Messages before the last Message.

Each writer call gets the Messages that leave, the cast names, and the last memory entry written before. A cut over 4,000 tokens of Messages goes to several writer calls of at most 4,000 tokens each, one after the other.

The engine sets the last Message of each scene, so the entries of one call cover its Messages with no gap. A writer call that fails is tried once more with the same prompt. A call that fails twice adds nothing.

A scenario sees the memory entries of the active Swipes through the next Action. "The next Action shows X" means that the narrator's `<MemoryEntries>` of that Action holds X. A scenario that names the writer's previous entry means the entry in the `<PreviousEntry>` of the writer's prompt.

A stop at the story memory writer call is a row of the Examples table of Scenario 39.4 in `dashboard.md`.

## Scenarios

### The cut writes memory entries

#### Scenario 46.1: A cut writes a memory entry before the narrator call, and the narrator call shows it before the history

```gherkin
Given a Game whose story is longer than the history space of its Connection
And a model that answers the story memory writer with one scene whose last Message the model names wrongly
When the client sends POST /action with command="look"
Then the story memory writer is called before the narrator
And its prompt holds the oldest Messages that leave the history
And the narrator's prompt holds a <StoryMemory> block before the first history entry
And the block holds the first Message of the story word for word in <Opening>
And the block holds the memory entry in <MemoryEntries>
And the narrator's prompt holds none of the other Messages that left
And the story log still holds every Message
When the client sends POST /action with command="wait"
Then the writer's prompt holds the memory entry as the previous entry
And the previous entry covers the first Message to the last Message that the first writer call held
And the first Message in the writer's prompt is the first Message after the ones that the first writer call held
```

#### Scenario 46.2: An Action whose history space leaves out no uncovered Message shows the memory entries again with no writer call

```gherkin
Given a Game whose last Narration keeps a memory entry from a cut
And its Connection's context window widened so that every Message after the covered ones stays in the history
When the client sends POST /action with command="wait"
Then the model receives no story memory writer call
And the narrator's prompt holds the same <StoryMemory> block
When the client sends POST /action with command="listen"
Then the narrator's prompt holds the same <StoryMemory> block again
```

#### Scenario 46.3: A later cut gives the writer the last memory entry and only the Messages it does not cover

```gherkin
Given a Game whose last Narration keeps a memory entry from a cut
And enough new Messages after it that the history is cut again
When the client sends POST /action with command="look"
Then the writer's prompt holds the last memory entry as the previous entry
And it holds only Messages newer than the ones that the memory entries cover
And the narrator's prompt holds the old memory entry and the new one in <MemoryEntries>, oldest first
```

#### Scenario 46.11: The story memory writer call asks for no more than the ceiling of the Connection

```gherkin
Given a Game whose story is longer than the history space of its Connection
And a Narration Connection whose ceiling is 700 tokens, under the answer size of a writer call
When the client sends POST /action with command="look"
Then the story memory writer call carries a response length of 700 tokens
```

#### Scenario 46.13: One cut writes at most one history space of the Messages that left

```gherkin
Given a Game saved before the story memory existed, whose story passes the history space of its Connection many times over
When the client sends POST /action with command="look"
Then the writer's prompt holds the oldest Messages of the story, up to one history space
And it holds none of the newer Messages that left the history
And the writer's prompt fits the context window of the Connection
When the client sends POST /action with command="wait"
Then the first Message in the writer's prompt is the first Message after the ones that the memory entries cover
```

### A cut of several writer calls

#### Scenario 46.14: A cut over the size of one writer call makes several writer calls, one after the other, before the narrator call

```gherkin
Given a Game whose cut leaves more than 4,000 tokens of Messages
When the client sends POST /action with command="look"
Then the model receives several story memory writer calls, one after the other, before the narrator call
And each writer call holds at most 4,000 tokens, or one Message
And the Messages of the calls follow each other with no gap
And the first writer call holds no previous entry
And each later writer call holds the last memory entry of the call before it as the previous entry
And the narrator's prompt holds the memory entries of every call, oldest first
```

#### Scenario 46.15: A failed call in a split keeps the entries of the earlier calls, and its Messages go to the next cut

```gherkin
Given a Game whose cut makes two writer calls
And a model whose first writer call answers, and whose second writer call fails on both tries
When the client sends POST /action with command="look"
Then the Narration is saved
And the narrator's prompt holds the memory entry of the first call
And the failure banner names the story memory and says the earlier memory entries are kept
When the writer answers again and the client sends POST /action with command="wait"
Then the writer's prompt holds the memory entry of the first call as the previous entry
And the first Message in the writer's prompt is the first Message of the failed call
```

### A history that fits

#### Scenario 46.4: A history inside its space makes no writer call and shows no story memory

```gherkin
Given a Game whose story fits the history space of its Connection
When the client sends POST /action with command="look"
Then the model receives no story memory writer call
And the narrator's prompt holds no <StoryMemory> block
And the next Action shows no memory entry
```

### Failure and the second try

#### Scenario 46.16: A writer call that fails once is tried again with the same prompt

```gherkin
Given a Game whose story is longer than the history space of its Connection
And a model whose first story memory writer try fails, and whose second try answers
When the client sends POST /action with command="look"
Then the model receives two story memory writer calls with the same prompt
And the narrator's prompt holds the memory entry of the second try
And the failure banner does not name the story memory
```

#### Scenario 46.5: A writer call that fails twice keeps the earlier memory entries, and its Messages go to the next cut

```gherkin
Given a Game whose last Narration keeps a memory entry from a cut
And enough new Messages after it that the history is cut again
And a model whose story memory writer fails on both tries
When the client sends POST /action with command="look"
Then the Narration is saved
And the narrator's prompt holds the earlier memory entry
And the failure banner names the story memory and says the earlier memory entries are kept
When the writer answers again and the client sends POST /action with command="wait"
Then the writer's prompt holds the earlier memory entry as the previous entry
And it holds the Messages that the failed call missed
```

#### Scenario 46.6: A writer answer that the response length cut off, on both tries, keeps the earlier memory entries

```gherkin
Given a Game whose last Narration keeps a memory entry from a cut
And enough new Messages after it that the history is cut again
And a model that stops its story memory writer answer at the response length, on both tries
When the client sends POST /action with command="look"
Then the Narration is saved
And the narrator's prompt holds the earlier memory entry
And the failure banner names the story memory and says the earlier memory entries are kept
And the next Action shows the earlier memory entry
```

### The calls and the story moves that read the story memory

#### Scenario 46.7: The Impersonate call shows the story memory

```gherkin
Given a Game whose last Narration keeps a memory entry from a cut
When the client sends POST /action with command="/impersonate I wave"
Then the Impersonate call's prompt holds the memory entry in a <StoryMemory> block
```

#### Scenario 46.8: A Retry shows the memory entries of the Messages before the last Message

```gherkin
Given a Game whose last Narration keeps a memory entry from a cut
And a second Action whose cut wrote a newer memory entry
When the client retries the last Narration with POST /swipe/new
Then the writer's prompt holds the memory entry of the first cut as the previous entry
And the narrator's prompt holds the memory entry of the first cut and the writer's new entry in <MemoryEntries>
And the narrator's prompt does not hold the newer memory entry of the second Action
```

#### Scenario 46.9: A swipe switch gives the memory entries of the chosen Swipe back

```gherkin
Given a Game whose last Narration has two Swipes, each with its own memory entry
When the client switches the last Narration back to its first Swipe
Then the next Action shows the memory entry of the first Swipe
And the next Action does not show the memory entry of the second Swipe
```

#### Scenario 46.10: A delete of the last Message gives the older memory entries back

```gherkin
Given a Game whose last Narration keeps a newer memory entry than the Message before it
When the client deletes the last Message with POST /history/delete
Then the next Action shows the older memory entry
And the next Action does not show the newer memory entry
```

#### Scenario 46.12: The event Narration call shows the story memory

```gherkin
Given a Game whose story is longer than the history space of its Connection
And a Character whose Trigger the Narration fires
When the client sends POST /action with command="look"
Then the Narration's cut writes a memory entry
And the event Narration call's prompt holds that memory entry in a <StoryMemory> block
```
