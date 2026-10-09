# 05 — Panel host integration

Type: prototype
Status: open
Blocked by: (none)
Assignee: (unclaimed)

## Question

Does the chosen panel wiring actually work end to end, and what breaks?

Charting settled the wiring: depend on `@luan.sh/pi-panels` and load its
extension from `node_modules` through our own `pi` manifest. That answer is a
plan, not a verified fact. This ticket proves it.

## What to prove

1. A pi package can depend on `@luan.sh/pi-panels`, and its `pi` manifest can
   reference `./node_modules/@luan.sh/pi-panels/src/extension.ts` so that one
   `pi install` loads the host.
2. A tab registered through `registerSidePanelProvider` from
   `@luan.sh/pi-libtui` appears in the host, even though the host and the
   contributor load from separate module roots.
3. Two hosts in one process do not conflict. The registry holds a single host
   slot, so a user who already installed `@luan.sh/pi-panels` separately could
   load two. Find out what happens.
4. The tab re-adds itself after a session restart, using `restoreTab`. Persisted
   layout does not include tab contents.

## Deliverable

A stub extension that adds one trivial tab, plus a short note recording what
worked and what did not. Link the note as an asset.

## Background

- `@luan.sh/pi-panels` 0.3.5 is MIT. It ships 766 lines across 5 files, and its
  manifest loads two extensions: its own, and
  `./node_modules/@luan.sh/pi-libtui/src/extension.ts`.
- The registry lives on `globalThis` under
  `Symbol.for("pi-panels/registry/v1")`, with a protocol and version check. That
  is what makes separate module roots safe.
- The host runs only in interactive TUI sessions. It does nothing in print mode,
  without a UI, or in a child process started with `PI_EMBEDDED_SIDE_CHAT=1`.
- Contributors must keep working when no host is installed. Check
  `ensureSidePanelRegistry(globalThis).hasHost()` before assuming a panel
  exists. Ticket 07 decides the fallback behaviour.

## Recommendation

Build the stub first. Do not start the real tab until this is proven.
