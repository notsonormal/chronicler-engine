# Statecharts of this engine

Generated with the skill at `.agents/skills/statechart/`, on 2026-10-29, after a read of the code. Copied from `tmp/statechart/` on 2026-11-05 (ticket 04). The first runs used a Temurin 21 JDK in `/tmp/statechart-tools/`. Since ticket 06, `mise` gives the runner its pinned tools.

Each folder holds one model:

| Folder | Model | Chart |
|---|---|---|
| `job-life/` | One Job on one Game: the holder, the steps, the failure line, Stop, and the Options run that follows a saved Narration | 7 states, 14 arrows, every rule holds |
| `autoplay/` | One Autoplay run: the run holds the Game between Jobs, the cycles count, and the run ends on its budget, a failure or a stop | 9 states, 18 arrows, every rule holds |
| `swipe-state/` | The state of one new Swipe: the pre-analysis write, the analysed write, the Retry of a typed Input, and Stop or a restart between the two writes | 7 states, 12 arrows, one rule breaks |

## Run one model again

The runner needs a JDK and `tla2tools.jar`. When `TLA2TOOLS` is not set, it asks `mise` for its
pinned tools (`tla2tools.jar` 1.7.4 and Java `temurin-25`), and it installs them when they are
missing. `mise` is in `~/.local/bin/mise` on this host (ticket 06). Do not set `TLA2TOOLS` or `JAVA`.

```sh
unset TLA2TOOLS JAVA
cd /workspace/chronicler-engine/.scratch/statechart-models/assets/00-code-models/job-life
/workspace/chronicler-engine/.agents/skills/statechart/scripts/statechart \
  JobLife.tla --view JobLife.view.mjs --out out-job-life
```

`--out` must be a new or empty folder. The run writes `report.md`, `chart.mmd`, and TLC's own log.

## Render a chart as SVG

Pele renders the `.mmd` file to SVG, next to it with the same name. Pele is installed outside the
repo, in `/tmp/pele-render/`. `render.mjs` in this folder is a copy of its script: it puts the
style block of `references/render.md` at the start of each SVG. If `/tmp` was cleared, install
Pele again first:

```sh
mkdir -p /tmp/pele-render && cd /tmp/pele-render
[ -d node_modules/pele ] || npm install pele@1.9.0
cp /workspace/chronicler-engine/.scratch/statechart-models/assets/00-code-models/render.mjs .
node render.mjs /workspace/chronicler-engine/.scratch/statechart-models/assets/00-code-models/*/out-*/chart.mmd
```

Each folder also holds a `chart.png`. It is a Chrome screenshot of the SVG, for a quick look only.
