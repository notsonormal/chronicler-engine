# Make `build_slot.py --status` give one verdict

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

`build_slot.py --status` printed `Build slot: free.` and on the next line `leftover .holder is stale: ... (pid 1212472, running 182s)`. The two lines read as a contradiction. Should the status print only the flock verdict, or label the holder line?

- **Label the holder line** "informational".
- **Print the holder line only when the slot is held.**

## Context

- Source: `/retro` 2026-10-19 on the whole-rebuild review: lead session `01a15614` and its subagents.
- Test-police `d448fab2` entry 19.
