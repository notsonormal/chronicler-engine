# Decide what a tier-1 test may observe

Type: grilling (HITL)
Status: resolved
Blocked by: —

## Question

A tier-1 (HTTP E2E) test acts through HTTP. What may it assert on: only the response, the response plus stored state, or the response plus stored state read through the application's read seams? And in what words may a spec Given or Then describe it?

## Context

- `tests/STRATEGY.md:12` says tier 1 checks "the server response a `curl` would see". Lines 22-23 say every spec scenario maps to an HTTP E2E test. Lines 28-34 send internal state (`last_trigger`, phase transitions) to the unit tier. The doc contradicts itself, and each spec file resolves it its own way. [known]
- A precedent already exists: [Spec restructure: HTTP-observable, endpoint-named](../../test-strategy-execution/issues/11-spec-restructure-http-observable-only.md) settled "specs cover HTTP-observable behaviour only". It never defined "observable", and the specs drifted past it. This ticket sharpens that rule. It does not reopen endpoint naming or the tier split.
- What the code does today [known, counted]:
  - 83 calls to `state.message_service.load_messages()` across 7 files in `tests/http/`, for example `tests/http/actions.rs:32`.
  - About 40 raw `GameState` field reads, mostly `final_state.narrative.input_buffer.status`, in `actions.rs` (13), `swipe_new.rs` (17), and `retrigger.rs` (9). List: [raw_gamestate_reads.txt](../assets/test-audit/raw_gamestate_reads.txt).
  - 65 of 146 scenarios name internal state in a Given or Then, for example `docs/specs/retrigger.md` 13.1, "Given … `narrative.last_trigger` is set". The 58 hard leaks come from a grep. The 7 soft ones are a lower bound. Detail: [specs.md](../assets/test-audit/specs.md).
  - `scripts/validate_feature_spec.py` passes (146/146). It checks tags and directories only and never reads spec prose or test bodies.
- `tests/http/settings.rs` 20.2–20.4 show the opposite failure. Each asserts only `body == "Settings saved!"`, so none proves the switch took effect.
- Options raised in the audit session:

| Option | A tier-1 test observes | Specs name | Cost |
|---|---|---|---|
| A. Strict | The response, plus follow-up GETs | Only what a response shows | Weaker tests. `POST /action` returns "Thinking...", so outcomes must be scraped from fragment HTML. Rejected in session as less useful. |
| B. Codify practice | The response, plus any state | Anything, including field names | No rework. Tier 1 becomes unit tests with a router in front, and ~40 tests break on internal reshapes. |
| C. Domain outcome | The response, plus stored state read through the application's read seams | `CONTEXT.md` terms only | Rewrites ~40 raw reads (status reads can use `GET /status/generating`). Some may need a new read seam. "Read seam" needs a definition. |

- The session leaned to C. The proposed leading word is **domain outcome**: a Given or Then describes a domain outcome in `CONTEXT.md` terms, or something a client sees in a response.
- This map's tickets place tests by `tests/STRATEGY.md`. Most of them add tier-2 browser tests, which this decision barely touches.

## Answer

**Option C — domain outcome.** A tier-1 test observes the response, plus stored state read through a read seam. The leading word is **domain outcome**.

**Read seam (the one-sentence definition).** A read seam is a read the application exposes to a driving adapter — an HTTP GET, or a method on an application read service/port — through which a test observes stored state; a test leaks when it dereferences a field of `GameState` or reads a storage row.

Reviewer test for one line: *does it call an HTTP GET or a named application read, or does it dereference a `GameState` field / read a storage row?*

### Sub-decisions

| Question | Decision | Reason |
|---|---|---|
| Storage reads as seams | **No.** Storage is not a read surface. Observed state goes through an HTTP GET or an application read service/port. | The recorder seam already has an application read — `GameViewQuery::list_latest_llm_messages()` exists in `src/application/games/view_query.rs` with zero callers. The other storage observation reads have HTTP/service substitutes or are persistence facts that belong to the storage tier. An allowlist is unnecessary. |
| Helpers | **Transitive.** The rule judges the read the helper performs. A helper may wrap a seam; it may not wrap a field reach-through. | A helper is not a seam (it is not exposed by the application). H2 would let `fn current_status(&state) { … .narrative.input_buffer.status }` pass behind a named call, reopening the hole. A check over `tests/http/` including helper bodies is exactly this rule. |
| Spec Given/Then vocabulary | **CONTEXT.md domain nouns.** A Given/Then names a domain thing only as a `CONTEXT.md` term, and a client-seen thing only as a response value or a DOM `id`/`class`. Rust identifiers, field paths, and module names are banned. Plain English is fine when the nouns are glossary terms. | Gives a reviewer one test: *is the noun a `CONTEXT.md` term or a client-seen value?* |

The rule governs assertions. Synchronization waits (for example `wait_idle`) may keep reading state.

### Migration of the existing corpus

**Split.** Field reads migrate now; scenario prose migrates on touch.

- **Now:** [Migrate tier-1 tests to observe through legal read seams](61-migrate-tier-1-test-reads.md). The 45 raw `GameState` field reads and the storage observation reads (~11 sites) move to HTTP GETs, `GameViewQuery`, `MessageService`, and `PromptPresetService`. The three persistence facts (`load_latest_snapshot`, `count_swipes_for_message`, `require_active_swipe_index`) reduce to storage-tier assertions.
- **On touch:** the 58 scenarios whose Given/Then names an internal identifier are reworded in `CONTEXT.md` terms when their spec file is next edited. [Rewrite the test strategy around one checkable tier-1 rule](32-rewrite-test-strategy.md) records this rule in `tests/STRATEGY.md`.
- **Nothing:** the 7 soft-leak scenarios (`worlds.md` 25.1–25.5, `prompt_presets.md` 21.16, 21.25) already read as clean domain prose, and under this decision their covering tests' `world_catalogue` / `prompt_preset_service` reads are legal seams. Their wording stands.

### Facts corrected against the ticket context

- 45 raw `GameState` field reads, not ~40 ([raw_gamestate_reads.txt](../assets/test-audit/raw_gamestate_reads.txt)).
- The `tests/http/settings.rs` 20.2–20.4 example is stale. [Fix the weak dashboard and settings tests](34-fix-weak-dashboard-and-settings-tests.md) added a follow-up `GET /fragment/settings` badge assertion; they are no longer the opposite failure.


## Done when

- One option is chosen, with the user's reasons recorded under `## Answer`.
- If C: "read seam" has a one-sentence definition that a reviewer can apply to a single line of test code.
- The answer states what happens to the 65 leaking scenarios and the ~40 raw reads: migrate now, migrate on touch, or leave. The map's *Not yet specified* entry graduates into tickets, or is ruled out of scope.
