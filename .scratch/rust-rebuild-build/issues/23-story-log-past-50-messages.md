# Decide what the story log shows past 50 Messages

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

Past 50 Messages the story log shows the 50 oldest (`take(MAX_LOG_DISPLAY)`), so the newest Messages are hidden and the last-entry controls act on a hidden Message (UI F1, P1; older than the rebuild). What does the story log show for a long story: the newest 50, all of them, or a window with a way to see earlier Messages? And where does that rule live (candidate S3: one story-window read, with the Retrigger rule from the `Tail`)? The answer adds the spec scenarios.

## Context

- Inputs: [the UI findings](../assets/ui-review/findings.md) F1 (a click deleted hidden Narration 67); [synthesis.md](../assets/architecture-review/synthesis.md) row S3 and §4 bug 2; FINDINGS "Rejected" TN-L4.
- Specs: `docs/specs/story_log.md`, `docs/specs/browser_story_log.md`. Skills: `/grilling`, `/domain-modeling`.
- From ticket 19 (resolved): the Retrigger button reads `Tail::retrigger_allowed` (R-65, an Impersonate Input included) and adds no condition of its own.

## Answer

### Q1: what the story log shows past 50 Messages (owner, 2026-10-20)

Option A: the story log shows the newest 50 Messages. Earlier Messages are not in the dashboard. Rejected: B (all Messages: each 2 s poll renders the whole story) and C (a "Show earlier" control: the poll must keep the larger window). The long-term answer (show earlier, paging, a separate history page, a tail-only poll) is a separate investigation: [docs/plans/story-log-earlier-messages-investigation-plan.md](../../../docs/plans/story-log-earlier-messages-investigation-plan.md). It is out of scope for this map.

### Q2: where the story-window rule lives (owner, 2026-10-20)

Option b, candidate S3: one story-window read. `GameViewQuery::story_window(game)` owns the size (50). In one storage read it returns the newest 50 entries, oldest first, and the `Tail` facts: the id of the last Message and `retrigger_allowed`. The HTTP adapter only renders: `AppState::render_story_log` loses `MAX_LOG_DISPLAY` and `take`. The one read closes a gap of the two reads of today (`entries` then `tail`): a Job could save a Message between them. The template keeps `loop.last` for the last-entry controls; with the newest window the last entry is the `Tail` by construction, and scenario 8.11 pins it. Rejected: a (fix in place; the size stays in HTTP and the gap stays) and c (a domain `StoryWindow` type; it holds only the size today).

### Q3: the term and the spec scenarios (owner, 2026-10-20)

`CONTEXT.md` has the new term **Story window** (added). The build task adds these two scenarios to `docs/specs/story_log.md` in the same commit as their tests in `tests/http/story_log.rs`. They are not in the spec yet: `validate_feature_spec.py` fails the gate on a scenario with no test.

```gherkin
#### Scenario 8.11: The story log of a long story ends at the last Message

Given a Game whose story holds 53 Messages
When the client GET /fragment/story-log
Then the response renders exactly 50 .log-entry
And the last .log-entry is the last stored Message
And the first .log-entry is the fourth stored Message
And the Delete and Retry controls sit on the last .log-entry

#### Scenario 8.12: Delete-last in a long story removes the entry the player sees

Given a Game whose story holds 53 Messages
And the client GET /fragment/story-log
When the client POST /history/delete
Then the response status is "200 OK"
And the client GET /fragment/story-log renders exactly 50 .log-entry
And the last .log-entry is the stored Message before the deleted one
And no .log-entry holds the deleted Message
And the first .log-entry is the third stored Message
```

No new browser scenario: 30.14 and 30.23 cover the browser side once the server sends the newest window. No separate long-story Retrigger scenario: the flag comes from the same read, and 8.11 pins the entry that carries the controls.
