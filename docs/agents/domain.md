# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- **`AGENTS.md`** at the repo root — project-wide agent guidelines.
- **`CONTEXT.md`** for the Chronicler Engine context.

There is no system-wide decision-record directory at the root by design — rationale for past decisions lives in the relevant `CONTEXT.md` or in archived plans.

## Missing files: proceed silently

If a referenced `CONTEXT.md` doesn't exist, **proceed silently**. Don't flag the absence, don't suggest creating it. Files are created lazily via `/domain-modeling` when the work actually needs them.

## Layout

```
/
├── AGENTS.md
└── CONTEXT.md
```

## Use the glossary's vocabulary

When your output names a domain concept (issue title, refactor proposal, hypothesis, test name), use the term as defined in the relevant `CONTEXT.md`. Don't drift to synonyms the glossary explicitly avoids.

If the concept isn't in the glossary yet, that's a signal — either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/domain-modeling`).

