# Decide the memory view of the dashboard

Type: prototype (HITL)
Status: open
Blocked by: 17, 20

## Question

How does a person read, edit and keep a memory entry in the dashboard? The answer is a rough prototype that the owner reacts to, made with `/impeccable` and `/prototype`, and the decisions a build ticket needs.

## Context

- **The rule** (map Notes, "The memory in principle"). The oldest memory entry leaves the prompt when the list passes its space, and it stays stored. A person can keep or edit an entry in the memory view.
- **What exists** ([Build the scene entries](17-build-scene-entries.md), Answer). The entries are rows of the table `memory_entries`, each tied to the Swipe whose Job wrote it. An entry never changes. The prompt drops the oldest entries first. No dashboard part shows them yet. Their write raises the story change kind.
- **The size** (same Answer). At Cycle 100 of the check trial, the list held 36 entries (about 8,000 tokens). The prompt showed 34 of them.
- **Questions to settle.** These are suspected, and the prototype sharpens them:
  - where the view lives in the dashboard;
  - what an edit is when an entry "never changes", for example a new version tied to the person's edit;
  - what "keep" does to the space of the list;
  - how an entry shows that it left the prompt;
  - what an edit does on a Retry or a swipe switch.
- Read `docs/diataxis/reference/frontend/dashboard.md` and `docs/diataxis/explanation/dashboard_design.md` first (map Notes, "UI").
