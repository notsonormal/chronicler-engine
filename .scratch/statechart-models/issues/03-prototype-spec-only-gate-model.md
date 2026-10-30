# Prototype: one TLA+ model of the gate, built from the spec only

Type: prototype
Status: resolved
Blocked by: 01

## Question

Does a TLA+ model of the gate, built from the docs and the spec only, find anything that the tests of [Research: which rule sentences of the gate the tests already pin](01-research-gate-rules-and-their-tests.md) do not pin?

Build one TLA+ model with the `statechart` skill, with a view in the reader's words, and run it twice:

1. **Guard on.** The TLA+ model as the docs describe it. Each break is a candidate divergence between the docs and the code.
2. **Guard off.** The same TLA+ model with the job id check removed (a constant). The TLA+ model must break. If it does not, the rules miss what the check protects.

Then replay each break of run 1 against the real code: write the path down as a tier-1 test sketch (with `ScriptedModel`, the hold of the LLM fake, and the pause point), and run it if the sketch is cheap. The replay says which side is wrong: the docs, the code, or the TLA+ model.

Deliver, as linked assets in `.scratch/statechart-models/assets/`:

- the TLA+ model, its config, its view, and the report and SVG of each run;
- a table: each invariant, the rule sentence it states (quoted), the row of ticket 01 it covers, the result of each run;
- each break with its replay: the path, the test sketch, and the verdict;
- the time the work took, and the number of states that TLC reached.

## Context

- **The discipline.** Write the TLA+ model from `docs/diataxis/reference/game_flow.md` and the gate scenarios in `docs/specs/`. Do not read `src/` before the first run. A TLA+ model written from the code can only agree with the code. Read `src/` after the first run, for the replay.
- **Scope of the TLA+ model.** The holder (none, a plain Job, an Autoplay run with or without its Job), the job id in the job entry, the saves of a Job (input, text, state update, Options, failure line, step), the late save of an ended Job, stop, reset, delete, a config write, and the run's own accept. The candidate invariants are the unpinned rows of ticket 01.
- **Toolchain.** See the map Notes: `JAVA` and `TLA2TOOLS` from `/tmp/statechart-tools`, Pele in `/tmp/pele-render`. Reinstall them if `/tmp` was cleared. Keep the constants small; TLC stops at 100,000 distinct states after the first break.
- **Earlier TLA+ models.** `tmp/statechart/` holds three TLA+ models that were written after a read of the code. Do not reuse their rules; their views can serve as examples of labels.
- **No landing.** The test sketches are candidates. This ticket lands no test and changes no code (owner, 2026-10-29).
- HITL: the owner judges each rule sentence that the TLA+ model states, and each verdict of a replay.

## Comments

### 2026-10-30: the TLA+ model and the replays are done; waiting for the owner's judgement

Asset: [The trial TLA+ model of the gate, built from the spec only](../assets/03-gate-spec-only-model.md), with the TLA+ model, the view, the configs and the report and SVG of each run in [03-gate-spec-only/](../assets/03-gate-spec-only/).

Draft answer, for the owner to judge (section 11 of the asset):

- **Yes, it found something.** Three rules that the code holds and `game_flow.md` does not state (G1: the end of a Job or a run removes only its own entry; G2: the accept of a run's loop passes only while its run holds the Game; G3: a run ends on a failure line only when its own Job wrote it). The spec states G2 (41.6, 41.7) and G1 for one order (41.17). No code defect.
- **Run 2 breaks.** With the job id check a constant, six rules break (R01, R05 to R09). R02, R03, R04 and R10 hold without the check: the "own id" checks of G1 to G3 protect them.
- **Six candidate tests**, all green on the code today and removed after the run: G1, G3, R05, R07, R08, R09. Each forces an order that no test forces, or reads a record that no test reads after a late answer.
- **Limits.** 5 of 15 rules hold by construction (R11 to R15). The Input save order (R01) is not reachable with the LLM fake hold. The delete case cannot break in this TLA+ model.
- **Cost.** About 50 minutes of work; the full check at the bounds of run 1 reached 15,798,130 states in 19 minutes; each break shows in under 4,000 states.

The ticket stays claimed until the owner has judged the rule sentences and the verdicts.

## Answer

Asset: [The trial TLA+ model of the gate, built from the spec only](../assets/03-gate-spec-only-model.md), with the TLA+ model, the view, the configs, and the report and SVG of each run in [03-gate-spec-only/](../assets/03-gate-spec-only/).

**Yes, it found things that the tests of ticket 01 do not pin, but no code defect.**

- **Three "own id" rules.** The end of a Job or of a run removes only its own entry (G1). The accept of a run's loop passes only while its run holds the Game (G2). A run ends on a failure line only when its own Job wrote it (G3). The code holds all three. The spec holds G2 (41.7) and G1 for runs (41.17). G3 is not clear in autoplay.md ("a Job" or "a Job of the run").
- **Not doc gaps (owner, 2026-10-30).** `game_flow.md` need not cover every use case: the specs hold the event rules, and the docs give an understanding. The trial shows the reverse: `game_flow.md` holds event rules that a spec already holds, and mechanics that the code holds (asset, section 12).
- **Run 2 breaks.** With the job id check a constant, R01 and R05 to R09 break. R02, R03, R04 and R10 hold without it: the "own id" checks protect them.
- **Six candidate tests**, all green on the code once and then removed: G1, G3, R05, R07, R08, R09. Each forces an order that no test forces, or reads a record that no test reads after a late answer.
- **Limits.** 5 of 15 rules (R11 to R15) hold by construction. No tier-1 test reaches a stop before the Input save. The delete case cannot break in this TLA+ model.
- **Cost.** About 50 minutes of work. Every break showed in fewer than 4,000 states; the full check of 15,798,130 states (19 minutes) found nothing more.
- **Not judged.** The quoted rule sentences of R01 to R10, and the candidate tests and spec edits. They stay candidates, out of the scope of the map.
- **For ticket 04.** The owner's leaning: the TLA+ model is a one-time audit that picks up gaps, and a TLA+ model in `tmp/` is written again for each audit.
