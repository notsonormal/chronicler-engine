# Catch hand-typed derived numbers in prose

Type: grilling (HITL)
Status: open
Blocked by: —

## Question

A generated count typed into hand-written docs becomes a second source of truth. The HTTP route count in `dashboard.md` was hand-bumped `52 → 56 → 57` across reviews. Each review spent effort *verifying* the number instead of questioning it. The user stopped it: "I just noticed that this was changed from 56 to 57. We shouldn't be manually counting and updating like this. Remove the number unless the number itself is automatically generated somehow."

`extract_http_routes.py` prints its count to stdout only; nothing validates a prose count.

How should a derived number appear in a doc?

- **Generator-written or absent.** If a generator knows the number, it writes it into the doc; otherwise the doc omits it.
- **A docs-hygiene drift phase.** The audit flags any prose number a generator could emit.
- **Both.** The phase catches what the generator route does not own.

## Context

- Source: `/reflect` 2026-10-07, sessions `01a1177a`, `01a117d3`, `01a108f1`.
- Evidence: `assets/reflect-2026-10-07/synthesis.md` (Accepted row 6; Backlog "Derived-number doc check").
- `scripts/extract_http_routes.py` — writes the route table and prints `Wrote … (N routes).` to stdout.
- `docs/diataxis/reference/frontend/http_routes.md` is generated; `dashboard.md`'s count was prose.
- Precedent: `AGENTS.md`'s `AUTO-STRUCTURE` block is generator-owned and byte-checked by the pre-commit hook.
