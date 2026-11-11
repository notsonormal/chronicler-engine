# A Narration wraps its paragraphs in XML speaker tags

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A Narration came back with its paragraphs wrapped in tags named after the Characters (`<Jezebel>…</Jezebel>`, `<Carla>…</Carla>`). It happened once. What must happen when it happens again: nothing, a count in the trial pack, an engine fix, or a preset rule?

## Context

Source: layout-trials ticket 15, [Try the history in the ME format](../../layout-trials/issues/15-history-in-me-format.md), judge session `03640625`. This ticket is a product fault, so it does not fit this map's "Out of scope" line on product defects. The owner filed it here on 2026-11-02 so that a second case has one place to go.

- **known** Story entry #552 of `tmp/lt15/new/trial-1` (Narration call 1088, `finish_reason` `stop`, 30 messages, history holds 28 entries) begins `<Jezebel>Jezebel closed the drawer…</Jezebel>` and then `<Carla>At the study door…</Carla>`. The rest of the entry is plain prose. The stored entry and the story log keep the tags.
- **known** No other entry of any trial pack has such a tag: `tmp/lt01`, `tmp/lt03`, `tmp/lt10`, `tmp/lt11`, `tmp/lt15/reference`, `tmp/lt15/probe` and the other 200 entries of `tmp/lt15/new`. The check was the regex `</?[A-Z][a-z]+>` on `story.json`.
- **known** Ticket 15 changed the history. Each entry is now one chat message, wrapped in `<ConversationHistory>` and `<LastMessage>`, and every other prompt block is an XML tag too (`<GameState>`, `<CharactersPresent>`, `<KnownCharacters>`). Before ticket 15 the model copied the `Narrator:` / `Julian:` labels of the history text.
- **inferred** The model copies the XML style of the prompt into its answer, as it copied the labels before. One case does not show a rate.
- **known** `sanitize_llm_output` (`src/application/prompting/sanitize.rs`) strips thought blocks, channel markers and turn markers. It does not strip other tags.
- **known** `scripts/trial.py` does not count tagged entries. A judge found this case only because it read the whole story.

## Options to decide

- **A.** No change now. A later trial or judge that sees it again adds the case here.
- **B.** `scripts/trial.py` counts the entries that hold a tag such as `</?[A-Z][a-z]+>` and prints the count in `pack.md`, so every trial shows the rate with no judge.
- **C.** `sanitize_llm_output` removes a tag pair whose name is a Character name, and keeps the text inside it.
- **D.** A sentence in the default narrator preset (`output_format`) asks for plain prose with no tags.
