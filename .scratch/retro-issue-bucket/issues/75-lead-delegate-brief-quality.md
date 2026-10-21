# Check a delegate brief for unresolved references and stale premises

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

Three briefs of the lead in this run cost their agents time. A fix row said "like the sibling role-health mapping" and named no file or symbol: about 1.5 minutes of guessing. A scout brief stated two premises that commits in the range had already made false: an `#[ignore]` anchor path and four `MockBackend` knobs. A brief asked for a consistency check that the skill `/code-consistency-check` covers, and did not name the skill. Is this a rule for the lead, and where does it live?

- **A short checklist in the delegate role docs or the after-plan workflow**: each reference names `file` and symbol; each premise was checked in the tree in this session; name the skill that the task matches.
- **No rule**: judgement, caught by the delegate's report.

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- Implementer `eaa0aa4a` thinking at 73; scout `699961ee` final message (two premises not confirmed); scout `e3690c86` Part B.

## Comments

- 2026-10-21, triage pass 2: Tickets [33](33-check-ui-names-in-ticket-copy.md) and [56](56-ticket-edit-lists-from-the-tree.md) folded in: check named UI controls in the tree; build each edit list from `git diff --stat` or `rg`. Note: 11's "scout has no shell" premise is stale (`.pi/agents/scout.md` grants `bash`).
