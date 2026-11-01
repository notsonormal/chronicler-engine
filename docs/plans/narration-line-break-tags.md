# Plan: Remove HTML line-break tags from model answers

**Date:** 2026-11-01
**Status:** Open. Waits for the merge of `prompt-flexibility` into `main` (see "Branch").
**Goal:** A line-break tag that a model writes (`<br>`, `<br/>`, `<br />`, `</br>`, in any case) does not reach the story text, the story log or a later prompt.

## Origin

The Impeccable critique of the Game tab found a literal `</br>` in the story log (`.scratch/dashboard-design/assets/game-tab-critique/report.md`). The owner put it outside the dashboard design map and asked for this plan (2026-11-01).

## Current state (known, 2026-11-01)

- **The source.** One Narration holds the tag: Swipe 300 of Message 300 in the probe DB (`target/ui-investigator/debug/chronicler_3001.db`). The Narrator model `deepseek-v4.1-flash` on OpenCode Go ended its answer with a line that holds only `</br>`. LLM message 588 has the tag in `raw_response_json`, so the model wrote it.
- **How often.** The trial packs `tmp/pf08`, `tmp/pf18`, `tmp/pf19` and `tmp/pf20` hold about 505 story entries. One entry has a tag, and it is the same Narration, because the probe Game is the `pf19` trial. No other HTML tag occurs.
- **The save path.** `LlmCallRecorder` (`src/application/llm_recorder.rs`) runs `sanitize_llm_output` (`src/application/prompting/sanitize.rs`) on every model answer of every call kind. It saves the result as `parsed_response`, and it gives the result to the caller. The function removes thought blocks, channel markers and turn markers. Then it trims the leading whitespace of each line. It does not know HTML tags.
- **The render path.** `markdown_to_html` (`src/adapters/driving/http/utils/view_models.rs`) turns every raw HTML event into text (`sanitize_markdown_event`). So the page shows `</br>` as text. This escape is a security rule, and it stays.
- **The history.** The stored text goes into later prompts. 27 later calls carried the `</br>`: 22 Narrator calls and 5 Quantifier calls. The model did not write the tag again. This is known for this one Game only.
- **The tests.** The Narrator seam contract (`src/application/narration/narrator_tests.rs`) pins the cleanup with eight cases, `narrator-cleanup-*`. Each case calls the helper `narrated_text`, which builds the Narrator with `ScriptedModel` as production does. `sanitize.rs` and the eight cases are the same on `main` and on `prompt-flexibility`.
- **The docs.** No doc describes the cleanup. The code and the seam cases are its only record, so this plan changes no doc.

## Decisions (agent, open to change)

- **Fix at save time, in `sanitize_llm_output`.** The function is the one place for model artifacts (the comment above it in `chat_completions.rs`: "`sanitize_llm_output` removes any marker that still leaks"). A fix there keeps the tag out of the stored text and out of later prompts. A fix at render time keeps the tag in the history, and it weakens the HTML escape for one tag.
- **Replace a tag with a space, not with a line break.** The recorder runs the function on every call kind. The Quantifier answer is JSON, and a raw line break inside a JSON string makes the JSON invalid. A space is safe in JSON and in prose. A tag alone on a line leaves an empty line, and Markdown ignores it.
- **Line-break tags only.** No other tag occurs in the evidence. A rule for every tag can remove text that a story needs, such as `<3` or an invented `<whisper>`.
- **Stored rows do not change.** See "Not in scope".

## Steps

1. **Test first.** Add two cases to the Narrator seam contract, after `narrator-cleanup-lines`. Both test names contain `line_break_tag`.
   - `// CASE narrator-cleanup-break-line: a line that holds only a line-break tag is removed from the answer.` The answer `"\"Read it.\"\n</br>"` gives `"\"Read it.\""`.
   - `// CASE narrator-cleanup-break-inline: a line-break tag inside a line becomes a space, in each of its forms.` Each of the answers `A<br>B`, `A<br/>B`, `A<br />B`, `A</br>B` and `A<BR>B` gives `A B`.

   Run `python build.py test-pattern "line_break_tag" --test-target lib`. Both cases fail.
2. **The change.** Add one `Lazy<Regex>` to `sanitize_llm_output` with the pattern `(?i)<\s*/?\s*br\s*/?\s*>`, and replace each match with `" "`. Apply it before the line trim, so the trim removes the space that a tag alone on a line leaves. Run the same test pattern. Both cases pass.
3. **The checks.** `python build.py clippy`, then `python build.py guardrails` (the `seam-cases` check and the unit test allowlist), then the full `python build.py`.

## What already exists

- `sanitize_llm_output` and its one caller, `LlmCallRecorder`. This is the only change site.
- `narrated_text` in `narrator_tests.rs`, the entry point of the eight cleanup cases.
- `markdown_to_html` and `sanitize_markdown_event`. They do not change.

## Not in scope

- **Rows saved before the fix.** The probe DB fixture of the dashboard design map holds one (Swipe 300). The owner can remove the tag with Edit on that Message.
- **Text that a player writes or edits.** It does not pass through the recorder. The render escape shows it as the player wrote it.
- **Other HTML tags, and Markdown faults in model answers.**
- **A prompt rule against HTML.** One tag in about 505 entries does not justify a prompt change. The map `.scratch/layout-trials/` owns the prompt layout.

## Failure modes

| Failure | Effect | Handling |
|---|---|---|
| The pattern matches text that is not a tag | The story loses characters | The pattern needs `<`, the letters `br`, and `>`, with only `/` and spaces between them. `<brother>`, `<3` and `x<br` do not match (checked with the same pattern in Python) |
| A tag inside a Quantifier JSON string | A space replaces it, and the JSON stays valid | The space decision |
| `parsed_response` differs from `raw_response_json` | The LLM Messages tab shows the clean text, and the raw answer keeps the tag | The thought-block cleanup works the same way today |
| Another model writes the tag often | Extra spaces and empty lines | The HTML render collapses spaces, and Markdown ignores empty lines |

## Branch

`narrator_tests.rs` differs between `main` and `prompt-flexibility` by about 500 lines. So a change on `main` now can conflict with that branch (inferred). Make the change on a short branch from `main` after `prompt-flexibility` merges. The owner approves the commit.

## Done when

- Both new cases pass.
- The full `python build.py` passes.
