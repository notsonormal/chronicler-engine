# Synthesize the three architecture reviews of the rebuilt code

Type: research
Status: open
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
