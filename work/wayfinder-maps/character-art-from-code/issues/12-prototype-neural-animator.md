# Prototype a neural animator on the headshots

Type: prototype (HITL)
Status: open
Blocked by: 11

## Question

After a head cut-out and a 512×512 frame, can THA3 (Talking Head Anime 3) turn `data/images/lisette_headshot.png` and `data/images/gabriella_headshot.png` into five Expressions and a blink pair that a reader tells apart at 80 px, with no visible tearing?

## Context

- Family 9 of the [fresh-view findings](../assets/fresh-view/findings.md), route A. Each Expression is one parameter vector, so an agent can write the Expression data as a small JSON (inferred).
- Use THA3, not THA4: THA3 weights are CC BY 4.0, THA4 weights are CC BY-NC (known, findings).
- The sandbox has no Nvidia GPU (known). The THA3 README offers a Colab notebook. The way to get a GPU is a question for the owner in [What "new Characters without an image model" means](11-grill-what-goal-3-means.md).
- Main risk (inferred): the headshots are tilted crops, and THA3 has strict input rules. Record the work that the cut-out needs for each Character.
- Run [the shared test of the approaches](../map.md) (map Notes), so the shots compare with tickets 05–08. Record the cost of a third Character.
- Throwaway: files under `tmp/`, no change to `src/`.

Done when the owner has seen the two Characters in the strip and the answer records the owner's verdict and the cost for each Character.
