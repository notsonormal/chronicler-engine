# Scan the code again with another model family, and compare with the catalogue

Type: research
Status: resolved
Blocked by: —

## Question

Which implicit state machines does a scan by another model family find when it does not see the catalogue? Which machines does only one of the two lists have? This is the outside check of the catalogue (owner, charting).

## Method

1. Run five read-only scouts, one for each area. Give each the brief and its area line from [`assets/scan-brief.md`](../assets/scan-brief.md), with no change, so the two scans can be compared. The scouts are Claude (owner, 2026-11-09). Do not give the scouts the catalogue or the first reports.
2. Compare the new lists with [the catalogue](../assets/catalogue.md). Match two machines by their state variables, not by their names.
3. For each machine that only one list has, read the code and give it one label: **missed** (a real machine), **not a machine** (plain data or a pure function), or **duplicate** (a part of another machine).
4. Count the machines that each scan found and the real machines that each scan missed.

## Output

- `assets/02-second-scan.md`: the comparison table, the labels with their evidence, and the counts.
- Add each **missed** machine to the catalogue, in its group, with a new row id.
- Add each missed machine to the open apply ticket of its area (03, 04 or 05), in its *Machines* list. If that ticket is resolved, add the machine to *Not yet specified* on the map.
- A plain defect that the scan finds goes to `work/issues/pending/`.

## Answer

Resolved 2026-11-09.

The second scan (five Claude Sonnet scouts, 2026-11-09) agrees with the catalogue on 53 of its 54 rows. It found one machine that the catalogue missed: the Game config state (narrator mode and the mode-bound preset slots), now row B12 in ticket 03. It missed one catalogue row, B8 (Swipe text against Swipe state), which is a weak machine. Its other nine extra sections are duplicates (4) or not machines (5).
Ticket 02 refuted two scan-1 claims (E3 World seeding, the G8 Test-button opener) and five scan-2 claims (the overflow guard ends as `ContextOverflow`, the edit dialog sends twice, a missing state row reports `MissingData`, Impersonate health contradicts the glossary, a second `updated_at` touch at boot in part).
Of the three high-impact inferred risks, the double send is refuted, and the status read and the detached in-flight hold are confirmed as behaviour questions.
Five plain defects went to `state-machines-work` (03 to 07). Ten behaviour questions and fifteen "to check" items went to the apply tickets 03, 04 and 05. Five doc mismatches went to "Also check" of ticket 05.
Details: [`assets/02-second-scan.md`](../assets/02-second-scan.md).
