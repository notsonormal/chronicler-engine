# Synthesize the three architecture reviews of the rebuilt code

Type: research
Status: resolved
Blocked by: 14, 15, 16

## Question

Three models ran `/improve-codebase-architecture` on the same tree, each without seeing the others or the whole-rebuild review (tickets 14, 15 and 16). Where do they agree, where do they differ, and how do their candidates map onto the findings of the whole-rebuild review? The answer is the input of [Decide how to respond to the review of the whole rebuild](13-respond-to-the-whole-rebuild-review.md).

- Inputs: [../assets/architecture-review/opus.md](../assets/architecture-review/opus.md), [../assets/architecture-review/glm.md](../assets/architecture-review/glm.md), [../assets/architecture-review/qwen.md](../assets/architecture-review/qwen.md), and [the review findings](../assets/whole-rebuild-review/FINDINGS.md).
- Merge candidates that name the same deepening, even under different names. Keep a candidate that only one model found, and label it so.
- Check each merged candidate against the tree: does the friction it names exist in the code? A candidate that the code does not support is listed as rejected, with the reason. Label each claim known or inferred.
- Map each merged candidate to the review findings it would remove or change (for example, review findings 3 to 6, the shape of `job.rs` and `gate.rs`), and list the review findings that no candidate touches.
- Note what each model saw that the others did not, as one short paragraph: a fact about the runs, not a ranking of the models.
- Read-only on the code.

## Done when

- [../assets/architecture-review/synthesis.md](../assets/architecture-review/synthesis.md) holds: the merged candidate table (candidate, models that found it, strength each gave, verified yes or no, review findings it touches), the rejected candidates, the review findings no candidate touches, and one recommended order of the candidates.
- The Answer gives the gist in five lines or fewer and the asset link.

## Answer

- The 18 run candidates make 6 merged candidates (M1 to M6, from 13 run candidates) and 5 single-model candidates (S1 to S5). The code supports every candidate; 5 smaller claims inside the run texts are rejected.
- Most agreement: the in-memory Storage adapter (all three runs; Opus deletes it, GLM and Qwen split the seam) and the Game rules with one room lookup (Opus and GLM).
- The candidates touch review findings 4, 5, 7, 12, 15, 17 and 19. No candidate touches findings 2 (the failure lines), 3 (the data-carrying kind) or 6 (`Job::run` and `failed`), so ticket 13 places them itself.
- The three possible bugs of the Opus run: the unrendered Retrigger direction is finding 12; the story log keeping the 50 oldest Messages is a defect that FINDINGS lists only as TN-L4 "not a regression"; the Retrigger button against `Tail::retrigger_allowed` for an Impersonate Input is new, and spec 14.3 agrees with the button. A new inferred gap: a preset delete does not check the preset slots of each Game.
- Recommended order: M4 analysis output type, M1 Game rules, S1 plan rules to the Tail with finding 3, M3 narration prompt rules, then S3, S2, M2, M5, M6, S4. Asset: [../assets/architecture-review/synthesis.md](../assets/architecture-review/synthesis.md) (session `9ffe3f17`).
