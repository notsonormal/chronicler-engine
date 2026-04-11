# Specification: Game Master Narration System

**Status:** Completed

## Objective
Transform the engine from a strict command parser into a hybrid free-text narrative engine. Player input that does not match a recognized system command is interpreted by an LLM acting as a **Game Master**, who narrates the outcome based on the current game state.

## System Commands vs Free Actions
The engine recognizes two categories of player input:

1. **System Commands**: Hard-coded actions that directly mutate engine state. These are not sent to the LLM. Examples: `look`, `walk to <target>`, `inventory`, `quit`.
2. **Free Actions**: Everything else. The player's raw text is forwarded to the Game Master LLM for narrative interpretation. The engine must never respond with an error message for non-empty free-text input.

## Game Master Role
The LLM operates as a Game Master / Narrator for the text adventure. Its context window is constructed from the current game state:

- **World Lore**: The `WorldCard.global_rules` provide persistent setting and lore context.
- **Room Context**: The current `Room.name` and `Room.description` ground the scene.
- **Present NPCs**: All `NpcCard`s located in the current room, including their `personality`, `scenario`, and `description`. The Game Master may voice any of these NPCs naturally in response to the player's action.
- **Player Identity**: The `PlayerCard.name` and `PlayerCard.description` for reference.

The Game Master must:
- Narrate the outcome of the player's stated action.
- Voice NPCs that would logically react to the player's action.
- Never act or speak on behalf of the player.

## Relationship to Existing `talk` Command
The `talk <npc> "message"` command is **soft-deprecated**. It continues to function, routing through the existing `generate_dialogue` path for direct one-on-one NPC conversations. However, players may achieve the same result by typing freely (e.g., `"Hello Carla!"`), which the Game Master will handle naturally by voicing the appropriate NPC.

## LLM Abstraction
Per the testing strategy defined in `00_testing_strategy.md`, the Game Master narration must follow the `LlmBackend` trait pattern. A `MockBackend` implementation must exist for deterministic unit testing without network calls.

## REPL Prompt Display
Before each input prompt, the engine displays the available system commands as clear, labeled actions based on the current room state. This replaces raw direction names with human-readable command hints:

```
[Move North] [Look] [Inventory] [Quit]
> _
```

The movement commands shown are derived dynamically from the current room's exits. Non-directional system commands (`Look`, `Inventory`, `Quit`) are always shown. Any text the player types that does not match these commands is treated as a Free Action.

## Example Session
```
=== Front Gates ===
Large rusted iron gates marking the entrance to the Redmist Estate...
You see: Carla
Exits: North

[Move North] [Look] [Inventory] [Quit]
> Hello Carla, I'm the new heir.

*Carla's eyes narrow behind her sunglasses as she studies you carefully.
She uncrosses her arms and extends a firm hand.* "So you're the one
Bernard mentioned in his will. I'm Carla — your bodyguard from here on
out. Let's head inside."

[Move North] [Look] [Inventory] [Quit]
> I examine the iron gates closely

*The gates are old, forged from heavy wrought iron with an ornate 'R'
worked into the metalwork. Rust creeps along the lower hinges, but the
frame remains solid. Carla watches you with mild impatience.*
```

## Boundaries
The Game Master is **narrative only** in this spec. It does not mutate engine state (e.g., it cannot move items into inventory or change the player's room). State mutation via LLM function calling is deferred to a future specification.
