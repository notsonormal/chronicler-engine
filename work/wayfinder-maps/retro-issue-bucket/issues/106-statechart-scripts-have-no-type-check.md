# The `/statechart` TypeScript has no working type check and no tests

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The `/statechart` skill is now a fork that this repo changes. Its scripts have no check that runs: how must a change to them be checked?

## Context

Source: `/retro` 2026-11-08, the coordinator session of statechart-models tickets 08, 10, 11, 12, 13 and 16 (`01a1bd22`), and its subagent sessions (t13 `41648274`, t16 `5bfd1d3b`).

- **known** `.agents/skills/statechart/package.json`: the `test` script runs `test/*.test.ts`, but no `test/` folder exists.
- **known** `tsconfig.json` asks for `types: ["node"]`, but nothing installs `@types/node`. t13: `npx tsc` gave "Cannot find type definition file for 'node'". t13 installed TypeScript into `tmp/t13/tc`, and t16 used that copy again.
- **known** That type check found an error that is already in HEAD: `exit` typed `number` in `scripts/run.ts`. t13 and t16 report "only the 2 errors that were there before".
- **known** t13 and t16 each compared the outputs of the three TLA+ models of `work/wayfinder-maps/statechart-models/assets/00-code-models/` before and after by hand (`diff -rq -x tlc.out`).

## Options to decide

- **A.** Add `typescript` and `@types/node` to `devDependencies`, a `check` script (`tsc -p .`), and a golden-output test on the three models. Fix the two old errors. Optional: a `build.py` step.
- **B.** Only the type check.
- **C.** Nothing: the fork changes seldom.
