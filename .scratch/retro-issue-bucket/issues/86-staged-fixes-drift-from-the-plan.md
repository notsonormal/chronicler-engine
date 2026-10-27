# Per-stage implement, review and fix loops move the code away from the approved plan

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A plan that runs in stages, with one implementation, one review and one fix pass for each stage, ends with code that the approved plan does not describe. Each review checks one stage against the plan. Each finding gets a local fix, and the implementer chooses the shape of that fix, usually by adding a counter, a flag or a rule. Nothing checks the whole code against the plan's intent before the final gate. So each fix looks reasonable alone, and the total grows into the patchwork that the plan was written to remove. The plan text can stay honest while this happens, because the deviations go into a Gate log and not into the design.

How should a staged plan keep its result close to its approved design?

## Context

- **First occurrence: the change-driven dashboard refresh** (archived as `old-docs/archived-plans/change-driven-dashboard-refresh.md`, branch `wf/change-driven-dashboard-refresh`, 2026-10-25). The server half held. The browser half grew into a set of hold rules and markers. Stage 3b alone added `data-reset-on-save`, `data-autosave`, `data-saved-flag`, `data-refusal-what`, `data-opens-editor` and `data-close-editor`, none of them in the plan. After the plan was done, about half of the `dashboard.md` markers existed only to protect drafts. That result is why `docs/plans/dashboard-visible-tab-refresh.md` was written.
- **Second occurrence: the visible-tab refresh** (`docs/plans/dashboard-visible-tab-refresh.md`, plan commit `ebae627e`, stages 1–6 committed through `577c358a`, stage 7 in progress, 2026-10-26).
  - The plan body changed by only 16 lines since `ebae627e`. The Gate log records each deviation. One deviation contradicts the plan body without a plan edit: stage 6 step 5 says to delete `INLINE_ERROR_CONTAINERS`, and stage 6 kept it.
  - Dashboard JS lines: `main` 1,444; plan start (`ebae627e`) 1,838; after stage 6 plus the stage 7 work tree 2,070. The new `assets/dashboard-screen.js` (502 lines) is the size of the `dashboard-live.js` it replaced (511 lines at `535f1a3c`). The plan's aim was a simpler loop.
  - The stage 6 and stage 7 review fixes added mechanisms that the plan's design does not name: a dialog session counter, a `data-saving` guard against a second submit, a `gameChanges` counter, a `serversMet` order list, a rule that asks do not reload while a skipped request is in flight, and a hold on the `hx-disabled-elt` controls (`tmp/review/stage6/review.md`, `tmp/review/stage7/review.md`). Each one fixes a real race. Nobody asked if a change to the design would remove the race.
  - The lead accepted each fix shape without that question.
- **Source.** The owner raised the problem on 2026-10-26, in the lead session of the visible-tab refresh plan, before stage 8.
