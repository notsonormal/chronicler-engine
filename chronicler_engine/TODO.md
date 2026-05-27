# QUICK TODO List

## Added: 2026-05-27

- [] When a character has left a scene, they should remain in the system prompt for at least 3 turns.
- [] After a character has left the scene, or after a certain amount of time we will need to auto summarize. We will also need to autosummaize
- [] We will need to stop sending chat history to the LLM past a certain budget. Instead, sent the summary. Perhaps storing the summary in a history or something. 

## Added: 2026-05-24
- [x] Need to split up the system prompt into different parts (role, writing style, output format, etc)

## Added: 2026-05-14

- [] Pressing "Reset Game" should cancel any LLM generation, as we don't want to have them being pushed into the new game
- [] This error is a problem: `[System] Entered unknown location: dynamic_1778882402`. We shouldn't be showing an error message here. For one, we shouldn't be writing an error message into the conversation history. For another a 'dynamic location' isn't an error state. It just be displayed normal naturally. Probably just updating the location in the message box to `Foyer (Dynamic)` instead of `foyer`. 
- [] The style of writing is a bit annoying. Should be an issue with the prompts
- [x] The list of LLM messages doesn't include the quantifier for some reason. 
- [] When you edit the player text and retry the next message, the player text is reverted
- [x] Should remove the meaningless 'sync' actions. Look, inventory, north/south/etc from the options and on the bottom left of the screen. Remove inventory from the system prompt as well
- [] If the "Send" text box is empty. It should trigger a new narrator text generation (e.g. like Silly Tavern/Marina)
- [] Move connections in a separate connections tab.
- [] Create a presets tab for configuring narrator/event/quantifier prompts in the connections tab. Also turn it into a normal set of list + add/edit
- [] Reimplement message swipes
- [] Need to support replacement strings (e.g. {{user}})

## Added: 2026-05-13

- [] Some modules do that have associated test files in many cases e.g. `chronicler_engine\src\engine\game_service\retry.rs, chronicler_engine\src\server\fragments\checkpoint.rs. Check whether to simply go through them and add coverage, or explictly add a guardrail to check that rs classes (not enums) over a certain length should always havev and associated _test.rs

## Added: 2026-05-12

- [] The error (the red run over the top) overlaps the [Reset Button]. It also disappears and reappears every time it loads
- [] Character should not be disappearing and appearing on the UI as it processes events, it's a bit annoying
- [] For characters, we only include their relationship with other characters if they are in the room. It would probably be better to include all their relationships, although that makes things complicated, since we won't have the full character card to establish context for that relationship.
    - Perhaps we also should include full character cards if we think they are relevant to the current scene, even if they aren't present?