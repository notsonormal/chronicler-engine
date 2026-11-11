# Decide the runtime World validation, and the role of validate_data.py after it

Type: grilling (HITL)
Status: open
Blocked by: 01

## Question

What is the verdict for each of these seven findings? Where does one World validation live, what does it check, and what task tickets put it in place?

- **The `default_scenario_id` that nothing reads (world-01).** The field is stored, but both `default_scenario()` functions return `scenarios.first()`.
- **The Game that cannot be played (world-02).** A World with no Scenario (`scenarios_json: []`, which is the form placeholder) or with a wrong `starting_room_id` passes the World form. `starting_room_id()` then falls back to the literal `"start"`.
- **Scenario Characters that do not exist (world-03).** Only `validate_data.py` checks them, at build time.
- **A Trigger Room that does not exist (world-05).** Only `validate_data.py` checks it, at build time. The Trigger never fires.
- **A Relationship partner that does not exist (world-07).** Nothing checks it, and the relationship silently leaves the prompt.
- **A World key that breaks the URLs (http-08).** Only `UNIQUE` checks the key. A key with `/ ? # %` is stored, and after that the dashboard cannot edit or delete the World.
- **A posture value that becomes the default (http-09).** Three routes refuse a bad value. The two World write paths silently replace it with the default.

The choices to put to the owner:

1. Where the validation lives: one domain function or constructor that the seed, the World create form and the World edit form all call. It covers the World card, its map and its Characters.
2. What it checks: the list above, plus exits if [ticket 01](01-grilling-game-rules.md) makes exits matter.
3. world-01: read `default_scenario_id`, or remove the field (enforcement preference 1).
4. The allowed characters of a World key.
5. The role of `scripts/validate_data.py` after the change: keep its JSON-schema checks and drop the cross-reference checks that Rust now does; call the Rust validation from it; or keep both.
6. A World in the dev database that fails the new validation: boot fails loudly and names it (map Notes, Q5 rule), or only its new Games are refused.

## Context

- Findings in full: world-01, 02, 03, 05, 07 in [the world report](../assets/world.md), http-08 and http-09 in [the http report](../assets/http.md). The coordinator confirmed world-01 and world-02 in the code.
- Known, at `8b23151c`:
  - `WorldManifest::default_scenario` and `WorldCard::default_scenario` return `scenarios.first()`, and `starting_room_id()` falls back to `"start"` (`src/domain/model/world.rs:62-77`).
  - `scripts/validate_data.py` checks the Room of the `default_scenario_id` Scenario, Scenario Characters and Trigger Rooms for the seed files only.
- **Two more map faults for this validation** (the audit of 2026-11-10, row N4 of [its list](../../fable-audit/assets/findings-not-carried.md)). The World form accepts a map with the same Room id twice, and a map with an exit to a Room that does not exist. Both were run on a probe server, and both answered 200. That audit does not fix them: they belong to the one validation of this ticket.
- Triggers and Relationships live in the Character files (`data/characters/<group>/`), not in `world.json`. So the validation spans three kinds of file.
- [Decide and close the wiring and config group](14-task-wiring-and-config-group.md) adds boot checks of the presets. It can use the same mechanism.
- Blocked by ticket 01 for choice 2 only.
- Use `/grilling` and `/domain-modeling`. Read the seven findings in the code again first (map Notes).

## Answer
