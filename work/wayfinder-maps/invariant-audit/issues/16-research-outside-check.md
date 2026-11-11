# Outside check: run the six-theme audit again

Type: research (AFK)
Status: open
Blocked by: 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15, and every task ticket that the grilling tickets graduate

## Question

After every verdict is in place, does the code match the verdicts? Are there new invariants that nothing enforces?

1. Run the six scouts again with [the scout brief](../assets/scout-brief.md), against the commit after the last task. Give each scout the list of findings of its theme and their verdicts from this map. Each scout confirms one of these for each finding: "enforced, fails loudly at the named point", "the reliance is removed", "accepted, and the reason is where the verdict says", or "does not match the verdict". The scouts must be agents that made no fix in this map (owner, charting, Q4 Option C).
2. Read the code of each finding that does not match, and of a sample of the others.
3. Count the new invariants by tier, as in [the report](../assets/REPORT.md).

## Context

- The asset of this ticket is a report like `assets/REPORT.md`: one row for each of the 53 findings, with its verdict and the result of the check, and a list of new findings.
- Where the new findings go is in "Not yet specified" on the map. This ticket only reports them.
- Use the role `scout` for the six scouts and the role `reviewer` for step 2.

## Answer
