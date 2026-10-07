# Take the quarantine count out of parallel-ticket conflicts

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

[Split Settings into Connections and Text Check sub-tabs](../dashboard-ui-review/issues/68-split-settings-sub-tabs.md) and [Remove the story-log ✓ and `POST /check-text`](../dashboard-ui-review/issues/70-remove-story-log-check.md) both lowered the same pin, `REQUIRES_MIGRATION_TEST_COUNT`, from 82 to 80 and 82 to 78. The two edits conflict by construction, and the coordinator resolved them by hand to 76.

The check is a one-way ratchet: `ratchet_exceeded = quarantine > REQUIRES_MIGRATION_TEST_COUNT`. A count below the pin passes, so lowering the pin is housekeeping, not a gate requirement. The conflict is avoidable.

How should the pin be maintained?

- **Add `--update-pin`.** One command recomputes and rewrites the pin after a merge.
- **Make lowering coordinator-owned.** The map Notes state that a per-ticket change leaves the pin alone; the coordinator lowers it once after a merge.
- **Derive the pin.** Replace the literal with a count computed at check time, and keep the ratchet as a recorded baseline.

## Context

- `scripts/validate_feature_spec.py:65` — "Pins the requires_migration quarantine: the count may only go down."
- `scripts/validate_feature_spec.py:68` — `REQUIRES_MIGRATION_TEST_COUNT = 76`.
- `scripts/validate_feature_spec.py:255` — `ratchet_exceeded = quarantine > REQUIRES_MIGRATION_TEST_COUNT`.
- Tickets 68 and 70 ran in parallel in worktrees `/workspace/wt68` and `/workspace/wt70`.
