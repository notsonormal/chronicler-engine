# Write the verify skill of the gate, and do its first run

Type: task
Status: resolved
Blocked by: 06

## Question

Write `.agents/skills/verify-gate/` with `/create-formal-verification-skill` (vendored by ticket 06). Then do its first run, and write the gaps that it finds to the trial log. Read the Answer of [ticket 04](04-decide-the-role-of-the-statechart-skill.md) first. It holds every decision below, and the grilling rounds above it give the reasons.

## The decisions (ticket 04)

- **The TLA+ model is made from the code.** Each action points to a function that a reviewer can open. One indivisible step in the code is one step in the TLA+ model.
- **Coverage.** `src/application/generation/gate.rs`, and the functions that take a hold of the gate (`GameWriter`, `JobWriter`, `BusyRegistry`) in `src/application/pipeline/job.rs`, `src/application/autoplay/driver.rs` and `src/application/games/catalogue.rs`. The coverage record names each function, with the sync commit. Ticket 08 may later read the record with a Python script, so keep it in a form that the script can read without a Markdown parser, for example a small TOML or JSON file in the skill folder.
- **The rules come from the spec.** Each rule quotes one gate scenario sentence of `docs/specs/`. Write the list of rules before you read `src/`. A rule written after a read of the code copies the code, and it can never break. Gate scenarios are in `docs/specs/dashboard.md` (some of the 39.x scenarios, for example 39.5; not all 39.x scenarios are about the gate) and `docs/specs/autoplay.md` (41.x, for example 41.6, 41.7 and 41.17). Other specs can hold some, for example `actions.md`, `reset.md`, `games.md` and `settings.md` (inferred).
- **Two lists in the skill.** The actions, each with its code function and its covering spec scenario. The rules, each with its quoted spec sentence.
- **The five comparisons**, with the TLA+ model as the hub:

  | Pair | How | A gap means |
  |---|---|---|
  | TLA+ model and code | Each rule fails once on a broken copy of the code, for example a removed check or a reordered step | The TLA+ model is wrong: fix it |
  | Spec to TLA+ model | TLC checks each rule | A code bug, or a wrong spec |
  | TLA+ model to spec | Each action and each branch has a covering scenario | A spec gap |
  | TLA+ model and tests | A tier-1 test forces the order of each break path | A test gap |
  | TLA+ model and docs | No sentence of `docs/diataxis/reference/game_flow.md` contradicts the chart | A doc error. A missing sentence is not a gap. |

  A rule that only `game_flow.md` states, with no spec scenario, is a spec gap.
- **Toolchain.** `mise` with the skill's pins (ticket 06). Small bounds: memory (8 GiB) is the limit of this host. Do not run TLC while a cargo build runs.
- **The skill is loadable by agents.** No `disable-model-invocation` in its frontmatter. Its description names the covered files, so that an agent that changes a gate function finds it. Refer to `/statechart` and `/maintain-formal-verification-skill` by name, and copy none of their commands.

## Inputs

- [`assets/00-code-models/`](../assets/00-code-models/): `JobLife.tla`, `Autoplay.tla` and `SwipeState.tla`, written from the code on 2026-10-29. `gate.rs` and `job.rs` have had more than 25 commits each since then, so read the code again.
- [The ticket 03 asset](../assets/03-gate-spec-only-model.md): rules R01 to R15, and six candidate test sketches (G1, G3, R05, R07, R08, R09). Most of its rules quote `game_flow.md`, so find their spec scenarios again. Five rules (R11 to R15) held by construction: the "able to fail" step must catch that kind.
- [The ticket 01 asset](../assets/01-gate-rules-and-their-tests.md): 39 gate rule sentences and the tests that pin them.

## Scope

- **Report only** (ticket 04, Q12). Land no test. Edit nothing in `src/`, `tests/`, `docs/specs/` or `docs/diataxis/`.
- **The scratch break of "Prove it before you hand off"** changes real code for a short time. Other sessions use this checkout at the same time. So make the scratch change in a linked git worktree under `tmp/`, and remove the worktree after the run.

## Output

- `.agents/skills/verify-gate/`, its TLA+ model, its configs, its view, and its coverage record with the sync commit.
- `assets/trial-log.md` (create it): one row for each run and each maintain pass. Columns: the date, the commit range, the minutes, the outcome (in sync, updated, bug or blocked), and the gaps by kind. Below the table, one entry for each gap: its kind, its path in the code's words, the rule or the action, and the spec scenario.
- The first row of the trial log, from the first run.

## Checks

- The skill ran once, as the next agent would run it, and it caught the scratch break.
- `python build.py validate-docs` passes.
- Do not commit.

## Answer

Append here: the gaps by kind (a count, and a link to the trial log), the bounds, the states that TLC reached, the time that the work took, and what the TLA+ model does not cover.

`.agents/skills/verify-gate/` is written, and its first run is row 1 of the [trial log](../assets/trial-log.md). The outcome is **bug**.

**The skill.**
- `SKILL.md`: the trigger, the run, how to read a break, the model update, the action table and the rule table.
- `model/VerifyGate.tla`: 29 actions, each one named after its code function.
- 7 configs.
- `model/mutants.toml`: 18 mutants.
- `check.py`: one command, `python3 .agents/skills/verify-gate/check.py`, with `--mutants` for the mutants.
- `mise.toml`: the pins of the statechart skill.
- `coverage.toml`: 44 functions in 4 files, with sync commit `e6715c60`.

**The rules.** There are 15 rules, each one quoted from `docs/specs/`. The rule list was written before any read of `src/`. Each rule breaks first on at least one mutant. That mutant mirrors a removed check or a dropped step of the code.

**The proof run.** The scratch break was a run id check removed from `GameWriter::end_autoplay`, in a linked worktree. A fresh agent followed the skill cold and judged the break a code bug. Its first run gave a false green, because the model had not yet followed the code. `SKILL.md` now says so at the top of *Run it*. The agent found eight more unclear points, and `SKILL.md` is fixed for them too. The worktree is removed, and on `e6715c60` the check gives exit 0 with only the known break.

**The gaps by kind** (the [trial log](../assets/trial-log.md) has each entry):

| Kind | Count |
|---|---|
| code | 1 |
| model | 1 |
| spec | 8 |
| test | 5 |
| doc | 1 |

- Code: `GameCatalogue::delete_game` checks the active Game outside the hold of `GameWriter::delete`, so a switch between the check and the delete makes it delete the active Game. The harm is low: only a thread preemption reaches that order. The fix mutant shows that a check inside the hold closes the race. **Correction (ticket 09):** it closes one order of three, see the Comments of ticket 09.
- Model: TLC tries the story writes and the config write only while the Game is busy.
- Test: 13 of the 18 break paths have a tier-1 test that forces them.

**The bounds and the states.**
- At most 2 Games, 2 plain Jobs or 1 run with a budget of 2 cycles, or 2 runs with a budget of 1 cycle.
- From 769 states (`runs`) to 147,768 states (`games`).
- One full check takes about 35 s, and the mutants take about 25 s. Peak memory is under 2 GiB.
- A larger bound costs much more: `run-and-jobs` with 3 Jobs has 3.1 million states and takes 2.5 min.

**The time.** 74 minutes, from 20:42 to 21:56 UTC.

**Not covered.**
- Storage faults, and the stop whose dock read fails (39.8).
- A Job that writes to its own Game only. The model holds this by construction.
- That each Job reaches its end. Only no deadlock is checked.
- The story content, and the count of writes of one Swipe state.
- A restart of the server.
- The `StillThinking` branch of `accept_for_autoplay`. The model has it, but TLC never reaches it.
- Any check of the code itself. The model follows the code only when an agent updates it, and ticket 08's staleness warning is what tells the agent to do that.
