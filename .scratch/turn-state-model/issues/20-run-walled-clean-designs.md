# Run three walled clean designs from the spec

Type: task
Status: resolved
Blocked by: 19

## Question

What design does an agent with only the reviewed spec produce? Three agents run, from three model families. Where they assume, ask, or disagree with each other, the spec has a gap. This task is step 3 of the clean-room route. It is AFK.

## The wall (ticket 17)

- A raw one-shot `pi` run, from the [research 15 asset](../assets/research-15-clean-room-method.md) §Method and commands:
  `pi --no-session --no-context-files --no-skills --no-extensions --no-prompt-templates --no-mcp --no-approve --no-tools -e <provider extension> --model <model> [--thinking <level>] -p "<prompt>"`.
  The provider extension is the one that the agent definitions use for the model's provider: `@lnilluv/pi-opencode-go-rotation` for the opencode-go models, and `@gotgenes/pi-anthropic-auth` for the Anthropic model. Each run loads only the extension of its own provider.
- Run it from a neutral temporary directory outside the repository (for example a `mktemp -d` under `/tmp`). From the repository root, the system prompt names `/workspace/chronicler-engine`.
- The prompt embeds `../assets/clean-run-prompt.md` and `../assets/clean-spec.md`, as reviewed in ticket 19, and nothing else. SillyTavern, Marinara, `CONTEXT.md`, the docs and every research asset stay out.
- **Models:** `opencode-go/glm-5.3-flash`, `opencode-go/kimi-k2.7-code` and `anthropic/claude-opus-5-5` at `--thinking xhigh`. GLM and Kimi are not from the spec writer's family (Opus). Opus is, so a gap that Opus fills the way the spec writer would does not show in its design (research 15 §1, limit 1). The GLM and Kimi seats keep that test. The Opus seat adds the strongest design.
- **Owner change, 2026-10-12.** Ticket 17 (decision 5) chose two clean agents, GLM and Kimi. The owner added the Opus seat in this ticket (option C of three: Opus in place of both, Opus in place of one, or Opus added). This is a run decision, not a requirement, so it is not in `clean-spec-rulings.md`.
- **First, a canary.** In the same directory, ask each model for its callable tools, any context file, and every system-prompt line that names a project or a path. Record the answers. Also check how `pi -p` takes a long prompt before the real run.
- The run has no `ask_owner`. The agent lists its questions in the design. The owner answers them in ticket 21.

## Output

- `../assets/clean-design-glm.md`, `../assets/clean-design-kimi.md` and `../assets/clean-design-opus.md`. Each holds the verbatim output, which is the whole transcript of a one-shot run.
- The leak check (`research-15-leakcheck.py`) on each design. A distinctive `src/` name is a question for the wall, not proof of a leak, because a common name can be invented again.
- A note of what each design recalls of SillyTavern. Three of the four probed models recall its swipe storage (research 15 §5).
- `../assets/clean-run-gaps.md`: one row per gap, tied to a spec section. The rows are every assumption, every question, and every point where the designs disagree. Each row says which designs raised it, so that ticket 21 can see a gap that only Opus filled. No owner answers go here.

## Done when

- The canary result is recorded in the Answer.
- All three designs are saved verbatim, and the leak check has run on each.
- The gap list exists.

## Answer

Resolved on 2026-10-12. All three walled runs finished. [`clean-run-gaps.md`](../assets/clean-run-gaps.md) holds 38 gap rows: 17 for the open questions and 21 others. In 23 of the 38 rows the designs split, the spec conflicts with itself, or only one design raises the point.

**The command (tested).** Each run used `pi --no-session --no-context-files --no-skills --no-extensions --no-prompt-templates --no-mcp --no-approve --no-tools`, with one provider extension and the prompt as one argument (`-p "$(cat prompt.md)"`):

| Seat | `-e` | `--model` | Time | Exit | Words |
|---|---|---|---|---|---|
| GLM | `@lnilluv/pi-opencode-go-rotation` | `opencode-go/glm-5.3-flash` | 9 min 22 s | 0 | 8,239 |
| Kimi | `@lnilluv/pi-opencode-go-rotation` | `opencode-go/kimi-k2.7-code` | 6 min 50 s | 0 | 6,624 |
| Opus | `@gotgenes/pi-anthropic-auth` | `anthropic/claude-opus-5-5 --thinking xhigh` | 12 min 9 s | 0 | 7,320 |

The three runs ran in parallel from `/tmp/tmp.t0mTvAZPtY`, a directory made by `mktemp -d`. The prompt (`clean-run-prompt.md`, an empty line, then `clean-spec.md`; 35,090 bytes, 6,148 words) had 0 leak hits. stderr was empty for each run. The prompt, the canary answers and the run times are kept in `tmp/cleanrun-20/`.

**The canary (tested).** It ran from `/tmp/cleanrun.u2Qd5l`, with the same flags and extensions. Each model reported: no callable tool; no context file; and no system-prompt line that names a project or a skill. The only path is the cwd. GLM and Kimi also quoted the lines of pi's own documentation paths (`/home/node/.npm-global/lib/node_modules/@earendil-works/pi-coding-agent/...`). Opus also quoted "Be concise and helpful." and "Show file paths clearly when working with files.". GLM read the directory name "cleanrun" as a hint of a test sandbox. So the real runs used a directory with a neutral `mktemp` name.

**Long prompts (tested).** A 35,198-byte filler prompt with a marker at each end went to GLM as one `-p` argument. GLM gave back both markers and the correct number of the last line. So `pi -p` takes the whole prompt.

**The leak check (tested).** `research-15-leakcheck.py` found 0 `src/` names, 0 `src/` paths and 0 `.rs` names in each design and in the gap list.

**Recall of SillyTavern and Marinara (inferred, from a read and a grep).** Only Opus names a product: "The terms Swipe, Impersonate and Guided Generation also exist in SillyTavern. No design idea is taken from it". No design uses a term specific to SillyTavern (`swipe_id`, `swipe_info`, `chat_metadata`, an out-of-range in-flight index, an integrity slug, checkpoints, branches) or to Marinara (the `(messageId, swipeIndex)` key, macro variables). All three keep a full game state per Swipe, as Marinara does (research 14). R-36 forces that choice, so it is not evidence of recall. Kimi's `activeSwipeIndex` and its camel-case record names look like code, but they match no name of either product.

**What the gap list shows.** These are inferred observations for ticket 21. They are not owner answers.

- **The designs agree on 15 rows.** Examples: R-38 wins over R-36 for the offered set (OQ-01); a Retry whose redo fires a Trigger adds an event Narration (OQ-02); notices are not Messages (OQ-04); a user regen is not analysed. The two conflicts that ticket 19 left open (OQ-01 and OQ-02) got the same answer from all three designs.
- **The largest splits are on the game state after a delete (OQ-11), the trigger context (OQ-14, B-01, B-05) and when a one-shot Trigger counts as fired (B-02).** These are the writer and pointer questions of the map's fog, at the level that a player observes. In G and K, a Retrigger can write an event again after the event is already in the story.
- **O found two conflicts inside the spec that G and K did not see.** B-04: R-17 offers Retrigger after a failed event, but after an Impersonate the last Message is an Input, so R-65 refuses it. B-16: R-01 parses an offered option such as "/options" as a command, but R-06 says that it goes in as free text. The spec writer's family found these two, so the shared-writer channel did not hide them.
- **Kimi asked no questions.** It answered each OQ and left its section 12 empty. It also added a `409 Conflict` answer and a dismiss control for notices. The spec asks for neither.
- **G answered three OQs only as questions with a default (OQ-05, OQ-06, OQ-11).** Its OQ-05 default departs from R-44 as written.

**Changes to this ticket.** The owner added the Opus seat (see Models). The title changed from "two" to "three" walled clean designs. Ticket 21's link text and ticket 22's input line changed to match.
