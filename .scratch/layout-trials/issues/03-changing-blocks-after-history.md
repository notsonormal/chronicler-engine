# Try the changing blocks after the history

Type: task (AFK)
Status: open
Blocked by: 01

## Question

Does the cached share go up when the parts that change during a story stop breaking the cache before the history? The candidate removes the `(in room)` / `(elsewhere)` marks from `<KnownCharacters>`, and moves `<GameState>` and `<CharactersPresent>` to after `<ConversationHistory>`.

## Context

- Target part: the cache. The cached share alone decides: keep the candidate when the share goes up (map Notes).
- This one candidate replaces two candidates of [Decide which parts of a Narrator-role call an author can change](../../prompt-flexibility/issues/06-the-layout-model.md): "no presence marks" and "`GameState` after the history" (owner, plan review, 2026-11-01).
- Measured on the last trial of Prompt flexibility ([the shared prefix of the cache candidates](../assets/cache-prefix-measure.md)):
  - Each of the two old candidates alone gains about 1 and 4 points.
  - With `<CharactersPresent>` moved too, the shared prefix of the Narration calls goes from 55.5 % to 83.9 %, and of the Impersonate calls from 60.7 % to 85.3 %. No pair breaks before the history.
  - When a Character moves or the Room changes, `<CharactersPresent>` changes too. So all three parts must move.
- `<CharactersPresent>` is not in the list of ticket 06. It holds the full cards of the Characters in the Room. After the history, these cards sit near the end of the prompt, so the output can change more than with `GameState` alone (inferred). Record an output change that you see. It does not decide.
- Caching rule 3 of [Find how prompt caching works for our providers and our layout](../../prompt-flexibility/issues/12-prompt-caching.md) put `GameState` after the cards and before the history. The measure shows that a Room change then still loses the history.
- The new order of the user message: `<KnownCharacters>` (with no marks), `<Persona>`, `<WorldLore>`, `<ConversationHistory>`, `<GameState>`, `<CharactersPresent>`, `<writing_style>`, `<output_format>`, `<PlayerInput>`, `<Guide>`. The style and format sections stay at the end, as the written rule "Prose constraints placed at the end of the user message" in `docs/diataxis/explanation/prompt_system_design.md` says.
- Known: `join_user_layers` in `src/application/prompting/assembler.rs` sets the order. `render_character_cards_layer` renders both card lists, so the trial code splits it into two layers. `render_known_character_entry` in `src/application/prompting/builders/sections.rs` writes the marks.
- Known: `render_and_fit` calls `join_user_layers(…, false)` to get the fixed part that the history space subtracts. Put the two moved layers in the list before `post_history_prompt`, not in the `with_tail` part. Otherwise the history space does not count them, and the overflow guard cuts more Messages (inferred).
- The change applies to every Narrator-role call (Narration, Event Narration, the regenerate of a user Input, and Impersonate), because they share the assembler. The Quantifier and Options calls do not change.

## Steps

1. Run [`prefix_measure.py`](../assets/prefix_measure.py) on the pack of [Run the baseline trial](01-baseline-trial.md). Record the line "no marks + GameState and CharactersPresent after". The measure screens the candidate (map Notes). If the gain is not clear on the new code, stop and ask the owner.
2. Make a linked worktree from `layout-trials` with a detached HEAD, and change the order there. The change is for the trial only, so it needs no tests.
3. Build it, copy the main probe DB into its target dir, and start it as a probe server on its own port (map Notes, "How a trial runs" and "Worktree limits").
4. Copy the setup file to `tmp/lt03/redmist_julian.json`, with `out` set to `tmp/lt03/redmist_julian`. Run one trial of 100 Cycles, with `--base` and `--db` for that server.
5. Check that the pack still shows the history and the call kinds (map Notes, "Failure modes"). Run `prefix_measure.py` on the new pack: its "today" line is the byte share of the candidate, and it should be near 84 % for the Narration calls.
6. Run `compare` with the baseline pack.

Done when the answer gives the cached share of both trials (in total and for the Narrator-role calls, from `pack.md`), the measured byte share of both packs, the verdict (keep or drop), any output change that you saw, and the worktree is removed. A kept candidate gets a new task ticket (map Notes).
