# How ST, ME and other tools keep long-term state

This asset gathers material for [14-long-term-state](../issues/14-long-term-state.md). It covers the
facts that a story changes through play. The memory of events is the sibling asset `long-term-memory.md`
([13-long-term-memory](../issues/13-long-term-memory.md)). Memory appears here only where one store keeps
both.

`known` means a source states it, or a stored record shows it. `inferred` means this note reads it from
code, a stored record or a document. `guessed` means this note does not know and says so.

Sources: SillyTavern at `06bde93` and Marinara Engine at `457e827` (the pinned commits of the map) and the
Marinara Agents catalogue. Web sources and repository metadata were read on 2026-10-31. Stored calls come
from `tmp/stage6/debug/chronicler_3060.db`, read-only.

## Short answer

- **known** Marinara Engine (ME) keeps state as a **per-Message snapshot of typed fields that a second model
  call extracts from the new text**. It stores it in `game_state_snapshots`, keyed by message and swipe, and
  its tracker prompts ask for JSON.
- **known** SillyTavern (ST) core keeps no such record. ST core has two rule-based mechanisms: chat
  variables that a slash command writes, and World Info timed effects that a keyword scan writes.
- **known** The state trackers in ST are all third-party extensions. The maintained ones (`zTracker`,
  `npc_state_delta`, `merged_world_tracker`) all copy the ME shape: extract after the reply, store per
  Message, inject a short block into the next prompt.
- **known** Every tool that keeps state per swipe keys it on the Message and the swipe index, and rewinds
  on a swipe. Our Swipe state already follows that shape.
- **known** No tool found here keeps what our story needs most: what one Character knows, and a durable
  change to a relationship or to an object. The tools that come closest keep free text and lose the rest.
- **inferred** Our engine records 5 kinds of fact today (Room, added Rooms, Characters present, encounters,
  the fired Trigger). A reader of the stored Game 6 finds 11 state changes, and the records hold 3 of them,
  each in part.

## 1. The kinds of state, by speed and by owner

The speed is how often the fact can change. The owner is the thing the fact belongs to.

| Kind | Owner | Speed | Example from our Game 6 | An ME tracker stores it | Our engine stores it today |
|---|---|---|---|---|---|
| Who is in the scene | the World, one Room | each Message | Carla leaves the scene, Gabriella enters | Character Tracker `presentCharacters` | **yes**: `scene.characters_present` |
| Where the Persona is | the World | each scene | Front Gates, then the entrance hall, then the study | World State `location` | **yes**: `movement.current_room_id` |
| What a Character wears and holds | one Character | each Message | Carla's jacket pocket holds the black box | Beholder, Character Tracker `outfit` | no |
| Body state, wounds, health | one Character | each Message, partly durable | Carla's old scars, tattoos | Beholder `wounds`, Character Tracker stats | no |
| Mood and current thought | one Character | each Message | Julian's cold knot of fear | Character Tracker `mood` and `thoughts` | no |
| How two Characters feel about each other | a pair | each scene, durable in part | Julian and Carla agree to work together | Relationship Tracker | **in part**: `relationships[].dynamic`, and nothing writes it |
| What one Character knows | one Character | each Message, durable | Julian learns that Bernard's death may not be natural | no ME tracker does this | no |
| Promises and open threads | the World | each scene, durable | Gabriella offers a tour of the estate | Memory Nag, Quest Tracker | no |
| Quests and their goals | the World | each scene | not in this excerpt | Quest Tracker | no |
| Items, money, gear | the Persona, one Character | each Message | the folder of accounts in Julian's hand, the phone with the tracker | Inventory Tracker, Quartermaster | **a field only**: `inventory`, and no prompt part reads it |
| Needs and condition bars | the Persona | each Message | not in this excerpt | Persona Stats | no |
| Date, time of day, weather, temperature | the World | each scene | the afternoon light throughout `#85` to `#92` | World State `date`, `time`, `weather`, `temperature` | no |
| Numbers with a range | one Character, the Persona | each Message | not in this excerpt | RPG stats `hp`, `pools`, ruleset sheets | no |
| A change to a Character card itself | the World, one Character | durable | not in this excerpt | Card Evolution Auditor proposes it | no |
| Which Triggers have fired | the World, one Character | durable | "Gabriella Introduction" fired | no ME equivalent | **yes**: `encounters.npcs[id].trigger_fired` |

Three speeds hold for every tool found here:

| Speed | Kinds | Who usually writes them |
|---|---|---|
| Each Message | who is in the scene, clothing, held items, body state, mood, needs, time of day | a second model call after the reply |
| Each scene | where the Persona is, relationships, promises, quests | the same second model call, or a rule |
| Durable | card text, what a Character knows for good, a healed or lasting injury | a model proposal that a person approves, or a rule |

## 2. The methods

### 2.1 Marinara Engine: the tracker agents

The measured facts of one tracker:

| Field | Value | Source |
|---|---|---|
| What it stores | JSON in one row of `game_state_snapshots` per Message and swipe: `date`, `time`, `location`, `weather`, `temperature`, `worldCustomFields`, `presentCharacters`, `recentEvents`, `playerStats`, `personaStats`, `rulesetLive`, `rulesetClocks` | `packages/server/src/db/schema/game-state.ts` (`known`) |
| Who writes it | a post-processing Agent, one model call per tracker per Message | `docs/agents/built-in-agents.md` (`known`) |
| How often | after each reply. The Card Evolution Auditor defaults to every 8 Messages. A run interval is settable to 100 | `docs/agents/built-in-agents.md` (`known`) |
| Where it goes in the prompt | a `user` message with `contextKind: "injection"`, before the newest history block, inside `<context>` with one wrapper per tracker | `services/generation/committed-tracker-context.ts`, `routes/generate/generate-route-utils.ts:624` (`known`) |
| Cost per cycle | one call per enabled tracker. The defaults of the Relationship Tracker are 5 Messages of context, 1600 output tokens and temperature 0 | `packages/relationship-tracker/agents.json` (`known`) |
| What the author defines | which trackers are on, the prompt template of each one, the connection, the output limit, "Add as Prompt Section", the custom world fields, the persona stat bars, the custom tracker fields, the quest list | `docs/roleplay/hud-and-trackers.md`, `docs/agents/built-in-agents.md` (`known`) |
| Swipe behaviour | its own row per `(messageId, swipeIndex)`. A regeneration clones the base row for the new swipe. The next typed Message commits the row of the active swipe. Agents running for a swipe that is no longer active are aborted | `services/storage/game-state.storage.ts` (`known`) |

The state-keeping part of the tracker family holds 9 agents: World State, Character Tracker, Beholder,
Relationship Tracker, Persona Stats, Custom Tracker, Inventory Tracker, Quest Tracker and Memory Nag. The
catalogue counts 13 tracker agents, and the other 4 change no state (Background, Expression Engine, World
Maps, Quartermaster). Each state-keeping tracker has its own prompt, its own schema and its own output
limit. Two behaviours matter for our design:

- **known** The incremental format. A tracker prompt can answer with `{"updates": [...], "removed": [...]}`
  and leaves every field it does not name alone. The host announces support with the marker
  `tracker_incremental_updates`. Without the marker the prompt asks for the full array and an omitted field
  is a deleted field (`packages/world-state/agents.json`).
- **known** The author edits and locks a value. `manualOverrides` records the edit, `fieldLocks` stops the
  tracker from changing it, and `hiddenTrackerFields` stops it from coming back. A tracker run retires an
  edit when the stored value changes (`game-state.storage.ts`).

### 2.2 Marinara Engine: the model writes state in its own call

| Method | What it stores | Who writes it | Where it goes | Swipe behaviour | Source |
|---|---|---|---|---|---|
| `update_game_state` tool | `location` only, or `time` only | the narrator model, by a tool call in the same reply | written to the row of that swipe after the reply is saved | the same row rule | `services/tools/tool-executor.ts:383`, `routes/generate.routes.ts:8127` (`known`) |
| Turn-game engine state | the private state of a game inside the chat (UNO, Chess, Poker) | the game engine, by rule | not a prompt part | its own table keyed by `(messageId, swipeIndex)`, "so regenerate / branch / undo rewind the game correctly" | `db/schema/game-engine-state.ts` (`known`) |

The tool refuses every update type except `location_change` and `time_advance`. Its hint to the model is
"Only location_change and time_advance are stored. Describe stat, inventory and quest changes in the
narration instead." (`known`, `tool-executor.ts:622`). The split is a design statement: the model may write
two cheap facts itself, and the trackers own the rest.

### 2.3 SillyTavern core: variables and timed effects

| Method | What it stores | Who writes it | How often | Where it goes | Author defines | Swipe behaviour | Source |
|---|---|---|---|---|---|---|---|
| Chat variables | `chat_metadata.variables`, a flat name to string map, and a global map | a slash command, a macro such as `{{setvar::hp::3}}`, or a Quick Reply | whenever the author's script runs | by a macro in any prompt text: `{{getvar::name}}`, `{{getglobalvar::name}}` | the names, the values, the script | none. The store is per chat, not per Message, so a swipe does not rewind it | `public/scripts/variables.js`, `public/scripts/macros/definitions/variable-macros.js` (`known`) |
| Quick Reply automation | nothing of its own. It runs the commands | the extension, at a named event: on user message, on AI message, on World Info activation, before generation, on chat change | at that event | written text goes wherever the command puts it | the author's Quick Reply set | none | `public/scripts/extensions/quick-reply/src/AutoExecuteHandler.js` (`known`) |
| World Info timed effects | `sticky`, `cooldown` and `delay` per entry, with a start and an end in Message counts, in `chat_metadata.timedWorldInfo` | the keyword scan of each generation | each generation | the entry itself goes where its position and depth put it | the number of Messages of each effect | **a reset.** An effect is dropped when "the chat is not advanced", that is when the last Message was swiped or deleted | `public/scripts/world-info.js:596-660`, [docs](https://docs.sillytavern.app/usage/core-concepts/worldinfo/) (`known`) |

**known** ST core has no state tracker. Every tracker in ST is a third-party extension. **inferred** The
official extension list holds no tracker either ([docs](https://docs.sillytavern.app/extensions/), read on
2026-10-31: Token Counter and Duplicate Finder are the nearest entries).

### 2.4 SillyTavern: the third-party trackers

Repository metadata from `api.github.com/repos/<owner>/<repo>` on 2026-10-31.

| Extension | What it stores | Who writes it | Where it goes | Author defines | Maintained |
|---|---|---|---|---|---|
| **zTracker** (fork of WTracker) | one snapshot per Message per Module. Each Module has its own schema | an extra model call, JSON, XML or TOON mode, optional grammar sampling, optional sequential per-field calls | an injectable block per Module, with a header, a "virtual character" mode and a transform preset. It can also chain another Module's history | the Modules, the schema, the prompt, the connection, the injection format, the world info filter, the trigger direction, "Skip First X Messages" | **yes**: last push 2026-10-23 ([README](https://github.com/Zaakh/SillyTavern-zTracker)) (`known`) |
| **RPG Companion** | one tracker state per swipe. Stats, scene widgets, present Characters with relationship badges and thoughts, inventory, quests | two modes. **Together**: the roleplay model writes a tracker block inside its own reply, and the extension extracts it and hides it. **Separate**: one extra call after the reply | the panel, plus prompt text built from the tracker | the stats, the fields, the widgets, the theme, the context Messages | community-maintained. The author moved to Marinara Engine ([README](https://github.com/SpicyMarinara/rpg-companion-sillytavern)) (`known`) |
| **StatSuite** | pose, location, outfit, pocket contents, health, injuries, hunger, per Character | a small purpose-trained model on its own connection, so the main model and its samplers do not matter | one Handlebars template, at depth 1, newest block only | the stat list, manual stats, presets locked to a Character, the template | prototype at 0.4, last push 2026-02-07 ([README](https://github.com/leDissolution/StatSuite)) (`known`) |
| **BetterSimTracker** | `Affection`, `Trust`, `Desire`, `Connection`, `Mood`, `Last Thought`, plus custom stats, one card per Message | an extra call, unified or sequential, with per-stat prompts | optional injection, trimmed first when it grows too large | the stats, the ranges, the prompts, the injection size | **abandoned**. The README says so ([README](https://github.com/ghostd93/BetterSimTracker)) (`known`) |
| **npc_state_delta** | a dossier per Character: who they are, how they look, how they speak, what they remember, how they feel | an extra call after each reply, which updates a dossier only on evidence | the dossiers of the present Characters go to the reply model | the fields of a dossier | **yes**: last push 2026-10-31 ([README](https://github.com/kohz87/npc_state_delta)) (`known`) |
| **merged_world_tracker** | World State, a session chronicle, NPC knowledge, story planning, NPC interiority, in five modules | an extra call per module | per module | the modules | **yes**: last push 2026-10-29 ([README](https://github.com/brasen56/merged_world_tracker)) (`known`) |
| **Beholder-ST** | clothing per body slot, held items, wounds, missing parts, species | a purpose-trained small extractor model, one prompt or five narrow passes | a paper-doll panel and the next prompt | the model, the prompt, the tracked lanes | **yes**: last push 2026-08-30 ([README](https://github.com/GetBeholder/Beholder-ST)) (`known`) |
| **SillyTavern-State** | free text from a set of author-written prompts, kept as Messages | one extra call per configured prompt, added to the chat as its own Message | it is a Message in the history, so the history cut applies to it | the prompts | **not maintained since 2024**: last push 2024-09-24 ([README](https://github.com/ThiagoRibas-dev/SillyTavern-State)) (`known`) |

Cost per cycle of the extensions above, with no source in the READMEs unless stated: one extra model call per
Message for zTracker, BetterSimTracker, npc_state_delta and merged_world_tracker. StatSuite and Beholder-ST
use a small purpose-trained model, so their cost is one short local call and the main model's cache is not
disturbed. RPG Companion's **Together** mode uses no extra call, because the roleplay model writes the block
inside its own reply. `known` for StatSuite and Beholder-ST (their READMEs name the model), `inferred` for the
rest.

**inferred** The maintenance picture is clear. The trackers that survive copy the ME shape (a typed
snapshot per Message, an injected block, an author-defined schema). The ones that stop copy the chat-Message
shape (a new Message per update), because the history cut deletes their output.

### 2.5 AI Dungeon, NovelAI and Inform

| Tool | What it stores | Who writes it | Where it goes | Swipe behaviour | Source |
|---|---|---|---|---|---|
| **AI Dungeon** scripts | `state`, a key-value store that persists across turns, with the script's own `memory` handles: Plot Essentials at `state.memory.context`, the Author's Note, the front memory | the script, by rule, at the hooks `onInput`, `onModelContext` and `onOutput`. The model writes no state | the script builds the text that the budget then fits | the store belongs to the Adventure. Scripts run again from the new text | [Scripting](https://help.aidungeon.com/scripting), [Scripting repo](https://github.com/AIDungeon/Scripting) (`known`) |
| **AI Dungeon** Story Cards | a `keys` list, an `entry` text and a `type`, per card | the author, or a script that creates cards | keyword match, then the card text enters the context. About 25% of the remaining tokens go to matching cards | not per swipe. A card stays active "for a variable period depending on your context size" | [Story Cards](https://help.aidungeon.com/faq/story-cards), [context budget](https://help.aidungeon.com/faq/what-goes-into-the-context-sent-to-the-ai) (`known`) |
| **AI Dungeon** Plot Essentials and Memory Bank | free text that the author writes, plus an automatic summary of past text | the author, or auto-summarization | always in the context, about 25% of the remaining tokens | not per swipe | [System Settings](https://help.aidungeon.com/understanding-settings) (`known`) |
| **NovelAI** lorebook and Memory | entries with activation keys, an `Always On` flag, an insertion position and an insertion order. Memory is one fixed text | the author | staged insertion, by order and position | not per swipe | [Lorebook](https://docs.novelai.net/en/text/lorebook/), [advanced settings](https://docs.novelai.net/en/text/editor/advancedsettings/) (`known`) |
| **Inform 7** | a typed model world: kinds, properties, relations. A relation is a yes/no question about a pair of things, such as "the apple is in the basket" | the author's rules, in the present tense | not a prompt. The world is the game | the world is rebuilt from the start of play | [WI 8.11 Now](https://ganelson.github.io/inform-website/book/WI_8_11.html), [WI 13 Relations](https://zedlopez.github.io/i7doc/WI_13.html) (`known`) |

**known** Inform 7 gives one sentence form three jobs. "S." states the fact at the start of play,
"if S, ..." tests it now, and "now S;" makes it true from now on. Inform refuses a change it cannot do, a
vague change, or a change in another tense ([WI 8.11](https://ganelson.github.io/inform-website/book/WI_8_11.html)).
**inferred** This is the strongest argument in the corpus for typed fields over free text: one authored
form is the starting value, the test and the change.

**known** AI Dungeon and NovelAI keep no record that changes through play. Their Story Cards and lorebook
entries are authored text, and a change to one is the author's edit or a script's write. All changing state
lives in an AI Dungeon script.

### 2.6 The reflection step as a state writer

| Field | Value | Source |
|---|---|---|
| What it stores | a higher-level statement in the memory stream, with pointers to the records that support it. It is not a field with a range | [paper](https://arxiv.org/html/2304.03442v2) (`known`) |
| Who writes it | one model call, at a threshold | same (`known`) |
| How often | when the summed importance of the new records passes 150. In practice about two or three times a day | same (`known`) |
| What it reads | the 100 newest records of the memory stream. It asks for 3 salient questions, retrieves for each one, then asks for 5 insights with evidence | same (`known`) |
| Where it goes | back into the memory stream, so it takes part in retrieval like any record. An agent can reflect on a reflection | same (`known`) |
| Swipe behaviour | none. The memory stream only grows | same (`inferred`) |

**inferred** The reflection step writes a summary of evidence, not a fact with an owner. It suits what a
Character knows. It does not suit a number, an item list or a room.

## 3. A change to a Character card itself

| Tool | What it changes | Who approves | How the authored card stays apart | Source |
|---|---|---|---|---|
| **ME Card Evolution Auditor** | `description`, `personality`, `scenario`, `first_mes`, `mes_example`, `creator_notes`, `system_prompt`, `post_history_instructions`, `backstory`, `appearance` | the person, in the **Review Character Card Updates** window. "Character card edits are a special case. They always ask for your approval first, even when **Review Agent Outputs** is off." | the edit goes to the card record, not to the chat state. Approving raises the card version and writes a version-history entry. A proposal whose `oldText` no longer matches is marked **stale** | `packages/card-evolution-auditor/agents.json`, `docs/agents/approvals-and-agent-suite.md` (`known`) |
| **ME** separation of the two records | the trackers write a scene snapshot, not a card. `PresentCharacter` holds a `characterId`, a `name`, an `appearance`, an `outfit`, a `mood`, a `thoughts`, custom fields and stats, and it names the card by id only | not applicable. The snapshot belongs to one swipe and leaves with it | the snapshot is a column of the chat's state row. The card is its own record, and only the card path writes it | `packages/shared/src/types/game-state.ts:62-89`, `db/schema/characters.ts` (`known`) |
| **ST Character Card Suggestions** | the card text fields, on an allowlist, one proposal at a time | the person, one by one | the extension keeps the evidence, the current text and the replacement text. The card is only written on approval | [README](https://github.com/zapers/SillyTavern-Character-Card-Suggestions) (`known`, from [evaluation-tools](evaluation-tools.md)) |
| **Our engine** | nothing. No Agent writes a card. The seed adds a card only when its id is absent | not applicable. There is no proposal and no approval | the only card is the World's card, in `data/` and in the `characters` table | `src/bootstrap/load.rs:152-159` (`known`) |

The auditor prompt asks for what to ignore: "Ignore temporary mood, current scene location, transient
clothing, injuries already healed, and vague implications", and "False positives are worse than missed
changes" (`known`). That list is the same line the trackers cross in the other direction. The auditor holds
the durable facts. The trackers hold the fast facts.

**inferred** The Card Evolution Auditor is the only tool found here that keeps the run of one Game apart from
the authored World. It does so by a record boundary (card row against chat row), a human gate and a version
history. Our engine has none of the three for state.

## 4. Swipes, retries, edits and deletes

| Method | Rule | Source |
|---|---|---|
| **Our engine** | each Swipe holds its own Swipe state, and the Game rules start from the state of the previous Swipe | `CONTEXT.md`, `src/domain/model/state/swipe_state.rs` (`known`) |
| **ME game state** | one row per `(messageId, swipeIndex)`. A regeneration starts from the reply before the one it replaces, with the Tracker Panel edits of the replaced reply laid over it. The next typed Message commits the active swipe's row | `services/storage/game-state.storage.ts` (`known`) |
| **ME turn-game state** | the same key, "so regenerate / branch / undo rewind the game correctly" | `db/schema/game-engine-state.ts` (`known`) |
| **ME agents** | an Agent run that belongs to a swipe that is no longer active is aborted before the commit of the next turn | `routes/generate.routes.ts:1437-1455` (`known`) |
| **ME manual edits** | an edit by the person is a `manualOverride`, retired when a later tracker run changes the field. A newer edit of the same value on another swipe retires the older one | `game-state.storage.ts` (`known`) |
| **RPG Companion** | "Per-Swipe Data Storage: Each swipe preserves its own tracker data" | [README](https://github.com/SpicyMarinara/rpg-companion-sillytavern) (`known`) |
| **zTracker** | a snapshot per Message, and a "Clear and recreate stale tracker sections" action. The README gives no rule for a swipe | [README](https://github.com/Zaakh/SillyTavern-zTracker) (`inferred`: no swipe rule is stated) |
| **ST chat variables** | no rewind. The map lives in `chat_metadata` for the whole chat | `public/scripts/variables.js` (`known`) |
| **ST World Info timed effects** | a reset, not a rewind. An effect whose start Message is still the last Message is dropped, "chat not advanced". A swiped or deleted last Message therefore clears it | `public/scripts/world-info.js:624-628` (`known`) |
| **AI Dungeon scripts** | no rewind. `state` belongs to the Adventure, and an undo does not restore it | `help.aidungeon.com/scripting` (`inferred`: the page does not state an undo rule) |
| **StatSuite** | the newest block only is sent, one block per Message | [README](https://github.com/leDissolution/StatSuite) (`inferred`: the README says "Only the latest stat block is being sent to the model") |

**inferred** The per-swipe key is the common answer where a tool can afford a store. A tool that keeps its
state in the chat itself (ST variables, AI Dungeon `state`) either ignores a swipe or resets.

**known** Our engine already has the store that ME needed a rule to build: the Swipe state is a column of
the Swipe row (`message_swipes.state`), so a Retry makes a row with its own state.

## 5. Errors: what happens when the writer is wrong

| Method | A wrong or missing answer | A drifted answer | Source |
|---|---|---|---|
| **ME tracker** | the update is logged and dropped, and the previous row stays. The reply is not held or failed | the next tracker run rewrites the whole array, or the person edits the value and locks it | `routes/generate/retry-agents-route.ts:3188-3190` (`known`) |
| **ME tracker, exact checks** | only the fields the schema knows are copied. A `character-tracker` answer with no `presentCharacters` keeps the stored snapshot. A lock stops the write | `manualOverrides`, `fieldLocks`, `hiddenTrackerFields`, and the **State Editor** and **Agent Suite** for a hand edit. **Clear Trackers** wipes the chat | `retry-agents-route.ts:3255`, `game-state.storage.ts`, `docs/roleplay/hud-and-trackers.md` (`known`) |
| **ME Beholder** | "It validates its own snapshot" | a fresh extraction each Message, plus the paper-doll panel to read it | `docs/agents/built-in-agents.md` (`known`) |
| **ME Card Evolution Auditor** | a proposal needs a verbatim `oldText`. "If oldText is not present in the card, skip it" | a stale proposal is dimmed and needs **Override stale**. **Regenerate** asks for fresh proposals | `packages/card-evolution-auditor/agents.json`, `docs/agents/approvals-and-agent-suite.md` (`known`) |
| **zTracker** | a status badge per run with an elapsed timer and a cancel. A chooser gates a Manual Module. A hidden world info filter cuts the noise | per-part regenerate of one section, one list item or one field, plus a cleanup of stale sections | [README](https://github.com/Zaakh/SillyTavern-zTracker) (`known`) |
| **ST World Info timed effects** | the effect is dropped when the entry is missing or is no longer configured for that effect | `/wi-set-timed-effect` and `/wi-get-timed-effect` read and write it by hand | `public/scripts/world-info.js:617-653`, `:1950-1967` (`known`) |
| **AI Dungeon script** | an error is contained, and the other Scripts keep running. The order of the list decides who has the last word | the author edits `state`, or the script rewrites the memory text | [Scripting](https://help.aidungeon.com/scripting) (`known`) |
| **Our engine, the Quantifier** | the Agent failure is logged as a warning and the Quantifier result is `default()`: no Characters present and no destination. `game_rules::apply` then **clears `characters_present`** and keeps the Room. Nothing reaches the player | no check, no re-extraction, no hand edit of the Swipe state | `src/application/agents/runner.rs:71-79`, `src/domain/model/utils/game_rules.rs:53-58` (`known`). **Correction:** wrong for a model failure, which keeps the previous Characters present through the `Low` confidence fallback. See [Decide what the state does when a state writer fails](../issues/15-failed-state-writer.md) |

**known** The stored Game 6 shows this in numbers. `llm_messages` holds 30 Quantifier calls and 24 of them
failed with `HTTP 400` (the `content_filter` fault of [10-quantifier-options-layout](../issues/10-quantifier-options-layout.md)).
The median Quantifier call was 2959 characters of system text and 6559 characters of user text, about 2380
tokens at four characters to a token. The median Narrator call was about 5420 tokens.
**inferred** The one state writer of the engine therefore added about 44% to the tokens of a cycle, and it
worked in 6 of 30 cycles.

## 6. Our story, read: the state changes and where each one is recorded

The excerpt is `#81` to `#93` of [the story transcript](autoplay-story-transcript.md). The records are read
from `tmp/stage6/debug/chronicler_3060.db` (Game 6, 50 Messages). The message numbers of the transcript and
the message ids of that DB agree for this run.

| # | State change a reader can see | Owner | Where our engine holds it today |
|---|---|---|---|
| 1 | Carla offers to work with Julian, and he agrees (`#81`, `#84`) | a pair | **nowhere.** `relationships[].dynamic` would hold it, and no writer sets it |
| 2 | Julian hands over his phone. Carla fits a GPS tracker and a secure line (`#82` to `#85`) | an item, a pair | **nowhere.** The Persona `inventory` is a list of names with no prompt part |
| 3 | Carla says Bernard's death may not be natural. Julian now knows it (`#83`) | one Character | **nowhere.** No record of what a Character knows |
| 4 | Gabriella appears and speaks (`#86`) | the World, one Room | **yes, in part**: `scene.characters_present` holds her and `encounters.npcs.gabriella` holds `times_met: 1` and `trigger_fired: {0: true}`. The same write **dropped Carla** from the Characters present |
| 5 | The Persona walks from the Front Gates to the entrance hall (`#85`), then to the study (`#90`) | the World | **yes, in part**: `movement.current_room_id` is `entrance_hall` from Message 85 and stays there. The map has a `library` Room named "Library & Study", and the Quantifier never named it |
| 6 | Gabriella offers a tour of the estate (`#88`) | the World | **nowhere.** No promise and no quest record |
| 7 | Jezebel and Mr. Aldridge are in the study (`#90`) | one Room | **nowhere.** `characters_present` stays on Gabriella. Jezebel has a card (`characters.id` 5) and never enters the state. Mr. Aldridge has no card, and an id with no card is dropped by `filter_map` |
| 8 | Jezebel holds out a folder of accounts (`#92`) | an item | **nowhere** |
| 9 | The afternoon does not move (`#85` to `#92`) | the World | **nowhere.** No date and no time record |
| 10 | Julian hides his distrust of Gabriella (`#87`, `#89`, `#93`) | the Persona, a pair | **nowhere** |
| 11 | Carla stands a few paces back while Gabriella talks (`#88`) | one Character | **in part**: `encounters.npcs.carla.currently_meeting` is `false` |

**known** The records that exist are lossy in a second way. Message 85 moved the Room to `entrance_hall`
while the text still had the two of them at the side gate. Message 86 emptied the Characters present of
everyone but Gabriella, and its text has Carla walk beside Julian ("Before they'd taken more than a dozen
steps").

**inferred** Of 11 visible changes, 3 are in a record (numbers 4, 5, 11) and each of the 3 is partial. The
answer the ticket expected holds: most of the story's state is in no record, and only the history text
carries it.

**known** The state does not grow with the story. Every stored Swipe state in Game 6 has exactly four keys:
`movement`, `scene`, `npc_encounter_log` and `trigger_context`. The `encounters` map grew to 2 Characters in
50 Messages.

## 7. Facts for the named later tickets

### For [06-the-layout-model](../issues/06-the-layout-model.md): a place for state in the prompt

- **known** Both reference apps put state in the **user half, near the newest history, inside its own
  wrapper**. ME injects a `user` message with `contextKind: "injection"` before the newest history block.
  ST injects an entry at a chosen position and depth. Inform has no prompt.
- **known** ME gives each tracker its own wrapper and its own toggle ("Add as Prompt Section"). The block is
  grouped under one `<context>`, and the Markdown variant carries a sentence that says the state is
  established and must not be re-described.
- **known** ME has an incremental state format, so an update names only the fields that changed. The block
  in the prompt still holds the whole current state of each enabled tracker.
- **inferred** We have two places to put a state block, and they trade against each other. The existing
  `<GameState>` layer sits early in the user half and holds one Room and one list today. A block there keeps
  the prefix stable and is a natural host of the durable kinds. A block after the history, as ME does, is
  fresher in the prompt and breaks the cache on each cycle
  ([12-prompt-caching](../issues/12-prompt-caching.md)).

> **Coordinator check (inferred, from [the caching asset](prompt-caching.md)).** The cache trade-off is the reverse. A provider cache reads a prefix, so a block that changes breaks the cache for everything after it. The early `<GameState>` block is one of the measured causes of a break on every call. A state block after the history breaks only itself and the parts after it: the history before it stays cached. So a block after the history is both fresher and better for the cache.

### For [10-quantifier-options-layout](../issues/10-quantifier-options-layout.md): the Quantifier as one state writer

- **known** ME separates two writers. The model writes `location` and `time` by a tool in its own call. A
  second call writes every other field. The split needs no new prompt part in the narrator call beyond the
  tool schema.
- **known** ME keeps the tool's answer to two types only, so a wrong model answer costs one line of
  narration rather than a corrupt snapshot.
- **known** Every other tool found here uses one extraction call after the reply, and none of them trusts
  the reply text to carry the state in a parseable form except RPG Companion's **Together** mode, which
  hides the block from the player.
- **inferred** A Quantifier that answers more fields keeps one call per cycle. Its parser then depends on
  more of its answer, so more of its prompt becomes a parser contract, which is the trade in ticket 10.
- **known** A failed ME tracker leaves the old row. A failed Quantifier empties our Characters present.
  The two failure rules differ, and ours is the unsafe one.

> **Coordinator check (known).** The player is not fully blind to the failure. A failure banner exists since commit `29fb773a` (2026-10-09), and the Settings page has a Quantifier role-health row (`src/adapters/driving/http/builders/headers.rs`). But the banner text is "Quantifier failed — keeping the Characters present" (`role_failure_effect`), and the code does the opposite: `game_rules::apply` sets `characters_present` from the empty result (`src/domain/model/utils/game_rules.rs:67-71`). The doc comment on `AgentRunner::after_narration` (`src/application/agents/runner.rs:69-70`) says this empty result is intended. Whether the banner showed during the Game 6 run is unknown.
>
> **Correction (2026-10-31).** The check above is wrong for a model failure. After 2 failed attempts, the Quantifier falls back to the previous Characters present (`Low` confidence, `static_character_result`), so the banner is correct. Only the runner's `Err` path clears them. See [Decide what the state does when a state writer fails](../issues/15-failed-state-writer.md).

### For [08-test-setup-and-trial](../issues/08-test-setup-and-trial.md): state changes in the reading pack

- **known** A state change is invisible in a transcript unless the pack also prints the state row. ME has
  three views of the same record: the tracker panel, the **State Editor** and a change stream
  (`game_state_patch` in its SSE reply).
- **inferred** The cheapest reading pack shows, per Cycle, the Swipe state of the Narration and the diff to
  the Swipe state of the message before it. A field that never changes, or that changes every Cycle without
  a visible cause, is then visible to the owner at once.
- **known** A stored Swipe state of Game 6 is 3839 to 4516 bytes of JSON (median 4516 over 50 Messages),
  and most of it is the full cards of the Characters present. **inferred** A pack that prints the full state
  per Cycle is mostly repeated card text, so a diff is both smaller and clearer.

## Limits

- **known** No live run of ST or ME was made (see [11-live-run-of-st-and-me](../issues/11-live-run-of-st-and-me.md)).
  Every ME Swipe, commit and error claim comes from code, not from a run.
- **inferred** The swipe rule of zTracker is not stated in its README. Its store is per Message, so a
  rewind is likely. This note does not claim it.
- **known** ME marks its incremental format inside the prompt itself, with the text
  `tracker_incremental_updates: supported`. The engine contract and the author-facing prompt are therefore
  the same text, and an author who edits the prompt can break the contract.
- **guessed** No source found here gives a cost figure for a tracker call in money or in tokens. The
  figures above count characters, or they come from the stored prompts of our own Game.
- **I don't know** whether zTracker, StatSuite or BetterSimTracker rewinds its state on a swipe. Their
  READMEs do not say, and only RPG Companion states a per-swipe rule. ME's rule is in its code, so that one
  is known.
- **I don't know** of a measured accuracy figure for any of these extractors. `evaluation-tools.md` lists
  judges, not extractor accuracy, and the StatSuite README offers its own errors as training data with no
  number.
- **known** This note does not design a state model, as the ticket asks. It gives the owner the facts to
  decide whether this map designs long-term state or only keeps room for it.
