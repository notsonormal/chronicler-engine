# Working notes of the audit

These notes are for a person or an agent that continues the work. `engine-defects-and-risks.md` has the findings, and "the report" below means that file. Until 2026-11-10 the two files had the names `REPORT.md` and `NOTES.md`.

## Method

1. Read the root docs (`CONTEXT.md`, `PRODUCT.md`, `CODING_STANDARDS.md`, `ENVIRONMENT.md`) and the report of the earlier audit (`work/wayfinder-maps/invariant-audit/assets/REPORT.md`), so that I did not repeat it.
2. Read the non-test source of these areas in full: the gate, the Job, the Autoplay loop, the narrator, the prompt assembler, the token budget, the LLM transport and provider, the recorder, the Agents (Quantifier, Options), the Game rules, the story reads, the transaction, the migrations, the settings, the wiring, the boot, the router, and the handlers of Action, history, settings, screen and debug. About 12,000 of the 21,600 non-test lines.
3. Read stored evidence: one trial reading pack (`tmp/ltm09/redmist_julian/trial-1`) and two dev databases (`target/ui-investigator/debug/chronicler_3001.db`, `target/lt11/debug/chronicler_3005.db`), both read-only.
4. Ran a probe server against a fake model endpoint to confirm the findings that a run can confirm.

## The probe

Folder: `/tmp/fable-probe` (outside the repository, so the server found no `.env`). It held a copy of `target/debug/chronicler_engine` and links to `assets/` and `data/`. The binary is from 20:55 on 2026-11-09. The last commit that changed `src/` is from 21:02 (`8eb0b057`), so the binary is probably that tree. I did not build it again.

```bash
# fake model endpoint: local-only/evidence/fake_llm.py, port 39111. The file `mode` picks the answer.
OPENROUTER_API_KEY=sk-FAKE-AUDIT-KEY-0001 ./chronicler_engine --port 39100 --host 127.0.0.1

# Finding 1: the key of the environment goes to the address of the form
curl -X POST http://127.0.0.1:39100/connections/test -H 'Origin: https://evil.example' \
  --data 'conn_name=x&conn_provider=openrouter&conn_model=m&conn_api_key=&conn_base_url=http://127.0.0.1:39111/v1'

# Finding 1: a stored key comes back in the form
curl -X POST .../connections/add --data 'conn_name=probe&conn_provider=openrouter&conn_model=fake-model&conn_api_key=sk-STORED-KEY-0002&conn_base_url=http://127.0.0.1:39111/v1'
curl -H 'Host: attacker.example' .../fragment/connections/<id>/edit     # value="sk-STORED-KEY-0002"

# Finding 5: mode "markdown", then POST /action, then GET /screen?parts=story-log
# Finding 4: mode "length", then POST /action; the status display says Ready
# Finding 2: mode "normal 2" (2 s for each call), POST /autoplay cycles=30, kill -INT, count the calls
# Finding 8: start the binary from an empty working directory; / is 200, every asset is 404
```

Results: `text/plain` as the content type of the form is refused with `415`, so only the urlencoded form passes. A foreign `Host` header gets `200`. The answer of `/` has no security header.

A mistake of mine to avoid: `pkill -f <pattern>` killed its own shell, as retro issue 90 says. Use `pgrep` with a bracket in the pattern and kill by id.

## Pass 2

The owner asked for a second pass with no stops for questions. The questions are in section 8 of the report.

1. **The rest of the source.** The World storage and handlers, the Game catalogue and handlers, the boot seed, the prompt preset service, the text check adapter, the failure kinds, and the four largest dashboard scripts (`dashboard-screen.js`, `dashboard-action-area.js`, `dashboard-story-log.js`, `dashboard-errors.js`).
2. **A long story on the probe.** `local-only/evidence/fake_llm2.py` answers with 150 words and two Characters present. `local-only/evidence/scaling_drive.py` ran ten Autoplay runs of 100 Cycles on one Game and read the cost after each run.
3. **A browser session.** Headless Chrome on port 9222 already ran in the container. I opened a new tab on the probe (`PUT /json/new`) and closed it at the end. A tab on `127.0.0.1:3001` was there before, and I did not touch it. For the phone width I put the page in a 390 px wide `iframe`, because the tools cannot resize the window.
4. **Three stored trials.** `tmp/ltm09/redmist_julian/trial-1`, `tmp/lt15/new/trial-1` and `tmp/lt15/reference/trial-1`. Each has `story.json` (the Swipe state of each entry) and `calls.jsonl` (the raw request and answer of each call). `local-only/evidence/trial_story_stats.py` reads both.
5. **The owner's records.** A scout (`known-or-new`, session `843c3347-5cb2-4305-b3c4-ba398a34efe4`) searched `.scratch/` and `docs/` for an earlier record of each finding. I checked its quotes for the decided and the known rows. The table is in section 5 of the report.
6. **The advisory database.** `cargo audit` is not installed, so I read the RustSec pages for the crates in `Cargo.lock`.

Things that pass 2 looked at and dropped:

- **A failed gate.** 15 of 64 full gates failed in the build history. A failed gate during development is the gate at work. Only a test that fails and passes on one tree is a fault, and one test does that (section 6 of the report).
- **A blank Settings screenshot.** One screenshot of the Settings tab was empty. The page had its content at that moment (`innerText` of 1,112 characters), and the next screenshot was correct. It was a fault of the capture.
- **Two tabs with two Games.** The owner has a ticket for it (`work/issues/pending/01-per-tab-games.md`).
- **The scroll-back past 50 entries.** The owner has a seed for it (`work/issues/pending/19-story-scroll-back.md`). Finding 13 is a different fault of the same window.

## Numbers

- Source: 21,582 non-test lines in `src/`, 46,005 with the unit tests, 53,425 lines in `tests/`. The gate runs 309 guardrail tests, 1,663 integration tests and 44 browser tests.
- Trial `ltm09`: 100 Cycles, 402 calls, 968 s, 2.95 M input tokens, 80 % cached. The Quantifier made 201 of the calls and used 13 % of the input tokens. Its calls were 44 % cached.
- Swipe state: 4.2 to 5.9 kB for each Swipe. The text of a Swipe: 0.7 to 1.0 kB. The Character cards are 72 % to 93 % of the state.
- `llm_messages`: 50 rows, 1.0 MB and 1.9 MB of request and response text in the two databases.
- Build history (last 1,000 rows): 64 full gates, 15 failed. Median of the good full gates since 2026-11-01: 319 s.

## Things I looked at and dropped

- **Blocking storage calls in async handlers.** Every handler calls SQLite on a runtime thread. For one player this costs nothing that I can measure. It matters only with many players, and the design has one active Game for the whole process.
- **SQLite settings.** No WAL and no `busy_timeout`. One process and one connection behind a mutex, so neither matters today.
- **The session key of the provider cache.** Impersonate and Narration share one session and have different prompt starts. The trial still shows 85 % cached tokens for both, so I found no problem.
- **The Retry of the opening Narration.** The model writes a new opening and does not see the authored one. This is a product question, and it is small.
- **(Pass 2 measured this, and it is now finding 12.) The Quantifier runs on an Impersonate Input but not on a typed Input.** So a move that the Persona only tries can change the Room before the narrator answers, but only when the narrator wrote the Input. `Tail::retrigger_record` shows that this is intended. I did not find the reason in the specs. A question for the owner, not a finding.
- **`{{char}}` and other SillyTavern macros.** `TemplateVars` knows `{{user}}` and six others. The shipped data uses no other macro. A card from SillyTavern would keep its `{{char}}` as text.
- **Quantifier prompt cost.** The prompt holds the full description of each Character present. The names and ids are enough to decide who is present. The saving is a few hundred tokens for each call.

## Open questions for the owner

They are in section 8 of the report, each with a recommendation.

## Not done

See section 7 of the report. The next audit step with the most value is a session with a real Connection and typed Inputs. Every story that I measured is an Autoplay run, so findings 11, 12 and 14 can be weaker when a person plays.
