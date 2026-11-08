# Find how Pocock's chief of staff and the LLM wiki keep memory

Type: research
Status: resolved
Blocked by: —

## Question

How does Pocock's chief of staff keep its memory? What is the LLM wiki, how is a wiki scoped (one goal, one user, or shared), and who writes and reads it? What has changed in the skill since `mattpocock/skills@4ae2317`? Which parts meet [the needs](../map.md#notes)?

## Context

- The owner named Pocock's chief of staff and its LLM wiki as the second model for this map (2026-11-04). The owner's reading: a base chief of staff, with goal folders for it to work in, each with its own wiki.
- The skill itself (`skills/in-progress/chief-of-staff/SKILL.md` at `4ae2317`) pursues one goal per session. It says: "Look for a shared wiki first… Only if there is none, agree a location with the user before the first write." It defines no goal folders.
- The mrn-wiki already covers the skill: `concepts/chief-of-staff-pattern.md`, `concepts/repo-as-memory.md` and `concepts/wayfinder-decision-map.md`. Start from them, and do not redo them. Clone details are in the map's Notes.

Points to cover:

1. **The LLM wiki.** Where the pattern comes from (check the source; do not assume), its operations (ingest, query, lint), its index, and "write history, not the present".
2. **Scope.** What the skill says, and what Pocock does in practice, from his repos, posts or talks. Label each claim known or inferred.
3. **The tracker.** How the chief relates to wayfinder maps and to the issue tracker (GitHub Issues upstream).
4. **Changes.** The commits to `skills/in-progress/` and `skills/engineering/wayfinder/` after `4ae2317`.
5. **The history voice.** How a wiki of dated claims avoids the failure of the 33 ADRs that this repo deleted on 2026-08-10 (`e0f22e00`). The owner's account is in `entities/chronicler-engine.md` of the mrn-wiki. This point feeds [Decide what a wiki holds and what it never holds](05-wiki-contents.md).
6. **The needs.** A table: the chief of staff against each of the six needs.

Write the result to `assets/chief-of-staff-wiki.md`, and link it from the answer.

## Answer

Asset: [How Pocock's chief of staff and the LLM wiki keep memory](../assets/chief-of-staff-wiki.md). It holds the origin of the LLM wiki, the scope of the skill against Pocock's own posts, the tracker link, the changes after `4ae2317`, a table of mechanisms against a stale read, the needs table, and the facts for tickets 04 to 08. Access date of every web source: 2026-11-05.

- **known** Karpathy's gist "LLM Wiki" (2026-04-04) is the origin. It has three layers (raw sources, the wiki, the schema), three operations (ingest, query, lint), an `index.md` and an append-only `log.md`. Its voice is a wiki "kept current".
- **known** "Write history, not the present" is Pocock's rule (commit `5beb35b`, 2026-10-19), not Karpathy's. The skill names no `log.md` and no schema file. I found no source where Pocock says why he changed the voice.
- **known** The skill defines no goal folders and no scope rule. It says to look for a shared wiki first, and to agree a location with the user only if there is none.
- **known** Pocock's posts show three scopes: one personal wiki (2026-06-29 to 2026-10-19), one wiki for each deliverable (2026-09-09), and one wiki for each long-running domain agent (2026-11-03). An agent writes each of them.
- **inferred** The scope follows the unit of work. No post shows one chief with several goal wikis, or goal folders inside one wiki. The public skill is not the whole practice, because PR #1387 moved a rule to a private skill.
- **known** The skill uses the tracker only for spec and tickets of large work. Its memory is not in the tracker. It does not name wayfinder.
- **known** Draft PR #1369 (2026-11-01, open): "Chief-of-staff may replace wayfinder, because the wiki outlives the session where a closed wayfinder map doesn't." It moves the destination, the fog and the out-of-scope list into a `destination.md` wiki page.
- **known** No commit to `main` changes the skill or wayfinder after `4ae2317`. Two pull requests exist and neither is merged: #1369 (open draft) and #1387 (closed unmerged on 2026-11-05).
- **known** The skill gives these mechanisms against a stale read: a date on each claim, a commit pin on a claim about code, no present tense about code, supersede and never rewrite, and a lint on a schedule. The skill has no script that checks any of them.
- **known** Of the six needs, the chief of staff fits need 3 (rules in one place) in one wiki. It fits needs 1, 4 and 6 in part, and does not fit need 2 (one-off items) or need 5 (close a goal as one unit).
- **known** Upstream moves from naming the tracker folder to pointing at the tracker doc (changeset #1208). `to-tickets` and the local tracker template still name `.scratch/` at `4ae2317`.
- **I don't know** Whether Pocock's own wikis were ever read as current state by mistake, whether his personal wiki has goal folders, and whether his domain agents run the `chief-of-staff` skill.

