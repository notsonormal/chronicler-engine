# Map: Replace find_free_fn_smells.py with module-allowlist rules

Status: open
Type: wayfinder:map
Assignee: (unassigned)

## Destination

A forward guardrail that replaces the signature-heuristic `find_free_fn_smells.py` (and its 21-entry per-function suppression list) with module-allowlist rules. Before the new rules land, the three parked-orchestrator free fns (`execute_action_impl`, `retry_last_response_impl`, `retrigger_event_impl`) are converted to methods on `DefaultApplicationService` — removing the only non-honest residue from the suppression list. After the new scanner ships, there is no growing per-function suppression list; honesty of a free fn is determined by the module category it lives in.

## Notes

- The mental model is the C# smell: static helper taking `Repository` as first param = wrong design. In Rust that becomes "free fn doing an operation that behaviorally belongs on its first-param type AND where the move is layer-safe." Cases like mappers, multi-input engine algorithms, persistence-boundary seams, generic utilities, and prompt builders are honest free fns, not smells.
- Provenance: `find_free_fn_smells.py` caught real smells during the backward sweep (e.g. `AppSettings::load/storage` was reverted to free fn `load_settings(storage)` in commit 14a69fb to avoid a domain → adapter dependency). The backward sweep is done; the 21-entry `SUPPRESSED_FREE_FNS` list is the residue of honest free fns that survived.
- Relevant docs: `chronicler_engine/docs/architecture/system.md` (carries the "Free fn Doctrine" section promised by the archived `plan-eliminate-free-function-smells-final.md`).
- Relevant scanner artefacts: `chronicler_engine/scripts/find_free_fn_smells.py`, `chronicler_engine/scripts/tests/test_find_free_fn_smells.py`, `chronicler_engine/build.py` (scanner wired as gate).
- Skills every session should consult: `/grilling`, `/domain-modeling` for design decisions; `/planning-and-task-breakdown` for implementation tasks.
- Standing preference: Ponytail (lite) + Caveman prose for implementation work. Subagent-driven where possible (see `AGENTS.md`). Do not grow a new per-function suppression list — that is the failure mode this effort replaces.

## Decisions so far

<!-- index — one-line gist per closed ticket -->

- **Real smell definition** — free fn is a smell iff the operation behaviorally belongs on its first-param type AND moving it onto the type is layer-safe. "First param is `&DomainType`" is only the first filter of a candidate, not a smell by itself.
- **Forward guardrail only** — backward sweep is complete; scanner enforces against regressions, does not re-audit the 18 honest free fns.
- **Replace, don't delete** — scanner stays as a gate, but its exemption mechanism changes from per-function suppression list to module-allowlist rules.
- **Three `*_impl` orchestrators convert to methods** — `execute_action_impl`, `retry_last_response_impl`, `retrigger_event_impl` move to `impl DefaultApplicationService` as `execute_action` / `retry_last_response` / `retrigger_event`. They are genuine method candidates parked as free fns only to keep the service's API surface slim (T3 cleanup rationale). Conversion removes the parked-orchestrator category from the scanner's residual list before the new rules land.
- **Scope: wide** — the conversions and the scanner replacement land as one effort, so the new scanner starts from a clean state with no parked-orchestrator accommodation.

## Not yet specified

<!-- fog — in-scope, not yet sharp enough to ticket -->

- Whether the new scanner needs a parser upgrade (e.g. syn-based AST) to express the signature constraints 03 may want. The current text-scanner masked-comments-and-brace-counting parser has known gaps (slices `&[T]`, qualified types like `crate::domain::...` surface as 2 UNKNOWN findings). Can't decide until 03 lands — if 03 rejects constraints entirely, the parser may not be needed and the UNKNOWNs become irrelevant.
- Fate of the existing `SUPPRESSED_FREE_FNS` list specifically — deleted wholesale, kept as deprecated reference, or replaced by machine-generated allowlist tests. Depends on 02 (file vs folder) and 03 (constraint pairing) answers, since those determine what the new exemption data structure looks like.
- How an advisory scanner integrates with `healthcheck.py` if 04 = advisory — a new `@register("free_fn_candidates")` check, or a separate script invoked manually. Subordinate to 04; not worth its own ticket until 04 lands.
- Whether the Free fn Doctrine section in `system.md` needs rewriting to match the new allowlist shape (file-level vs folder-level naming in its examples). Subordinate to 02; likely a follow-up task inside 05 rather than its own ticket.

## Out of scope

<!-- work ruled beyond the destination -->

- Another backward sweep of the 18 honest free fns. The backward sweep is done; they stay where they are.
- Rewriting the T3 cleanup history (deleting `query_handlers` / `MessageEditingService` passthrough methods). That cleanup landed for good reasons; the three `*_impl` conversions do not bring back the deleted 14 passthrough methods.
- Replacing clippy's `wrong_self_convention` / `len_without_is_empty` family. The scanner complements clippy, not replaces it.
- Parser-class changes to the scanner beyond what the new rules need (e.g. full Rust AST via syn). Out of scope unless a chosen rule类别 requires it.
