# Find how ST, ME and other tools keep long-term state

Type: research
Status: resolved
Blocked by: —

## Question

Which facts of a story change through long play, and how do SillyTavern, Marinara Engine and other interactive-fiction tools record them: relationships, what a Character knows, mood, injuries, items, quests, time, and changes to a Character card itself? Which call writes each change, how does an author define which state exists, how does the state reach a prompt, and how does it follow a swipe?

## Context

- **Owner's view (this session).** The engine has no long-term state beyond the Room and the Characters present. Long play and Autoplay need state that changes, and memories added to Characters. It is a large unknown and context for most decisions of this map. [Find how ST, ME and other tools keep long-term memory](13-long-term-memory.md) covers the memory of events. This ticket covers the facts that are true now.
- **What our engine keeps (known).**
  - The Swipe state: the Room, the added Rooms, the Characters present, the encounters, the fired Triggers and the event record (`src/domain/model/state/swipe_state.rs`, `CONTEXT.md`).
  - The Quantifier returns only the Characters present and a destination (`src/domain/model/quantifier.rs`). The Game rules apply it to the Swipe state with no model call.
  - Each Character's `relationships` entry has a `static` and a `dynamic` text, and the prompt shows `dynamic` when it is set (`src/domain/model/character.rs`). **inferred** Nothing writes `dynamic` during play: only the JSON sets it.
  - Characters and Personas have an `inventory` list. Triggers fire on encounter counts.
- **Where a writer could run (known, `CONTEXT.md`).** An Agent runs at a Hook: the after-narration Hook or the end-of-Job Hook (`src/application/agents/`).
- **Start points in ME (known, from [Find the tools that evaluate characters and prompts](../assets/evaluation-tools.md)).** The tracker agents (World State, Character Tracker, Beholder, Relationship Tracker, Persona Stats, Custom Tracker, Inventory Tracker, Quest Tracker) and the Card Evolution Auditor, which proposes durable card changes every 8 messages and names what to ignore (temporary mood, place, clothes, healed injuries). Also the RPG stats and tracker fields on a card (`rpgStats`, `trackerCustomFieldDefaults` in `packages/shared/src/types/character.ts`) and the Game Mode state (`packages/server/src/db/schema/game-state.ts`). Clone: `/tmp/research/marinara`.
- **Start points elsewhere.** ST has no core tracker. Find the ST extensions that track state, and say whether each is maintained. Interactive-fiction tools: AI Dungeon (Story Cards, the state that scripts keep), NovelAI, and text-adventure state models (flags, variables, a world model as in Inform). Methods with primary sources: state extraction after each reply, structured JSON state against free text, and the Generative Agents reflection step as a state writer.

Points to cover:

1. **Kinds of state.** List the kinds that the tools track. Sort them by how fast they change (each Message, each scene, durable) and by owner (the World, one Character, a pair of Characters, the Persona).
2. **For each method:** what it stores (free text or typed values), which call writes it (an extra model call, a rule, or the user), how often, where it goes in the prompt, its cost per cycle, and what an author defines beforehand (the fields, their ranges, their starting values).
3. **Changes to a card.** How does a tool change a Character card itself, as ME's Card Evolution Auditor does? Who approves the change? How does the tool keep the authored card apart from the changes of one Game?
4. **Swipes and edits.** How does each method keep its state true when a Message is swiped, retried, edited or deleted? Compare with our rule that each Swipe holds its own Swipe state.
5. **Errors.** What do the tools do when the writer call gets a state wrong, or when it drifts from the story? Is there a check, a manual edit, or a fresh extraction?
6. **Our story, read (the outside check).** In [the story transcript](../assets/autoplay-story-transcript.md) or a stored Autoplay Game, list the state changes that a reader can see (a relationship that moved, an item given, a promise, a time of day). For each one, say which of our records holds it today. **inferred** The answer is mostly "none".

## Output

An asset `assets/long-term-state.md` with:

- the kinds of state, sorted by speed and by owner
- a table of methods: what is stored, who writes it, when, where it goes, its cost, what the author defines, and its Swipe behaviour
- the state changes in our story and where each one is recorded today
- facts for [Decide which parts of a Narrator-role call an author can change](06-the-layout-model.md) (a place for state in the prompt), [Decide which parts of a Quantifier or Options call an author can change](10-quantifier-options-layout.md) (the Quantifier as one state writer among several) and [Shape the test setup and the trial](08-test-setup-and-trial.md) (show state changes in the reading pack)

This ticket finds facts. It does not design a state model. The answer must give the owner what they need to decide whether this map designs long-term state or only keeps room for it.

## Answer

Asset: [How ST, ME and other tools keep long-term state](../assets/long-term-state.md).

- **known** ME is the reference for our shape. Every tracker is a second model call after the reply, it
  writes typed JSON into one row of `game_state_snapshots` keyed by `(messageId, swipeIndex)`, and an
  injected `user` message before the newest history block carries the state into the next prompt.
- **known** ME separates the writers. The narrator model may change only `location` and `time`, by a tool in
  its own call. A tracker call owns every other field.
- **known** SillyTavern core has no tracker. It has chat variables that a slash command writes, and World Info
  timed effects (`sticky`, `cooldown`, `delay`) that the keyword scan writes. A swipe *resets* a timed effect
  ("chat not advanced") and never rewinds a variable, because both live in `chat_metadata`, not in a Message.
- **known** Every ST state tracker is a third-party extension. The maintained ones (zTracker, npc_state_delta,
  merged_world_tracker) copy the ME shape. The abandoned one (BetterSimTracker) and the stale one
  (SillyTavern-State) keep their output as chat Messages, and the history cut deletes it.
- **known** ME changes a card only by a proposal that a person approves, with a verbatim `oldText` check, a
  stale marker and a card version history. It never writes on its own. The tracker `PresentCharacter` is a
  scene snapshot that names the card by id, so the card record and the Game record stay apart.
- **known** Inform 7 shows the alternative to extraction: one authored sentence form is the starting value
  (`S.`), the test (`if S`) and the change (`now S;`). Typed fields with a range, not free text.
- **known** AI Dungeon and NovelAI keep no state that changes through play. All AI Dungeon state lives in a
  script's `state` key-value store, which no swipe rewinds.
- **known** Our engine keeps 5 kinds today: `movement.current_room_id`, the added Rooms,
  `scene.characters_present`, `encounters` (with `trigger_fired`) and `trigger_context`. `relationships[].dynamic`
  and `inventory` exist as fields and no writer and no prompt part uses them.
- **known** The outside check on Game 6: 11 visible state changes, 3 of them in a record, each in part.
  Jezebel has a card and never enters the state. A Character with no card is dropped by `filter_map`.
- **known** The failure rule is the unsafe part. A failed Quantifier is a logged warning and
  `QuantifierResult::default()`, and `game_rules::apply` then clears `characters_present` and keeps the Room.
  In the stored Game 6, 24 of 30 Quantifier calls failed. Coordinator check (known): a failure banner and a Settings role-health row exist, but the banner says "keeping the Characters present" while the code clears them.
  **Correction (2026-10-31, [Decide what the state does when a state writer fails](15-failed-state-writer.md)):** wrong. A failed model call keeps the previous Characters present through the `Low` confidence fallback. Only the `Err` path of the runner (no saved text or no Room) clears them. The banner is correct.

Facts the later tickets need:

- **06-the-layout-model:** both apps put state in the user half, near the newest history, inside its own
  wrapper. The two candidate places for us trade against each other: the early `<GameState>` layer keeps the
  cache prefix stable, a block after the history is fresher. The state of one Swipe is 3839 to 4516 bytes of
  JSON, mostly repeated card text, so a reading pack wants a diff.
- **10-quantifier-options-layout:** the Quantifier is today the only state writer, it fails silently, and its
  median call costs about 2380 tokens against the Narrator's about 5420. ME's split (a two-type tool call plus
  a full extraction call) is one way to keep the narrator prompt free of a parser contract.
- **08-test-setup-and-trial:** a state change is invisible in a transcript. The reading pack should print the
  Swipe state per Cycle, or its diff, or the owner cannot see a change, a stuck field or a lost Character.
