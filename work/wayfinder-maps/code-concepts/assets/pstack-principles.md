# Seed: the pstack principles

Source: <https://github.com/cursor/plugins/tree/main/pstack/skills>, the 24 `principle-*` skills (read 2026-10-21). The guide that explains how pstack uses them is `pstack/docs/guide/08-principles.md`. A local copy may exist in `tmp/pstack/`, but `tmp/` is scratch and can be deleted.

## How pstack uses them

- Each principle is one skill. Its description gives the trigger and the stance, and most stances name their opposite inline ("X instead of Y").
- The names are steering words. "Use *subtract before you add*" redirects an agent in one phrase, because the agent has already read the rule behind the name.
- `/poteto-mode` reads the index at the start of every multi-step task and applies the principles. That is enforcement on top of the work. This map does not do that (Out of scope).

## How this map differs

| | pstack | This map |
|---|---|---|
| Kind | Prescriptive: what to do | Descriptive: what this code does |
| Reach | The same for every repo | Specific to this repo |
| Level | Stances | Stances, patterns and idioms |
| Opposite | Named to reject it | Named so an agent can choose it on purpose |

## The split (charting session, inferred)

**Code shape:** a scout can see these in the code or the tests. They are candidate lens entries.

- **boundary-discipline**: concentrate guards at system boundaries; trust internal types; business logic in pure functions.
- **model-the-domain**: encode the domain in a data structure (state machine, typed model, registry) instead of scattered conditionals.
- **type-system-discipline**: illegal states unrepresentable, branded primitives, parse at boundaries, exhaust variants, derive from authoritative schemas.
- **separate-before-serializing-shared-state**: remove the sharing first; serialize structurally only when one writer is a real invariant.
- **make-operations-idempotent**: converge to the same end state after partial runs, crashes and retries.
- **migrate-callers-then-delete-legacy-apis**: migrate and delete in one wave; no compatibility layers.
- **minimize-reader-load**: count layers to trace and state to hold; collapse one-caller wrappers.
- **test-behavior-not-implementation**: assert what users of the code observe.
- **encode-lessons-in-structure**: a lint, a check or a script instead of more text.

**In between:** these are process principles, but they leave marks in the code (few layers, no shims, data structures chosen first).

- **foundational-thinking**, **laziness-protocol**, **subtract-before-you-add**, **outcome-oriented-execution**.

**Process:** how an agent works. These are out of scope.

- **attack-the-premise**, **build-the-lever**, **exhaust-the-design-space**, **experience-first**, **explain-the-number**, **fix-root-causes**, **guard-the-context-window**, **never-block-on-the-human**, **prove-it-works**, **redesign-from-first-principles**, **sequence-verifiable-units**.
