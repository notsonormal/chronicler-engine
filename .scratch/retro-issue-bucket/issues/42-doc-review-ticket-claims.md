# Check the tickets a doc review is about to make false

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

The `/document-review` pass found that `swipe_new.md` I.9, scenario 11.10 and `game_flow.md:69` disagreed, and ruled "keep one wording, in I.9". Applying it rewrote `docs/diataxis/reference/frontend/dashboard.md:38` and deleted two rules from it — including "names only an action the dashboard itself offers", which ticket 11's Answer records as *that doc's* job. The loss was caught only by the later code review's spec axis ("**11 — wrong (a ticket claim is now false)**"), and fixed in another pass. The doc review had no reason to know the line was load-bearing: a resolved ticket said so, in `.scratch/`, which the review never read.

Where should the check live?

- **The doc review reads the ticket Answers.** Before deleting a claim, it greps the effort's `.scratch/**/issues/*.md` for the sentence or the file it names, and treats a hit as a constraint to keep or to correct in the ticket.
- **The after-plan brief supplies them.** The brief already names the tickets under review; it also names the claims those Answers record as delivered, so the pass starts from the record instead of rediscovering it.
- **A deletion must carry its ticket edit.** A doc line whose removal makes a ticket's Answer false is deleted only together with the correction to that Answer.

## Context

- Source: `/retro` 2026-10-11 on the phase2-after-plan pass.
- Doc-review session `01a12c59-98f2-7575-a39d-4899f5086d09` (reviewer), "Doc set": `swipe_new.md` I.9 against 11.10 and `game_flow.md:69` — "these three disagree on what a failed retry leaves behind … Fix: keep one wording, in I.9."
- The Lead accepted it; commit `151c4579` "docs(specs): correct the phase-2 review's stale spec lines" replaced the `dashboard.md:38` sentence.
- Spec-axis session `01a12c70-90fa-72fb-844d-ee138e30611f`, finding 11: ticket `11-show-the-token-budget-remedy.md:51` says "`dashboard.md`: the 'Failures and Health' section now states the invariant — a generation failure's sentence is drawn in full, and names only an action the dashboard itself offers"; `git show 151c4579 -- docs/diataxis/reference/frontend/dashboard.md` shows both rules gone.
- Restored in `41eeef0a`. Cost: one doc-fix commit, one review finding, one fix pass.
- Related: [Widen the stale-reference sweep to `.agents/skills/`](15-stale-references-in-skills.md) is the other direction (a removed identifier left live in prose).
