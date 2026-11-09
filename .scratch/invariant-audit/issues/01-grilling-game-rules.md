# Decide the Game rules: Quantifier output, exits, Trigger order and Retrigger

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

What is the verdict for each of these six findings, and what task tickets put the verdicts in place?

- **The empty scene from a Quantifier answer with names (llm-01).** An answer that leaves out `characters_present`, or names the Characters by name and not by id, empties the Characters present. Every Character then gets a `Left` transition.
- **The duplicate Room from a destination name (llm-02).** A destination that is a Room name and not a Room id makes a new dynamic Room. The same place twice gives two Rooms.
- **Movement that ignores exits (llm-03).** Nothing reads `Room::exits`, so the Persona can move to any Room of the map.
- **The random Trigger order (world-04).** When two Triggers match, the one that fires depends on the iteration order of a `HashMap`. That order is random for each map instance, so a Retry can fire a different event.
- **The second event of a one-shot Trigger (story-06).** Delete the event Narration, then press Retrigger: the same one-shot event is written into the story again. A Retrigger trusts the event record and skips the fired mark.
- **The Trigger index of an encounter (world-06).** An encounter keeps `trigger_fired` by the index of the Trigger in its Character's list. Nothing makes the index name the same Trigger for the whole life of a Game.

The choices to put to the owner:

1. What an answer that leaves out the field means: no reading (Low confidence, so the Swipe state keeps its Characters present), or an empty scene.
2. How a Character or a destination resolves: by id only, or by id and then by name. Over which Rooms: the map Rooms, the added Rooms of the Swipe state, or both. When a new dynamic Room is made, and how two answers for one place become one Room.
3. Exits: must the Persona move only through an exit of the current Room? If yes, what happens to a destination that no exit reaches (stay, a new Room, or accept it with a warning), and does the Quantifier prompt list only the Rooms that the Persona can reach? If no: remove `Room::exits` from the code and the World files (enforcement preference 1), or accept the field as author notes.
4. The order when several Triggers match: Character id order (`CharacterCard::in_id_order` exists, and the prompt already uses it), the order of the World files, or a priority that the author sets.
5. What a Retrigger means for a one-shot Trigger after its event Narration was deleted: allowed (Retrigger exists to repeat), or refused. What the player sees in each case.
6. The index of world-06: accept it (a Character row does not change during a Game, so state the reason), or key the fired mark by a stable Trigger id (a new field in the Character file schema).

## Context

- Findings in full: llm-01, 02, 03 in [the llm report](../assets/llm.md), world-04 and world-06 in [the world report](../assets/world.md), story-06 in [the story report](../assets/story.md). The coordinator confirmed llm-01, llm-02 and world-04 in the code, and read the event-record half of story-06.
- Known, at `8b23151c`:
  - `extract_characters` labels any JSON that parses as High confidence, also when the list is empty (`src/application/agents/quantifier/utils/parser.rs:33-42`). `process_quantifier_result` then drops every id that is not a Character key of the World (`orchestration.rs:160-185`).
  - The Quantifier prompt prints each Character and Room with its `id` and its `name` side by side (`quantifier/prompt.rs:40-49`). So an answer with names is a natural answer, and llm-01 is more likely than its report says.
  - `walk` matches Room ids only (`src/domain/model/utils/game_rules.rs:136`). `Room::new_dynamic` keys the new Room by a timestamp, not by the destination text.
  - `evaluate_triggers` iterates `characters.values()` of a `HashMap`, and `fired_trigger` takes `.next()` (`game_rules.rs:87-90,166-174`).
  - `Tail::retrigger_record` allows a Narration or an Impersonate Input with an event record in its current state (`src/domain/model/tail.rs:47-58`).
- Earlier decisions and specs:
  - [Keep the state when a state writer fails](../../prompt-flexibility/issues/23-keep-state-when-a-writer-fails.md) (prompt-flexibility) says that a failed or missing after-narration Agent keeps the Room and the Characters present. It bounds choice 1.
  - The spec of Retrigger is `docs/specs/retrigger.md`. Read it before choice 5: the answer may already be there.
- [Ticket 02](02-grilling-runtime-world-validation.md) waits for choice 3. If exits matter, the World validation must check that each exit names a Room.
- The default Quantifier preset is `data/prompt_presets/quantifier/default.json`. An author can edit the preset, and the parser has only one alias (`npcs_in_room`).
- Use `/grilling` and `/domain-modeling`. Six choices take two rounds at most. Read the findings in the code again first (map Notes).

## Answer
