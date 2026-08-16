# Slash-command parser for steering entry

Type: task
Status: pending

## Question

Replace the `Action::FreeAction` stub with a slash-command parser recognizing `/narrator`, `/impersonate`, `/guide`.

Per the design synthesis (`../research/04-design-synthesis.md`, Q10):

1. Today `Action` (`src/domain/model/action.rs`) has one variant `FreeAction(String)` and `Action::parse` copies input verbatim. `FreeAction` is a leftover from removed non-LLM movement — it carries no weight and is replaced, not extended-by-habit.
2. Add a parser that recognizes `/narrator <text>`, `/impersonate [text]`, `/guide <text>` prefixes in the single `command` field of `ActionForm` (`src/adapters/driving/http/action/handlers/actions.rs`). One input box, one form field, one endpoint — the slash-command convention across ST, Marinara, and GG.
3. Dispatch each command to its feature's pipeline path. Plain input (no slash) keeps the existing `process_action` / `continue_narration` dispatch.
4. The parser is the entry point for tickets 07 (guide), 08-specs, 09 (impersonate), and the narrator generate path (ticket 10). Keep the dispatch targets decoupled so each feature ticket can land independently.