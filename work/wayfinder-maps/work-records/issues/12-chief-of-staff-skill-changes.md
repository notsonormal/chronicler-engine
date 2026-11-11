# Decide the local changes to the chief-of-staff skill

Type: grilling
Status: open
Blocked by: 10

## Question

The repo brings in Pocock's chief-of-staff skill as a local skill, `.agents/skills/chief-of-staff/`, adapted from upstream (owner, 2026-11-12). Which changes does the local copy make, and what is the reason from this repo for each one?

## Context

- **The owner's choices (2026-11-12, after ticket 10).** The skill was never installed, so no session can load it, and the tracker doc cites a skill that is not there. The owner chose a ticket on this map, not the later chief-of-staff map. The owner chose a local adapted skill over an unchanged install through `skills-lock.json`. The cost is a manual merge whenever upstream changes.
- **Upstream.** `mattpocock/skills@4ae2317`, `skills/in-progress/chief-of-staff/SKILL.md` (75 lines) and `agents/openai.yaml`. Draft PR #1369 may make it replace wayfinder.
- **The rule of ticket 05.** [Decide what a wiki holds and what it never holds](05-wiki-contents.md): a change to the skill needs a reason from this repo, and it must say that it is a change. Its Wiki section stays as written.
- **Candidate changes (not decided).**
  - The wiki is `work/wiki/`, and the chief reads `docs/agents/issue-tracker.md` for the records.
  - Subagents run through the `delegate` roles of this harness.
  - Large work goes through `/wayfinder`, because `to-spec` and `to-tickets` are not installed. `../retro/` and `../implement-spec/` exist here.
  - How the copy records its upstream commit and its list of changes, so a later merge can find them.
- **Out of this ticket.** Where a chief's goal lives (the prompt alone, or a dated wiki page as in PR #1369), and how a chief asks fewer questions. Those stay with the later chief-of-staff map.
