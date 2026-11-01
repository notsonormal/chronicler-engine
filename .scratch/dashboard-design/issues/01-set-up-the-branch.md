# Set up the branch `dashboard-design`

Type: task (HITL)
Status: open
Blocked by: the merge of `prompt-flexibility` into `main` (owner approval)

## Question

Make the branch `dashboard-design` from `main`, and finish the setup on it, so that every later ticket of this map starts from the same files.

## Context

- **Already committed (owner, 2026-11-01).** The Impeccable setup is on `prompt-flexibility`, and it reaches `main` with that merge: `PRODUCT.md`, `.pi/skills/impeccable/` (the skill without its binary), `.impeccable/critique/` (the first critique report), this map, and the two `.gitignore` parts (the binary, and the `.impeccable/` block).
- **Decided (owner, 2026-11-01): `.impeccable/` tracks the shared design records only.** `.gitignore` ignores `.impeccable/*` and allows `config.json`, `design.json`, `critique/` and `surfaces/` again. Local and runtime state stays out of git: `config.local.json`, `hook.*.json`, `live/`, `build/`, `review/`, `mocks/`. The critique reports are the history of the outside check, and `critique-storage trend` and `/impeccable polish` read them.
- Put `PRODUCT.md` into the doc checks (map Notes, "The root docs"): add it to `ROOT_DOCS` in `scripts/vale_lint.py` and in `scripts/validate_docs.py`. `PRODUCT.md` already passes Vale (its two faults were fixed on 2026-11-01). `scripts/tests/test_validate_docs.py` asserts some members of `ROOT_DOCS`. Add one assert for `PRODUCT.md`.
- Name `PRODUCT.md` and `DESIGN.md` in `AGENTS.md`, with one line on when to read them: before any change to the dashboard. `DESIGN.md` does not exist yet, so say that [Write `DESIGN.md`](04-write-design-md.md) makes it. Use `/writing-for-agents`.
- A commit needs the owner's approval (`AGENTS.md`). The agent prepares the commit and shows the file list; the owner approves.
- Other efforts work on `main` at the same time. Do not move, stage or commit their files.
- Make the probe DB fixture (map Notes): copy `target/ui-investigator/debug/chronicler_3001.db` to `tmp/dashboard-design/probe-fixture.db` with the SQLite backup API (`python3 -c "import sqlite3; sqlite3.connect(SRC).backup(sqlite3.connect(DST))"`), which is safe while the server runs. A plain file copy of a live DB can be inconsistent.

Done when `dashboard-design` exists, the doc-check and `AGENTS.md` changes are committed on it with the owner's approval, the fixture exists and opens with 41 Messages in its Game, `python build.py validate-docs` and `python build.py vale` pass with `PRODUCT.md` in scope, and the answer records the commit.
