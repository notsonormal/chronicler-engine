# Close the two create and delete races

Type: task (AFK, decide and do)
Status: open
Blocked by: —

## Question

Close two races between overlapping requests. Choose each fix under the enforcement order of the map Notes, and put it in place.

- **Two Games with one stable name (storage-07).** `create_game` and `reset_game` read the names in one storage call and write in a later one. The schema has no `UNIQUE` on `games.name`. The candidate fixes: `UNIQUE(games.name)` and one retry, or the name scan and the insert in one storage call.
- **A Game that names a deleted preset (storage-10).** `PromptPresetService::delete_preset` checks in four storage calls and deletes in a fifth, with no hold over the sequence. The candidate fixes: the checks and the delete in one storage call, or a guard on `GameWriter`.

## Context

- Findings in full: storage-07 and storage-10 in [the storage report](../assets/storage.md). The coordinator did not read these in the code.
- **Existing names.** Under the Q5 rule a new `UNIQUE(games.name)` migration fails loudly on two Games with the same name. Look in the dev database for duplicates first. If any exist, the migration stops the server until a Game is renamed, so stop and ask the owner (decide-and-do rule: it changes what the player sees).
- A guard on `GameWriter` changes `gate.rs`, which `verify-gate` covers (map Notes).
- The six end conditions of the map Notes apply.

## Answer
