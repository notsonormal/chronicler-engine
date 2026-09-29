# Decide whether duplicate connection and preset names are allowed

Type: task (AFK)
Status: open
Blocked by: —

## Question

Adding two connections with the identical name creates two identical-looking cards (only the model line distinguishes them), and adding a preset with an existing name is equally accepted. Should duplicate names be rejected on create, or accepted with a visible disambiguator? Which of the two surfaces do it?

## Context

- Finding 05.F4 (P2). DOM probe after a duplicate add: two cards titled `QA T05 Conn A` with details `qa/throwaway-a` and `qa/throwaway-a2`; screenshot 57; preset duplicate on screenshot 50; ticket 05 answer.
- Connection ids are timestamp-derived (`conn-<millis>`), so nothing breaks technically — the cost is telling two cards apart, including which one holds the Narrator/Quantifier role.
- Related: [Decide one save model for panel forms](16-decide-save-model.md) and the Games copy sweep (21) if the decision is a naming/display convention.

## Done when

- The decision is in the ticket answer, and the implementation either lands here or graduates into a new ticket.
- `python build.py` is green. Commit after user approval.
