# Make an empty Message text follow ticket 03

Type: task (AFK, decide and do)
Status: open
Blocked by: 03

## Question

Put in place the rule of [ticket 03](03-grilling-small-product-rules.md) (rule 4) for an empty Message text (llm-04). Choose where the rule lives under the enforcement order of the map Notes.

Today `edit_history_handler` passes `form.text` to `GameWriter::edit` with no check (`src/application/generation/gate.rs:560-571`), and `set_swipe_text` writes it. The transport drops an empty part only when it is the system part, so `{"role":"user","content":""}` reaches the provider. The narrator's own answer is already guarded.

The candidate places: a type for Message text that cannot be empty (enforcement preference 2), or a check in `GameWriter::edit` (preference 5).

## Context

- Finding in full: llm-04 in [the llm report](../assets/llm.md). The coordinator confirmed it in the code.
- The browser does not guard it either: the edit textarea has no `required` (`assets/dashboard-story-log.js:71-77`). A browser guard is not the enforcement. The server is.
- `GameWriter::edit` is in `gate.rs`, which `verify-gate` covers (map Notes).
- The six end conditions of the map Notes apply.

## Answer
