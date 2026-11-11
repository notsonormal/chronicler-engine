# Add the TLA+ model layer to docs/AGENTS.md, and rewrite game_flow.md

Type: task (HITL)
Status: resolved
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

## Decided before the work

- **One page per diagram (owner, 2026-11-09).** The generated pages in `docs/diataxis/reference/diagrams/` stay one page per diagram (13 today). Do not merge them into two pages. Link to each page from the section of `game_flow.md` that it shows.

## Checks

- `python build.py validate-docs` and `python build.py vale` pass.
- `python build.py` passes.
- The owner approves the draft.
- Do not commit.

## Progress (2026-11-10, not resolved)

Draft 2, waiting for the owner's review. Draft 1 (2026-11-09) cut only the gate sentences and linked all 13 diagrams, so it grew from 2,277 to 2,322 words. The owner rejected it: more prose, not less, and the diagrams explain themselves.

- **`game_flow.md`: 821 words (was 2,277).** It holds names, owners and invariants only. One sentence names the TLA+ model, and *Document References* has one diagram link. Cut because a spec holds the rule: the failure lines (`failure_display.md`), the status display (`dashboard.md`, `autoplay.md`), Retry (`swipe_new.md`), when a Retrigger is allowed (`retrigger.md`), the `400`/`500` plan refusals, the busy refusals, the config write, the Options run after a failure. Cut because the TLA+ model holds the order: the stop, reset, delete and Autoplay accept orders, the switch order, the order of the two Swipe-state writes. Cut as code detail: the movement fields, the `GenerationOrder::write` list, the steps list. The `#trigger-evaluation` anchor stays (guardrails.md links it).
- **The diagram pages.** Each lead was about 50 words of the same text. `check.py` now writes only the part of each page that differs: `Scope: …` and the TLC state count of a chart, `Rule: …` of a sequence, and the PROPERTY note when it applies. The shared text is in two sentences in the first section of `game_flow.md`. `MODEL_WORDS` is gone.
- **`docs/AGENTS.md` row:** in commit `068b682f` (owner).
- Checks: `check.py` exit 0, 13 pages written. `python build.py` passes (`logs/build_20261110_183829_3214818.log`).
- Open: the busy refusal comes before every other check. No spec and no TLA+ model step holds it now. It needs a spec scenario by the gap rule.

### 2026-11-10, round 3 (owner): anchors and cross-document references

- **The owner's view.** An agent sees the whole repo, and `docs/AGENTS.md` is always loaded. So a doc does not need to route to every other doc. Keep only the cross-document references that are useful.
- **`DOC:` anchors.** 22 modules now point at the doc that names their area: `storage.md` (6: storage `games.rs`, the World and Persona catalogues, `map.rs`, `game_name.rs`, `message_types.rs`), `frontend/dashboard.md` (10: text check, Worlds HTTP), `frontend/http_routes.md` (2: debug), `architecture_system.md` (4: `application/errors.rs`, three `mod.rs`). 26 stay on `game_flow.md`, which names their part.
- **Document References.** The rule of `docs/AGENTS.md` already allows only the paired Reference or Explanation doc and a doc that the body hands a point to. 13 lines that broke it are gone, in 7 docs. Unchanged: generated `guardrails.md`, the diagram pages, and the pairs.
- `python build.py` passes (`logs/build_20261110_185130_3241702.log`).
- Not applied yet: the wording findings 5 to 18 of the review of 2026-11-10.

## Answer

**Done (known).** `game_flow.md` holds the names, the owners and the invariants of the game flow, in 821 words (was 2,277). One sentence names the TLA+ model of the gate, and one link goes to its diagrams. `docs/AGENTS.md` has the TLA+ model row (commit `068b682f`). `python build.py` passes (`logs/build_20261110_190354_3271846.log`).

- **The rewrite.** A rule that a client sees is in its spec. An order between parts that run at the same time is in the TLA+ model. Fine-grained detail is in the code. Draft 1 cut the gate sentences only and linked all 13 diagrams, so it grew. The owner rejected it.
- **The diagram pages.** `check.py` writes one line on top of each page: `Scope: …` and the state count of a chart, or `Rule: …` of a sequence. The text that was the same on each page is gone. The scopes say "a stop" (the action), not "Stop" (the button).
- **The review (`/technical-writing`, `/writing-for-agents`).** Findings 5 to 18 are applied: one meaning for "order", "plain Job" only, the hold of the registry defined, no "land nowhere", split sentences, "TLC" named once in `game_flow.md` and "the model checker" on the pages. Findings 1 to 3 (a router section to the specs) are rejected (owner): the agent searches the repo, and `docs/AGENTS.md` is always loaded.
- **`DOC:` anchors.** 22 modules point at the doc that names their area (`storage.md`, `frontend/dashboard.md`, `frontend/http_routes.md`, `architecture_system.md`). 26 stay on `game_flow.md`.
- **Document References.** 13 links in 7 docs broke the rule of `docs/AGENTS.md` (only the paired doc and a doc that the body hands a point to), and they are gone.
- **The busy refusal before the other checks.** It had no spec and no TLA+ model step. New scenarios: story_log.md 8.18 (an edit of no Message, a delete of a Message that is not the last) and swipe_switch.md 36.8 (an index past the last Swipe, a Message that is not the last), each with a test in `tests/http/races.rs`. The setup of 36.3 is now the helper `held_job_on_two_swipes`, shared with 36.8. No code change: `GameWriter::require_idle` already runs first.
