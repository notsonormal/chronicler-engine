# Add the staleness warning of the verify skill to build.py

Type: task
Status: open
Blocked by: —

## Question

Add a check to `build.py` that warns when code that `verify-gate` covers changed after its sync commit. [Ticket 09](09-grilling-decide-the-gaps-and-the-verify-skill.md) keeps `.agents/skills/verify-gate/` (Q3).

## The decisions (ticket 04)

- **A warning, not a failure** (Q5). The owner chose a warning.
- **In the end summary of the build log** (Q14). Agents read only the last lines of the newest log (`AGENTS.md`, Development Loop: `tail -n 10`).
- **The warning names `/maintain-formal-verification-skill`**, and the agent that sees it runs the maintain pass before it ends its task (Q10). Write that rule into the warning text.
- **By function, not by file** (Q6). The coverage record of `verify-gate` (ticket 07) names functions. An edit elsewhere in a covered file gives no warning. So the check reads the diff of each function from the sync commit, not only the commit list of each file.
- **No JDK.** The check uses git and Python only.
- **Every `build.py` run shows the warning** (ticket 09, Q8), the narrow `test-pattern` loops too. The check also reads the working tree, not only `git log -L <sync>..HEAD`: agents do not commit without approval (`AGENTS.md`), so a check of commits alone shows nothing until the owner commits.

## Context

- `git log -L :<function>:<file> <sync>..HEAD` follows one function. For Rust, git finds the function bounds only with a `*.rs diff=rust` line in `.gitattributes` (inferred). `.gitattributes` has no such line today (known).
- The idiom of other checks: `scripts/validate_feature_spec.py` pairs spec scenarios with tests.
- Ticket 14 adds a second check of the same kind: a generated diagram that is older than the TLA+ model. The two checks can share code.

## Checks

- Python tests in `scripts/tests/`.
- `python build.py` (full gate) passes.
- Do not commit.
