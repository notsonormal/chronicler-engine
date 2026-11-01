# Decide whether anything lands without the owner

Type: grilling (HITL)
Status: open
Blocked by: 16

## Question

May any change land without the owner's approval? If yes, which kinds of change, and with which checks?

## Context

- **Covers** C-19 (the agent lands its own small changes), with C-17 (a deterministic risk sort of a diff), C-18 (a second model as the strict reviewer) and C-50 (Labs). See `assets/candidates.md`, themes D and G.
- **The rule agreed in ticket 08.** If C-19 drops, C-17 drops with it, and C-18 counts as covered by the `code-review` skill, with its record in the work report. C-50 joins only if C-19 stays.
- **Chronicler (known).** `AGENTS.md:415` says "Don't commit without explicit approval". The permission config (`.pi/extensions/pi-permission-system/config.json`) allows every `bash` command, `git commit` included. So the written rule is the only barrier. The median commit has 377.5 changed lines, and p90 is 3049 (K-41).
- **Hex (known).**
  - 60 of 64 agent PRs merged, with a median of 3 s from open to merge (K-04).
  - `.github/agent-policy/check.ts` (118 lines, no AI) sorts each diff. The verdicts were 46 `clean`, 17 `flagged` and 1 `dax` (K-02, K-05).
  - A `flagged` diff needs the label `agent-strict-review`.
  - `.hivemind.toml` verifies, builds and installs in batches of up to 4.
  - 8 of 59 pipeline commits are `fix(ci)` repairs, and none was reverted (K-07).
  - The criteria of the strict reviewer are not public, so Chronicler would write its own.
- **Input.** The work report decision from [Draft the work report](./13-task-draft-the-work-report.md), as confirmed in [Confirm the recommendations of tickets 10 to 15 and 20](./16-grilling-confirm-the-recommendations.md). Landing without the owner needs a durable record of each change.
- **Why it waits for ticket 16** (owner, 2026-11-01). Ticket 17 needs only the work report from ticket 13. It waits for ticket 16 on purpose, so that the HITL sessions come late and together.
- **Likely end.** A yes is larger than one session, so it leaves the map as a question for a new map. A no ends in "the rule stays".
- **Skills.** `/grilling` and `/domain-modeling`.
- **When done.** Add `## Answer` and set `Status: resolved`.
