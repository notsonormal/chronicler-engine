# Decide what a wiki holds and what it never holds

Type: grilling
Status: resolved
Blocked by: 01, 02, 03

## Question

What goes into the wiki of an initiative, and what goes into the `general` wiki? What never goes into a wiki? Which voice does a wiki use: dated history, or the current state? How does a wiki stay true?

## Context

- **Research (tickets 01 to 03).** Read the `## Answer` of [01](01-claude-projects.md), [02](02-chief-of-staff-wiki.md) and [03](03-other-layouts.md) first. Each asset ends with "Facts for tickets 04 to 08". Zoom into the asset only for the detail.
- **The tension.** The chief-of-staff skill writes history: "Date every claim", "add a new dated claim and mark the old one superseded. Don't rewrite the old one." The docs of this repo are current-state by design. The 33 ADRs were deleted on 2026-08-10 (`e0f22e00`) because they were "full of historical information that no longer matters, and then were constantly referred to for current information" (owner, recorded in the mrn-wiki page `entities/chronicler-engine.md`).
- **The owner's position (2026-11-04).** The ADRs described the whole application. A chief-of-staff wiki is local to its goal, so it is a different thing.
- **Page kinds of the skill.** "A decision, a recurring problem, a subsystem, a user preference." Charting proposed a rule: no subsystem pages, because `docs/diataxis/` holds the architecture.
- **Candidate contents (charting).** The goal, and how each map serves it. The owner's preferences, each with its date and source. The standing rules that span maps (see [the charting facts](../assets/charting-facts.md), "Rules copied between maps"). Recurring problems, such as trial noise and the judge method.
- **Overlap.** A map's Decisions so far is already an index of decisions. Say how a wiki differs from it, so that a decision lives in one place.
- The `SCHEMA.md` of the mrn-wiki is one worked example of a wiki's rules.

## Answer

Grilled with the owner on 2026-11-11.

1. **The repo has one wiki.** An initiative has no wiki of its own, and `general` has none. Separate wikis make each write a choice of which wiki, and a wrong choice hides the entry from the other initiatives. In one wiki, a page links to what it affects. This overturns the working view "each initiative and `general` has `wiki/`" and the shape in the answer of [Decide the names of the work records](04-names.md). The names do not change.
2. **The wiki follows the Wiki section of the chief-of-staff skill as written** (`mattpocock/skills@4ae2317`, `skills/in-progress/chief-of-staff/SKILL.md`). That covers its page kinds (a decision, a recurring problem, a subsystem, a user preference), `index.md`, its three operations (ingest, query, lint), the history voice ("Write history, not the present"), and the agent as the only writer.
3. **A map and a ticket are sources.** A page cites them and does not copy them, as the skill says for every source. So a decision page that links to its ticket does not give the decision a second home.
4. **The layout ticket decides the rest**: where the one wiki is, how `index.md` shows the pages of one initiative, and what `general` holds without a wiki.

The owner's reasons:

- **The history voice is not the ADR problem.** The ADRs described the whole application, and agents read them as the current state. I recommended a current-state voice on that basis, and the owner rejected it.
- **No re-engineering of the skill.** The first round proposed a "rule" page kind, a ban on subsystem pages, a "never holds" list, an approval step for rules, and a size limit. Each one changed the skill with no reason from the owner's purposes, so each one is dropped. A change to the skill needs a reason from this repo, and it must say that it is a change.
- **One wiki departs from the Claude Projects model.** That model separates the memory of unrelated projects. The initiatives here share one repo and one owner.

Known costs: a chief's memory is in the shared wiki and not in its initiative folder (need 4), and the index grows with the repo.
