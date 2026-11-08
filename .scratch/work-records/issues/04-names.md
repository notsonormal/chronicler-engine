# Decide the names of the work records

Type: grilling
Status: resolved
Blocked by: 01, 02, 03

## Question

What are the names of: the folder of one long goal, the folder of everything with no goal, a one-off item with no map, and the top folder that replaces `.scratch`? Does the map's own name, "Work records", stay?

## Context

- **Research (tickets 01 to 03).** Read the `## Answer` of [01](01-claude-projects.md), [02](02-chief-of-staff-wiki.md) and [03](03-other-layouts.md) first. Each asset ends with "Facts for tickets 04 to 08". Zoom into the asset only for the detail.
- **Working names (owner, 2026-11-04).** Expedition, errand and `general`. The owner chose `general` over "base camp". The top folder is visible, with no dot.
- **The rule from charting (proposal).** Name each folder for what it holds, as `maps/` and `errands/` are.
- **Candidates for the top folder (charting).** `expeditions/` (it names what it holds), `work/` (plain; the mrn-wiki uses `work/` for in-flight efforts), and `tracker/` (the role in software-factory terms; but a wiki is not a tracker item).
- **Words that are taken.** See [the charting facts](../assets/charting-facts.md), "The words": "project" and "goal" are common in prose, `CONTEXT.md` avoids "campaign", and "task", "job" and "brief" already have meanings here.
- The research tickets may show names that other tools use. A name that a well-known tool uses for the same thing is a point in its favour.

## Answer

Grilled with the owner on 2026-11-08. The rule behind every name: a name says what the thing is, means one thing in this repo, is known to a fresh agent, and is not a word of the game or its story. The owner overturned "expedition" because its only basis was the wayfinder travel image.

| Thing | Name | Why |
|---|---|---|
| The folder of one long goal | **initiative** | It has no use in the repo. It has an end, and outside the repo it means a goal that holds several smaller pieces of work (Jira puts it above epics). |
| The folder with no goal | **`general`** | It has the same subfolders as an initiative, but it is **not** an initiative. It has no destination and never closes. Its wiki holds the rules for all initiatives. |
| A one-off item with no map | **issue**, with no new term | Upstream wayfinder says "every map and ticket is an issue", and the tracker doc and the triage skills say "issue". A one-off is an issue directly under an initiative or `general`, in `issues/`. A ticket is a child issue in `wayfinder-maps/<map>/issues/`. The location tells which is which. |
| The top folder that replaces `.scratch` | **`work/`** | It names what it holds. The owner's mrn-wiki uses `work/` for in-flight efforts. |
| The memory folder | **`wiki/`** | Karpathy's "LLM Wiki" gist and the chief-of-staff skill use the word. |
| The folder of maps | **`wayfinder-maps/`** | "Map" is a game word: `CONTEXT.md` ("a place on the map of a World"), `src/domain/model/map.rs` and the `maps` table. Prose keeps "map", and says "wayfinder map" only where the game map could be meant. `map.md` keeps its name. |
| "Work records", "the needs" | no defined terms | Docs say "the `work/` folder". This map keeps its title. "The needs" stays this map's own word. |

The shape these names give (layout is ticket 06, placement is ticket 07):

```
work/
├── general/          wiki/  wayfinder-maps/  issues/
├── <initiative>/     wiki/  wayfinder-maps/  issues/
└── _closed/
```

Rejected:

- **expedition**: no outside meaning, and its only basis was the wayfinder image.
- **project**: 540 uses in `.scratch/` prose, and it also means a repository.
- **goal**: 3,062 uses. It stays the prose word for what an initiative pursues.
- **epic**: its outside meaning fits one map, not a goal with several maps.
- **mission**: OpenRig and Factory use it.
- **effort**: the tracker doc uses `.scratch/<effort>/` for one map's folder.
- **errand**: not a tracker word, and a word of the game's story text in trial evidence.
- **chore**, **item**: too narrow and too vague.
- **`initiatives/`**: `general` and `_closed/` are not initiatives.
- **`tracker/`**: a wiki is not a tracker item.
- **`records/`**: "record" is a game term (**Event record**).
- **`memory/`**, **`notes/`**: Claude Code's model-written store, and too vague.

Facts for later tickets:

- **06.** The repo root has a tracked symlink `plans -> docs/plans`. The plans move must handle it.
- **08.** A ticket uses `Status:` `open`, `claimed` and `resolved`. A one-off issue uses the triage statuses. The tracker doc must say which set applies where.
