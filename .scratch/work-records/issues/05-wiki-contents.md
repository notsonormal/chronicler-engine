# Decide what a wiki holds and what it never holds

Type: grilling
Status: open
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
