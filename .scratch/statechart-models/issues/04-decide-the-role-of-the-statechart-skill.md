# Decide the role of the statechart skill in this repo

Type: grilling
Status: open
Blocked by: 02, 03

## Question

Which role does the statechart skill get in this repo?

| Role | What stays in the repo | What enforces it |
|---|---|---|
| Drop it | nothing; remove `.agents/skills/statechart/` | — |
| Ad-hoc design aid | the skill only; TLA+ models stay in `tmp/` | nothing |
| Committed TLA+ models with paired rule ids | TLA+ models next to the docs they state, and rule ids in doc sentences | a validator that pairs each rule id with an invariant |
| Test generator only | the skill, and the tests each break yields | the existing `SCENARIO` and `CASE` validators |
| Trace validation (added by ticket 02) | the TLA+ model, a Trace spec, a transition log in `BusyRegistry::hold` behind the `testing` feature | TLC replays the logs of the tier-1 tests in a build step; needs a JDK in the build |
| One-time audit (added by ticket 03) | the skill and a short repo-local addendum to it; a new spec-only TLA+ model in `tmp/` for each audit; the findings go to the specs and the tests | nothing; a trigger starts an audit, for example a new flow or a change to the gate |
| Committed TLA+ models paired with spec scenario ids (added by ticket 03) | TLA+ models; each invariant names the spec scenario that states its rule | a validator that pairs each invariant with a scenario, as `validate_feature_spec.py` pairs tests with scenarios; no rule ids in doc sentences |

Decide with the yield of [Prototype: one TLA+ model of the gate, built from the spec only](03-prototype-spec-only-gate-model.md) in hand, and with the methods and costs of [Research: how other teams tie a TLA+ model to code](02-research-tying-a-tla-model-to-code.md). A role that ticket 02 shows as better may replace a row of this table.

## Context

- The owner's question (map Notes): can a TLA+ model find gaps between the spec and the code, or tie the docs to the code, or is it only a mechanical check of the code?
- The base rate so far (known): three TLA+ models written after a read of the code, no code defect, one break that `game_flow.md` already accepts.
- The cost so far (known): a JDK of about 210 MB and `tla2tools.jar`, not on this host by default; one session for the three charts.
- The decision graduates fog on the map: where a rule of a TLA+ model pairs, the toolchain, a repo-local addendum to the skill, the trigger for a new audit, and the provenance of the vendored skill. Write which of them the chosen role needs.
- Ticket 02 ranks the rows for this repo: paired rule ids with one tier-1 test per rule first, trace validation second as the upgrade path, with a named trigger for the move. Its open points belong here: the gate only, or the gate plus the Autoplay loop; whether a JDK in the build is acceptable at all; where the rule ids live.
- Ticket 03 answered (see its Answer): no code defect; three "own id" rules that the code holds and the spec holds in part; six candidate tests; 5 of 15 rules true by construction; every break in fewer than 4,000 states, and nothing more in the full check of 15,798,130 states.
- **The owner's leanings (2026-10-30), not a decision:**
  - `game_flow.md` need not cover every use case. The specs hold the event rules, and the docs give an understanding. So a role that adds rule ids or rules to `game_flow.md` goes against this leaning.
  - The TLA+ model is used to pick up gaps between the specs, the docs and the code.
  - A TLA+ model in `tmp/` is written again for each audit. A reused TLA+ model goes stale with no signal, and a new spec-only one stays independent of the code.
- Use `/grilling` and `/domain-modeling`, one question at a time. The owner answers for themselves.
