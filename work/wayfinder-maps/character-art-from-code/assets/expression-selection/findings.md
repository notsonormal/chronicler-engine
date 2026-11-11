# How other apps choose an Expression

Ticket: [02 Research: how other apps choose an Expression](../../issues/02-research-expression-selection.md)

Date of reading: 2026-11-02. Labels: **known** (read in source or official docs), **inferred** (a conclusion from known facts), **guessed** (no source).

Terms: "Expression" is the working name from the map. The apps call it an expression or a sprite.

## Short answer

Four ways to pick an Expression exist in the apps read.

1. A small local classifier reads the text (SillyTavern "Local", Marinara avatar "mood reader").
2. An extra LLM call reads the text (SillyTavern "Main API", Marinara "Expression Engine").
3. The narrator writes the Expression as a tag in its own output (Marinara Game Mode).
4. A word list reads the text (Marinara keyword tier).

Chronicler can share the Quantifier call only for method 2 or 3. The classifier and the word list make no model call.

## Method table

| # | Method | Who reads the text | Runs when | Extra model call | Extra cost | Extra wait | Expression list: who defines it |
|---|---|---|---|---|---|---|---|
| 1 | Local classifier (SillyTavern) | A DistilBERT model on the server | After the reply is saved. During streaming, at most once each 10 s | No | None. A one-time download of about 100 MB | Tens to hundreds of ms (**inferred** from the Marinara worker comment for a similar model; SillyTavern gives no figure) | Fixed: the 28 GoEmotions labels. The user can add custom names, but the classifier never returns them |
| 2 | Extra LLM call, "Main API" (SillyTavern) | The chat model | After the reply is saved. Waits for the stream to finish | Yes, one for each reply | Input of the last message plus a short prompt ("Limited Context"), or the whole prompt ("Full Context"). The PR below measured 4× to 20× fewer tokens with the last 3 messages | One full model round trip | The default 28 labels, or only the labels that the Character has a picture for ("filter available") |
| 3 | Extra LLM call, "Expression Engine" agent (Marinara Roleplay) | The chat model, or the agent's own connection | Post-Processing: after the reply is complete. Can run in a batch with other agents or alone | Yes, or a share of one batched call | Input: up to 6000 characters of reply, 2 recent messages (1200 characters each), the latest user message, and the list of sprites. Output: JSON with one entry for each owner | One model round trip. A batch adds no round trip but a longer output | The Character's own sprite names. The user uploads and names them. Quick-add suggests names (inferred: the 17 labels of `SPRITE_EXPRESSION_LABELS`) |
| 4 | Narrator-written tag (Marinara Game Mode) | The Game Master model, in its own reply | Inside the narration call | No | A few output tokens for each line | None | A default list of 16 labels plus `custom`. The Character's own sprite names replace it when the Character has any |
| 5 | Word list (Marinara keyword tier) | A local word list, several languages | Instantly, also while streaming | No | None | Under 1 ms (**inferred**) | The 17 labels `SPRITE_EXPRESSION_LABELS` |
| 6 | Local mood reader (Marinara 2D/3D avatars) | A multilingual model, `Horizon-Labs/multilingual-emotions-small`, in a worker thread | For each sentence, also while streaming. Averaged with the word list | No | About 300 MB download, about 1 GB memory in use | Tens to hundreds of ms for each text (worker comment) | 28 GoEmotions labels, mapped to six emotions for 3D avatars |

## SillyTavern: Character Expressions extension

Source: `public/scripts/extensions/expressions/index.js` and `src/endpoints/classify.js` in the SillyTavern repo (release branch, read 2026-11-02). Docs: <https://docs.sillytavern.app/extensions/expression-images/>.

### What reads the text

Known. The setting `extension_settings.expressions.api` has these values (`EXPRESSION_API`, `index.js` line 83): `local`, `extras` (a legacy server), `llm`, `webllm` and `none`.

- `local`: the browser posts the text to `/api/extra/classify`. The server runs a transformers.js `text-classification` pipeline and returns the top five labels. The page uses `classification[0].label` (`index.js` `getExpressionLabel`, `classify.js`). The default model is `Cohee/distilbert-base-uncased-go-emotions-onnx` (`default/config.yaml`, key `extensions.models.classification`). The docs name a 6-label model as the alternative.
- `llm` ("Main API"): the page calls `generateRaw` ("Limited Context": the last message and a system instruction) or `generateQuietPrompt` ("Full Context": the whole chat and the character card). The default prompt is: `Ignore previous instructions. Classify the emotion of the last message. Output just one word, e.g. "joy" or "anger". Choose only one of the following labels: {{labels}}` (`DEFAULT_LLM_PROMPT`). If the backend supports JSON schema, the call is constrained to an enum of labels with `top_k: 1` (`onTextGenSettingsReady`, `getJsonSchema`).
- `webllm`: the same prompt on an in-browser model through an extension.

### When it runs

Known. A timer calls `moduleWorker` every 2 s (`UPDATE_INTERVAL`). It classifies only when the last Character message text has changed. With the `llm` API it waits until streaming has finished. With the other APIs it classifies during streaming, at most once each 10 s (`STREAMING_UPDATE_INTERVAL`). One call runs at a time (`inApiCall`). The classifier sees only the last Character message. For text under 500 characters it trims to a full sentence. For longer text it keeps the first 250 and the last 250 characters (`sampleClassifyText`). **Inferred:** the middle of a long message never reaches the local classifier.

### The Expression list and who defines it

Known. The default list has 28 names, the GoEmotions labels (`DEFAULT_EXPRESSIONS`): admiration, amusement, anger, annoyance, approval, caring, confusion, curiosity, desire, disappointment, disapproval, disgust, embarrassment, excitement, fear, gratitude, grief, joy, love, nervousness, optimism, pride, realization, relief, remorse, sadness, surprise, neutral.

The user uploads one picture for each name into `characters/<name>/`. The file name is the label. Many pictures for one label are allowed. Then the extension picks one at random. A user can add custom label names, but the list of custom names is global, not for each Character (issue [#3918](https://github.com/SillyTavern/SillyTavern/issues/3918), as the issue describes it). A custom label can be chosen only by the `llm` API (when it is in the labels sent to the model) or by hand with `/expression-set`. **Inferred** from the code: the local classifier returns only its own labels.

### When the answer is not on the list

Known. Three layers.

1. The `llm` parser tries JSON, then a fuzzy match (Fuse.js) against the labels, then a substring match. If all fail, it throws and `getExpressionLabel` returns the fallback (`parseLlmResponse`).
2. A failed request, no connection after 3 s, empty text, or API `none` return the fallback expression (`getExpressionLabel`).
3. If the Character has no picture for the label, `chooseSpriteForExpression` shows the picture of the fallback expression. The default fallback is `joy` (`DEFAULT_FALLBACK_EXPRESSION`). The user can set "No fallback" (show nothing) or "Default emojis" (built-in emoji pictures).

### Cost and extra wait

- `local`: no money. About 100 MB download once. Results are cached by text in server memory (`cacheObject` in `classify.js`).
- `llm`: one model call for each Character reply. Cost depends on the prompt type. The "Full Context" mode sends the whole prompt. Pull request [#2273](https://github.com/SillyTavern/SillyTavern/pull/2273) (author's measurement, Czech chat, not reviewed by a maintainer in the text read): the last 3 messages cut tokens 4× in one test and 20× in another, with little loss of quality. That PR was closed with merge conflicts. **Known** from the PR page; whether a later change took the idea is not known.
- The wait: the classification starts after the reply ends and after the next 2 s timer tick. **Inferred.** The picture changes after the story text is already shown.

### Known failures

- The fallback overwrites an Expression that a script set, unless the script waits 2 s. A new option fixed it in release 1.19.0 (issue [#5547](https://github.com/SillyTavern/SillyTavern/issues/5547), closed). The reporter asks for a regex mode: his model ends each reply with `Emotion: <name>`. This is the "narrator-written tag" method.
- Reasoning text of a thinking model reaches the classifier (issue [#3364](https://github.com/SillyTavern/SillyTavern/issues/3364)).
- Custom labels are global and not for each Character (issue [#3918](https://github.com/SillyTavern/SillyTavern/issues/3918)).
- Users on a text-generation backend found that the `llm` mode sends the whole chat prompt (Reddit, [r/SillyTavernAI 1h5503l](https://www.reddit.com/r/SillyTavernAI/comments/1h5503l/sillytavern_textgenerationwebui_image_prompt_and/)). Known from the post title and snippet only; I did not read the full thread.
- The 6-label model gave users few Expressions (Reddit, [r/SillyTavernAI 133afua](https://www.reddit.com/r/SillyTavernAI/comments/133afua/how_do_i_allow_for_the_character_expressions_to/)).
- The local classifier reads only the last message in the sampled form. It "ignores your character card and the wider scene" (third-party page <https://tavernsprite.com/tools/sillytavern-expression-classifier/>; a secondary source). **Inferred** from the code: it also reads action text with the `*` and `"` marks removed (`sampleClassifyText`).
- **Guessed, not sourced:** a classifier trained on Reddit comments (GoEmotions) reads narrated prose as `neutral`. Marinara's own code comment states this for a similar model (see below), so the claim holds for that model; it is not tested on the SillyTavern model.

### Add-on: Expressions+

Source: <https://github.com/Tyranomaster/expressions-plus/> (README). Known. It adds rules on top of the same local classifier: a range rule (a label when the score of one emotion falls in a band) and a combination rule (a label when several emotions score close). Its default profile adds 22 labels such as `anxious`, `awe` and `contempt`. It also splits a long message into segments. It states the internal limit is 500 tokens, not characters. **Inferred:** the community needed more labels than the 28, and made them from the score vector, not from a second model.

## Marinara Engine

Source: repo <https://github.com/Pasta-Devs/Marinara-Engine> (cloned 2026-11-02, release v2.5.1). Docs: `docs/characters/sprites.md`, `docs/agents/built-in-agents.md`, `docs/agents/avatars.md`. Code: `packages/server/src/services/agents/agent-executor.ts`, `packages/server/src/routes/generate/expression-agent-utils.ts`, `packages/shared/src/utils/expression-keywords.ts`, `packages/shared/src/types/avatars.ts`, `packages/server/src/services/avatars/emotion-classifier.ts`, `packages/server/src/services/game/gm-prompts.ts`.

### Roleplay mode: the Expression Engine agent

Known. It is an optional agent in the phase "Post-Processing": "runs after the reply is complete" (`docs/agents/built-in-agents.md`, `docs/agents/agents-overview.md`). It is one LLM call. `buildExpressionAgentMessages` (`agent-executor.ts` line 2654) builds a compact request:

- a role text: "You are a specialized expression-selection agent. Keep the request compact and return only JSON.";
- `<available_sprites>`: for each owner, `Name (id): label, label, ...`, built from the sprites the owner has;
- `<recent_context>`: the last 2 messages, 1200 characters each (`EXPRESSION_AGENT_RECENT_CONTEXT_MESSAGES`, `EXPRESSION_AGENT_CONTEXT_CHAR_LIMIT`);
- `<latest_user_message>` for the Persona;
- `<assistant_response>`: the reply, up to 6000 characters;
- an instruction: exactly one expression for each owner.

The model returns JSON with `{characterId, characterName, expression, transition}` for each owner. The Persona is an owner too, and its expression comes from the latest user message.

Batching: known. The agent can share one call with other tracker agents. It runs alone if the user turns off "batch with other agents" (`shouldRunAgentIndividually`, line 2063). **Inferred:** by default, the Expression Engine can ride in the same call as the World State and other trackers.

### Expression list and who defines it

Known. The list is for each owner. It is the names of the sprites that the owner has, from two sources: Expressions, Full-body, or both (`buildAvailableSpriteCharacter`). The "Quick add" row suggests names. **Inferred:** they come from `SPRITE_EXPRESSION_LABELS` (17): neutral, happy, sad, angry, surprised, embarrassed, thinking, laughing, worried, scared, disgusted, love, smirk, crying, determined, hurt, blushing. The user can name any sprite. Names with a shared prefix before an underscore are variants (`happy_01`).

### When the answer is not on the list

Known (`expression-agent-utils.ts`). Four layers of repair, no failure shown to the user:

1. `resolveCharacter`: an unknown Character id is matched by id, name, name without title, then by containment.
2. `resolveExpression`: exact, normalised, then containment match on the owner's own names. A prefix group picks a random variant. No match: the entry is removed with a warning.
3. `completeRequiredSpriteExpressionEntries`: if the model omitted a required owner, the code fills one in. It first tries the keyword list on the source text. If that gives nothing, it picks a stable hash of the text among the broad (not "smirk", "sly", "wink", "flirt") expressions. This is a pseudo-random but repeatable choice.
4. If the call returns no entries, it logs "Expression agent returned no expression entries — filling required targets" and fills all targets.

### Cost and extra wait

Known: one LLM call for each reply (or a share of a batch), after the reply is complete. Input about up to 6000 characters of reply plus 2400 of context plus the sprite list. **Guessed in tokens:** roughly 2000–3000 input tokens in the worst case, and a short JSON output. The wait is one model round trip after the reply. The call can use its own, cheaper connection (**inferred**: agents have their own connection setting; not read in code).

### Roleplay: the sprite while streaming

Known. While a reply streams, the client uses `detectExpressionKeyword` from the word list (`SpriteOverlay.tsx` line 119, `expression-keywords.ts`). The word list is the "Roleplay sprite fallback when no Expression Engine result is there". The header of `expression-keywords.ts` says the same list also serves the agent's fallback and the avatar classifier. The word groups are in priority order. The first group that matches wins. General emotions come before specific ones.

### Avatars (2D/3D): the mood reader

Known (`emotion-classifier.ts`, `docs/agents/avatars.md`). Two tiers: the word list (instant, always there) and a local model, `Horizon-Labs/multilingual-emotions-small` (about 300 MB, about 1 GB memory, unloads after 10 idle minutes, not on lite installs, Android or machines under 3 GB). The code header says: "Never an LLM, never LLM-written tags, never a Decision model: those run after generation and cannot follow the stream." It averages the model with a keyword hit, because "the model reads dialogue well but calls most narrated prose neutral, and explicit words ("tears", "laughs") are the strongest evidence there is." The user doc says each sentence gets its own mood, so the face changes while the reply is read.

### Game Mode: the narrator writes the tag

Known (`gm-prompts.ts` line 1753; `docs/agents/avatars.md` "Faces in Game mode"). The Game Master prompt asks for lines in the form `[Name] [main|side|whisper|thought] [neutral|happy|sad|angry|surprised|scared|disgusted|thinking|laughing|crying|blushing|smirk|embarrassed|determined|confused|sleepy|custom]: "Dialogue"`. It adds: "use these EXACT names when the character has any listed expression; only fall back to the defaults if the character has no listed sprites". The parser reads the last bracket (`game-narration-text.ts` line 286 onward). No extra call. Each dialogue line has its own Expression, so one reply can show several faces.

### Known failures in Marinara

I did not find user complaints in the sources I could open (no Marinara issue search finished). What the code itself records as failures:

- The model omits an owner, uses an unknown name, or returns no entries. The repair layers above exist for this.
- The model returns a Character name instead of an id.
- A narrated reply reads as neutral for the local model (code comment).
- The persona who left the scene was shown again (issue #7378, named in a code comment in `expression-agent-utils.ts`; I did not open the issue).

## What this means for Chronicler

### The Quantifier today

Known. The Quantifier runs after each generation and reads the new text (`LatestMessage`), the room, the Characters present before, the recent history and the list of Character ids and names (`src/application/agents/quantifier/prompt.rs`). It returns `QuantifierResult { present, destination }` (`src/domain/model/quantifier.rs`). The Game rules apply the result with no model call (`CONTEXT.md`). It has its own Connection slot (`CONTEXT.md`: Slot). The test fixtures use `max_tokens: 300` and `temperature: 0.2` (`agent_tests.rs` line 499) (a test value, not the production default).

### Which outside methods can share the Quantifier call

| Method | Can share the Quantifier call? | Why |
|---|---|---|
| 2 / 3. LLM reads the text and picks from a list | **Yes** (**inferred**) | The Quantifier already has the new text, the Characters and their ids. The prompt adds each present Character's list of Expression names. The JSON gets one more field for each Character present. Cost: a longer prompt (the list) and a few more output tokens for each Character. No extra round trip |
| 4. Narrator-written tag | Not the Quantifier. It shares the **Narration** call | The Narrator writes the tag in the story text. That needs a tag format in the prompt and a strip step before the text is stored or shown. Zero extra calls. But the tag can leak into the story text, and the Narrator's output rule changes (**inferred**) |
| 1 / 6. Local classifier | No | No model call. Needs a model download and memory. Reads the whole message as one text, so it cannot say which Character feels what in a scene with more than one Character (**inferred**) |
| 5. Word list | No | No model call. Free and instant, but covers only words that name a feeling. Works as the fallback when the Quantifier fails (Marinara uses it so) |

### Cost and wait for Chronicler

A turn costs money and takes several seconds (`PRODUCT.md`). **Inferred** ranking, from the code read:

| Method | Extra model calls | Extra wait | Extra tokens | Local resources |
|---|---|---|---|---|
| Field in the Quantifier JSON | 0 | Slightly longer output | Prompt: the lists. Output: a few tokens for each Character | None |
| Narrator tag | 0 | None | A few output tokens for each line | None |
| A new, separate LLM call | 1 | One round trip | Up to the whole reply | None |
| Local classifier | 0 | Tens to hundreds of ms | None | 100–300 MB disk, up to 1 GB memory |
| Word list | 0 | Under 1 ms | None | None |

### Failures the Chronicler design must meet

Each of these happens in at least one app read.

1. **The answer is not on the list.** Every app has a repair chain: exact match, normalised match, containment or fuzzy match, then a fallback Expression.
2. **The model omits a Character.** Marinara fills it in. For Chronicler: keep the last Expression of that Character, **or** the default one. (Which one is a decision for the grilling ticket.)
3. **The Character has no picture for that Expression.** SillyTavern shows the fallback Expression. Marinara drops the entry.
4. **A list of labels that does not fit the Character.** SillyTavern users complain about global custom labels. Marinara avoids it with a list for each Character.
5. **Text that is mostly narration reads as neutral** (local models, Marinara code comment).
6. **Thinking text reaches the reader** (SillyTavern #3364). Chronicler must give the reader the saved story text only. **Inferred:** the Quantifier already gets saved text (`HookInput.text`).
7. **The picture changes late.** In all methods that wait for the full reply, the picture changes after the text shows. The streaming fix is a word list or a small model, which Chronicler may not need if its story text appears whole. **Not known:** whether Chronicler streams the reply to the screen. I did not check.
8. **Many Characters in one reply.** A message-level classifier gives one label for the message. The Quantifier and the Game Master tag give one label for each Character.
9. **A Swipe or a retry.** SillyTavern skips a placeholder text `...` and uses the fallback. Chronicler stores the Quantifier result for each Swipe state, so this is likely a non-issue (**inferred**).

## Surfaced

- **Input for the grilling ticket "what chooses the Expression".** The three live options are: a field in the Quantifier JSON (cheapest, Characters are named), a tag written by the Narrator (no extra call, text leak risk), and a word list as the fallback. The local classifier does not fit: it has no Character, and it adds a heavy download.
- **Fog: the Expression list.** A list for each Character (Marinara) avoids the "custom label" problem of SillyTavern. A fixed list is easier to prompt. Two apps use 17 to 28 labels. A Character with fewer Expressions needs a defined fallback.
- **Fog: the Persona's Expression.** Marinara gives the Persona an Expression from the latest user message, not from the reply. Chronicler's Quantifier reads the new text, which may or may not carry the Persona's feeling. Needs a decision.
- **Fog: unknown Marinara complaints.** I did not find user reports on Marinara Expression failures. A later reading of its GitHub issues could add some.
- **Fog: the real cost of the longer Quantifier prompt.** Needs a count on a real Game: tokens of the Expression lists for the Characters present, and the change in Quantifier latency. A probe could measure this without sending an Action only if it reads stored LLM messages.
