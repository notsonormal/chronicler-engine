# Isolate the UI investigator's probe server

Type: task
Status: resolved
Blocked by: 02

## Question

`chronicler-ui-investigator` drives the real dashboard against the user's live game.
On 2026-10-04 a probe ran a real LLM turn in the active game and it had to be removed by
hand. The skill says nothing about mutation.

## Work

Run the probe server so it cannot open the live game's database. The server DB path is
`<exe dir>/chronicler_<port>.db` (`src/bootstrap/run.rs:52-58`), so a separate
`--target-dir` gives the probe its own database. The chosen shape is:

```bash
python build.py run --target-dir <probe dir> -- <server flags>
```

Point the skill's Capture step at that server, and state that a probe must never be
pointed at the live dashboard. The cost is one cold build.

The separate-port and copied-binary variants were considered and rejected in
[Decide which mechanisms to build](02-decide-mechanisms-to-build.md) in favour of the
explicit `--target-dir`.

## Done when

- The skill gives the exact command to start an isolated probe server.
- A probe run leaves its DB file under the probe target dir, not the live one.

## Answer

Resolved 2026-10-05. `.agents/skills/chronicler-ui-investigator/SKILL.md` now starts its own
isolated server. The Serve step gives the exact command:

```bash
python build.py run --target-dir tmp/probe-server -- --world redmist_estate --port 3001
```

and states the never-rule at the moment of action: "A probe must never be pointed at the live
dashboard" — followed by the positive target, "Drive only the server this step started." The
Drive step navigates to port 3001. Default port moved from 3000 to 3001, outside the
integration-test band 3010–3050 (`tests/test_config.json`).

Verification (all re-checked 2026-10-05):

- `src/bootstrap/run.rs` derives `db_dir` from `current_exe().parent()` and opens
  `<exe dir>/chronicler_<port>.db`. Confirmed.
- `build.py` `run` execs `<target-dir>/debug/chronicler_engine` (`_target_paths` +
  `_exec_pending`), so the probe's exe dir is `tmp/probe-server/debug`. Confirmed.
- The exact command parses: `target_dir=tmp/probe-server`, profile dir
  `tmp/probe-server/debug`, forwarded flags `--world redmist_estate --port 3001`, bind port
  3001. Confirmed by calling `build.parse_args`.
- Port band: integration tests allocate 3010–3050 (`tests/test_config.json`,
  `docs/diataxis/reference/coding_standards/integration_test_standards.md:150`), so 3001 is
  outside it. 3011 and 3020 would both have collided.
- Empirical: the built binary copied to `tmp/probe-verify/debug/` and run on port 3141 wrote
  `tmp/probe-verify/debug/chronicler_3141.db`, served `/` 200, and created **no**
  `target/debug/chronicler_3141.db`. The scratch dir was removed afterwards.

The cold compile of `tmp/probe-server` itself was not exercised; its only unwritten step is
`cargo build`, and every step after it is the above. `tmp/` is gitignored, so the probe target
dir leaves no tracked residue.
