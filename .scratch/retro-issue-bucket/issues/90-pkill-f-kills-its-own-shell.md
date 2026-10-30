# `pkill -f` kills the shell that runs it

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The bash tool runs a whole command line as one `bash -c "<command>"` process. `pkill -f <pattern>` matches the full command line of every process, so it also matches that shell and sends it SIGTERM. The command ends with exit 143 and no output, and any later part of the command line does not run. It has happened in two sessions.

Should anything change, and where?

## Context

- Source: `/retro` 2026-10-29, lead session `01a1743c-7303-77ca-953f-4e72116e9d9f` (the action-answers Final gate and its fix pass, branch `wf/change-driven-dashboard-refresh`). The lead ran `pkill -f 'playwright_chromiumdev_profile-ndmiLy' ; …` to stop a hung headless Chrome; the shell died with exit 143.
- An earlier session (2026-10-25, same lead session, observation on tooling constraints) hit the same failure and noted "kill by captured PID instead". The note did not prevent the repeat.
- The safe forms, used elsewhere in the same session: `pgrep -f '[c]hronicler_engine…' | xargs -r kill` (the bracket regex does not match its own literal text) or `kill $(cat <pid file>)`.
- **Options seen in the retro.** A: a permission rule for `pkill -f` in `.pi/extensions/pi-permission-system/config.json` set to "ask" (only the user can apply it; it interrupts the user for a low-risk command). B: one line in `ENVIRONMENT.md` (read mainly when a build is slow). C: no change; one failed call, and the agent recovers. The lead's view in the retro was C or B. Compare Won't do 63 (`rg -r`), where the owner kept the option of a deny rule.
