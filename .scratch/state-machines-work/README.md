# State machines: work tickets

This folder has no map. It holds the work that the [state-machines-next](../state-machines-next/map.md) map hands off:

- **Plain defects** that the sweep of implicit state machines found. A plain defect needs no decision, so it comes here at once.
- **Work tickets** for each disposition other than "leave it": make a machine explicit in the code, give it a TLA+ model and a verify skill, or generate its diagram. Ticket 06 of that map files them after the owner accepts them.

Tickets are `issues/NN-<slug>.md`, numbered from `01`. Each has a `Status:` line from `docs/agents/triage-labels.md`. The work-records map can move this folder later.
