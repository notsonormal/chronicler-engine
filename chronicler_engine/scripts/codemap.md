# chronicler_engine/scripts/

## Responsibility
Python helper scripts for the Rust engine — data validation, documentation indexing, test structure enforcement, coverage reporting, and asset extraction.

## Design Patterns
- **Guardrail Scripts**: Enforce project conventions programmatically.
- **Build Automation**: `generate_docs_index.py` runs as a pre-commit hook.
- **Data Validation**: JSON schema validation against `data/schemas/`.

## Files
| File | Purpose |
|------|---------|
| `validate_data.py` | JSON schema validation for worlds, characters, maps, settings |
| `generate_docs_index.py` | Auto-generates `docs/README.md` index from markdown files |
| `install_git_hooks.py` | Installs pre-commit hook for docs index regeneration |
| `check_test_structure.py` | Enforces 1:1 test file mapping (no inline `#[cfg(test)]` blocks) |
| `parse_coverage.py` | Parses `cargo tarpaulin` coverage output |
| `coverage_summary.py` | Generates coverage summary reports |
| `refine_character_json.py` | Batch character JSON processing/refinement |
| `extract_images.py` | Asset extraction from data files |
| `extract_sillytavern_png.py` | SillyTavern character card PNG extraction |
| `kimi_hook_wrapper.py` | Kimi Code CLI session start hook wrapper |
