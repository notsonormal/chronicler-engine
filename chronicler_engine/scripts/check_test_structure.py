import re
import sys
from pathlib import Path


def check_inline_test_blocks(src: Path) -> list[str]:
    """Inline `#[cfg(test)] mod X { ... }` blocks are forbidden in src/."""
    errors: list[str] = []

    for rs_file in src.rglob("*.rs"):
        content = rs_file.read_text(encoding="utf-8")
        for _ in re.finditer(r"#\[cfg\(test\)\]\s*mod\s+\w+\s*\{", content):
            errors.append(f"Inline test block found: {rs_file}")

    return errors


def find_module_root(tests_file: Path) -> list[Path]:
    """Find candidate declaration files for a `*_tests.rs` file.

    A directory `foo/` is declared either by `foo/mod.rs` (inside `foo/`) or by
    `foo.rs` in the parent directory. For files at the crate root (`src/`),
    the declaration lives in `lib.rs` / `main.rs`. Return all candidates that
    exist so the caller can search each.
    """
    candidates: list[Path] = []
    parent = tests_file.parent

    mod_rs = parent / "mod.rs"
    if mod_rs.exists():
        candidates.append(mod_rs)

    parent_name = parent.name
    parent_rs = parent.parent / f"{parent_name}.rs"
    if parent_rs.exists():
        candidates.append(parent_rs)

    # Crate-root tests (e.g. src/cli_tests.rs) are declared in lib.rs / main.rs.
    if parent.name == "src":
        for crate_root in (parent / "lib.rs", parent / "main.rs"):
            if crate_root.exists():
                candidates.append(crate_root)

    return candidates


def check_tests_registration(src: Path) -> list[str]:
    """Every `*_tests.rs` file must be declared as `mod <stem>;` in its module root.

    Catches the recurring class of bug where a worker adds a `*_tests.rs` file but
    forgets the `mod` declaration in the sibling `mod.rs`, leaving the tests
    silently orphaned (never compiled, never run).
    """
    errors: list[str] = []

    for tests_file in src.rglob("*_tests.rs"):
        stem = tests_file.stem
        module_roots = find_module_root(tests_file)
        if not module_roots:
            errors.append(
                f"No module root found for {tests_file} "
                f"(expected sibling mod.rs or parent {stem.rsplit('_', 1)[0]}.rs)"
            )
            continue

        pattern = rf"\bmod\s+{re.escape(stem)}\b"
        registered = any(
            re.search(pattern, root.read_text(encoding="utf-8")) for root in module_roots
        )
        if not registered:
            roots_str = " or ".join(str(r) for r in module_roots)
            errors.append(
                f"{tests_file.name} not registered in {roots_str} "
                f"(missing `mod {stem};` declaration)"
            )

    return errors


def check() -> int:
    src = Path("src")
    if not src.exists():
        print("ERROR: src/ directory not found.")
        return 1

    errors: list[str] = []
    errors.extend(check_inline_test_blocks(src))
    errors.extend(check_tests_registration(src))

    tests_dir = Path("tests")
    if tests_dir.exists():
        errors.extend(check_tests_registration(tests_dir))

    if errors:
        print("TEST STRUCTURE VIOLATIONS:")
        for e in errors:
            print(f"  {e}")
        print(
            "\nAll unit tests must live in separate files, and every `*_tests.rs` "
            "file must be declared in its module root. See AGENTS.md for the standard."
        )
        return 1

    print("Test structure OK.")
    return 0


if __name__ == "__main__":
    sys.exit(check())
