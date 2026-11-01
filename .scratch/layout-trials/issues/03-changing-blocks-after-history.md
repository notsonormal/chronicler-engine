# Try the changing blocks after the history

Type: task (AFK)
Status: resolved
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
- Known, on the baseline pack ([Run the baseline trial](01-baseline-trial.md), `tmp/lt01/prefix-measure.md`): today 56.3 % (Narration) and 57.4 % (Impersonate). With the three-part candidate: 82.0 % and 84.0 %, and no pair breaks before the history. "GameState after the history" alone now gains 10.5 points for the Narration calls (pf20: about 4). The three-part candidate still gains 15 points more than that, so it stays one candidate. Step 1 is done.
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

## Answer

Resolved on 2026-11-01. Verdict: **keep**. The cached share rose from 47.6 % to 68.9 % in total, and from 55.2 % to 78.8 % for the Narrator-role calls. The map rule is "keep when the share goes up".

### How it ran

- **known** The trial ran in a linked worktree, `/workspace/chronicler-engine-lt03`, made from `layout-trials` (`1a0c9162`) with a detached HEAD. Nobody committed anything.
- **known** The change is in `tmp/lt03/candidate.diff` (76 lines, 2 files). It is trial code only and has no tests.
  - `render_known_character_entry` in `sections.rs` writes `- <name>` with no `(in room)` or `(elsewhere)` mark.
  - `render_character_cards_layer` in `assembler.rs` is split into `render_known_characters_layer` and `render_characters_present_layer`.
  - `fixed_user_layers` (the ticket calls it `join_user_layers`) now lists: KnownCharacters, Persona, WorldLore, history, GameState, CharactersPresent, `post_history_prompt`. The two moved layers sit in the fixed list, so the history space counts them. `tail_layers` is unchanged.
- **known** The server ran on port 3003 on a copy of the main probe DB (SQLite backup call, schema version 34). The Connection `conn-1793449507061-3a1cb15f` was in it. `build.py run` started the server from the worktree and linked `.env` itself. I did not read the key. The main probe server on port 3001 stayed up and untouched.
- **known** The first Narration and Impersonate calls showed the new order. User prompt blocks of a Narration: `KnownCharacters`, `Persona`, `WorldLore`, `ConversationHistory`, `GameState`, `CharactersPresent`, `writing_style`, `output_format`, `PlayerInput`. An Impersonate call has no `Persona` and no `PlayerInput`, as before. The known-Character lines carry no mark (`- Gabriella`). `<ConversationHistory>` is in every Narrator-role prompt.
- **known** One run of 100 Cycles, no retry. The pack is `tmp/lt03/redmist_julian/trial-1`. The setup file is `tmp/lt03/redmist_julian.json` (`out` = `tmp/lt03/redmist_julian`).

### The numbers

| Item | Baseline (`tmp/lt01`) | This trial (`tmp/lt03`) |
|---|---|---|
| Cycles played | 100 of 100 | 100 of 100 |
| Failed calls | 0 | 0 |
| Wall time | 1280 s | 1114 s |
| Tokens in / out | 3,067,165 / 52,637 | 3,055,352 / 45,077 |
| Cached input, total | 47.6 % | **68.9 %** (2,104,448) |
| Narration cached | 52.8 % | 77.8 % |
| Impersonate cached | 57.9 % | 79.9 % |
| Event narration cached | no call | 48.4 % (1 call, 2,048 of 4,233) |
| Narrator-role calls cached (Narration, Impersonate, Event narration) | 55.2 % | **78.8 %** |
| Quantifier cached | 3.8 % | 7.3 % |

- **known** The Narrator-role share is my sum of the kind rows of `pack.md` (cached tokens over input tokens). `pack.md` does not print it.
- **known** The shared byte prefix, from `prefix_measure.py` (`tmp/lt03/prefix-measure.md`), line "today":

| Call kind | Baseline | This pack | Prediction on the baseline pack |
|---|---|---|---|
| Narration | 56.3 % | **83.4 %** | 82.0 % |
| Impersonate | 57.4 % | **85.2 %** | 84.0 % |
| Quantifier | 39.0 % | 43.5 % | no change expected |

- **known** In this pack no pair of calls breaks before the history (0 of 99 for each kind, against 37 and 35 in the baseline).
- **known** The provider reports 68.9 % cached against about 84 % shared bytes. The byte share stays an upper bound. The gap is the rule of the provider: it caches whole blocks of 128 tokens, and the Quantifier calls read little from the cache (ticket 02).
- **inferred** The cached share of the Narrator-role calls gains 23.6 points, and the byte measure said about 26. The measure predicted the gain well.
- **inferred** The Quantifier share (3.8 % to 7.3 %) is not an effect of the candidate: those prompts did not change. Ticket 02 found that the provider alone decides those hits.

### Pack checks (the layout failure mode)

- **known** The pack has 100 Narration calls, 100 Impersonate calls, 201 Quantifier calls and 1 Event narration call (402 calls). The `pack.md` story entries show the Room, the Characters present and "history holds N entries" for the Narrator-role calls. The history columns and the call kinds are intact.
- **known** The baseline had no Event narration call. This story fired one Trigger. That is a story difference and not a code difference.

### History cut

- **known** The first cut is at call 1174, the 141st call of this pack, and it drops 23 entries. In the baseline it is the 153rd call. 131 of 402 calls carry a cut (baseline: 124 of 400).
- **guessed** The cut comes a little earlier because this story is a little longer in the prompt (median user prompt of a Narration: 59,345 characters against 58,196). The two moved layers are in the fixed part, so the history space counts them as before. I did not test the cause.

### Output change (does not decide)

- **known** Median Narration length: 191 words in the baseline, 171 in this trial (pf20: 175). Median Input length: 160 words in the baseline, 131 in this trial (pf20: 126). Tokens out: 52,637 against 45,077.
- **inferred** The new numbers are close to pf20, and the baseline is the high one. One trial for each side cannot tell the candidate from the story noise.
- **known** I compared the first entries only (entry 1 is the same text in both packs; entry 2 differs in the Input). I did not read the two stories in full. So I saw no sign that the Characters present near the end of the prompt changed the Narration, and I did not look for one in depth.
- The owner can read `tmp/lt03/compare.md` for the output of the Characters present near the end (map: the output notes are for the output tickets).

### Files

- Pack: `tmp/lt03/redmist_julian/trial-1`
- Comparison with the baseline: `tmp/lt03/compare.md`
- Prefix measure: `tmp/lt03/prefix-measure.md`
- Trial code: `tmp/lt03/candidate.diff`
- Log of the run: `tmp/lt03/run.log`

### Worktree

- **known** The worktree is removed (`git worktree remove`) and the port 3003 server is stopped. `git worktree list` shows only the main checkout. The plain remove refused, because the two changed files were modified. I saved their diff to `tmp/lt03/candidate.diff` first, then used `--force` on this worktree only.
- **known** A mistake of mine: `trial.py compare` wrote its output into `tmp/lt01/redmist_julian/compare.md` (beside the first pack) and replaced the baseline comparison with pf20. I moved the new output to `tmp/lt03/compare.md` and wrote the baseline comparison again with `trial.py compare tmp/lt01/redmist_julian/trial-1 tmp/pf20/redmist_julian/trial-1 --out tmp/lt01/redmist_julian/compare.md`. The text is built from the same two packs, so it is the same file in content (inferred: I did not keep a byte copy of the old one). Use `--out` the next time.
