# What "new Characters without an image model" means

Type: grilling (HITL)
Status: open
Blocked by: 10

## Question

Goal 3 of the map Notes says "new Characters without an image model". Does it mean no image model at any step, or no image model for each new Expression or outfit, so that a painted picture made once by an artist or an image model is an allowed input?

## Context

- [Research: art from code with a fresh view](10-research-fresh-view.md) found that no family gives the painted look, new Characters with no image model, Expressions and clothes together ([findings](../assets/fresh-view/findings.md)). The routes that come close keep a painted picture as the pixel source and move it with data.
- Under "no image model at all", [Prototype a neural animator on the headshots](12-prototype-neural-animator.md) and [Prototype a layer split and a rig](13-prototype-layer-split-and-rig.md) leave the route: both need a painted picture for each Character and a model at bake time.
- Under "no image model for each Expression or outfit", both stay. The painted pictures in `data/images/` already exist for the current Characters.
- A related point: the routes A and B need an Nvidia GPU at bake time (Colab, a rented GPU or a Hugging Face demo). The sandbox has none (known). Ask whether an author may need a GPU.
- Use `/grilling` and `/domain-modeling`.
