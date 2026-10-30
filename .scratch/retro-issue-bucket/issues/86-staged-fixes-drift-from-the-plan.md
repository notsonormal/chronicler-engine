# Per-stage implement, review and fix loops move the code away from the approved plan

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A plan that runs in stages, with one implementation, one review and one fix pass for each stage, ends with code that the approved plan does not describe. Each review checks one stage against the plan. Each finding gets a local fix, and the implementer chooses the shape of that fix, usually by adding a counter, a flag or a rule. Nothing checks the whole code against the plan's intent before the final gate. So each fix looks reasonable alone, and the total grows into the patchwork that the plan was written to remove. The plan text can stay honest while this happens, because the deviations go into a Gate log and not into the design.

How should a staged plan keep its result close to its approved design?

## Context

- **First occurrence: the change-driven dashboard refresh** (archived as `old-docs/archived-plans/change-driven-dashboard-refresh.md`, branch `wf/change-driven-dashboard-refresh`, 2026-10-25). The server half held. The browser half grew into a set of hold rules and markers. Stage 3b alone added `data-reset-on-save`, `data-autosave`, `data-saved-flag`, `data-refusal-what`, `data-opens-editor` and `data-close-editor`, none of them in the plan. After the plan was done, about half of the `dashboard.md` markers existed only to protect drafts. That result is why `old-docs/archived-plans/dashboard-visible-tab-refresh.md` was written.
- **Second occurrence: the visible-tab refresh** (`old-docs/archived-plans/dashboard-visible-tab-refresh.md`, plan commit `ebae627e`, stages 1–6 committed through `577c358a`, stage 7 in progress, 2026-10-26).
  - The plan body changed by only 16 lines since `ebae627e`. The Gate log records each deviation. One deviation contradicts the plan body without a plan edit: stage 6 step 5 says to delete `INLINE_ERROR_CONTAINERS`, and stage 6 kept it.
  - Dashboard JS lines: `main` 1,444; plan start (`ebae627e`) 1,838; after stage 6 plus the stage 7 work tree 2,070. The new `assets/dashboard-screen.js` (502 lines) is the size of the `dashboard-live.js` it replaced (511 lines at `535f1a3c`). The plan's aim was a simpler loop.
  - The stage 6 and stage 7 review fixes added mechanisms that the plan's design does not name: a dialog session counter, a `data-saving` guard against a second submit, a `gameChanges` counter, a `serversMet` order list, a rule that asks do not reload while a skipped request is in flight, and a hold on the `hx-disabled-elt` controls (`tmp/review/stage6/review.md`, `tmp/review/stage7/review.md`). Each one fixes a real race. Nobody asked if a change to the design would remove the race.
  - The lead accepted each fix shape without that question.
- **Source.** The owner raised the problem on 2026-10-26, in the lead session of the visible-tab refresh plan, before stage 8.
- **Third run: the action-answers plan did not repeat the failure mode** (`old-docs/archived-plans/dashboard-action-answers.md`, fixed point `e31c30b5`, stages 1–5 through `70bf8dbc`, Final gate `a68ad764`, fix pass `fcd76b9f` and `067ac8f2`, 2026-10-28 to 2026-10-29). Checked in lead session `01a1743c-7303-77ca-953f-4e72116e9d9f` on the owner's request.
  - **No mechanism growth on the client.** The Final plan check (`tmp/final-review/aa-verify.md` §9) traces every client mechanism to a plan decision. The only exception is the fallback string "No detail.". On the server, two small items were accepted in the stage 2 review but not written into the plan body: the 64 KiB read limit and the pass-through of an unbuilt `409`. The pass-through was later deleted (owner Q8).
  - **The client shrank at every gate.** Dashboard JS: 1,995 → 1,995 → 1,907 → 1,770 → 1,717 → 1,684 (Final), then 1,686 after the fix pass.
  - **Design questions went to the owner, not into local fixes.** Six decisions came up during the stages and went into the plan body as O11–O16. Example: stage 4 review M1 (a stale Delete) became O12, the last-Message check, through a plan edit. In the fix pass, the builder stopped at F1 and did not add a focus mechanism. A regression of B2 went to the owner as Q14.
  - **What held:** the stage-7b rule (a fix that adds a mechanism edits the plan body or goes to the owner); the JS line count recorded at every gate; the Final plan check's list of every mechanism and its source.
  - **A smaller, different gap remained: rules built only in part.** Three plan rules covered fewer paths than the plan said, and the stage reviews passed them:
    - O15 (a failed update names the typed name) held only when the last save step failed (review P2).
    - A2 (a request that names a gone record answers `400`): a Connection Edit answered `500` (review AP-1).
    - A5 (every answer has a sentence) had the unbuilt-`409` exception.

    No mechanism grew here; a rule covered fewer paths. A stage review checks the diff of one stage and does not ask "does every path follow rule X?". The Final plan check asked that and found all three.
  - **One cause was in the approved plan itself.** A6 gave `answer_for_error` an input that held "whether the route is a settings route" (plan line 174). The design therefore classified errors in two ways, although its aim was "one classifier". No stage review can catch a flaw that the approved plan contains. It was fixed after the Final gate (owner Q6).
