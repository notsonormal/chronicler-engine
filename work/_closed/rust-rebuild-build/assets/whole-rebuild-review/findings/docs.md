# Docs (step 4)

Source: scout `docs-hygiene` (session 69dd27b4). Lead applied fixes 1–7 and 9; `validate_docs.py` passes after them.

| # | Finding | File | Action |
|---|---|---|---|
| DOC-1 | Deployment Contract named `saves/` and `prompts/` folders that do not exist (pre-existing, also at `bc7d6cd4`) | `reference/architecture_system.md` | fixed |
| DOC-2 | Spec scenario numbers cited in a Diátaxis doc | `reference/coding_standards/integration_test_standards.md` | fixed |
| DOC-3 | `game_flow.md` said the newest Game is picked by `updated_at`; code orders by newest Message, then `updated_at` (`storage.md` is right) | `reference/game_flow.md` | fixed: rule lives in `storage.md` only |
| DOC-4 | Deprecated term "turn" | `reference/narrative/prompt_system.md` | fixed |
| DOC-5 | Rationale in a Reference doc, naming a type that does not exist (`StateRepository`) | `reference/architecture_system.md` | fixed: sentence dropped |
| DOC-6 | Past-design sediment ("in the older design") | `explanation/storage_design.md` | fixed |
| DOC-7 | `RwLock` named; the engine has none now | `explanation/architecture.md`, `explanation/rust_idioms.md` | fixed |
| DOC-8 | `AppState` collaborator bullets are a field dump | `reference/architecture_system.md` | open, low (inferred); left for the owner — rewriting the section is larger than this pass |
| DOC-9 | Negative explaining on the job entry | `reference/storage.md` | fixed |

No stale removed name found anywhere else; specs spot-checked against code with no finding.
