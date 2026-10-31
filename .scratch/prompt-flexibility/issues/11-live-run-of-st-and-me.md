# Decide whether to run ST and ME live

Type: grilling
Status: open
Blocked by: —

## Question

Do we run SillyTavern or Marinara Engine live with the original cards and the same model? If yes, which calls does the run cover?

## Context

- [How ST and ME let an author change a call](02-st-me-prompt-configuration.md) answered the layout question from source alone ([asset](../assets/st-me-prompt-configuration.md)). A full live comparison adds nothing to the layout decisions (inferred by the researcher).
- One fact remains that source cannot give: does a real model treat a trailing `system` impersonation prompt (ST, `public/scripts/openai.js:1382`) differently from a trailing `user` one (ME, `packages/server/src/routes/generate.routes.ts:7495`)? This bears on [Decide which parts of a Narrator-role call an author can change](06-the-layout-model.md).
- Options to weigh: no live run; a live run of the Impersonate call only; or the same check on this engine, with the role of its last part changed, and no ST or ME install.
- An automated, repeated comparison with ST and ME is out of scope (map).
