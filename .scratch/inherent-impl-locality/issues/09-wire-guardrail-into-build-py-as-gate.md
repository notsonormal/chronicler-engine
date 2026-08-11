# 09 — Wire guardrail into build.py as gate

Type: task
Status: ready-for-agent
Blocked by: 01

## Question

Promote `guardrails_inherent_impl_locality` from audit-only to a gating step in `build.py`'s step list.

Current state:
- Rule exists in `tests/infrastructure/guardrails/inherent_impl.rs` (built in 01).
- Rule is wired into `cargo test --test guardrails` (the existing harness from `guardrails/mod.rs`).
- `build.py` already runs `cargo test --test guardrails` as one of its steps (per the existing guardrails integration — verify by reading `build.py`).
- All violations from 02 are resolved by 03, 04, 05, 06, 07, 08.

What this ticket does:
- Verify `cargo test --test guardrails` is already in `build.py`'s step list. If yes, the gate is already in place once refactor tickets land — no build.py change needed. If no, add the step.
- Confirm the rule runs in gate mode (hard failure, no advisory path).
- Verify there are no `#[allow(...)]` suppression annotations on the rule (rust allow on a test would disable it silently — scan for any).
- Remove the `audit-only` framing from the rule's module doc comment (rewrite as "enforced" language).

Constraints:
- `build.py` must remain green at completion.
- Do NOT retire `scripts/find_free_fn_smells.py` — that's issue 10 in `.scratch/free-fn-scanner-rules/`, out of scope.
- Do NOT modify the rule logic itself in this ticket — only its framing/wiring.

Acceptance:
- `cargo test --test guardrails guardrails_inherent_impl_locality` passes on `main`.
- `build.py` full run is green.
- Rule module doc says "enforced" not "audit-only".
- No `#[allow(...)]` on the rule function.
