# Collapse the Prompt Presets add forms

Type: task (AFK)
Status: open
Blocked by: —

## Question

The Prompt Presets panel always shows an empty Add form under each category. Each form is taller than the presets above it, and the panel is 3609px tall. Hide each Add form behind an "Add preset" button that opens it.

## Context

- Finding 4.5. Screenshots 12, 13.
- The scroll-container and width problems (4.6, 4.7) belong to [Decide the panel layout convention and supported viewports](19-decide-layout-convention.md). Do not fix them here.

## Done when

- The Add forms are closed by default and open on request.
- Tests are placed by `tests/STRATEGY.md`. Existing prompt-preset browser tests (`browser_prompt_presets.md` 28.x) still pass or are updated.
- `python build.py` is green. Commit after user approval.
