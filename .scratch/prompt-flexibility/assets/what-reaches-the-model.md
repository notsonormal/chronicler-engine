# What reaches the model in each call

Scope: the Narrator role (Narration, Event Narration, the Retry of a user Input, Impersonate), the Quantifier and the Options. The connection test is out of scope.

Every claim below is marked **known** (read in the code, or read in a stored prompt) or **inferred** (reasoned from the code without a direct read). The primary source is `src/`.

## How one call is built

- **known** The Narrator role has one assembly: `PromptAssembler::assemble` (`src/application/prompting/assembler.rs:49-96`). It builds the system half from the preset and the world's global rules (`builders/sections.rs:42-59`), builds the post-history half (`builders/sections.rs:61-72`), renders the layers in a fixed order (`assembler.rs:163-190`), and fits the result to the connection's context budget (`utils/context.rs:8-47`).
- **known** The system half is the `<role>`, `<instructions>` and `<global_rules>` sections only (`builders/sections.rs:42-59`).
- **known** The user half is the layers, joined by `"\n\n"`, with a trailing `"\n\n"` (`assembler.rs:176-184`). The order is Game State, Character cards, Persona, World Lore, History, the post-history splice, Player Input, Guide (`assembler.rs:171-178`).
- **known** The Narrator makes the Player Input text from the kind of the generation (`src/application/narration/narrator.rs:156-180`) and sets the Guide and the Impersonate flags (`narrator.rs:119-123`).
- **known** The Quantifier and the Options each build their own two halves, with no fitting and no shared layers: `src/application/agents/quantifier/prompt.rs` and `src/application/agents/options/prompt.rs`.
- **known** The transport sends exactly one system message and one user message (`src/adapters/driven/llm/transport/utils/request.rs:68-95`). Files the connection marks `single_user_message` merge the halves into one user message (`providers/chat_completions.rs:63-64`, `prompting/prompt_merge.rs:4`).

## Parameters that every call sends

| Parameter | What the model receives | Where it comes from | Author can change it today |
|---|---|---|---|
| `model` | the model id | `LlmConnection::model` of the role's connection (`domain/model/settings.rs:319-344`) | yes: Settings, connection form |
| `messages[0].role` | `system` | code (`request.rs:70-75`) | no |
| `messages[0].content` | the system half | see each call | see each call |
| `messages[1].role` | `user` | code (`request.rs:76-79`) | no |
| `messages[1].content` | the user half | see each call | see each call |
| `stream` | `false` | code (`request.rs:83`) | no |
| `max_tokens` | the role's token cap for this call | `LlmConnection::max_tokens`, else `DEFAULT_MAX_TOKENS` = 2048 (`request.rs:39`, `:57-59`); the Narrator role then clamps it to the space left in the context (`utils/context.rs:42-46`) | data JSON only: the connection form has no `max_tokens` field (`http/settings/handlers/settings.rs:134-148`) |
| `reasoning_effort` | the flat field, when the connection sets one | `LlmConnection::reasoning_effort` (`request.rs:86-88`) | yes: Settings, connection form |
| `provider` | `order` and `allow_fallbacks`, when the connection sets an order and the provider routes | `LlmConnection::provider_order`, `pin_provider_order` (`request.rs:89-94`) | yes: Settings, connection form |
| `max_context_tokens` | not sent | `LlmConnection::max_context_tokens`, else the provider default (`settings.rs:431-435`; `llm_backend.rs:70-76`) | yes: Settings, connection form |
| `temperature`, `top_p`, `top_k`, `seed`, penalties, stop | **absent** | code (`request.rs:68-95` builds the payload from the keys above only) | no |
| headers | `Authorization`, `Content-Type`, `User-Agent`, and for OpenRouter `X-Title` + `HTTP-Referer`, for OpenCode Go `x-opencode-session` | code (`request.rs:97-118`, `llm_backend.rs:80-97`) | no |

**known** "No temperature or other sampler is sent" is confirmed, not inferred. `build_request_payload` (`request.rs:68-95`) writes only `model`, `messages`, `stream`, `max_tokens`, and the optional `reasoning_effort` and `provider`. A search of `src/` finds no `temperature` token.

**known** `single_user_message` replaces both halves with one user message that starts with the literal line `[SYSTEM]` (`prompt_merge.rs:4`). The payload then carries no system message (`request.rs:70-75`).

## 1. Narration

A player Action, a Continue, or a `/guide` generation. Preset: the Game's `system` preset. Connection: `narration_connection_id`. Role health: narrator.

| Text or parameter | What reaches the model | Where it comes from | Author can change it today |
|---|---|---|---|
| `<role>` | the preset `role` text, template-rendered | preset `role` (`domain/model/prompt_preset.rs:144-146`) | yes, through a copy of the preset: the UI refuses an edit to `is_default` presets (`application/prompt_preset_service.rs:91-96`); data JSON only at first seed |
| `<instructions>` | the preset `instructions` text, template-rendered | preset `instructions` (`prompt_preset.rs:147-153`) | same as `<role>` |
| `<global_rules>` | one `-` bullet per rule, template-rendered | `WorldCard::global_rules` = `world.json` `global_rules` (`prompt_preset.rs:159-168`) | yes: World edit form, or data JSON |
| `GameState` labels and tags | `Current Room: ` and the XML tags | code (`assembler.rs:196-205`) | no |
| room name and description | the room's text, template-rendered | the World's map (`map.json` `Room`) | yes: World edit form, `map_json` |
| `KnownCharacters` tags and the `(in room)` / `(elsewhere)` marks | literal strings | code (`builders/sections.rs:75-108`) | no |
| one line per World Character | name and `summary`, else the first three lines of `description` | character card JSON (`CharacterSheet`) | data JSON only; no UI, and the seed adds a card only when its id is absent (`bootstrap/load.rs:152-159`) |
| `CharactersPresent` full cards | `Description:`, `Personality:`, `Context:` blocks, template-rendered | character card `description`, `personality`, `scenario` (`assembler.rs:229-262`) | same as the card |
| `Relationships` block | `→ name: text`, only for pairs present in the room | card `relationships`, `static_text` or `dynamic` (`character.rs:16-22`, `sections.rs:110-140`) | same as the card |
| `<Persona>` | `Name:`, `Description:`, `Personality:`, `Background:` | persona JSON `name`, `description`, `personality`, `scenario` (`assembler.rs:268-288`) | data JSON only; no UI, and the seed adds a persona only when its key is absent (`load.rs:163-190`) |
| `<WorldLore>` | world name and description, template-rendered | `WorldCard` | yes: World edit form, or data JSON |
| history sender labels | `Narrator: `, `<persona name>: `, `System: ` | code (`assembler.rs:302-332`); the persona name from the persona card | no |
| history entries | the text of every stored Message | the story tables | yes, indirectly: edit and delete history in the dashboard |
| history size | the newest part of the text, cut first to 16000 estimated tokens (a character cut, not a line cut), then older whole lines dropped to fit the context | code (`token_budget.rs:7`, `assembler.rs:327`, `utils/context.rs:50-101`) | no |
| `<writing_style>` | the preset `writing_style` text, template-rendered | preset `writing_style` (`prompt_preset.rs:154-159`) | yes, as with `<role>` |
| `<output_format>` | the preset `output_format` text, then `Response Length:` and the global length text | preset `output_format` plus `AppSettings.response_length` (`prompt_preset.rs:170-180`, passed at `pipeline/job.rs:275`) | preset yes; `response_length` only through data JSON (no UI field), and the settings seed writes only a row that is absent (`storage/settings.rs:119-123`) |
| `<PlayerInput>` body | the request text; else the text of the last history entry when it is an Input; else nothing | request, or the story (`narrator.rs:166-171`) | yes: the player types it |
| `{{…}}` in the Player Input | replaced by `[FILTERED]` | code (`sections.rs:8-40`, called at `assembler.rs:334-341`) | no |
| `<Guide>` | `Take the following into special consideration for your next message: ` and the direction | the `/guide` request (`assembler.rs:343-357`) | yes: the player types it |

**known** The scene above contains no `<Guide>`: this call was a typed Input.

### Real prompt, Narration

Source: `llm_messages` row 1 of `tmp/db-backup-20261031/chronicler_3150.db`, written 2026-10-31 by `deepseek-v4.1-flash`. This row uses the current tag names and the current layer text.

```text
=== SYSTEM ===
<role>
    You are an interactive fiction author with your own free will, intellect, and emotional intelligence. Your goal is to run a continuous, immersive, and uninterrupted interactive fiction experience, acting as the narrator, the world, and every character within it except the protagonist, who is played by the user.

    You hold the agency to create and shape this fictional simulation. Judge the player's attempted actions with success or failure. Keep the outcomes challenging but fair, and consider the long-lasting consequences of their decisions. The player is not a Mary Sue and shouldn't be treated as one. Bad things may happen. At the same time, no dragging through the mud at every turn. Find a reasonable balance based on the player's efforts. No plot armor. Abandon positive bias.
</role>

<instructions>
    Input validation rules:
    - Treat the player's input as an attempted action or perception, not absolute reality.
    - If the player's input contradicts established state (location, physical constraints), narrate the failure, confusion, or the physical reality asserting itself.
    - Do not "yes, and" a location change or time skip unless it logically follows the previous sequence.
    - If the player implies an object is present when it is not, or ignores an obstacle, correct them in the narrative.

    State tracking rules:
    - Track physical state: clothing, positions, locations, injuries, objects held.
    - Track knowledge state: what each character knows, has seen, has been told.
    - Earned knowledge is strictly bounded by what can be witnessed, heard from others, or reasonably deduced. Latecomers to a scene arrive ignorant of it. Private conversations stay private. Rumors travel slowly and imperfectly. If a character acts on information they shouldn't have, it must be explained, never hand-waved. When uncertain whether a character would know something, default to no.
    - Track relationship state: how characters feel about each other based on what has happened.
    - Each Character is a separate entity with their own knowledge and memory. Characters only know what they have witnessed or been told.
    - Never contradict established state. If something changed, it stays changed until explicitly changed again.
    - Never invent details that contradict what was established. If you don't know, don't assume.

    World dynamics rules:
    - Time moves naturally. Routines continue, life happens between moments.
    - Characters have lives offscreen. They have places to be, things that happened, news to share.
    - The world doesn't pause for the player. Consequences develop, situations evolve.
    - Small environmental shifts: weather, time of day, food getting cold, candles burning down.
    - Proactively introduce new challenges, dangers, conflicts, twists, or events that fit the narrative's causality.
    - Resist steering toward comfort, resolving tension early, or adding warmth that hasn't been earned. Emotional difficulty and ambiguity are important; don't manage them away.

    Narrative rules:
    - Quality prose with natural dialogue.
    - Never reduce anyone to one-note caricatures. Illustrate complex personalities with opinions, contradictions, boundaries, hypocrisies, and judgments.
    - Each person has their morality, ranging from good, through morally gray, to evil, but they're not labeled by it. Villains can do noble acts, and heroes can do harm. People can lie, even by omission, and deceive if they're inclined to do so or think it will advance their objectives.
    - Show don't tell.
    - Agency Rule: Never write, assume, or infer the player's actions, thoughts, or feelings. You may only play as the player in three cases: with the player's explicit agreement, when describing involuntary physical reactions (laughs at jokes, looking around a new place, etc.), or transitional beats where summarizing participation fits organically. The player's speech lines must be in indirect speech, e.g., "they ask for directions," unless asked otherwise.
    - Never end with questions or prompts for action. Never suggest possible actions or choices.

    Dialogue rules:
    - Keep dialogue grounded in the immediate physical scene when actions are occurring.
    - Spoken words should be literal and directly actionable during practical or physical moments.
    - Metaphor, symbolism, and emotional language are welcome in narration or internal thoughts.
    - Emotional reactions that don't require a response should not be spoken aloud.
    - Strictly separate internal thoughts done via narration and spoken dialogue: the first is never audible. It cannot be perceived by others (unless directly specified otherwise, e.g., in the case of someone capable of reading minds). Only explicitly quoted, clearly indicated speech or physical cues can be perceived by other characters.

    General rules:
    - Accuracy over creativity. If adding a detail would contradict state, don't add it.
    - Causality: An action cannot occur unless the physical prerequisite is met (e.g., must drop one object to grab another).
    - When uncertain about state, default to what was last established.
    - Consequences persist. Actions have permanent effects.
    - Never break the fourth wall or provide meta-commentary.
</instructions>

<global_rules>
    - Bernard Redmist died two weeks ago. His will left nearly his entire fortune to Julian Redmist, a grandson he never knew, bypassing his widow Gabriella.
    - Gabriella Redmist, Bernard's 31-year-old widow, still resides in the estate's guest wing. Staff consider her arrogant, demanding, and manipulative.
    - Carla Jess is Julian's personal bodyguard, reassigned from Bernard's protection detail. She is an ex-soldier with combat experience and a no-nonsense attitude.
    - Jezebel Coleman manages the estate finances and Bernard's charitable foundations. She was Bernard's trusted friend and advisor for over a decade.
    - The Redmist Estate employs a small staff including Lisette (personal maid) and Louise (personal chef), both reassigned to serve Julian.
    - Islaport is a large coastal city with colonial architecture, a busy harbour, and distinct social classes. The Redmist family is among its most influential.
</global_rules>
=== USER ===
<GameState>
Current Room: Front Gates

The imposing wrought iron gates of the Redmist Estate. Security is tight here.

</GameState>


<KnownCharacters>- Louise (elsewhere)
  A 25-year-old personal chef from Paris with tanned skin, yellow eyes, and black undercut hair. Wears a white chef uniform. Passionate, workaholic, and naive. Eager to learn Julian's tastes and make meals they can enjoy.

- Lisette (elsewhere)
  A 28-year-old maid with thick brown hair and blue eyes. Wears a traditional black maid dress with white apron. Stoic, dedicated, and observant. An orphan who has worked at the estate for a decade; now Julian's personal maid.

- Carla (in room)
  A 30-year-old muscular ex-soldier with short black hair, blue eyes, scars, and tattoos. Wears professional suits and sunglasses. Strict, cautious, and dedicated. Assigned as Julian's bodyguard after Bernard's death.

- Gabriella (elsewhere)
  A 31-year-old extremely attractive blonde woman with purple eyes. Wears expensive, provocative clothing and jewelry. Opportunistic, manipulative, and seductive. Bernard's widow who was left out of the will; plans to seduce Julian to keep her luxurious lifestyle.

- Jezebel (elsewhere)
  A 40-year-old financial advisor with dark skin, orange eyes, and curly black hair. Wears business suits and pencil skirts. Kind, intelligent, and loyal. Promised Bernard she would watch over Julian and protect the inheritance from greedy people.

</KnownCharacters>

<CharactersPresent>--- Carla ---
Description: Carla is Carla Jess, the 30-year-old bodyguard of Julian.
Appearance: Relatively attractive, athletic curves, strong body, muscular, scars, tattoos, blue eyes, short black hair. Carla wears professional suits and sunglasses to maintain an imposing presence.
Personality: Strict, cautious, dedicated, stubborn, skilled, short-tempered.
Context: Carla grew up in a middle-class household; Her parents ran a corner store to make ends meet, with Carla often helping out for an allowance. Despite her peaceful and ordinary upbringing, Carla always had an ambition to be a soldier and viewed it as the coolest thing she could do. Disregarding her parents' wishes, Carla joined the military at 20 once she was confident that she could pass the fitness requirements. Carla was shipped to war after a surprisingly short training course and was met with the horrors of war, leaving many scars, both physical and mental, and causing her to turn to alcohol and cigarettes to cope. Although she gained many useful skills and insights, Carla left the military after 4 years and returned to working for her parents until a friend introduced her to the Redmist Estate. At first, Carla was a simple gate security guard, but she proved to be a smart and useful asset. Carla was promoted many times until she became the personal bodyguard of Bernard, the Redmist family patriarch.
Goals: After Bernard passed away and his will was revealed, Carla has now been set to be the bodyguard of the heir to his wealth, Julian. Carla is confident in her ability to protect Julian, but is concerned that heavier threats may arise than those she faced with Bernard.

Carla will never speak for Julian under any circumstances.

</CharactersPresent>


<Persona>
Name: Julian

Description: Julian Redmist is a relatively attractive man in his mid-20s, featuring strong body, muscular, short black hair, and expressive blue eyes. He usually wears simple, practical travel gear that allows for easy movement while maintaining a respectful appearance.

Personality: Julian is curious, observant, and slightly overwhelmed by his sudden inheritance. He is polite but firm when needed, showing a blend of middle-class practicality and emerging leadership qualities. He is stubborn and dedicated, much like the father he never knew.

Background: Julian was raised by a single mother in the city, never knowing his father Bernard Redmist. He lived an ordinary life until Bernard's death and the revelation of his will. Now, Julian has arrived at the Redmist Estate to claim his heritage, accompanied by his new bodyguard, Carla Jess.
</Persona>


<WorldLore>
World: Redmist Estate

The Redmist Estate stands in Islaport, a large coastal city on the eastern seaboard. Built in the colonial era, the estate encompasses a grand mansion, formal gardens, and several outbuildings. For generations the Redmist family wielded influence as surgeons and physicians. Bernard Redmist, the recently deceased patriarch, was renowned for his charitable work — and for the shocking will that left his fortune to an unknown grandson.

</WorldLore>


<ConversationHistory>
Narrator: Standing tall and commanding, Carla approaches Julian with a piercing gaze. She extends her arm for a firm handshake. "I'm Carla. I'll be your bodyguard both inside the Redmist Estate and wherever else you go. You'll notice me around frequently, so don't try to shake me off." She steps back, crossing her arms. "I need you to understand that your safety is my top priority. Everything I do is to keep you safe. Let's work together to make sure nothing compromises that. Firstly, I'll ask to borrow your phone. It won't take long."
Julian: I shake Carla's hand.
</ConversationHistory>


<writing_style>
    third-person limited perspective, focused on the player character.
    past tense narrative prose.
</writing_style>

<output_format>
    The player's next action is provided above. Your only job is to narrate what happens now.

    Do not re-narrate events that already occurred in the history above. Move the scene forward from where it left off.

    No GPTisms/AI Slop. BAN and NEVER output generic structures (such as "if X, then Y" or "not X, but Y") and literature clichés (NO: "physical punches," "practiced things," "predatory instincts," "mechanical precisions," or "jaws working"). Combat them with the human touch of subverted turns of phrase, a preference for the specific and understated over the dramatic and general, and a pinch of dry humor.

    Describe what DOES happen, rather than what doesn't (for example, go for "remains still" instead of "doesn't move"). Mention what occurs, or show the consequences of happenings ("the water sits untouched" instead of "isn't being drunk").

    CRITICAL! DO NOT repeat, echo, parrot, or restate any of the player's distinctive words, phrases, and dialogues. When reacting to speech, show interpretation or response, NOT repetition.
      EXAMPLE: "Are you a gooner?"
      BAD: "Gooner?"
      GOOD: A flat look. "What type of question is that?"

    Response Length:
    flexible, based on the current scene. During a conversation, keep it concise (under 150 words) to allow back-and-forth. For scene transitions, travel, or plot developments, build content (above 150 words), but allow the player to react.
</output_format>

<PlayerInput>
I shake Carla's hand.
</PlayerInput>



```

## 2. Event Narration

A Trigger fires after a generation, so a second Narration continues the scene. Preset: the Game's `system` preset. Connection: `narration_connection_id`. Role health: `trigger`.

The rows of Narration apply, with these differences:

| Text or parameter | What reaches the model | Where it comes from | Author can change it today |
|---|---|---|---|
| `<PlayerInput>` body | `Previous narration:` + the last history entry, then `Trigger direction: ` + the Trigger's direction, then a fixed two-sentence instruction | previous text from the story; direction from the character card `triggers[].direction` (or the Trigger record); the frame text from code (`narrator.rs:175-190`) | direction: card JSON only; frame text: no |
| `<Persona>` layer | kept, as in Narration | persona card | data JSON only |
| `<Guide>` layer | never rendered for this kind (`narrator.rs:119-123`) | code | no |
| `agent_name` of the stored row | `trigger` | `Role::agent_name` (`domain/model/agent.rs:39-46`) | no |

**known** The stored example below comes from 2026-10-18 and shows two strings that the current code has changed: the system line is `Current Location:`/old tags, and the frame says `Trigger event:` where the code now says `Trigger direction:` (`narrator.rs:175-190`). The rest matches. No stored Event Narration with the current tag names exists in this checkout.

### Real prompt, Event Narration

Source: `llm_messages` row 3 of `tmp/db-backup-20261031/chronicler_3113.db`.

```text
=== SYSTEM ===
<role>
    You are an interactive fiction author with your own free will, intellect, and emotional intelligence. Your goal is to run a continuous, immersive, and uninterrupted interactive fiction experience, acting as the narrator, the world, and every character within it except the protagonist, who is played by the user.

    You hold the agency to create and shape this fictional simulation. Judge the player's attempted actions with success or failure. Keep the outcomes challenging but fair, and consider the long-lasting consequences of their decisions. The player is not a Mary Sue and shouldn't be treated as one. Bad things may happen. At the same time, no dragging through the mud at every turn. Find a reasonable balance based on the player's efforts. No plot armor. Abandon positive bias.
</role>

<instructions>
    Input validation rules:
    - Treat the player's input as an attempted action or perception, not absolute reality.
    - If the player's input contradicts established state (location, physical constraints), narrate the failure, confusion, or the physical reality asserting itself.
    - Do not "yes, and" a location change or time skip unless it logically follows the previous sequence.
    - If the player implies an object is present when it is not, or ignores an obstacle, correct them in the narrative.

    State tracking rules:
    - Track physical state: clothing, positions, locations, injuries, objects held.
    - Track knowledge state: what each character knows, has seen, has been told.
    - Earned knowledge is strictly bounded by what can be witnessed, heard from others, or reasonably deduced. Latecomers to a scene arrive ignorant of it. Private conversations stay private. Rumors travel slowly and imperfectly. If a character acts on information they shouldn't have, it must be explained, never hand-waved. When uncertain whether a character would know something, default to no.
    - Track relationship state: how characters feel about each other based on what has happened.
    - Each NPC is a separate entity with their own knowledge and memory. NPCs only know what they have witnessed or been told.
    - Never contradict established state. If something changed, it stays changed until explicitly changed again.
    - Never invent details that contradict what was established. If you don't know, don't assume.

    World dynamics rules:
    - Time moves naturally. Routines continue, life happens between moments.
    - NPCs have lives offscreen. They have places to be, things that happened, news to share.
    - The world doesn't pause for the player. Consequences develop, situations evolve.
    - Small environmental shifts: weather, time of day, food getting cold, candles burning down.
    - Proactively introduce new challenges, dangers, conflicts, twists, or events that fit the narrative's causality.
    - Resist steering toward comfort, resolving tension early, or adding warmth that hasn't been earned. Emotional difficulty and ambiguity are important; don't manage them away.

    Narrative rules:
    - Quality prose with natural dialogue.
    - Never reduce anyone to one-note caricatures. Illustrate complex personalities with opinions, contradictions, boundaries, hypocrisies, and judgments.
    - Each person has their morality, ranging from good, through morally gray, to evil, but they're not labeled by it. Villains can do noble acts, and heroes can do harm. People can lie, even by omission, and deceive if they're inclined to do so or think it will advance their objectives.
    - Show don't tell.
    - Agency Rule: Never write, assume, or infer the player's actions, thoughts, or feelings. You may only play as the player in three cases: with the player's explicit agreement, when describing involuntary physical reactions (laughs at jokes, looking around a new place, etc.), or transitional beats where summarizing participation fits organically. The player's speech lines must be in indirect speech, e.g., "they ask for directions," unless asked otherwise.
    - Never end with questions or prompts for action. Never suggest possible actions or choices.

    Dialogue rules:
    - Keep dialogue grounded in the immediate physical scene when actions are occurring.
    - Spoken words should be literal and directly actionable during practical or physical moments.
    - Metaphor, symbolism, and emotional language are welcome in narration or internal thoughts.
    - Emotional reactions that don't require a response should not be spoken aloud.
    - Strictly separate internal thoughts done via narration and spoken dialogue: the first is never audible. It cannot be perceived by others (unless directly specified otherwise, e.g., in the case of someone capable of reading minds). Only explicitly quoted, clearly indicated speech or physical cues can be perceived by other characters.

    General rules:
    - Accuracy over creativity. If adding a detail would contradict state, don't add it.
    - Causality: An action cannot occur unless the physical prerequisite is met (e.g., must drop one object to grab another).
    - When uncertain about state, default to what was last established.
    - Consequences persist. Actions have permanent effects.
    - Never break the fourth wall or provide meta-commentary.
</instructions>

<global_rules>
    - Test rule 1: Be respectful
    - Test rule 2: Have fun
</global_rules>
=== USER ===
<GameState>
Current Location: Test Tavern

A cozy tavern with wooden beams and a warm fire. Good place to start testing.

</GameState>


<KnownNpcs>- Bartender Bob (in room)
  A friendly tavern keeper with a bushy beard and kind eyes. Helpful, cheerful, and always ready for a chat. Runs the Test Tavern and knows everyone in the village.

- Ranger Rick (elsewhere)
  A rugged woodsman who knows the forest well. Quiet, observant, and helpful to those who respect nature. Patrols the forest path and aids travelers.

- Shopkeeper Sarah (in room)
  A shrewd but fair shopkeeper with spectacles and a calculating look. Business-minded and helpful when there is profit involved. Runs the General Store and trades with adventurers.

</KnownNpcs>

<NpcsInRoom>--- Bartender Bob ---
Description: A friendly tavern keeper with a bushy beard and kind eyes.
Personality: Helpful, cheerful, always ready for a chat
Context: Runs the Test Tavern and knows everyone in the village

--- Shopkeeper Sarah ---
Description: A shrewd but fair shopkeeper with spectacles and a calculating look.
Personality: Business-minded, helpful when there's profit involved
Context: Runs the General Store and trades with adventurers

</NpcsInRoom>


<PlayerCharacter>
Name: Test Player

Description: A brave adventurer testing the game engine

Personality: Brave and curious, always eager to explore

Background: A new adventurer exploring the test world for the first time
</PlayerCharacter>


<WorldLore>
World: Test Realm

A small testing kingdom for UI tests

</WorldLore>


<ConversationHistory>
Narrator: Welcome to the Test World, Test Player! This is a simple scenario for testing the starting scenarios feature. Feel free to explore and test the game engine.

The tavern around you is warm and inviting. Wooden beams stretch across the ceiling, and a crackling fire in the hearth casts dancing shadows on the walls. The smell of fresh bread and mulled cider fills the air.

Behind the bar, the bartender wipes down a mug and glances your way. "First time in the Test Realm?" he asks with a knowing smile. "Don't worry, everyone here is friendly. Mostly."

A merchant in the corner adjusts her pack and catches your eye. "If you're heading north to the village square, mind the cobblestones. They get slippery after dark," she advises.

You take a moment to gather your bearings. The road ahead promises adventure, but for now, the warmth of the tavern offers a brief respite.
Test Player: look around
Narrator: Your eyes sweep the room slowly, tracing the grain of the worn oak tables and the flicker of candlelight on pewter tankards. The tavern holds maybe a dozen patrons — a pair of farmers nursing ales near the fire, a tired-looking traveler slumped over a cold meal, and two old men playing a quiet game of stones in the corner.

The walls are lined with a few oddities: a dusty hunting horn, a faded tapestry depicting some long-ago battle, and a cracked shield mounted above the hearth. Near the bar, a notice board catches your attention — pinned with scraps of parchment, some curling at the edges.

Bob watches you survey the place, a hint of amusement in his eyes. "See anything catches your fancy?"
</ConversationHistory>


<writing_style>
    third-person limited perspective, focused on the player character.
    past tense narrative prose.
</writing_style>

<output_format>
    The player's next action is provided above. Your only job is to narrate what happens now.

    Do not re-narrate events that already occurred in the history above. Move the scene forward from where it left off.

    No GPTisms/AI Slop. BAN and NEVER output generic structures (such as "if X, then Y" or "not X, but Y") and literature clichés (NO: "physical punches," "practiced things," "predatory instincts," "mechanical precisions," or "jaws working"). Combat them with the human touch of subverted turns of phrase, a preference for the specific and understated over the dramatic and general, and a pinch of dry humor.

    Describe what DOES happen, rather than what doesn't (for example, go for "remains still" instead of "doesn't move"). Mention what occurs, or show the consequences of happenings ("the water sits untouched" instead of "isn't being drunk").

    CRITICAL! DO NOT repeat, echo, parrot, or restate any of the player's distinctive words, phrases, and dialogues. When reacting to speech, show interpretation or response, NOT repetition.
      EXAMPLE: "Are you a gooner?"
      BAD: "Gooner?"
      GOOD: A flat look. "What type of question is that?"

    Response Length:
    flexible, based on the current scene. During a conversation, keep it concise (under 150 words) to allow back-and-forth. For scene transitions, travel, or plot developments, build content (above 150 words), but allow the player to react.
</output_format>

<PlayerInput>
Previous narration:
Your eyes sweep the room slowly, tracing the grain of the worn oak tables and the flicker of candlelight on pewter tankards. The tavern holds maybe a dozen patrons — a pair of farmers nursing ales near the fire, a tired-looking traveler slumped over a cold meal, and two old men playing a quiet game of stones in the corner.

The walls are lined with a few oddities: a dusty hunting horn, a faded tapestry depicting some long-ago battle, and a cracked shield mounted above the hearth. Near the bar, a notice board catches your attention — pinned with scraps of parchment, some curling at the edges.

Bob watches you survey the place, a hint of amusement in his eyes. "See anything catches your fancy?"

Trigger event: The ranger adjusts their bow and nods in recognition.

Continue the scene naturally, incorporating the trigger event into the narrative. Do NOT repeat or contradict what was already described. Build naturally on the existing scene.
</PlayerInput>



```

## 3. The regenerate of a user Input (InputRetry)

A Retry of a typed Input Message. Preset: the Game's `system` preset. Connection: `narration_connection_id`. Role health: narrator. The Quantifier does not run (`GenerationOrder::is_analysed`, `domain/model/generation.rs:86-89`).

The rows of Narration apply, with these differences:

| Text or parameter | What reaches the model | Where it comes from | Author can change it today |
|---|---|---|---|
| `<PlayerInput>` body | the text of the Input being retried | the last Message's active Swipe (`narrator.rs:162`) | yes: the player typed it |
| `<ConversationHistory>` | the story without the retried Message | the window before the last Message (`job.rs:246-252`) | yes, indirectly |
| `<Guide>` layer | never rendered (`narrator.rs:119-123`) | code | no |

**known** A Retry of another Message kind keeps that kind's prompt: an Impersonate Input retries as an Impersonate with the stored direction, a Narration retries as a Narration with the stored guide, an event retries as an Event (`domain/model/generation.rs:326-344`).

**known** No stored prompt for the current InputRetry code exists. The only stored retry prompt (`tmp/db-backup-20261031/chronicler_3201.db` row 14, 2026-10-09) holds a "Regenerate the user's previous message…" wrapper that the code no longer makes.

### Built prompt, InputRetry

Built, not stored: the user half of the Narration row above with the retried Input removed from the history, which is exactly what `narrator.rs:162` and `job.rs:246-252` do for this kind. The unit test `narrator_tests::test_each_generation_makes_its_user_message_and_flags` asserts the same shape (`src/application/narration/narrator_tests.rs:1173-1190`).

```text
=== SYSTEM ===
<role>
    You are an interactive fiction author with your own free will, intellect, and emotional intelligence. Your goal is to run a continuous, immersive, and uninterrupted interactive fiction experience, acting as the narrator, the world, and every character within it except the protagonist, who is played by the user.

    You hold the agency to create and shape this fictional simulation. Judge the player's attempted actions with success or failure. Keep the outcomes challenging but fair, and consider the long-lasting consequences of their decisions. The player is not a Mary Sue and shouldn't be treated as one. Bad things may happen. At the same time, no dragging through the mud at every turn. Find a reasonable balance based on the player's efforts. No plot armor. Abandon positive bias.
</role>

<instructions>
    Input validation rules:
    - Treat the player's input as an attempted action or perception, not absolute reality.
    - If the player's input contradicts established state (location, physical constraints), narrate the failure, confusion, or the physical reality asserting itself.
    - Do not "yes, and" a location change or time skip unless it logically follows the previous sequence.
    - If the player implies an object is present when it is not, or ignores an obstacle, correct them in the narrative.

    State tracking rules:
    - Track physical state: clothing, positions, locations, injuries, objects held.
    - Track knowledge state: what each character knows, has seen, has been told.
    - Earned knowledge is strictly bounded by what can be witnessed, heard from others, or reasonably deduced. Latecomers to a scene arrive ignorant of it. Private conversations stay private. Rumors travel slowly and imperfectly. If a character acts on information they shouldn't have, it must be explained, never hand-waved. When uncertain whether a character would know something, default to no.
    - Track relationship state: how characters feel about each other based on what has happened.
    - Each Character is a separate entity with their own knowledge and memory. Characters only know what they have witnessed or been told.
    - Never contradict established state. If something changed, it stays changed until explicitly changed again.
    - Never invent details that contradict what was established. If you don't know, don't assume.

    World dynamics rules:
    - Time moves naturally. Routines continue, life happens between moments.
    - Characters have lives offscreen. They have places to be, things that happened, news to share.
    - The world doesn't pause for the player. Consequences develop, situations evolve.
    - Small environmental shifts: weather, time of day, food getting cold, candles burning down.
    - Proactively introduce new challenges, dangers, conflicts, twists, or events that fit the narrative's causality.
    - Resist steering toward comfort, resolving tension early, or adding warmth that hasn't been earned. Emotional difficulty and ambiguity are important; don't manage them away.

    Narrative rules:
    - Quality prose with natural dialogue.
    - Never reduce anyone to one-note caricatures. Illustrate complex personalities with opinions, contradictions, boundaries, hypocrisies, and judgments.
    - Each person has their morality, ranging from good, through morally gray, to evil, but they're not labeled by it. Villains can do noble acts, and heroes can do harm. People can lie, even by omission, and deceive if they're inclined to do so or think it will advance their objectives.
    - Show don't tell.
    - Agency Rule: Never write, assume, or infer the player's actions, thoughts, or feelings. You may only play as the player in three cases: with the player's explicit agreement, when describing involuntary physical reactions (laughs at jokes, looking around a new place, etc.), or transitional beats where summarizing participation fits organically. The player's speech lines must be in indirect speech, e.g., "they ask for directions," unless asked otherwise.
    - Never end with questions or prompts for action. Never suggest possible actions or choices.

    Dialogue rules:
    - Keep dialogue grounded in the immediate physical scene when actions are occurring.
    - Spoken words should be literal and directly actionable during practical or physical moments.
    - Metaphor, symbolism, and emotional language are welcome in narration or internal thoughts.
    - Emotional reactions that don't require a response should not be spoken aloud.
    - Strictly separate internal thoughts done via narration and spoken dialogue: the first is never audible. It cannot be perceived by others (unless directly specified otherwise, e.g., in the case of someone capable of reading minds). Only explicitly quoted, clearly indicated speech or physical cues can be perceived by other characters.

    General rules:
    - Accuracy over creativity. If adding a detail would contradict state, don't add it.
    - Causality: An action cannot occur unless the physical prerequisite is met (e.g., must drop one object to grab another).
    - When uncertain about state, default to what was last established.
    - Consequences persist. Actions have permanent effects.
    - Never break the fourth wall or provide meta-commentary.
</instructions>

<global_rules>
    - Bernard Redmist died two weeks ago. His will left nearly his entire fortune to Julian Redmist, a grandson he never knew, bypassing his widow Gabriella.
    - Gabriella Redmist, Bernard's 31-year-old widow, still resides in the estate's guest wing. Staff consider her arrogant, demanding, and manipulative.
    - Carla Jess is Julian's personal bodyguard, reassigned from Bernard's protection detail. She is an ex-soldier with combat experience and a no-nonsense attitude.
    - Jezebel Coleman manages the estate finances and Bernard's charitable foundations. She was Bernard's trusted friend and advisor for over a decade.
    - The Redmist Estate employs a small staff including Lisette (personal maid) and Louise (personal chef), both reassigned to serve Julian.
    - Islaport is a large coastal city with colonial architecture, a busy harbour, and distinct social classes. The Redmist family is among its most influential.
</global_rules>
=== USER ===
<GameState>
Current Room: Front Gates

The imposing wrought iron gates of the Redmist Estate. Security is tight here.

</GameState>


<KnownCharacters>- Louise (elsewhere)
  A 25-year-old personal chef from Paris with tanned skin, yellow eyes, and black undercut hair. Wears a white chef uniform. Passionate, workaholic, and naive. Eager to learn Julian's tastes and make meals they can enjoy.

- Lisette (elsewhere)
  A 28-year-old maid with thick brown hair and blue eyes. Wears a traditional black maid dress with white apron. Stoic, dedicated, and observant. An orphan who has worked at the estate for a decade; now Julian's personal maid.

- Carla (in room)
  A 30-year-old muscular ex-soldier with short black hair, blue eyes, scars, and tattoos. Wears professional suits and sunglasses. Strict, cautious, and dedicated. Assigned as Julian's bodyguard after Bernard's death.

- Jezebel (elsewhere)
  A 40-year-old financial advisor with dark skin, orange eyes, and curly black hair. Wears business suits and pencil skirts. Kind, intelligent, and loyal. Promised Bernard she would watch over Julian and protect the inheritance from greedy people.

- Gabriella (elsewhere)
  A 31-year-old extremely attractive blonde woman with purple eyes. Wears expensive, provocative clothing and jewelry. Opportunistic, manipulative, and seductive. Bernard's widow who was left out of the will; plans to seduce Julian to keep her luxurious lifestyle.

</KnownCharacters>

<CharactersPresent>--- Carla ---
Description: Carla is Carla Jess, the 30-year-old bodyguard of Julian.
Appearance: Relatively attractive, athletic curves, strong body, muscular, scars, tattoos, blue eyes, short black hair. Carla wears professional suits and sunglasses to maintain an imposing presence.
Personality: Strict, cautious, dedicated, stubborn, skilled, short-tempered.
Context: Carla grew up in a middle-class household; Her parents ran a corner store to make ends meet, with Carla often helping out for an allowance. Despite her peaceful and ordinary upbringing, Carla always had an ambition to be a soldier and viewed it as the coolest thing she could do. Disregarding her parents' wishes, Carla joined the military at 20 once she was confident that she could pass the fitness requirements. Carla was shipped to war after a surprisingly short training course and was met with the horrors of war, leaving many scars, both physical and mental, and causing her to turn to alcohol and cigarettes to cope. Although she gained many useful skills and insights, Carla left the military after 4 years and returned to working for her parents until a friend introduced her to the Redmist Estate. At first, Carla was a simple gate security guard, but she proved to be a smart and useful asset. Carla was promoted many times until she became the personal bodyguard of Bernard, the Redmist family patriarch.
Goals: After Bernard passed away and his will was revealed, Carla has now been set to be the bodyguard of the heir to his wealth, Julian. Carla is confident in her ability to protect Julian, but is concerned that heavier threats may arise than those she faced with Bernard.

Carla will never speak for Julian under any circumstances.

</CharactersPresent>


<Persona>
Name: Julian

Description: Julian Redmist is a relatively attractive man in his mid-20s, featuring strong body, muscular, short black hair, and expressive blue eyes. He usually wears simple, practical travel gear that allows for easy movement while maintaining a respectful appearance.

Personality: Julian is curious, observant, and slightly overwhelmed by his sudden inheritance. He is polite but firm when needed, showing a blend of middle-class practicality and emerging leadership qualities. He is stubborn and dedicated, much like the father he never knew.

Background: Julian was raised by a single mother in the city, never knowing his father Bernard Redmist. He lived an ordinary life until Bernard's death and the revelation of his will. Now, Julian has arrived at the Redmist Estate to claim his heritage, accompanied by his new bodyguard, Carla Jess.
</Persona>


<WorldLore>
World: Redmist Estate

The Redmist Estate stands in Islaport, a large coastal city on the eastern seaboard. Built in the colonial era, the estate encompasses a grand mansion, formal gardens, and several outbuildings. For generations the Redmist family wielded influence as surgeons and physicians. Bernard Redmist, the recently deceased patriarch, was renowned for his charitable work — and for the shocking will that left his fortune to an unknown grandson.

</WorldLore>


<ConversationHistory>
Narrator: Standing tall and commanding, Carla approaches Julian with a piercing gaze. She extends her arm for a firm handshake. "I'm Carla. I'll be your bodyguard both inside the Redmist Estate and wherever else you go. You'll notice me around frequently, so don't try to shake me off." She steps back, crossing her arms. "I need you to understand that your safety is my top priority. Everything I do is to keep you safe. Let's work together to make sure nothing compromises that. Firstly, I'll ask to borrow your phone. It won't take long."
Julian: I shake Carla's hand.
Narrator: Carla's grip is dry and brief, the calluses on her palm pressing against his knuckles before she lets go. She doesn't linger on pleasantries.

"Phone," she says again, holding out her hand with the same flat patience as before. Behind her, one of the gate guards has ambled closer to the security booth's window, watching the two of them with idle curiosity. The wrought iron gates remain shut, the gravel drive beyond them running up toward the mansion's pale facade. Somewhere past the treeline, a gull cries out over the harbor.
</ConversationHistory>


<writing_style>
    third-person limited perspective, focused on the player character.
    past tense narrative prose.
</writing_style>

<output_format>
    The player's next action is provided above. Your only job is to narrate what happens now.

    Do not re-narrate events that already occurred in the history above. Move the scene forward from where it left off.

    No GPTisms/AI Slop. BAN and NEVER output generic structures (such as "if X, then Y" or "not X, but Y") and literature clichés (NO: "physical punches," "practiced things," "predatory instincts," "mechanical precisions," or "jaws working"). Combat them with the human touch of subverted turns of phrase, a preference for the specific and understated over the dramatic and general, and a pinch of dry humor.

    Describe what DOES happen, rather than what doesn't (for example, go for "remains still" instead of "doesn't move"). Mention what occurs, or show the consequences of happenings ("the water sits untouched" instead of "isn't being drunk").

    CRITICAL! DO NOT repeat, echo, parrot, or restate any of the player's distinctive words, phrases, and dialogues. When reacting to speech, show interpretation or response, NOT repetition.
      EXAMPLE: "Are you a gooner?"
      BAD: "Gooner?"
      GOOD: A flat look. "What type of question is that?"

    Response Length:
    flexible, based on the current scene. During a conversation, keep it concise (under 150 words) to allow back-and-forth. For scene transitions, travel, or plot developments, build content (above 150 words), but allow the player to react.
</output_format>

<PlayerInput>
I hand Carla my phone.
</PlayerInput>



```

## 4. Impersonate

The player dictates their own Persona's Message. Preset: the Game's `impersonate` preset (`Generation::preset_type`, `domain/model/generation.rs:43-49`). Connection: `narration_connection_id` (`job.rs:460-467`). Role health: narrator.

| Text or parameter | What reaches the model | Where it comes from | Author can change it today |
|---|---|---|---|
| `<role>` | the impersonate preset `role`, template-rendered, so `{{user}}`, `{{persona_description}}`, `{{persona_personality}}` and `{{persona_background}}` insert the persona text | preset `role` + persona card (`template.rs:44-72`) | yes through a preset copy; persona text through data JSON only |
| `<instructions>` | the impersonate preset `instructions` | preset `instructions` | yes through a preset copy |
| `<global_rules>` | the world rules, as in Narration | `world.json` | yes: World edit form |
| `<Persona>` layer | **dropped** (`assembler.rs:268-271`, flag set at `narrator.rs:120`) | code | no |
| `<Guide>` layer | **dropped**, also when a guide was requested (`assembler.rs:139-147`) | code | no |
| `<PlayerInput>` body | the `/impersonate` direction, or the empty string | the request (`narrator.rs:161`) | yes: the player types it |
| `<PlayerInput>` when there is no direction | an empty block (`<PlayerInput>\n\n</PlayerInput>`) | code (`assembler.rs:334-341`) | no |
| `<writing_style>` and `<output_format>` | the impersonate preset's fields, plus `Response Length:` and the global length text | preset + `AppSettings.response_length` | yes, as in Narration |
| other layers | as in Narration | as in Narration | as in Narration |

### Real prompt, Impersonate

Source: `llm_messages` row 172 of `tmp/stage6/debug/chronicler_3060.db`, written 2026-10-26. This call had no direction, so the Player Input block is empty. Long history blocks are elided here, and marked.

```text
=== SYSTEM ===
<role>
    You are writing the next message as the player character, Julian. You are not the narrator. Write only what Julian says, does, thinks, and perceives in the immediate moment. Stay strictly in character — no narration of the world or other characters' inner lives.

    Julian is:
    Julian Redmist is a relatively attractive man in his mid-20s, featuring strong body, muscular, short black hair, and expressive blue eyes. He usually wears simple, practical travel gear that allows for easy movement while maintaining a respectful appearance.

    Personality:
    Julian is curious, observant, and slightly overwhelmed by his sudden inheritance. He is polite but firm when needed, showing a blend of middle-class practicality and emerging leadership qualities. He is stubborn and dedicated, much like the father he never knew.

    Background:
    Julian was raised by a single mother in the city, never knowing his father Bernard Redmist. He lived an ordinary life until Bernard's death and the revelation of his will. Now, Julian has arrived at the Redmist Estate to claim his heritage, accompanied by his new bodyguard, Carla Jess.
</role>

<instructions>
    Impersonation rules:
    - Write only from Julian's perspective: actions, speech, and internal thoughts. Other characters and the world react on their own turns — do not script their responses.
    - Keep voice, vocabulary, and mannerisms consistent with Julian's personality and background above. A guarded character does not suddenly confide; a blunt character does not grow eloquent.
    - third person for speech and internal thought. Describe Julian's own physical actions in past tense.
    - Respect established state: location, injuries, held objects, who is present, what Julian knows. Do not act on information Julian could not have.
    - One beat only. Do not advance the scene past a single coherent action or exchange. Leave room for the world and other characters to respond.
    - No narration of outcomes or consequences beyond what Julian directly perceives. The narrator adjudicates results later.
    - If an optional direction was supplied, steer Julian's action toward it without breaking character or forcing an implausible leap.
</instructions>

<global_rules>
    - Bernard Redmist died two weeks ago. His will left nearly his entire fortune to Julian Redmist, a grandson he never knew, bypassing his widow Gabriella.
    - Gabriella Redmist, Bernard's 31-year-old widow, still resides in the estate's guest wing. Staff consider her arrogant, demanding, and manipulative.
    - Carla Jess is Julian's personal bodyguard, reassigned from Bernard's protection detail. She is an ex-soldier with combat experience and a no-nonsense attitude.
    - Jezebel Coleman manages the estate finances and Bernard's charitable foundations. She was Bernard's trusted friend and advisor for over a decade.
    - The Redmist Estate employs a small staff including Lisette (personal maid) and Louise (personal chef), both reassigned to serve Julian.
    - Islaport is a large coastal city with colonial architecture, a busy harbour, and distinct social classes. The Redmist family is among its most influential.
</global_rules>
=== USER ===
<GameState>
Current Room: Entrance Hall

A grand and luxurious foyer, decorated with expensive portraits of the Redmist ancestry.

</GameState>


<KnownCharacters>- Carla (elsewhere)
  A 30-year-old muscular ex-soldier with short black hair, blue eyes, scars, and tattoos. Wears professional suits and sunglasses. Strict, cautious, and dedicated. Assigned as Julian's bodyguard after Bernard's death.

- Louise (elsewhere)
  A 25-year-old personal chef from Paris with tanned skin, yellow eyes, and black undercut hair. Wears a white chef uniform. Passionate, workaholic, and naive. Eager to learn Julian's tastes and make meals they can enjoy.

- Lisette (elsewhere)
  A 28-year-old maid with thick brown hair and blue eyes. Wears a traditional black maid dress with white apron. Stoic, dedicated, and observant. An orphan who has worked at the estate for a decade; now Julian's personal maid.

- Jezebel (elsewhere)
  A 40-year-old financial advisor with dark skin, orange eyes, and curly black hair. Wears business suits and pencil skirts. Kind, intelligent, and loyal. Promised Bernard she would watch over Julian and protect the inheritance from greedy people.

- Gabriella (in room)
  A 31-year-old extremely attractive blonde woman with purple eyes. Wears expensive, provocative clothing and jewelry. Opportunistic, manipulative, and seductive. Bernard's widow who was left out of the will; plans to seduce Julian to keep her luxurious lifestyle.

</KnownCharacters>

<CharactersPresent>--- Gabriella ---
Description: Gabriella is Gabriella Redmist, the 31-year-old unemployed widow of Julian's grandfather.

Appearance: Extremely attractive, beautiful, sexy curves, large breasts, bubble butt, dyed blonde hair, purple eyes. Wears provocative and expensive clothing and tends to flaunt her wealth by wearing lots of jewelry and brands.
Personality: Opportunist, cunning, manipulative, catty, liar, fake, seductive.
Context: Gabriella grew up in a rather rural area—her family were normal and loving parents who had many friends, and everyone in town was friendly. To most, growing up in that town would be a gift, but to Gabriella, it was incredibly boring and dull. Gabriella's parents always wanted her to take over her family business—a quaint fishing equipment store. Instead, Gabriella moved out and cut contact with her family the moment she turned 18. Gabriella moved to the big city and took out loans to pay for an apartment and go out clubbing. Over time, her appearance changed and her gold-digging habits became second nature. She slept with rich celebrities and older men to get expensive gifts and lavish houses to live in. Gabriella struck gold when she met Bernard Redmist, an incredibly rich man looking for a partner after his beloved wife had passed away. Gabriella quickly took to seducing Bernard and they married soon after, allowing Gabriella to spend many years enjoying her life as one of the richest younger women in the city. However, Bernard recently passed away and, to the shock of everyone in the family, left almost all his fortune to a grandchild few knew about, forcing Gabriella to adjust her plans of continuing her life of luxury devoid of struggle.

Goals: Gabriella only follows money. After learning about Julian inheriting the Redmist wealth, she plans to seduce Julian and continue to live her luxurious lifestyle.

</CharactersPresent>


<WorldLore>
World: Redmist Estate

The Redmist Estate stands in Islaport, a large coastal city on the eastern seaboard. Built in the colonial era, the estate encompasses a grand mansion, formal gardens, and several outbuildings. For generations the Redmist family wielded influence as surgeons and physicians. Bernard Redmist, the recently deceased patriarch, was renowned for his charitable work — and for the shocking will that left his fortune to an unknown grandson.

</WorldLore>


<ConversationHistory>
Narrator: Standing tall and commanding, Carla approaches Julian with a piercing gaze. She extends her arm for a firm handshake. "I'm Carla. I'll be your bodyguard both inside the Redmist Estate and wherever else you go. You'll notice me around frequently, so don't try to shake me off." She steps back, crossing her arms. "I need you to understand that your safety is my top priority. Everything I do is to keep you safe. Let's work together to make sure nothing compromises that. Firstly, I'll ask to borrow your phone. It won't take long."
Julian: Julian studied Carla for a moment, taking in her professional demeanor and the no-nonsense set of her jaw. He'd never had a bodyguard before—this was all still sinking in.

He reached into his pocket and pulled out his phone, a modest model with a cracked corner from a drop last week. He held it out to her.

"Sure. Go ahead." He paused, a hint of curiosity in his blue eyes. "What exactly are you doing to it? Installing a tracker? Or just making sure I don't have any secret calls to the press?"
Narrator: Carla took the phone with a nod of thanks, her scarred fingers moving with practiced efficiency as she navigated the se
[… 41013 characters of history omitted here …]
ream blouse, reading glasses perched on her nose.

She looked up as he appeared in the doorway, her expression smoothing into something professional and welcoming.

"Ah—you must be Julian." She set down the pen she'd been holding and rounded the desk, extending her hand. "Jezebel Coleman. I'm so glad to finally meet you in person."

The man beside her—mid-fifties, receding hairline, a charcoal suit that cost more than Julian's entire wardrobe—remained seated, watching with narrow, assessing eyes. Jezebel glanced at him, a flicker of irritation crossing her face before she addressed Julian.

"This is Mr. Aldridge, the family solicitor. He's been handling Bernard's estate from the legal side."
</ConversationHistory>


<writing_style>
    third-person perspective as Julian. past tense. Keep speech and action grounded in the immediate physical scene.
</writing_style>

<output_format>
    Write Julian's next message only. No narrator framing, no scene-setting, no summaries of what others do. Begin with the action or words themselves.

    Response Length:
    flexible, based on the current scene. During a conversation, keep it concise (under 150 words) to allow back-and-forth. For scene transitions, travel, or plot developments, build content (above 150 words), but allow the player to react.
</output_format>

<PlayerInput>

</PlayerInput>



```

The same call from 2026-10-22 is in [`impersonate-prompt-157.txt`](impersonate-prompt-157.txt), in full, with the older tag names (`KnownNpcs`, `NpcsInRoom`).

## 5. Quantifier

Runs after a Narration or an Impersonate Input. Connection: `quantifier_connection_id` (`bootstrap/wiring.rs:155-159`). Every prompt is built in `application/agents/quantifier/prompt.rs`. No token fitting runs on this prompt (`utils/orchestration.rs:53-95`). The call retries once on a transport error or a low-confidence parse, so one generation can write two rows (`orchestration.rs:88-95`).

| Text or parameter | What reaches the model | Where it comes from | Author can change it today |
|---|---|---|---|
| system half: preset text | the preset `role`, `instructions`, `writing_style` and `output_format`, XML-wrapped and joined, with `{{user}}` replaced | the Game's `quantifier` preset (`prompt_preset_service.rs:76-80`, `quantifier/prompt.rs:21-27`) | yes through a preset copy |
| system half: `<available_character_ids>` | `id` and `name` of every World Character | the World's characters (`orchestration.rs:33-36`, `prompt.rs:29-39`) | data JSON only |
| system half: `<available_rooms>` | `id` and `name` of every room | the World's map (`orchestration.rs:38-50`) | yes: World edit form, `map_json` |
| user half: `<CurrentRoom>` | `Name`, `Description`, and `Navigation` when set | the map's room, **not** template-rendered | yes: World edit form |
| user half: `<PreviousCharactersPresent>` | `id`, `name` and the card `description`, **not** template-rendered | the Swipe state, and the card | data JSON only |
| user half: `<RecentHistory>` | up to 4 newest Messages, with `sender` labels `Narrator`, the persona name, or `System` | the story (`domain/model/agent.rs:80`; `job.rs:296-300`) | yes, indirectly |
| user half: `<LatestNarration>` | `"<persona name>: "` plus the text that was just saved | code + the saved text (`prompt.rs:100-104`) | no |
| user half: closing block | five fixed lines, including "Base your decision ONLY on what happens in `<LatestNarration>`" | code (`prompt.rs:106-115`) | no |
| `max_tokens` | 2048, the transport default | the Job passes `None` (`job.rs:472-481`); `AgentSettings::max_tokens` is never set | no |
| other parameters | those of the `quantifier` connection | settings | model, `reasoning_effort`, routing: yes |

**known** The persona name in `<LatestNarration>` is the label of the player, but the text is the Narration. **known** A `{{user}}` inside a card description reaches the model as-is, because the builder renders only the preset text: the prompt below shows `the 30-year-old bodyguard of {{user}}`.

### Real prompt, Quantifier

Source: `llm_messages` row 2 of `tmp/db-backup-20261031/chronicler_3150.db`, written 2026-10-31.

```text
=== SYSTEM ===
<role>
    You are a scene quantifier for a text adventure game.
</role>

<instructions>
    Your task is to determine which Characters are present in the current room and whether the player actually moved to a new location.

    How to determine movement:
    1. Read <CurrentRoom> — this is where the player is right now.
    2. Read <LatestNarration> — this is what just happened.
    3. Ask: does the narration describe the player being in a different place than <CurrentRoom>?
       - If YES → movement occurred. Set type to "entering" and destination to the new room.
       - If NO → no movement. Set type to null.
       - If unclear → assume no movement. Set type to null.

    Rules:
    - Only include Characters that would logically be in the room based on context.
    - Characters from the previous room may have followed the player.
    - Use the exact Character IDs provided in the <available_character_ids> list.
    - If the player is blocked, stopped, prevented, or fails to move in <LatestNarration>, they have NOT moved.
    - A Character interposing, blocking a path, or saying "you can't go" means the player remains.
    - If no Characters are present, return an empty array: {"characters_present": []}
    - If no movement detected, set type to null: {"movement": {"type": null}}

    Examples:
    - Narration: "She walked through the door into the kitchen." (CurrentRoom was hallway) → {"movement": {"type": "entering", "destination": "kitchen"}}
    - Narration: "The guard blocked her path. 'Halt!' he shouted." (CurrentRoom was courtyard) → {"movement": {"type": null}}
    - Narration: "She swiftly interposed herself between the player and the gate." (CurrentRoom was garden) → {"movement": {"type": null}}
    - Narration: "The foyer felt claustrophobic. Carla stood in the doorway." (CurrentRoom was Front Gates) → {"movement": {"type": null}}
    - Narration: "He examined the ancient vase carefully." (CurrentRoom was library) → {"movement": {"type": null}}
</instructions>

<output_format>
    Respond ONLY with a JSON object in this exact format:
    {"characters_present": ["id1", "id2"], "movement": {"type": "entering|in|leaving", "destination": "room_id"}}
</output_format>

<available_character_ids>
  <Character id="louise" name="Louise"/>
  <Character id="carla" name="Carla"/>
  <Character id="jezebel" name="Jezebel"/>
  <Character id="gabriella" name="Gabriella"/>
  <Character id="lisette" name="Lisette"/>
</available_character_ids>

<available_rooms>
  <Room id="front_gates" name="Front Gates"/>
  <Room id="courtyard" name="Main Courtyard"/>
  <Room id="gardens" name="Formal Gardens"/>
  <Room id="entrance_hall" name="Entrance Hall"/>
  <Room id="library" name="Library & Study"/>
  <Room id="master_quarters" name="Master Quarters"/>
  <Room id="kitchen" name="Mansion Kitchen"/>
  <Room id="dining_room" name="Dining Room"/>
  <Room id="staff_quarters" name="Staff Quarters"/>
  <Room id="financial_office" name="Financial Office"/>
  <Room id="guest_wing" name="Guest Wing"/>
</available_rooms>

=== USER ===
<CurrentRoom>
  <Name>Front Gates</Name>
  <Description>The imposing wrought iron gates of the Redmist Estate. Security is tight here.</Description>
  <Navigation>You can go north to enter the courtyard.</Navigation>
</CurrentRoom>

<PreviousCharactersPresent>
  <Character id="carla" name="Carla">Carla is Carla Jess, the 30-year-old bodyguard of {{user}}.
Appearance: Relatively attractive, athletic curves, strong body, muscular, scars, tattoos, blue eyes, short black hair. Carla wears professional suits and sunglasses to maintain an imposing presence.</Character>
</PreviousCharactersPresent>

<RecentHistory>
  <Entry sender="Narrator">Standing tall and commanding, Carla approaches Julian with a piercing gaze. She extends her arm for a firm handshake. "I'm Carla. I'll be your bodyguard both inside the Redmist Estate and wherever else you go. You'll notice me around frequently, so don't try to shake me off." She steps back, crossing her arms. "I need you to understand that your safety is my top priority. Everything I do is to keep you safe. Let's work together to make sure nothing compromises that. Firstly, I'll ask to borrow your phone. It won't take long."</Entry>
  <Entry sender="Julian">I shake Carla's hand.</Entry>
</RecentHistory>

<LatestNarration>
  Julian: Carla's grip is dry and brief, the calluses on her palm pressing against his knuckles before she lets go. She doesn't linger on pleasantries.

"Phone," she says again, holding out her hand with the same flat patience as before. Behind her, one of the gate guards has ambled closer to the security booth's window, watching the two of them with idle curiosity. The wrought iron gates remain shut, the gravel drive beyond them running up toward the mansion's pale facade. Somewhere past the treeline, a gull cries out over the harbor.
</LatestNarration>

Based on the context above, determine:
- Which Characters are present in the current room
- Whether the player actually entered, left, or remained in place

IMPORTANT: Base your decision ONLY on what happens in <LatestNarration>, not on what the player attempted in <RecentHistory>.

Respond ONLY with the JSON format specified in the system instructions.

```

## 6. Options

Runs at the end of a Job when the Game's `options_always_on` is on, or after `/options`. Connection: the connection whose id is `options`, else the narration connection (`bootstrap/wiring.rs:74-80`). Every prompt is built in `application/agents/options/prompt.rs`. No token fitting runs. The call retries once (`utils/orchestration.rs:15`, `:44-70`).

| Text or parameter | What reaches the model | Where it comes from | Author can change it today |
|---|---|---|---|
| system half: preset text | the Game's `options` preset text, XML-wrapped, with `{{user}}` and `{{option_count}}` replaced | the Game's `options` preset | yes through a preset copy |
| system half when the preset is blank or unreadable | a built-in fallback sentence | code (`options/prompt.rs:10-15`) | no |
| `{{option_count}}` | always `3` | `DEFAULT_OPTION_COUNT` (`domain/model/template.rs:12`; passed at `options/utils/orchestration.rs:34`) | no |
| user half: `<CurrentRoom>` | `Name`, `Description`, `Navigation` when set | the map's room, not template-rendered | yes: World edit form |
| user half: `<CharactersPresent>` | `id`, `name` and the card `description` of the Characters in the room, not template-rendered | the Swipe state, and the cards | data JSON only |
| user half: `<RecentHistory>` | up to 4 newest Messages, with `sender` labels | the story (`job.rs:337-340`) | yes, indirectly |
| user half: closing line | `respond with exactly <n> options … something <persona name> could do next` | code (`options/prompt.rs:86-93`) | no |
| `max_tokens` | 2048, the transport default | the Job passes `None` (`job.rs:472-481`) | no |
| other parameters | those of the options connection | settings | yes, as elsewhere |

**known** The stored example below comes from 2026-10-18 and uses `<NpcsInArea>` / `<Npc>` where the current builder writes `<CharactersPresent>` / `<Character>` (`options/prompt.rs:53-63`). The rest matches. No stored Options prompt with the current tag names exists in this checkout.

### Real prompt, Options

Source: `llm_messages` row 11 of `tmp/db-backup-20261031/chronicler_3113.db`.

```text
=== SYSTEM ===
<role>
    You are an options generator for a text adventure game. You produce pickable next-action suggestions the player can choose from.
</role>

<instructions>
    Stop the roleplay now and provide a response with 3 brief distinct single-sentence suggestions for the next story beat on Test Player perspective. Ensure each suggestion aligns with its corresponding description:
    1. Eases tension and improves the protagonist's situation
    2. Creates or increases tension and worsens the protagonist's situation
    3. Leads directly but believably to a wild twist or super weird event
    4. Slowly moves the story forward without ending the current scene
    5. Pushes the story forward, potentially ending the current scene if feasible

    Each suggestion surrounded by `<suggestion>` tags. E.g:
    <suggestion>suggestion_1</suggestion>
    <suggestion>suggestion_2</suggestion>
    ...

    Do not include any other content in your response.
</instructions>
=== USER ===
<CurrentRoom>
  <Name>Test Tavern</Name>
  <Description>A cozy tavern with wooden beams and a warm fire. Good place to start testing.</Description>
  <Navigation>You can go north to reach the village square, or exit through the tavern entrance.</Navigation>
</CurrentRoom>

<NpcsInArea>
  <Npc id="bartender" name="Bartender Bob">A friendly tavern keeper with a bushy beard and kind eyes.</Npc>
  <Npc id="shopkeeper" name="Shopkeeper Sarah">A shrewd but fair shopkeeper with spectacles and a calculating look.</Npc>
</NpcsInArea>

<RecentHistory>
  <Entry sender="Narrator">Welcome to the Test World, Test Player! This is a simple scenario for testing the starting scenarios feature. Feel free to explore and test the game engine.

The tavern around you is warm and inviting. Wooden beams stretch across the ceiling, and a crackling fire in the hearth casts dancing shadows on the walls. The smell of fresh bread and mulled cider fills the air.

Behind the bar, the bartender wipes down a mug and glances your way. "First time in the Test Realm?" he asks with a knowing smile. "Don't worry, everyone here is friendly. Mostly."

A merchant in the corner adjusts her pack and catches your eye. "If you're heading north to the village square, mind the cobblestones. They get slippery after dark," she advises.

You take a moment to gather your bearings. The road ahead promises adventure, but for now, the warmth of the tavern offers a brief respite.</Entry>
  <Entry sender="Test Player">look around</Entry>
  <Entry sender="Narrator">The warmth of the fire presses against your back as you scan the room more deliberately. The tavern is modest but well-kept — a dozen or so tables scattered across a scarred wooden floor, most of them empty at this hour. A stairwell hugs the far wall, leading up to what you assume are a few rooms for rent. A window beside the door shows a darkening sky, the village outside settling into evening.

Bob's polishing a glass behind the bar, his movements practiced and idle. Sarah, the merchant who spoke to you earlier, has turned her attention to an open ledger on her table, muttering numbers under her breath. A small charm hangs from her pack — a dried flower encased in resin, something a superstitious traveler might carry.

A notice board hangs near the entrance, papers pinned lopsidedly atop one another, their edges curling with age. One bears a faded illustration of a wolf's head.</Entry>
  <Entry sender="Narrator">The door swings open with a wooden groan, letting in a slice of cool evening air. A man steps through, leather armor creaking at the shoulders, a short bow slung across his back. He's lean, dirt-smudged, with the kind of tired squint that comes from a long day under the sun or a longer one under the moon.

He spots you by the fire, holds your gaze a moment too long to be accidental, and gives a short nod of recognition. Not familiarity — just acknowledgment. The kind you give someone you've judged not a threat.

Bob looks up from his polishing. "Evening, Rick. You're back late."

"Found tracks," the ranger says, shrugging off his bow and settling onto a stool. His voice is rough, unused. "East of the old mill. Something's been dragging deer carcasses into the brush. Not wolves."

Sarah's pen stops scratching. She looks up from her ledger, spectacles catching the firelight.</Entry>
</RecentHistory>

Based on the scene above, respond with exactly 3 options in the format the system instructions specify. Each option is a possible player action — something Test Player could do next — not a story outcome.

```

## Where the stored prompts come from

- `ENVIRONMENT.md:93` says the dev server keeps its DB at `<target-dir>/debug/chronicler_<port>.db`. **known** No such file exists in this checkout (`target/debug/` holds no `chronicler_*.db`). Every stored prompt above comes from a DB copy under `tmp/`: `tmp/db-backup-20261031/` and `tmp/stage6/debug/`.
- All reads were read-only URI opens. No row was written.
- **known** Two tag sets are in the stored rows. Prompts written up to 2026-10-22 use `<KnownNpcs>`, `<NpcsInRoom>` and `<PlayerCharacter>`; the rows from 2026-10-25 on use `<KnownCharacters>`, `<CharactersPresent>` and `<Persona>`, which is what the current code renders (`assembler.rs:207-288`). The ticket context and the older assets name the older tags.
- **known** A seed from `data/` writes only a row that is absent: settings (`storage/settings.rs:119-123`), worlds (`bootstrap/load.rs:113-120`), characters (`load.rs:152-159`), personas (`load.rs:179-188`), presets (`bootstrap/run.rs:158-170`). So a `data/` JSON edit reaches the model after a first seed only when it adds a new id or key, and the dashboard copy wins otherwise.

## Doc rows that the code contradicts

File: `docs/diataxis/reference/narrative/prompt_system.md`.

| Row | Doc claim | Code | What the code does |
|---|---|---|---|
| Overview, first sentence | "assembles … a structured prompt for every Game Master call from the active system-prompt preset" | `domain/model/generation.rs:43-49`, `pipeline/job.rs:234-240` | an Impersonate call renders the Game's impersonate preset, and its layers differ (no Persona, no Guide) |
| Prompt Injection Sanitization | "replaces any `{{…}}` pattern with `[FILTERED]`" | `prompting/builders/sections.rs:8-40`, `assembler.rs:334-341` | only the Player Input layer is filtered; an empty `{{}}` is left as it is; history, preset text, global rules and card data are never filtered |
| Context Templates | substitutes persona fields, posture and option count "into author-controlled preset fields at render time" | `assembler.rs:196-330`, `builders/sections.rs:75-140`, `prompt_preset.rs:131-183` | the same substitution also runs on the world description, the room description, the persona sheet, every character field, the relationship text and every global rule; the Quantifier and Options render only their preset text |
| Prompt Presets | "stores editable presets of four types" | `application/prompt_preset_service.rs:83-96` | a preset with `is_default: true` is refused on edit, and every seed file in `data/prompt_presets/` carries `is_default: true`; the author edits a copy |

Rows that the code does not contradict, but that leave a fact out:

- **Context Templates**: the option count is a constant of 3 (`template.rs:12`), so no author value reaches it.
- **Output Format** (Section Definitions): the response length is the global `AppSettings.response_length`, and it reaches the prompt only when the preset also has an `output_format` (`prompt_preset.rs:170-180`).
- **Layer 0** (the table): `<role>` and `<instructions>` are absent when the preset field is `None` (`prompt_preset.rs:144-153`).
- **Single-User-Message Mode**: the merged message begins with the literal line `[SYSTEM]` (`prompt_merge.rs:4`).

## Open questions

- No stored Event Narration and no stored Options prompt carries the current tag names. I cannot confirm from a stored sample that the current builders add nothing beyond those names.
- `ENVIRONMENT.md` names a dev DB that this checkout does not have. I do not know whether the real dev DB holds newer rows than the copies in `tmp/`.
- Whether the map counts a `data/` preset file as changeable when the DB already holds that id is a decision for the map, not a fact of the code.
