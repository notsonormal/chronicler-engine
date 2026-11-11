# C4: Genre features that Chronicler lacks

Status labels: **known** = read in a primary source or in the repo. **inferred** = follows from known facts. **guessed** = no source.

## Short answer

Chronicler lacks about 15 features that players and authors of this genre use. The owner's records name or scope about half of them (memory, state, prompt tuning, Portraits, dashboard layout, keyboard path, tab navigation). The records do not name the rest. The biggest unnamed gaps are: streaming, Character and World authoring in the dashboard, import and export of a Game, branching, multi-Character scenes, LLM provider breadth, and backup. Details follow.

Limit of this report: the notes from my first reading pass were compacted. I re-checked the repo claims below with `rg`. I re-fetched the Kobold, RisuAI, AI Dungeon and SillyTavern pages. The Marinara claims come from the local clone of its `docs/`. I did not fetch NovelAI (search rate limit). I did not open a SillyTavern page on group chats or branches (see "inferred" below).

## Sources used (all primary)

| Product | Source | Used for |
|---|---|---|
| SillyTavern | https://docs.sillytavern.app/extensions/ (fetched) | built-in and installable extension list; multi-user link |
| SillyTavern | https://docs.sillytavern.app/usage/worldinfo.md (search snippet) | World Info / Lorebooks |
| SillyTavern | https://github.com/SillyTavern/SillyTavern-Docs `Usage/Chatting` (search snippet) | swipe, edit on any message |
| Marinara Engine | local clone `docs/` (peek-prompt, group-chats, where-data-is-stored, localization, sprites, comfyui, tts-setup, combat, memory, avatars) | feature list |
| RisuAI | https://github.com/kwaroran/RisuAI README, wiki "RisuAI Basics", https://risuai.net/ (search snippets) | feature list |
| KoboldAI Lite / KoboldCpp | https://koboldcpp.net/docs/features/lite/ , https://github.com/LostRuins/koboldcpp/wiki , https://github.com/lostruins/lite.koboldai.net/ | modes, Undo/Redo/Branch, TTS, save formats |
| AI Dungeon | https://help.aidungeon.com/faq/how-to-play , `/faq/the-basics`, `/new-player-guide` | Do/Say/Story/See, Story Cards, Plot Essentials, Author's Note |

## Gap list

"Named?" says whether the owner's records already name the gap. I checked `PRODUCT.md`, `CONTEXT.md`, `.scratch/*/map.md`, `.scratch/*/idea.md`, the `work/wayfinder-maps/prompt-flexibility/assets/`, `.scratch/story-scroll-back/`, `work/_closed/platform-choice/` and `docs/specs/`.

| # | Gap | Who has it (source) | Named? | Evidence of the owner's state |
|---|---|---|---|---|
| 1 | **Streaming of a narration** (text appears token by token) | Kobold Lite streams by default (SSE or polled) (koboldcpp FAQ). ST streams (named in the same FAQ as an SSE client). | **No** (as a gap) | The only hit is `docs/specs/actions.md:417`, invariant I.3, which says "Streaming ... produce no duplicates". `docs/diataxis/explanation/dashboard_design.md` says the page polls and holds no stream. **Inferred:** this is a design choice for the dashboard, but no record weighs it against a turn that "takes several seconds" (`PRODUCT.md`, Operating Context). |
| 2 | **Character and World authoring in the dashboard** (create, edit, import a Character card or a World) | ST, Marinara (characters/creating-and-editing-characters.md), RisuAI (character list and settings sidebar), Kobold Lite (Scenarios, card import) | **Partly** | The owner knows that dashboard Worlds get no Characters (task brief). The owner's format research (`card-and-world-format.md`) covers card formats. I found no ticket for a Character editor or a card import button. **Inferred** gap. |
| 3 | **Import and export of a Game, a story or a card** (share a story, move to another machine) | Kobold Lite: story JSON, KAISTORY, ST cards and chats (koboldcpp wiki). AI Dungeon: shared Scenarios. RisuAI: RisuRealm sharing. ST and Marinara: card PNG/JSON import. | **No** for a Game or story. The card formats are researched. | `PRODUCT.md` names a public release. No record names a story export or a "download my Game" control. |
| 4 | **Branching a story** (copy the Game at an Entry and play on in parallel) | Kobold Lite has a Branch button (koboldcpp wiki). Marinara has Chat Branches (docs/chats/group-chats.md defines the term). ST has branches and checkpoints (**inferred**, no ST page fetched). | **No** | `work/issues/pending/19-story-scroll-back.md` names scroll-back, not branching. Retry, Edit and Delete act only on the last entry (`work/wayfinder-maps/long-term-memory-and-state/map.md:54`). A mistake 10 Messages back cannot be repaired. |
| 5 | **Edit, rewind or retry an older Message** | ST: "Click the Edit button on any message" (Chatting doc). Kobold Lite: Back, Redo, "edit any part of the text". | **Yes, as a constraint** | Known and recorded as a design fact. Not recorded as a wish. Kobold's Redo (undo of a Delete) has no Chronicler equivalent (`rg 'undo|redo'` finds only Swipe-redo wording). |
| 6 | **Several Characters that speak in one scene** (group chat) | ST (group extensions: Group Greetings, Group SendAs), Marinara (group-chats.md, no fixed limit), RisuAI (README: Group Chats) | **Partly** | Chronicler uses a Narrator, so this may not apply. `CONTEXT.md` has Narrator Mode. Group-chat features appear only in the owner's ST/ME research assets. No ticket decides "Characters speak, not only narrate". |
| 7 | **Lorebook keyed on words, with rules** (activation by keyword, depth, insertion order, regex) | ST World Info (worldinfo.md), Kobold World Info, AI Dungeon Story Cards, RisuAI lorebook | **Yes** | `card-and-world-format.md` and `long-term-memory.md` cover `character_book`. `WorldLore` exists. Not a new lead. |
| 8 | **Memory, Author's Note, Plot Essentials** (a player-written note that sits in the prompt) | Kobold Lite: Memory, Author's Note, World Info (FAQ). AI Dungeon: AI Instructions, Plot Essentials, Story Cards, Author's Note (help FAQ). ST: Author's Note, Objective extension. | **Yes** | `work/wayfinder-maps/long-term-memory-and-state/`. Open question: whether the player edits memory by hand. |
| 9 | **Automatic summary or vector memory of old chat** | ST: Chat Vectorization (built-in). Kobold Lite: TextDB. Marinara: memory agent (docs/agents/memory.md) | **Yes** | `long-term-memory-and-state/map.md` and `marinara-memory.md`. |
| 10 | **Text-to-speech and voice input** | Kobold Lite (TTS, Whisper, "Narration streaming"). Marinara (media/tts-setup.md). ST (TTS, RVC extension). | **No** | `rg` finds TTS only inside card JSON. |
| 11 | **Scene images and image generation** | ST Image Generation (built-in), Marinara ComfyUI (media/comfyui.md), Kobold Lite (Stable Diffusion inline), AI Dungeon "See" mode | **Partly** | `work/wayfinder-maps/character-art-from-code/` covers Portrait art from code. Nothing covers an image of the scene made by a model. |
| 12 | **Expressions and sprites that follow the story** | ST Expression Images, Marinara sprites, RisuAI Emotion Images | **Yes** | `work/wayfinder-maps/character-art-from-code/map.md` (goal 1). |
| 13 | **Prompt inspector** (see what the model got) | ST Prompt Inspector (extension), Marinara Peek Prompt (docs/chats/peek-prompt.md) | **No gap** | Chronicler has the LLM Messages tab and the LLM record dialog. Chronicler is ahead on this. **Inferred:** the Marinara "peek before send" (live preview of the next prompt) has no Chronicler counterpart. Minor. |
| 14 | **Provider breadth** (OpenAI, Claude, Gemini, local servers; OpenAI-compatible) | RisuAI (README: OpenAI, Claude, Gemini, DeepInfra, Ooba, OpenRouter), Kobold, ST, Marinara | **Partly** | `LlmBackendType` has `OpenRouter`, `Ollama`, `OpenCodeGo` and `Mock` (`src/domain/model/llm_backend.rs:12`). The Connection form offers three (`SELECTABLE`). `chat_completions.rs` covers "every OpenAI-compatible host", so a custom base URL may cover the rest. **Inferred:** a player with a plain OpenAI, Anthropic or LM Studio endpoint needs a base-URL field. I did not verify that the form has one. |
| 15 | **Accounts, multi-user, and server-side saves for several people** | ST has multi-user mode (link on the extensions page). Kobold has multiplayer and `--password` (koboldcpp wiki). | **Yes** | `PRODUCT.md:40`: "Whether a release needs accounts or stays a local install." Open decision. |
| 16 | **Two Games open at once** | ST and Marinara keep many chats open | **Yes** | `work/issues/pending/01-per-tab-games.md`. |
| 17 | **Backup and data location** | Marinara docs/data/where-data-is-stored.md (a `data` folder, `DATA_DIR`, what to copy). RisuAI cloud sync (DeepWiki, not primary). Kobold autosave and server save slots. | **No** | `ENVIRONMENT.md:95` says where the dev DB is. There is no player-facing note and no "export my data" control. SQLite beside the binary means a player moves one file, but only a record would say so. |
| 18 | **Mobile layout** | RisuAI, Marinara (chat-settings.md: full-width panel on a phone), Kobold Lite ("Mobile friendly") | **Yes** | `PRODUCT.md`: "A phone layout is wanted, but not built yet." `dashboard-design/map.md` names the overflow at phone width. |
| 19 | **Accessibility** | Marinara: keyboard order for chats, localization of accessible names (docs/development/localization.md) | **Yes** | `work/wayfinder-maps/dashboard-design/issues/10-keyboard-and-screen-reader-path.md`. Chronicler has the skip link, live regions and `aria-label` on icon buttons. |
| 20 | **Interface language** | Marinara has a localization guide (docs/development/localization.md). | **No** | `rg -i 'localiz|i18n'` over the records finds nothing. Low priority for a first release. **Guessed.** |
| 21 | **Action modes of the input** (Do, Say, Story, See) | AI Dungeon (help: How do I play) | **Partly** | Chronicler has `/guide`, `/impersonate`, `/options` and Narrator Mode. No record compares them with Do/Say/Story. Likely a design choice, not a gap. |
| 22 | **Continue** (extend the last text without a new Action) | Kobold Lite "Generate More"; ST Continue | **Unclear** | The word "Continue" appears in `prompt-flexibility` tickets for a model call that follows a Narration with no `<PlayerInput>` (issue 19). I did not find a player control that names it. **Inferred:** a Continue after a Narration exists in the prompt. I did not check whether the dashboard offers it. |
| 23 | **Regex or text replacement on output** | ST regex extension, RisuAI regex, Marinara (`use_regex` in lorebook entries) | **Partly** | Named only as a card-format field in `card-and-world-format.md`. `sanitize_llm_output` has one fixed `<br>` rule (`work/issues/pending/21-narration-line-break-tags.md`). No player-defined rules. |
| 24 | **Extension or plugin system** | ST (install from a Git URL), Marinara (optional agent packages), RisuAI (plugins in README) | **No** | Not named. Probably out of scope for a game with a data-driven World. **Guessed.** |
| 25 | **Tool use: web search, MCP** | Kobold Lite Tools tab (MCP, web search), ST Web Search extension | **No** | `rg` finds MCP and web search only in a platform-choice research asset. Unlikely wanted for a closed World. **Guessed.** |
| 26 | **Dice or a chance rule for an Action** | Kobold adventure roll modifier (lite.koboldai.net), ST D&D Dice, Marinara combat (docs/game/combat.md) | **Unclear** | Chronicler has Triggers and a Quantifier. I found no record about luck or a skill check. **Guessed** as a gap. |

## The gaps the owner's records do not name

Ranked by my judgement of their weight for a public release. The ranking is **inferred**.

1. **Streaming (1).** Every compared product shows text as it arrives. A turn of "several seconds" with a blank wait is the biggest feel gap. The owner chose polling for the dashboard. No record weighs the choice for the narration itself.
2. **Character and World authoring in the dashboard (2).** A released player cannot make a Character. This blocks the "game" from being made by anyone but the owner.
3. **Import and export (3) and backup (17).** A player cannot take a story out, share it, or move machines with a documented step.
4. **Branching and repair of an older Message (4, 5).** The player can fix only the last entry. All other compared products let the player go back or fork.
5. **Provider breadth (14).** Check the Connection form for a base-URL field before anything else.
6. **TTS (10), scene images (11).** Common in the genre. Probably later.
7. **Interface language (20), regex rules (23), plugins (24), tools (25), dice (26).** Low weight.

## Where Chronicler is ahead (so the owner does not read this as a pure deficit list)

- **Engine-kept game state** (Room, Characters present, Triggers) is the stated difference in `PRODUCT.md`. None of the compared products keeps authoritative state outside the model, from what I read. **Inferred**, because Marinara has a World State agent that writes state with a model.
- **Autoplay** with a budget. I found no equivalent in the compared products' docs (I found Idle prompting in ST, which differs: it prompts after the user is idle). **Inferred.**
- **LLM Messages tab**: the stored record of every model call.
- **Options** (suggested actions) match Marinara's "options" wording and a HypeBot-like idea in ST. Not a gap.

## What I could not check

- NovelAI (rate limit hit).
- SillyTavern group chats and branches pages. Rows 4 and 6 for ST come from extension names (Group Greetings, Group SendAs, Timelines) on the extensions page. That is **inferred**.
- Whether the Connection form lets the player enter a base URL (row 14).
- Whether the dashboard has a Continue control (row 22).
- The Character.AI side of "one hosted product". AI Dungeon is the hosted product here.
