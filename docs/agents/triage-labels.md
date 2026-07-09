# Triage Labels

The skills speak in terms of five canonical triage roles. This file maps those roles to the `Status:` line values used in `.scratch/` ticket files.

| Role                  | Status line value      | Meaning                                   |
| --------------------- | ---------------------- | ----------------------------------------- |
| `needs-triage`        | `Status: needs-triage` | Maintainer needs to evaluate              |
| `needs-info`          | `Status: needs-info`   | Waiting on reporter for more information  |
| `ready-for-agent`     | `Status: ready-for-agent` | Fully specified, ready for an AFK agent |
| `ready-for-human`     | `Status: ready-for-human` | Requires human implementation           |
| `wontfix`             | `Status: wontfix`      | Will not be actioned                      |

When a skill mentions a role (e.g. "apply the AFK-ready triage status"), write the corresponding `Status:` value from this table.
