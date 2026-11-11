# Decide the local changes to the chief-of-staff skill

Type: grilling
Status: resolved
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

## Answer

Grilled with the owner on 2026-11-12. The local copy is `mattpocock/skills@4ae2317`, `skills/in-progress/chief-of-staff/SKILL.md`, with the changes below. Every other part stays as upstream wrote it: the frontmatter, the opening, Subagents, Schedules, the rest of the Wiki section, the Retro procedure and Strategic View.

**The changes.**

1. **The wiki location (Wiki section).** Upstream's "Look for a shared wiki first … Only if there is none, agree a location with the user before the first write" becomes one sentence: "The shared wiki is `work/wiki/`. `docs/agents/issue-tracker.md` describes it and the other work records." The reason: need 4 (a chief finds its memory in the one wiki) and need 6 (one doc finds any record). This repo has already answered the question that upstream tells the chief to ask.
2. **Large chunks of work (Procedures).** "write a spec with `../to-spec/SKILL.md`, break it into tickets with `../to-tickets/SKILL.md` …" becomes: chart a map with `../wayfinder/SKILL.md`, then implement its build tickets with `../implement-spec/SKILL.md`. The reasons: `to-spec` and `to-tickets` are not installed here. Large work in this repo goes through wayfinder (17 maps), and the local `implement-spec` was already adapted to wayfinder tickets (the coordinator owns `map.md`). Draft PR #1369 moves upstream the same way. The owner rejected installing `to-spec` and `to-tickets`, and rejected keeping both routes.
3. **No `agents/openai.yaml`.** It is Codex metadata, and pi does not read it. `disable-model-invocation: true` does the same job.

**Unchanged on purpose.**

- **Subagents.** "All work should be done in subagents" stays. The root `AGENTS.md` lets "a skill that runs several subagents" keep its own split, so the two rules do not conflict. The `delegate` roles reach the chief through `AGENTS.md`, and restating them would put the rule in two places (need 3).
- **Schedules** and the Lint sentence "Suggest this as a schedule". Pi has no recurring schedules (inferred from its docs), and "Harness-permitting" already covers that. A substitute rule would be a change with no reason from use (the rule of ticket 05).
- **Retro.** `../retro/SKILL.md` exists. The local retro differs from upstream only in its step 4, which the chief replaces anyway.

**Where the upstream record lives.** Only here, in this answer: the source path, the commit and the list above. The skill folder holds no `UPSTREAM.md` and no comment. The owner chose this over a file beside `SKILL.md`. A later merge starts from this ticket.
