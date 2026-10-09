# 11 — Install and acceptance

Type: task
Status: open
Blocked by: 04, 05, 06, 07
Assignee: (unclaimed)

## Question

The destination is "installed and in use". What exact steps install it, and what
checklist proves it works?

This is the last ticket. It closes when the package is installed and the
checklist passes.

## Steps to perform

1. Place the package at `.pi/extensions/pi-explain/` (ticket 01).
2. Run `npm install` in `.pi/extensions/pi-explain/`. pi does not install the
   dependencies of a local package, and `node_modules/` is gitignored.
3. Add `"extensions/pi-explain"` to the `packages` array in
   `.pi/settings.json`.
4. Confirm pi loads it once: `pi config` shows the extension, no extension
   error is reported, and the extension does not load twice.
5. Run the acceptance checklist below.

## Acceptance checklist

- [ ] `/explain off` leaves ordinary answers untouched.
- [ ] `/explain md <topic>` writes a Markdown artifact under `.explain/` and the
      panel shows it.
- [ ] `/explain diagram <topic>` writes a Mermaid artifact, and the panel shows a
      rendered picture.
- [ ] `/explain html <topic>` writes a self-contained HTML page, and the panel
      shows a preview.
- [ ] The artifact opens in the browser from the panel.
- [ ] A second `/explain` run replaces or adds to the panel without a restart.
- [ ] A new pi session restores the panel state and the tab.
- [ ] The main context grows by a small, bounded amount per explanation. Record
      the number that ticket 04 measured.
- [ ] With the panel host absent, the fallback from ticket 07 behaves as
      designed.
- [ ] In print mode, `/explain` still writes the artifact and returns the path.
- [ ] The repository gate still passes: `python build.py --no-browser`.

## Deliverable

A short install note linked as an asset, and the completed checklist appended as
this ticket's answer.

## Background

- The repository this package lives in has its own gate. Do not leave it red.
- `.explain/` is project local. The user's repository needs a `.gitignore` entry
  for it. Decide whether the extension writes that entry or only documents it.

## Recommendation

Run the checklist against a real terminal that supports the Kitty or iTerm2
graphics protocol. Record the terminal used.
