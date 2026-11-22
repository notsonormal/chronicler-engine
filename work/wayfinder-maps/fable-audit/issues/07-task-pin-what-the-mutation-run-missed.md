# Pin what the mutation run missed, and delete one step that does nothing

Type: task
Status: open
Blocked by: —

## Question

Put the four changes below in place (owner, 2026-11-22, D23 of [the decisions](../decisions.md)). They come from the mutation run of the audit: section 5 of [the environment report](../assets/development-environment.md), and row N15 of [the list](../assets/findings-not-carried.md).

A mutant is one small change to the code, such as `<` to `<=`, or a function body replaced by a fixed value. A mutant is "missed" when no test fails. The run used the unit tests of the library only (`--lib`). So check first that no test under `tests/` catches the mutant: make the change by hand and run the tests of that area.

### 1. Delete the code-fence step of the Quantifier parser

- **Measured.** `extract_json_from_code_fence` (`src/application/agents/quantifier/utils/parser.rs:114`) can return `None`, an empty string or `"xyzzy"`, and every unit test passes (4 missed mutants).
- **Read it.** `try_parse_json_full` tries three steps: the whole answer, the content of a code fence, and the text from the first `{` to the last `}`. The third step reads a fenced answer too. So the second step decides nothing.
- **The change.** Delete the function and its step. A test with a fenced answer must pass before and after the delete.

### 2. Tests that make each state check fail

- **Measured.** `assert_state_consistency`, `assert_room_exists`, `assert_character_consistency` and `assert_encounters_consistency` (`src/domain/model/utils/game_rules.rs:224-272`) can each return `Ok(())`, and every unit test passes.
- **Read it.** The checks run in the shipped build: `default = ["testing"]` switches the `diagnostics` feature on. No test gives them a state that is wrong.
- **The change.** One test for each of the three parts, with a Swipe state that the check must refuse: a current Room that is on no map and is no added Room, a Character present that the World does not have, and an Encounter that breaks its rule. Read each check for the exact rule first.
- If a check cannot fail for any state that the Game rules can make, say so in `## Answer`. Do not write a test that builds an impossible state only to cover the line.

### 3. Tests for the boundaries of the token budget

- **Measured.** Seven missed mutants in `src/application/prompting/token_budget.rs`:
  - `history_reserve_tokens` (`:22`) can return 0 or 1, and its division can be `%` or `*`.
  - `TokenBudget::response_tokens` (`:66`): the `>` at `:76` can be `>=`, the `+` at `:79` can be `*`, and the `+` at `:85` can be `-`.
- **Why it matters.** These numbers decide how long an answer may be, and when the history is cut.
- **The change.** Tests with values on each side of each boundary: a context size where the reserve is the fixed 1,024 tokens, one where it is an eighth of the context, and the two cases of `response_tokens` on each side of its comparison.

### 4. Direct tests for the two sanitize functions

- **Read it.** No test calls `sanitize_llm_output` (`src/application/prompting/sanitize.rs:8`) or `sanitize_for_prompt` (`src/application/prompting/builders/sections.rs:8`) by name. Both are covered only through their callers.
- **The change.** A table test for `sanitize_llm_output`: a leading channel marker, a `<thought>` block, a channel-thought block, a turn marker, and text with none of them.
- **Two other changes touch these functions.** Do them first, or agree the order with their agents:
  - Change 7 of [ticket 02](02-task-engine-fixes.md) moves the escape of story text into the assembler. `sanitize_for_prompt` may then have no use. The weekly retro proposes its delete (R16). Test what is left after that change.
  - Change 5 of [ticket 06](06-task-engine-fixes-second-group.md) looks at the trim of every line in `sanitize_llm_output`.

## Rules for every change

- A new test must fail when the mutant of its row is put in by hand. Say in `## Answer` that you checked this for each row.
- Line numbers are from commit `e8d6512a`. Find the site again by its name before you edit.
- Use `python build.py test-pattern "<pattern>" --test-target lib` during the work. Run `python build.py --no-browser` once at the end.
- Do not commit.

## Not in scope

- **The retry condition of the Quantifier** (`quantifier/utils/orchestration.rs:129`, `:139`, 4 missed mutants). The loop ends at the same place for each comparison, so the condition is redundant, not wrong. Changes 15 and 17 of ticket 06 (D20, D25) rewrite this loop.
- **The write of `currently_meeting`** (`game_rules.rs:116`, 1 missed mutant). Change 8 of ticket 02 deletes the field.
- **Three single boundaries:** `Lt` in `Encounters::check_condition` (`trigger.rs:98`), one `>` in `HistoryCut::of` (`history_cut.rs:38`), and the Low-reading retry above. The owner accepted the four points of this ticket, and these were not among them. They are small: add them only if the test of a neighbour row makes it one more line.
- A second mutation run. It takes about 35 s for each mutant with the build slot. `work/wayfinder-maps/unclebob-quality-tools/` owns that plan.

## What already exists

- The lists of the run: `assets/local-only/evidence/mutants/<part>/mutants.out/missed.txt` (git-ignored, on the machine of the audit). The command to repeat it: `assets/local-only/evidence/mutants/run_shards.sh`.
- `work/wayfinder-maps/unclebob-quality-tools/assets/03-mutation-engines.md`: the method and its rules.
