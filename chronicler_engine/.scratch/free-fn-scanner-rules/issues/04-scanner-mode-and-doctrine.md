# 04: Scanner mode — gate vs advisory — and relationship to Free fn Doctrine

Status: open
Type: grilling
Assignee: (unassigned)
Blocked by: (none)

## Question

Should the new scanner be a **build-failing gate** (like the current script) or an **advisory report** reviewed at PR time? And what is the relationship between the scanner and the Free fn Doctrine section in `system.md` — is the scanner the enforcement arm of the doc, or is the doc the source of truth with the scanner advisory?

## Context

The current script is a gate: `build.py` runs it and fails on any unsuppressed SMELL. That stance was defensible when the rule was a per-function suppression list — every flagged function was a real candidate that had been triaged.

Under the new module-allowlist rules (per 02) and optional signature constraints (per 03), the scanner is structurally weaker:

- A pure module-allowlist cannot catch a method-shaped free fn added inside an allowed module.
- Signature constraints (if adopted) are coarse filters, not proof — they flag candidates, not smells.

If the scanner is a gate under these weaker rules, two failure modes appear:

1. **False negatives pass silently** — a real smell inside an allowed module is not caught, but the gate is green. The gate gives false confidence.
2. **Edge-case flags break the build** — a free fn that fails its category signature constraint (per 03) but is actually honest stops the build until someone widens the constraint or adds an annotation. This is the suppression-list growth problem in miniature.

### Two stances

**Gate stance** — scanner fails build on unsuppressed flags. Maintains enforcement bite. Requires the rules to be tight enough that false positives are rare. Hard to achieve with module-allowlist + signature rules.

**Advisory stance** — scanner emits a candidate report (file, line, category, shape, reason). Does not fail build. Reviewer triages at PR time. Module-allowlist becomes noise reduction (skip obvious honest categories), not enforcement. Width of allowlist is tolerable because the output is review, not a gate. The Free fn Doctrine in `system.md` is the source of truth; the scanner surfaces candidates for a human to judge against the doctrine.

### Relationship to Free fn Doctrine

The `system.md` "Free fn Doctrine" section (promised by `plan-eliminate-free-function-smells-final.md`) is the semantic source of truth — it defines what honest free fns look like per category. The scanner is a mechanical approximation of it.

- If scanner = gate, the scanner *is* the enforcement arm of the doctrine. The doctrine doc becomes commentary on the gate.
- If scanner = advisory, the doctrine doc is the source of truth; the scanner surfaces candidates for a human to judge against the doctrine. PR reviewer consults `system.md` to decide.

## Recommendation

**Advisory mode + doctrine doc as source of truth.** Gate-mode was defensible under the per-function suppression list because each flag was already triaged. Under module-allowlist rules the scanner cannot carry that weight, and gate-mode either gives false confidence (false negatives silent) or recreates the suppression-list problem (edge cases break builds until suppressed). Advisory mode keeps the scanner useful for surfacing candidates without overpromising what static analysis can decide. Remove the scanner from `build.py`'s gate; keep it as a `healthcheck.py` advisory check (`python scripts/healthcheck.py free_fn_candidates`) per the existing advisory-check pattern in `abstraction-antipattern-healthcheck-plan.md`.
