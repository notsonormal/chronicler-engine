# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- **`CONTEXT-MAP.md`** at the repo root — points at one `CONTEXT.md` per context.
- **`CONTEXT.md`** for each context in scope (e.g. `chronicler_engine/CONTEXT.md`).
- **`docs/adr/` inside the relevant context** — e.g. `chronicler_engine/docs/adr/` for engine decisions.

There is no system-wide `docs/adr/` at the root by design — all ADRs are context-scoped.

## Missing files: proceed silently

If a referenced `CONTEXT.md` or `docs/adr/` doesn't exist, **proceed silently**. Don't flag the absence, don't suggest creating them. Files are created lazily via `/domain-modeling` when the work actually needs them.

This repo's `CONTEXT-MAP.md` lists three contexts (Chronicler Engine, Docker, Scripts) but only the engine context has been materialized. Skills reading the Docker or Scripts context should silently treat them as empty until work in those areas forces the glossary into existence.

## Layout

```
/
├── CONTEXT-MAP.md
├── chronicler_engine/
│   ├── CONTEXT.md
│   └── docs/adr/
├── docker/
│   └── CONTEXT.md          (created lazily)
└── scripts/
    └── CONTEXT.md          (created lazily)
```

## Use the glossary's vocabulary

When your output names a domain concept (issue title, refactor proposal, hypothesis, test name), use the term as defined in the relevant `CONTEXT.md`. Don't drift to synonyms the glossary explicitly avoids.

If the concept isn't in the glossary yet, that's a signal — either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/domain-modeling`).

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding:

> _Contradicts ADR-0007 (event-sourced orders) — but worth reopening because…_
