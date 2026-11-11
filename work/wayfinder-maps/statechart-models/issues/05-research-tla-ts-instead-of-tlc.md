# Research: tla-ts instead of TLC

Type: research
Status: resolved
Blocked by: —

## Question

Does [tla-ts](https://github.com/aidenybai/tla-ts) give the same checks as TLC for the TLA+ models of this map, at a lower cost or with a better fit to this repo?

TLC needs a JDK 11+ and `tla2tools.jar` (see the **Toolchain** note of the map). This host has neither, and the [tying research](02-research-tying-a-tla-model-to-code.md) found that a JDK in the build lowers two of its methods. A checker without a JDK can change that cost.

Answer each point. Name the source: the repo, its docs, its code, its issues. Label each fact as known, inferred or guessed.

1. **What it is.** Is tla-ts a TLA+ checker, a TLA+ parser, a TypeScript DSL that is like TLA+, or a different thing? Does it read `.tla` files, or must each TLA+ model be written again in TypeScript?
2. **Coverage of the language.** Which TLA+ and PlusCal features does it support? Check the features that the [ticket 03 TLA+ model](../assets/03-gate-spec-only/GateSpecOnly.tla) and its `configs/*.cfg` use: `CONSTANTS` per config, invariants, temporal properties and fairness, `VIEW`, symmetry, state constraints.
3. **Equal results.** Run (or say why you cannot run) tla-ts on the ticket 03 TLA+ model. Compare it to the TLC runs in `assets/03-gate-spec-only/runs/`: the number of distinct states, the broken invariants, the counterexample traces. Where it differs, say why.
4. **Speed and memory.** Compare the wall time and the memory of the two checkers on the same configs. Say the hardware.
5. **The statechart skill.** Does `.agents/skills/statechart/` (its runner, its trace views, its Pele SVGs) work with tla-ts output, or does it need changes? Which changes?
6. **Toolchain fit.** What does it need: Node or Bun version, npm packages. Can `mise` or `build.py` install it? Does the repo already have that runtime?
7. **Maturity.** Age, release history, number of maintainers, open defects, test suite, licence. Is there evidence that its results were checked against TLC?
8. **Other checkers.** Note, without a deep study, other checkers without a JDK (for example Apalache needs a JVM too; say so if true) only if they change the answer.

End with one recommendation: use tla-ts in place of TLC, use it beside TLC, or keep TLC. Give the reason, and say which "Not yet specified" items of the map (the toolchain) the answer changes.

Write the findings to `assets/05-tla-ts-instead-of-tlc.md` and link it here.

## Answer

**Keep TLC.** Findings: [Research 05: tla-ts in place of TLC](../assets/05-tla-ts-instead-of-tlc.md).

tla-ts is a real TLA+ checker, not a DSL. It reads the `.tla` file and the `.cfg` file unchanged, and on the ticket 03 TLA+ model it agrees with TLC on **eleven configs**: the same distinct states, the same first broken rule and the same exit code, and the same trace on the three configs that I diffed (**known**, run on 2026-10-31 against TLC 2.19 / `tla2tools.jar` 1.7.4). The state counts match exactly, from 38 states to 2,728,171.

It cannot take TLC's place yet, because the `statechart` skill reads three things that no release of tla-ts writes: TLC's `-tool` message framing, the `-dump dot,actionlabels` state graph, and SANY's XML tree (**known**). tla-ts `0.0.3` accepts those flags and ignores them, and it writes no dump file at all. The maintainer has the matching open ticket (#228) and an open pull request (#232, 116 files) that adds both, and no release ships it. A second gap: tla-ts writes `TheInputIsSaved(1)` where the skill's trace reader expects `TheInputIsSaved`.

| | Answer |
|---|---|
| Speed | about equal on this host. The README's "2× faster" did not reproduce; the same config gave TLC 40 to 52 s and tla-ts 51 to 102 s across three passes. |
| Memory | tla-ts used 34 % to 75 % of TLC's peak RSS below 500,000 states, and about the same at 2.7 million. |
| Toolchain | Node only, one npm package, no JDK. `npx --yes tla-ts@0.0.3` works today. The repo already has Node. |
| Maturity | 5 days old, one maintainer, `0.0.3` the only release, MIT, but 823 recorded TLC results in its conformance suite. |

The JDK cost that [the tying research](02-research-tying-a-tla-model-to-code.md) put on its places 2 and 3 is a JDK **in the build**. That cost drops for a plain check of the TLA+ models, and it stays for a chart from the skill. So the **toolchain** item of the map splits in two, and no other "Not yet specified" item changes.

Re-run the comparison with `ab.sh` in [05-tla-ts/](../assets/05-tla-ts/). Raw TLC and tla-ts output is in [05-tla-ts/runs/](../assets/05-tla-ts/runs/).
