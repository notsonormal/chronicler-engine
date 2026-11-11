# Map: Dashboard design

Labels: wayfinder:map
Status: open

## Destination

All six tabs of the dashboard follow one chosen direction and one `DESIGN.md`. Each design decision of this map is kept and built, or dropped with its reason. A final `/impeccable critique` scores every tab at 28/40 ("Good") or higher. The Game tab scored 23/40 at the start. One person who has never seen the game plays it for 10 minutes and says aloud what they think. Each place where that person stops is fixed, or set aside with its reason.

## Notes

- **Origin (owner, 2026-11-01).** The owner has no "UI sense" and looked for skills that help. Impeccable was chosen and installed. Its first critique of the Game tab scored 23/40 and found three P1 and two P2 issues: [the Game tab critique](assets/game-tab-critique/report.md). This map turns that critique, and the work that it opened, into decisions.
- **Product context.** `PRODUCT.md` (written by `/impeccable init` with the owner): the owner plays and tunes equally now; a public release is the goal; desktop first. SillyTavern and Marinara Engine are the reference applications: [Reference apps](assets/reference-apps.md). `CONTEXT.md` holds the domain names.
- **Scope (owner, charting).** All six tabs (Game, Settings, Prompt Presets, Worlds, Games, LLM Messages) and the shared visual system: type, colour, buttons, spacing.
- **Phone (owner, charting).** A rule only: no decision may block a phone layout later. The phone layout itself is out of scope.
- **Behaviour (owner, charting).** Changes to engine behaviour are in scope. A behaviour change edits its spec in `docs/specs/` and its integration test (`docs/AGENTS.md`, "Direction of edits"). The browser selectors in `docs/specs/browser_*.md` are enforced by `tests/browser/`, so a restyle keeps them or changes the spec and the test with them.
- **The browser tier also checks layout (known).** `tests/browser/stub/layout.rs` measures the rendered page: a Character portrait between 50 and 120 px wide, `flex-wrap`, `overflow`, `flex-direction`, the textarea `max-height` and the line clamp of the status line. A restyle that changes one of these changes its test and its spec in the same build ticket. Every build ticket runs `python build.py browser` before the full gate.
- **How decisions are made (owner, charting).** The owner picks the big choices from screenshots: the direction, the type and the colour. The agent decides the details by the rules of `DESIGN.md`. A prototype ticket shows 2–3 versions with the real story of the probe DB, and the owner picks one. A prototype is throwaway: CSS and JavaScript injected into the probe page, or a worktree build.
- **The outside check (owner, charting).** `/impeccable critique` on each tab, at the start ([Score the five other tabs](issues/02-score-the-other-tabs.md)) and at the end. Every tab must reach 28/40.
- **A fixed method for the check (review, 2026-11-01).** The score is a model's judgement, so a start score and an end score compare only when the method is the same. Every critique of this map uses Impeccable 4.8.0, two isolated agents (A on the `reviewer` role, B on the `scout` role), 1280×800, and the probe DB fixture. An update of Impeccable happens only between tickets, and it re-scores the Game tab first. Ticket 02 measures how much one tab's score moves between two runs.
- **A second check by a person (owner, review).** Every score comes from a model, which judges designs that a model made. `PRODUCT.md` asks that a new player understands each screen without help. At the end, one person who has never seen the game plays for 10 minutes and thinks aloud. The owner writes down each place where the person stops.
- **The root docs (owner, review).** `PRODUCT.md` and `DESIGN.md` go through the doc checks: the `ROOT_DOCS` lists of `scripts/vale_lint.py` and `scripts/validate_docs.py`. `AGENTS.md` names both files, so UI work outside Impeccable reads them too. `DESIGN.md` is the source of the design values, and `assets/styles.css` follows it.
- **Fonts (owner, charting).** Self-hosted font files in `assets/` only, with no outside request. A font needs a licence that allows this, such as the SIL Open Font License.
- **`DESIGN.md` (owner, charting).** Written from the chosen direction, not from the current look. The current dark theme is not binding (`PRODUCT.md`).
- **Decide, then build (owner, charting).** A design ticket only decides. A kept decision gets a new build ticket. The build ticket makes the change test first when behaviour changes, and ends with `/simple-after-plan`: the comment pass, the doc review, `/code-review`, the full `python build.py` with the browser tier, and the retro.
- **One branch (owner, charting).** All build work lands on one branch, `dashboard-design`, made from `main` after `prompt-flexibility` merges (owner, review), and merged at the end. Design tickets need no branch, so they start before the merge. Other efforts change the same files in parallel (for example `layout-trials`), so rebase on `main` often. The owner chose this over one short branch per build ticket, and accepted the merge-conflict risk.
- **Probe server (known).** `/chronicler-ui-investigator`: `python build.py run --target-dir target/ui-investigator -- --world redmist_estate --port 3001`. Its DB (`target/ui-investigator/debug/chronicler_3001.db`, `ENVIRONMENT.md`) holds the Game "Redmist Estate — 31 Oct 2026 (7)" with 41 Messages. A design or critique session never sends an Action: a turn costs money and changes the Game.
- **The probe DB fixture (review, 2026-11-01).** Every screenshot and every score of this map uses the same story. [Set up the branch](issues/01-set-up-the-branch.md) copies the probe DB to `tmp/dashboard-design/probe-fixture.db`. A probe server in a worktree, or after a lost DB, starts from a copy of the fixture at `<target-dir>/debug/chronicler_<port>.db`. Never point a server at the fixture itself: a newer branch can migrate it.
- **Impeccable (known).** Installed at `.pi/skills/impeccable/` (version 4.8.0). Its 21 MB engine binary is git-ignored. The launcher `.pi/skills/impeccable/scripts/impeccable` downloads the binary on its first run, so a fresh checkout needs network access once. Run `.pi/skills/impeccable/scripts/impeccable context` once per session. A plain URL scan fails here, because Chrome cannot start its own sandbox. Scan the running Chrome instead: `impeccable detect --json --cdp 9222 --target <part of the page URL>`. A critique runs two isolated agents (design review and detector) and saves its report in `.impeccable/critique/`. Git tracks `config.json`, `design.json`, `critique/` and `surfaces/` of `.impeccable/`, and ignores the rest, which is local or runtime state (owner, 2026-11-01). The HTML is in Askama templates inside Rust strings, so Impeccable's live mode cannot write back to the source (inferred).
- **Names (charting).** "Direction", "pending row", "Play area" and "Author area" are working names. No `CONTEXT.md` change. A name that reaches product copy goes through `/domain-modeling` first.
- **Skills.** `/impeccable` (its `critique`, `typeset`, `layout`, `clarify`, `animate`, `document`, `polish` commands), `/prototype`, `/grilling`, `/domain-modeling`, `/research`, `/chronicler-ui-investigator` and `/simple-after-plan`. Read `docs/diataxis/reference/frontend/dashboard.md` and `docs/diataxis/explanation/dashboard_design.md` first.

## What already exists

- **Stop for a single Job (known).** The engine can stop a single Job. `GameWriter::stop` (`src/application/generation/gate.rs`) releases any busy Game, and Scenario 39.4 in `docs/specs/dashboard.md` specifies a stop at each model call of a Job. Only the screen hides it: `autoplay_controls_answer` (`src/adapters/driving/http/layout/handlers/action_controls.rs`) renders `#stop-btn` for an Autoplay run only, and `answer_words.rs` names the route "Autoplay run".
- **The status words (known).** `step_label` and `busy_answer` in `src/adapters/driving/http/layout/handlers/endpoints.rs` hold every status line. A copy change to the status is one function.
- **The busy state reaches the screen (known).** `GameKind::BusyState` (`src/domain/model/change.rs`) is a change kind, so a busy change raises the revision and the screen loads again.
- **Fonts and assets (known).** The router serves `assets/` at `/assets` (`ServeDir` in `src/adapters/driving/http/builders/router.rs`), and the packaging step of `build.py` copies `assets/`. A self-hosted font needs a file and an `@font-face` rule, and no Rust change.
- **Design tokens (known).** `assets/styles.css` defines 51 custom properties and has a global `:focus-visible` rule. `CODING_STANDARDS.md` has no CSS rule, so `DESIGN.md` will be the only design rule set.
- **The design explanation (known).** `docs/diataxis/explanation/dashboard_design.md`, "The SillyTavern lineage", says that the dashboard takes its interaction model from SillyTavern. A direction other than "better chat" makes that section wrong, and its build ticket rewrites it.
- **Test tools for the build tickets (known).** The browser stub tier (`tests/browser/stub/`) and `ScriptedModel` (`src/test_support/scripted_model.rs`), which holds a model call, so a test can see a Job in each step.

## Failure modes

| Failure | Effect | Handling |
|---|---|---|
| The probe DB is lost, migrated or changed by a sent Action | Screenshots and scores no longer compare | The probe DB fixture (Notes). Restore a copy of it |
| A probe server in a worktree starts with an empty DB | The prototype shows no story | Start it from a copy of the fixture (Notes) |
| A prototype changes the markup with a script, and the screen loads again (a revision change, a server restart) | The morph removes the changed markup | Inject CSS into `<head>`, which a load keeps. Re-apply markup changes on `screen:loaded`, or build the version in a worktree |
| The score moves between two runs of the same critique | 28/40 can pass or fail by chance | A fixed method (Notes). Ticket 02 measures the spread |
| A restyle breaks a layout check of the browser tier | The full gate fails late | `python build.py browser` in every build ticket (Notes) |
| A self-hosted font fails to load | The page falls back to the system font, and no check fails | The build ticket of the fonts adds a browser check that the font loaded (`document.fonts.check`) |
| A prototype of a Job needs a running Job | A real Action costs money and changes the Game | Show the busy states as static versions. The build ticket tests the real states with `ScriptedModel` |
| `main` moves while `dashboard-design` is open | Merge conflicts in `assets/styles.css` and the templates | Rebase often (Notes, "One branch") |

## Decisions so far

<!-- the index — one line per closed ticket: enough to judge relevance, then zoom the link for the detail the ticket holds -->

## Not yet specified

- **The layouts of the five other tabs.** They graduate after [Score the five other tabs](issues/02-score-the-other-tabs.md) and [Decide the navigation and what a released player sees](issues/09-navigation.md): one ticket per tab or per group of tabs, from their scores and their place in the navigation.
- **Reader settings.** Both reference apps let the reader set the text width and size. After [Write `DESIGN.md`](issues/04-write-design-md.md), decide whether one fixed default is enough.
- **The first view of a new Game, and help for a stranger.** The critique did not see a new Game with only the opening Narration, and it scored help at 1 of 4. The navigation decision shows where help and the first-run state belong.
- **A copy pass.** Player words for every label and message ("Enter an action…", "Retrigger Event", status lines), after the layouts settle (`/impeccable clarify`).
- **The final check.** Re-run `/impeccable critique` on all six tabs with the fixed method, and run the test with a person (Notes). Both become tickets when the last build ticket exists.

## Out of scope

- **The layout of the prompts** (review, 2026-11-01): [Layout trials](../layout-trials/map.md) owns it. "Layout" in this map means the layout of the screen only.
- **The phone layout** (owner, charting): a later effort. Inside this map, the phone is a rule only (Notes).
- **The literal `</br>` in a Narration** (owner, review): a fault of the text handling, not a design decision. The plan is `work/issues/pending/21-narration-line-break-tags.md`. The probe DB fixture keeps the old tag in Swipe 300, so a critique on the fixture still sees it.
