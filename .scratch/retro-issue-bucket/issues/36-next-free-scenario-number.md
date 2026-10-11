# Show the next free scenario number for a spec

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

To add Scenarios 24.15–24.19, the implementer had to learn that 24.3, 24.8 and 24.9 are retired. They moved to `browser_options.md` (26.1–26.3) and must not be reused. It took four `git log -S` calls and a read of `scripts/validate_feature_spec.py` to find this. No file records retired numbers.

How should an agent find the next free number?

- **`validate_feature_spec.py --next <spec>`.** It prints the highest declared number per section and the next free one.
- **A retired-numbers line in each spec.** For example `Retired: 24.3, 24.8, 24.9 (moved to browser_options.md)`. The script would refuse a reuse.
- **Do nothing.** Taking the highest number plus one is already safe if the rule is "never reuse". The cost is a few calls.

## Context

- Source: `/retro` 2026-10-11, implementer session `01a12ac6-5809` (entries around 11:45:09–11:45:22).
