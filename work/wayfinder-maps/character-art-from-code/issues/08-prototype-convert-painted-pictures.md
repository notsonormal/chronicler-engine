# Prototype a conversion of the painted pictures

Type: prototype (HITL)
Status: claimed
Blocked by: 04

## Question

Can code turn a painted picture from `data/images/` into flat cel art near the reference look, with Expression parts on top?

## Context

- Start from `lisette_headshot.png` and `gabriella_headshot.png`. Try colour reduction to a few flat tones, dark outlines from the edges, and a crop to a head that fills the frame. Then put eye, brow and mouth parts over the face for each Expression.
- Conflicts to record, not to hide:
  - **Goal 3 (map Notes).** A new Character still needs a painted picture, so an image model stays in the loop.
  - **The target style.** The target is art that looks good in an anime style (map Notes, 2026-11-02). Chibi is not required, so the painted pictures are a fair start point. Test whether the converted art looks good next to the other approaches.
  - **Alignment.** Expression parts must sit on a face that each picture places in a different spot (guessed: this needs a face position for each Character).
- The sandbox has Pillow (known). Check for other image tools before you add one.
- Run [the shared test of the approaches](../map.md) (map Notes), so the four prototypes compare.
- Throwaway: files under `tmp/`, no change to `src/` or `data/images/`.

Done when the owner has seen the two converted Characters in the strip and the answer records the steps, the faults and the owner's verdict.
