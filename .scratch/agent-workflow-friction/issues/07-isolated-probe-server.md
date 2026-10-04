# Isolate the UI investigator's probe server

Type: task
Status: open
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
