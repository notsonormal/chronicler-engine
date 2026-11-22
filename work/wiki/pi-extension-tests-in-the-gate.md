# The Node tests beside the pi extensions run in the gate

These claims describe the working tree on 2026-11-22. The change that made them is `d90e9821`, committed on that day.

- As of `a39ef8d0`, no gate step ran the TypeScript unit tests under `.pi/extensions/`. `node --test <file>` ran them, and a test could break while the gate stayed green.
- On 2026-11-22, `build.py` gained the step `pi-tests`: `node --test '.pi/extensions/**/*.test.ts'`, placed in `_GATE_CHECKS` after `py-tests`. The full gate and `--no-browser` run it.
- The pattern is single-quoted, so the shell leaves it alone and Node expands `**`. That runs a test file at any depth, including `pi-plan-mode/test/plan-mode.test.ts`, which a one-level glob missed. On 2026-11-22 the step ran 99 tests in five extensions.
- The step needs Node 22 or later, because Node strips the TypeScript types. The machine ran Node 24.22.1 on 2026-11-22. The `build.py` docstring names this requirement beside the Vale one.
- Node prints a `MODULE_TYPELESS_PACKAGE_JSON` warning per test file on 2026-11-22, because the repo's `package.json` names no module type. The warning did not fail the step.
- Two Python tests pin the step table: `scripts/tests/test_build_cli.py` (`EXPECTED_COMMANDS`) and `scripts/tests/test_build_cli_gate.py` (the gate's step count). On 2026-11-22, adding the step failed both until they named it.

To run one extension's tests alone, call Node on its files:

```bash
node --test .pi/extensions/mrn-context/*.test.ts
```

Related: [The pi-note sources of `mrn-context`](pi-note-sources.md).
