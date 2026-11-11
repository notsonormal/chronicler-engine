# Path moves of the work records

The move to `work/` (ticket 10) moved each path in the left column to the path in the right column. A folder moved whole, with its files. `move_to_work.py --reverse` reads this table.

| Before | After |
|---|---|
| `.scratch/_closed` | `work/_closed` |
| `.scratch/autoplay-follow-ups/issues/01-per-tab-games.md` | `work/issues/pending/01-per-tab-games.md` |
| `.scratch/autoplay-follow-ups/issues/02-narrator-role-health-after-a-stop.md` | `work/issues/pending/02-narrator-role-health-after-a-stop.md` |
| `.scratch/autoplay-follow-ups/issues/03-autoplay-runs-history-table.md` | `work/issues/pending/03-autoplay-runs-history-table.md` |
| `.scratch/autoplay-follow-ups/issues/04-rename-form-survives-the-games-panel-poll.md` | `work/issues/closed/04-rename-form-survives-the-games-panel-poll.md` |
| `.scratch/character-art-from-code` | `work/wayfinder-maps/character-art-from-code` |
| `.scratch/code-concepts` | `work/wayfinder-maps/code-concepts` |
| `.scratch/conceptual-integrity` | `work/wayfinder-maps/conceptual-integrity` |
| `.scratch/dashboard-design` | `work/wayfinder-maps/dashboard-design` |
| `.scratch/fable-audit` | `work/wayfinder-maps/fable-audit` |
| `.scratch/invariant-audit` | `work/wayfinder-maps/invariant-audit` |
| `.scratch/layout-trials` | `work/wayfinder-maps/layout-trials` |
| `.scratch/long-term-memory-and-state` | `work/wayfinder-maps/long-term-memory-and-state` |
| `.scratch/metacognitive-audit` | `work/wayfinder-maps/metacognitive-audit` |
| `.scratch/pi-explain` | `work/wayfinder-maps/pi-explain` |
| `.scratch/prompt-flexibility` | `work/wayfinder-maps/prompt-flexibility` |
| `.scratch/retro-issue-bucket` | `work/wayfinder-maps/retro-issue-bucket` |
| `.scratch/state-machines-next` | `work/wayfinder-maps/state-machines-next` |
| `.scratch/state-machines-work/issues/01-bug-added-room-id-from-whole-seconds.md` | `work/issues/pending/05-bug-added-room-id-from-whole-seconds.md` |
| `.scratch/state-machines-work/issues/02-bug-overflow-guard-asks-for-a-one-token-answer.md` | `work/issues/pending/06-bug-overflow-guard-asks-for-a-one-token-answer.md` |
| `.scratch/state-machines-work/issues/03-bug-times-met-counts-every-entry.md` | `work/issues/pending/07-bug-times-met-counts-every-entry.md` |
| `.scratch/state-machines-work/issues/04-bug-world-update-writes-the-map-outside-the-transaction.md` | `work/issues/pending/08-bug-world-update-writes-the-map-outside-the-transaction.md` |
| `.scratch/state-machines-work/issues/05-bug-a-world-without-its-map-stops-the-seeding.md` | `work/issues/pending/09-bug-a-world-without-its-map-stops-the-seeding.md` |
| `.scratch/state-machines-work/issues/06-bug-games-list-order-docs-disagree.md` | `work/issues/pending/10-bug-games-list-order-docs-disagree.md` |
| `.scratch/state-machines-work/issues/07-bug-preview-send-loses-its-draft-game.md` | `work/issues/pending/11-bug-preview-send-loses-its-draft-game.md` |
| `.scratch/state-machines-work/issues/08-bug-failed-job-with-no-failure-line.md` | `work/issues/pending/12-bug-failed-job-with-no-failure-line.md` |
| `.scratch/state-machines-work/issues/09-question-accept-keeps-the-options-failure.md` | `work/issues/pending/13-question-accept-keeps-the-options-failure.md` |
| `.scratch/state-machines-work/issues/10-bug-trigger-choice-follows-hash-order.md` | `work/issues/pending/14-bug-trigger-choice-follows-hash-order.md` |
| `.scratch/state-machines-work/issues/11-bug-a-refused-accept-on-a-deleted-game-raises-its-counters.md` | `work/issues/pending/15-bug-a-refused-accept-on-a-deleted-game-raises-its-counters.md` |
| `.scratch/state-machines-work/issues/20-bug-world-posture-and-detail-saves-can-overwrite-each-other.md` | `work/issues/pending/16-bug-world-posture-and-detail-saves-can-overwrite-each-other.md` |
| `.scratch/state-machines-work/issues/30-bug-dashboard-doc-omits-forget-points-priority-and-client-state.md` | `work/issues/pending/17-bug-dashboard-doc-omits-forget-points-priority-and-client-state.md` |
| `.scratch/state-machines-work/issues/31-bug-a-refused-request-ends-the-pending-state-of-an-accepted-send.md` | `work/issues/pending/18-bug-a-refused-request-ends-the-pending-state-of-an-accepted-send.md` |
| `.scratch/statechart-models` | `work/wayfinder-maps/statechart-models` |
| `.scratch/story-scroll-back/idea.md` | `work/issues/pending/19-story-scroll-back.md` |
| `.scratch/unclebob-quality-tools` | `work/wayfinder-maps/unclebob-quality-tools` |
| `.scratch/weekly-retro-2026-11-04` | `work/wayfinder-maps/weekly-retro-2026-11-04` |
| `.scratch/work-records` | `work/wayfinder-maps/work-records` |
| `docs/plans/drain-requires-migration-quarantine.md` | `work/issues/pending/20-drain-requires-migration-quarantine.md` |
| `docs/plans/narration-line-break-tags.md` | `work/issues/pending/21-narration-line-break-tags.md` |
| `docs/plans/parallel-ticket-runs-without-commits.md` | `work/issues/pending/22-parallel-ticket-runs-without-commits.md` |
| `docs/plans/story-log-earlier-messages-investigation-plan.md` | `work/issues/pending/23-story-log-earlier-messages-investigation-plan.md` |

## Removed

The move dropped these files. Git history keeps them.

- `.scratch/autoplay-follow-ups/map.md`
- `.scratch/state-machines-work/README.md`
- `docs/plans/` and the `plans` symlink at the repo root
