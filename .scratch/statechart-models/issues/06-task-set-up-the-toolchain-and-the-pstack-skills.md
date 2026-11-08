# Set up the toolchain and the two pstack skills

Type: task
Status: resolved
Blocked by: —

## Question

Make the tools of the trial ready on this host, and prove that they reproduce a known run. The trial is the decision of [ticket 04](04-decide-the-role-of-the-statechart-skill.md) (see its Answer).

1. **Install `mise` on this host.** The vendored skill `.agents/skills/statechart/` pins its tools in `scripts/run.ts` (`findTlc`): `tla2tools.jar` 1.7.4 with a SHA-256 checksum, and Java `temurin-25`. When `TLA2TOOLS` is not set, it asks `mise` where the two tools are, and it runs `mise install` when they are missing. It needs the network for that. `JAVA`, when set, only replaces the `java` binary. This host has no `mise` and no system JDK today.
2. **Copy two pstack skills by hand** into `.agents/skills/`: `create-formal-verification-skill/` and `maintain-formal-verification-skill/`. The source is `cursor/plugins`, folder `pstack/skills/`, commit `2776ddd2355d6d6c8747d3964600f28daf62f049` (2026-11-05). Each skill is one `SKILL.md`. Do not add a `skills-lock.json` entry: the six other pstack copies in this repo have none (ticket 04, Q9). Do not edit the copies.
3. **Run `statechart` with the pinned tools** on the ticket 03 TLA+ model, with `TLA2TOOLS` and `JAVA` not set, so that `mise` installs the tools:

   ```sh
   cd .scratch/statechart-models/assets/03-gate-spec-only
   ../../../../.agents/skills/statechart/scripts/statechart GateSpecOnly.tla \
     --config configs/GateSpecOnly.cfg --view GateSpecOnly.view.mjs --out <new empty folder under tmp/>
   ```

   Compare the report with `runs/run1-docs/report.md`: the same broken rules and the same shortest paths. The first TLC runs used Temurin 21, and the pin is Temurin 25, so a difference is possible. Report it if you find one.
4. **Record the cost**: the disk use of the `mise` data folder, and the peak memory of the TLC run. This host has 8 GiB, and memory is its limit (`ENVIRONMENT.md`). Do not run TLC while a cargo build runs (`python scripts/build_slot.py --status`).

## Context

- The old toolchain was a Temurin 21 JDK and `tla2tools.jar` in `/tmp/statechart-tools/`, and Pele 1.9.0 in `/tmp/pele-render/` for the SVGs. Leave them. They stop being needed after this ticket, except Pele. Say in the Answer whether Pele must also move out of `/tmp`.
- `assets/00-code-models/README.md` gives the old run commands with `JAVA` and `TLA2TOOLS`. Update its "Run one model again" section to the `mise` way.
- Apart from `scripts/test/`, the vendored `statechart` is the same as pstack at `2776ddd2` (ticket 04, 2026-11-05). Do not update it.

## Checks

- The run of step 3 ends, and its report matches, or the Answer explains each difference.
- `python build.py validate-docs` passes.
- Do not commit. The owner decides the scope of each commit.

## Answer

**The toolchain is ready, and it reproduces the ticket 03 run exactly** (2026-11-05).

- **`mise`.** Version `2026.11.1 linux-x64 (2026-11-04)`. Installed with the official script, `curl -fsSL https://mise.run | sh`, into `~/.local/bin/mise`. That folder is already on `PATH`, so no shell activation was added. The runner found `mise` with no other setup.
- **The install.** With `TLA2TOOLS` and `JAVA` not set, the first `statechart` run called `mise install` itself. `mise ls` now shows `http:tla2tools 1.7.4` and `java temurin-25.0.5+8.0.LTS`.
- **The versions that ran.** `tlc.out`: `TLC2 Version 2.19 of 08 August 2024 (rev: 5a47802)` (this is `tla2tools.jar` 1.7.4), on `Eclipse Adoptium 25.0.5`, 1 worker, 2048 MB heap.
- **The comparison (known).** `report.md` and `chart.mmd` of the new run are byte-for-byte the same as `assets/03-gate-spec-only/runs/run1-docs/`: the same broken rules, the same shortest paths, the same "Rules not checked" list. Temurin 25 in place of Temurin 21 made no difference. The runner exits with code 1, because rules break; that is the expected result for this config. A second check: `00-code-models/job-life` with the `mise` tools gives the same `report.md` and `chart.mmd` as `out-job-life/` (exit 0).
- **Disk use.** `~/.local/share/mise` (the installed JDK and jar): 305 MB. `~/.cache/mise` (the downloaded archives, `downloads-cas`): 138 MB; `mise cache clear` can remove it. The `mise` binary: 170 MB. Total about 610 MB.
- **Cost of one run.** Peak memory of the TLC `java` processes: 550 MiB RSS (a 0.1 s sample of `ps -C java`; the host has no GNU `time`). Wall time: 29 s with the install, 21 s without it. The build slot was free during each run.
- **Pele.** It need not move out of `/tmp`. The `statechart` skill says to install it outside the repo, and only the SVGs need it, not the checks. A reinstall is one `npm install pele@1.9.0` (4 MB). The only file that `/tmp` could lose for good was the render script, so `assets/00-code-models/render.mjs` now holds a copy of it.
- **The two pstack skills.** `.agents/skills/create-formal-verification-skill/SKILL.md` (43 lines) and `.agents/skills/maintain-formal-verification-skill/SKILL.md` (58 lines), fetched from `raw.githubusercontent.com/cursor/plugins/2776ddd2355d6d6c8747d3964600f28daf62f049/pstack/skills/<name>/SKILL.md`. The GitHub contents API at that commit lists only `SKILL.md` in each folder. Not edited, and no `skills-lock.json` entry.
- **The README.** `assets/00-code-models/README.md`: "Run one model again" now uses the `mise` way, and "Render a chart as SVG" gives the reinstall commands for Pele.
- **Left in place.** `/tmp/statechart-tools/` (Temurin 21 and the old jar) and `/tmp/pele-render/`. The runs of this ticket are in `tmp/t06/`.
- **Not known.** Whether `~/.local/bin`, `~/.local/share/mise` and `~/.cache/mise` survive a rebuild of this container. If they do not, run the install script again; the first `statechart` run then installs the tools again (needs the network).
