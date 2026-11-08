# Add the TLA+ model layer to docs/AGENTS.md, and rewrite game_flow.md

Type: task (HITL)
Status: open
Blocked by: 14

## Question

1. Add the TLA+ model as a layer to `docs/AGENTS.md`, *Where a fact lives* (ticket 09, Q2): "the order of steps between parts that run at the same time, and the rules that hold in every order" live in the TLA+ model of a verify skill, `.agents/skills/verify-<area>/model/`.
2. Rewrite `docs/diataxis/reference/game_flow.md` as a whole (ticket 09, Q6). It is not a cut of single sentences.

The owner reviews the draft before it lands. The shape of the document is the owner's decision.

## The rules for the rewrite (ticket 09)

- Specs keep the observable promises. Diátaxis keeps the names, the invariants and the reasons. The TLA+ model keeps the orders.
- `game_flow.md` restates no order that the TLA+ model holds. It links to the diagrams of ticket 14 and names the TLA+ model in one sentence (ticket 04, comparison 5 as changed by ticket 09).
- Sections that no TLA+ model covers stay prose in the new form: *The State of a Game*, *The Story Window*, *Trigger Evaluation*, *Retry Flow*, *Movement Branch* and others (known: run 1 covers about 60 of 171 lines).
- The sentences that ticket 11 and ticket 12 moved into spec scenarios go.

## Start material

- A sort of the gate sentences, made in round 3 of ticket 09 as a proposal. The owner asked for a full rewrite instead, so it is input, not a decision:
  - always true, stays: "a Game has at most one holder"; the two kinds of holder; no saved record holds the holder; `GameWriter` is the only code that opens a transaction; no hold of the registry spans an LLM call.
  - an order, goes to the TLA+ model: the span of a run's hold; the id check of each save of a `JobWriter`; the reset in one hold.
  - a rule "when X, then Y", goes to a spec: the busy rule refuses story writes; a config write passes during a plain Job; a stop clears the holder, the failure line and the Options failure.
- Section 12 of [the ticket 03 asset](../assets/03-gate-spec-only-model.md): the start list of an earlier trim.
- Skills: `/document-review`, `/diataxis-doc-review`.

## Checks

- `python build.py validate-docs` and `python build.py vale` pass.
- `python build.py` passes.
- The owner approves the draft.
- Do not commit.
