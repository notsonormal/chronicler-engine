# 01 — Where pi-explain lives in this repository

Type: grilling
Status: resolved
Blocked by: (none)
Assignee: pi (this session)

## Question

Where does the `pi-explain` package live, and how does pi load it?

Charting assumed the package belongs to this repository (chronicler-engine).
That answer was given but not confirmed against a concrete path. Settle the path
and the loading mechanism.

## Options

- **Option A — `.pi/extensions/pi-explain/`, registered in `.pi/settings.json`
  as `extensions/pi-explain`.** Follows the existing `pi-plan-mode` convention,
  which is already listed in that file as `extensions/pi-plan-mode`. Project
  local, loads only after project trust.
- **Option B — `.pi/extensions/pi-explain/`, unregistered.** Loaded only when
  someone passes `-e` explicitly.
- **Option C — user level, `~/.pi/agent/extensions/pi-explain/`.** Available in
  every project, but outside this repository and therefore outside the map's
  destination.

## Background

- `.pi/` is tracked by this repository's git. Its `.gitignore` ignores
  `.pi/extensions/**/*.js`, `**/*.d.ts`, `**/node_modules/`, and
  `**/package-lock.json`. Extension TypeScript sources are tracked.
- `.pi/settings.json` currently lists `extensions/pi-plan-mode` and
  `npm:@agnishc/edb-todo`. Relative paths resolve from the settings file's own
  directory, which is why the entry omits the `.pi/` prefix.
- The repository's own gate and pre-commit hook regenerate four index files, and
  a permission extension lives at
  `.pi/extensions/pi-permission-system/config.json`. Ticket 10 checks the
  impact.

## Recommendation

Option A. It matches how `pi-plan-mode` is wired.

## Consequence to accept

Option A makes the explainer work only inside chronicler-engine.

## Answer

Option A, with two refinements. The user confirmed this on 2026-10-09.

- **Path:** `.pi/extensions/pi-explain/`. TypeScript sources are tracked. The
  existing `.gitignore` already excludes `*.js`, `*.d.ts`, `node_modules/` and
  `package-lock.json` under `.pi/extensions/`.
- **Loading:** add `"extensions/pi-explain"` to the `packages` array in
  `.pi/settings.json`, next to `"extensions/pi-plan-mode"`. The key is
  `packages`, not `extensions`, which the ticket background had wrong. pi loads
  the package only after project trust.
- **Refinement 1 — no root `index.ts`.** pi auto-discovers any
  `.pi/extensions/*/index.ts` (`docs/extensions.md`). Auto-discovery loads the
  entry file only and ignores the `pi` manifest. The manifest is required to
  load `@luan.sh/pi-panels` from `node_modules`. So the entry is `src/index.ts`,
  declared in `package.json` under `pi.extensions`, the same as `pi-plan-mode`.
- **Refinement 2 — manual `npm install`.** `docs/packages.md`: "Local packages
  are not installed or modified, so their dependency tree remains the package
  author's responsibility." Each checkout needs `npm install` in
  `.pi/extensions/pi-explain/`. Ticket 11 now has this step. The map Note that
  said no install step is needed is corrected.
- **Accepted consequence:** `/explain` works only inside chronicler-engine.
