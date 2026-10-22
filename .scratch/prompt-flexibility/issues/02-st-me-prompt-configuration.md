# How ST and ME let an author change a call

Type: research
Status: open
Blocked by: —

## Question

How do SillyTavern and Marinara Engine let an author change the layout and the parameters of an LLM call: which parts, through what mechanism, and with which default texts?

## Context

- Start from what the repo already has: `docs/external_applications/sillytavern_prompt_system.md` (2026-08), `marinara_engine.md` (its "Game Mode Prompt Architecture" is the closest match to this engine), `marinara_engine_system_prompt.md` (our preset fields `role`, `instructions`, `output_format` follow ME's sections) and `.scratch/_closed/steering-and-guided-generation/research/03-sillytavern-core.md`. Check them against the pinned commits and fill the gaps. Do not rewrite them.
- Pinned commits and clone location: see the map's Notes.
- Known from the first research (2026-10-22): ST builds its prompt in the browser (`public/scripts/openai.js`). Its Prompt Manager has an ordered list of prompts and markers (`default/content/presets/openai/Default.json`). The `impersonation_prompt` is appended last. ME builds its prompt on the server (`packages/server/src/services/prompt/assembler.ts`) from sections and markers. For Impersonate it drops the non-marker sections unless a dedicated Impersonate preset is chosen, and it appends `DEFAULT_IMPERSONATE_PROMPT` (`packages/shared/src/constants/impersonate.ts`) as the last user message.
- Points to cover in ST: the order and the on/off switch of each prompt, the role of each prompt, injection position and depth, markers, Post-History Instructions, the card's own `system_prompt` and `post_history_instructions`, Author's Note, World Info positions, the instruct and context templates (Text Completion), names in history, the impersonation prompt, the continue nudge, and the parameters for each preset (max tokens, samplers).
- Points to cover in ME: sections, markers, groups, choice blocks, the Impersonate preset, connection and template, agents, and the parameters for each call.

## Output

An asset `assets/st-me-prompt-configuration.md`: one table for each app (part of a call, mechanism, changeable by an author, source path), and an appendix with the default prompt texts verbatim. Mark known and inferred. List the corrections that `docs/external_applications/` needs; the build plan carries them.
