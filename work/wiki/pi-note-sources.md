# The pi-note sources of `mrn-context`, and the usage note that left

These claims describe the working tree on 2026-11-22. The change that made them is `ff93cd9e`, committed on that day.

- From 2026-10-21 to 2026-11-22, `mrn-context` posted three notes into a pi turn: the model of the turn, the newest build log of the session, and the Anthropic 5h and 7d usage. From 2026-11-22 it posts the first two.
- On 2026-10-21, commit `a925fed1` added the usage source to `.pi/extensions/mrn-context/index.ts`. It read the cache that `@sreetej510/pi-usage` writes (`~/.pi/agent/usage-cache.json`) and posted a note whenever the 5h or 7d window crossed a bucket of `[0, 30, 60, 80, 90, 100]`. The aim was that the agent would change its behaviour under budget pressure.
- On 2026-11-01, commit `c290cc68` added `.pi/extensions/rate-limit-wait`. It holds a run at the Anthropic 5h limit until the window resets, then continues the run. A 7-day limit is not waited for.
- On 2026-11-22, the owner judged the usage note no longer worth its cost. Two reasons: the auto-wait makes the 5h wall a pause rather than a loss, and the 7d wall has never fired, so a warning about it would ask for a change that the agent cannot make. The 730 notes across 508 session files, many at 3% to 20% used, were noise. I agreed.
- The note carried no instruction. `/home/node/.pi/agent/AGENTS.md` is 16 lines and says nothing about usage or limits, and the project `AGENTS.md` says nothing either. So the reaction to the note was always implicit.

What read the account windows on 2026-11-22:

- `rate-limit-wait` reads the `anthropic-ratelimit-unified-*` headers into `~/.pi/agent/rate-limit-wait.json`. `/rate-wait` shows both windows on demand.
- `delegate` reads that same state file to choose the spare model of a role (`pickModel`, `.pi/extensions/delegate/lib.ts`).

Nothing read the `pi-usage` cache except the note that left; `rg` found no other reader in `.pi/` or `settings.json`.

On 2026-11-22 the note logic moved out of `index.ts` into `sources.ts`, with `sources.test.ts` and `index.test.ts` beside it. The owner asked for the tests; the split was the agent's own choice, to get the sources under test at all.

Related: [The Node tests beside the pi extensions run in the gate](pi-extension-tests-in-the-gate.md).
