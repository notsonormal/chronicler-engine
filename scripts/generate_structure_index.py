"""Generate AGENTS.md structure index from module summaries."""

from __future__ import annotations

import re
from pathlib import Path


def extract_module_info(filepath: Path) -> tuple[str, str] | None:
    """Extract summary from a Rust file.

    Accepts two shapes:
      - `//! [DOC: ...]` on line 1 + `//! summary` on line 2 (anchor style)
      - `//! summary` on line 1 (anchor-less, e.g. src/test_support/ per AGENTS.md)

    Returns (filename, summary) or None if not a valid module.
    """
    try:
        lines = filepath.read_text(encoding="utf-8").splitlines()
        if not lines:
            return None

        line1 = lines[0].strip()
        if not line1.startswith("//!"):
            return None

        first = line1.removeprefix("//!").strip()

        if first.startswith("[DOC:"):
            if len(lines) < 2:
                return None
            line2 = lines[1].strip()
            if not line2.startswith("//!"):
                return None
            summary = line2.removeprefix("//!").strip()
        else:
            summary = first

        if not summary:
            return None

        return (filepath.name, summary)
    except Exception:
        return None


def build_bullet_structure(src_dir: Path) -> str:
    """Build a bullet-point representation of the source structure."""
    tree: dict = {}
    root_files: list[tuple[str, str]] = []

    for rs_file in src_dir.rglob("*.rs"):
        if rs_file.name.endswith("_tests.rs") or rs_file.name.endswith("_test.rs"):
            continue

        rel_path = rs_file.relative_to(src_dir)
        parts = [str(p) for p in rel_path.parts]

        info = extract_module_info(rs_file)
        if not info:
            continue

        if len(parts) == 1:
            root_files.append(info)
            continue

        node = tree
        for segment in parts[:-1]:
            node = node.setdefault(segment, {})
        node.setdefault("__files__", []).append(info)

    lines: list[str] = []
    lines.append("- **src/**")

    if root_files:
        for filename, summary in sorted(root_files):
            lines.append(f"  - `{filename}` — {summary}")

    for top_dir in sorted(tree.keys()):
        lines.append(f"  - **{top_dir}/**")
        # Top-level aggregator `mod.rs` files stay hidden; deeper ones render.
        _render_node(lines, tree[top_dir], indent=2, include_mod=False)

    return "\n".join(lines)


def _render_node(lines: list[str], node: dict, indent: int, include_mod: bool) -> None:
    """Recursively render one tree node: file bullets first, then subdirs."""
    pad = "  " * indent
    for name, summary in sorted(node.get("__files__", [])):
        if not include_mod and name == "mod.rs":
            continue
        lines.append(f"{pad}- `{name}` — {summary}")
    for subdir in sorted(k for k in node.keys() if k != "__files__"):
        lines.append(f"{pad}- **{subdir}/**")
        _render_node(lines, node[subdir], indent + 1, True)


def _extract_docstring_summary(path: Path) -> str:
    """Read the first line of a Python file's module docstring."""
    try:
        content = path.read_text(encoding="utf-8")
        match = re.search(r'"""(.+?)"""', content, re.DOTALL)
        return match.group(1).split("\n")[0].strip() if match else "No summary"
    except Exception:
        return "No summary"


def build_python_bullets(scripts_dir: Path, engine_dir: Path) -> str:
    """Build bullets for repo-root .py files plus the scripts/ directory."""
    lines: list[str] = []

    for py_file in sorted(engine_dir.glob("*.py")):
        lines.append(f"- `{py_file.name}` — {_extract_docstring_summary(py_file)}")

    lines.append("- **scripts/**")

    if not scripts_dir.exists():
        lines.append("  - (no scripts directory)")
        return "\n".join(lines)

    scripts: list[tuple[str, str]] = []
    for py_file in sorted(scripts_dir.glob("*.py")):
        scripts.append((py_file.name, _extract_docstring_summary(py_file)))

    for filename, summary in scripts:
        lines.append(f"  - `{filename}` — {summary}")

    return "\n".join(lines)


def main() -> int:
    """Main entry point."""
    engine_dir = Path(__file__).parent.parent
    src_dir = engine_dir / "src"
    scripts_dir = engine_dir / "scripts"
    agents_md = engine_dir / "AGENTS.md"

    if not agents_md.exists():
        print(f"AGENTS.md not found: {agents_md}")
        return 1

    content = agents_md.read_text(encoding="utf-8")

    rust_bullets = build_bullet_structure(src_dir)
    python_bullets = build_python_bullets(scripts_dir, engine_dir)

    new_structure = f"""## Structure
<!-- AUTO-STRUCTURE START -->
{rust_bullets}
{python_bullets}
<!-- AUTO-STRUCTURE END -->"""

    structure_match = re.search(
        r"## Structure\n<!-- AUTO-STRUCTURE START -->.*?<!-- AUTO-STRUCTURE END -->",
        content,
        re.DOTALL,
    )

    if not structure_match:
        structure_match = re.search(r"## Structure\n```.*?```", content, re.DOTALL)

    if not structure_match:
        structure_match = re.search(r"## Structure\n", content)

    if structure_match:
        old_structure = structure_match.group(0)
        new_content = content.replace(old_structure, new_structure)
        agents_md.write_text(new_content, encoding="utf-8")
        print("  Updated STRUCTURE section in AGENTS.md")
    else:
        print("  Warning: Could not find STRUCTURE section in AGENTS.md")
        return 1

    print("  Structure index regenerated successfully")
    return 0


if __name__ == "__main__":
    exit(main())
