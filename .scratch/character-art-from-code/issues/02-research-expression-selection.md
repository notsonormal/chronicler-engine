# Research: how other apps choose an Expression

Type: research (AFK)
Status: resolved
Blocked by: none

## Question

How do SillyTavern, Marinara Engine and similar apps choose the Expression of a character picture from the story, and what fails?

## Context

- Candidates to read: the SillyTavern Character Expressions extension (its classifier, its LLM mode, its default Expression list), Marinara Engine, and other roleplay front ends that show character pictures.
- For each app, record: what reads the text (a local classifier, an LLM call, tags that the narrator writes), when it runs, the list of Expressions and who defines it, what happens when the answer is not on the list, and what users complain about.
- Chronicler already runs the Quantifier after each generation. It records the Characters present and the destination of the Persona (`QuantifierResult`, `src/domain/model/quantifier.rs`). Note which outside methods could share that call.
- A turn costs money and takes several seconds (`PRODUCT.md`). Note the cost and the extra wait of each method.
- Use `/research`. Prefer primary sources: source code and the official docs.

Done when a summary is linked as an asset of this ticket, and the answer lists the methods with their cost, their Expression lists and their known failures.

## Answer

Full summary with sources and a method table: [findings](../assets/expression-selection/findings.md).

| Method | App | Extra model call | Cost and wait | Expression list | Known failures |
|---|---|---|---|---|---|
| Local classifier | SillyTavern "Local"; Marinara avatars | No | 100–300 MB download, tens to hundreds of ms | Fixed: 28 GoEmotions labels | Reads one message as one text. Reads narration as `neutral`. Custom labels are global |
| LLM call on the reply | SillyTavern "Main API"; Marinara Expression Engine | Yes, one for each reply (Marinara can batch it) | One model round trip. Up to the whole prompt in "Full Context" | 28 defaults, or the names of the Character's own pictures | Name not on the list. Character omitted. Wrong Character name. Thinking text read |
| Narrator-written tag | Marinara Game Mode | No | A few output tokens for each line, no wait | 16 defaults plus `custom`, or the Character's own names | Tag leaks or is wrong. Needs a strip step |
| Word list | Marinara keyword tier | No | Under 1 ms | 17 labels | Covers only words that name a feeling |

- Every app repairs a wrong answer in layers: exact match, normalised match, fuzzy or containment match, then a fallback Expression (SillyTavern default `joy`; Marinara a stable pick from broad Expressions).
- Only the "LLM call on the reply" method and the narrator tag fit a turn that costs money. The first can share the Quantifier call (one more field for each Character present, no extra round trip). The second shares the Narration call.
- The local classifier does not name a Character and adds a heavy download. It does not fit Chronicler.

### Surfaced

- **Input for the grilling ticket "what chooses the Expression":** Quantifier field, Narrator tag, or word-list fallback. See findings, "What this means for Chronicler".
- **Fog, the Expression list:** a list for each Character (Marinara) avoids the global custom-label problem of SillyTavern. A fallback rule is needed for a Character with few Expressions.
- **Fog, the Persona's Expression:** Marinara reads it from the latest user message. What Chronicler reads is not decided.
- **Fog, the real cost of the longer Quantifier prompt:** not measured. Needs a token count on a real Game.
- **Fog, Marinara user complaints:** none found. A later reading of its GitHub issues could add some.
- **Not known:** whether Chronicler streams the reply to the screen. This decides whether a late picture change matters.
