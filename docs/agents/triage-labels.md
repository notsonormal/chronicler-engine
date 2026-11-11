# Triage Labels

The skills speak in terms of five canonical triage roles. This file maps each role to the status folder of a one-off issue (see `issue-tracker.md`).

| Role                  | Folder        | Meaning                                   |
| --------------------- | ------------- | ----------------------------------------- |
| `needs-triage`        | `pending/`    | Maintainer needs to evaluate              |
| `needs-info`          | `pending/`    | Waiting on reporter for more information  |
| `ready-for-agent`     | `ready/`      | Fully specified, ready for an AFK agent   |
| `ready-for-human`     | `ready/`      | Requires human implementation             |
| `wontfix`             | `closed/`     | Will not be actioned                      |

Two states have no triage role. Work that has started is in `in-progress/`. A finished issue is in `closed/`, beside the `wontfix` issues.

When a skill mentions a role (e.g. "apply the AFK-ready triage status"), move the issue to the folder of that role with `python scripts/move_issue.py <NN> <pending|ready|in-progress|closed>`.
