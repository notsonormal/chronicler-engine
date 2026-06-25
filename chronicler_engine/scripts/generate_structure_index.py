"""Generate AGENTS.md structure index from module summaries."""

from __future__ import annotations

import re
from collections import defaultdict
from pathlib import Path


def extract_module_info(filepath: Path) -> tuple[str, str] | None:
    """Extract DOC anchor and summary from a Rust file.

    Returns (filename, summary) or None if not a valid module.
    """
    try:
        lines = filepath.read_text(encoding="utf-8").splitlines()
        if len(lines) < 2:
            return None

        line1 = lines[0].strip()
        line2 = lines[1].strip()

        if not line1.startswith("//! [DOC:"):
            return None

        if not line2.startswith("//!"):
            return None

        summary = line2.removeprefix("//!").strip()

        return (filepath.name, summary)
    except Exception:
        return None


def build_bullet_structure(src_dir: Path) -> str:
    """Build a bullet-point representation of the source structure."""
    modules: dict[str, list[tuple[str, str]]] = defaultdict(list)

    for rs_file in src_dir.rglob("*.rs"):
        if rs_file.name.endswith("_tests.rs") or rs_file.name.endswith("_test.rs"):
            continue

        rel_path = rs_file.relative_to(src_dir)
        parts = [str(p) for p in rel_path.parts]

        info = extract_module_info(rs_file)
        if not info:
            continue

        filename, summary = info
        dir_path = "/".join(parts[:-1]) if len(parts) > 1 else "__root__"
        modules[dir_path].append((filename, summary))

    lines: list[str] = []
    lines.append("- **src/**")

    all_dirs = set(modules.keys())
    all_dirs.discard("__root__")

    top_level_dirs = set()
    for dir_path in all_dirs:
        top = dir_path.split("/")[0]
        top_level_dirs.add(top)

    if "__root__" in modules:
        root_files = sorted(modules["__root__"])
        for filename, summary in root_files:
            lines.append(f"  - `{filename}` — {summary}")

    for top_dir in sorted(top_level_dirs):
        lines.append(f"  - **{top_dir}/**")

        direct_files = sorted([f for f in modules.get(top_dir, []) if f[0] != "mod.rs"])
        for filename, summary in direct_files:
            lines.append(f"    - `{filename}` — {summary}")

        subdirs = set()
        for dir_path in all_dirs:
            if dir_path.startswith(top_dir + "/"):
                subdir = dir_path.replace(top_dir + "/", "")
                if "/" not in subdir:
                    subdirs.add(subdir)

        for subdir in sorted(subdirs):
            full_path = f"{top_dir}/{subdir}"
            lines.append(f"    - **{subdir}/**")

            files = sorted(modules.get(full_path, []))
            for filename, summary in files:
                lines.append(f"      - `{filename}` — {summary}")

            nested_subdirs = set()
            for dir_path in all_dirs:
                if dir_path.startswith(full_path + "/"):
                    nested = dir_path.replace(full_path + "/", "")
                    if "/" not in nested:
                        nested_subdirs.add(nested)

            for nested in sorted(nested_subdirs):
                nested_path = f"{full_path}/{nested}"
                lines.append(f"      - **{nested}/**")
                nested_files = sorted(modules.get(nested_path, []))
                for filename, summary in nested_files:
                    lines.append(f"        - `{filename}` — {summary}")

    return "\n".join(lines)


def build_python_bullets(scripts_dir: Path, engine_dir: Path) -> str:
    """Build a bullet-point representation of Python scripts."""
    lines: list[str] = []
    lines.append("- **scripts/**")

    if not scripts_dir.exists():
        lines.append("  - (no scripts directory)")
        return "\n".join(lines)

    scripts: list[tuple[str, str]] = []

    build_py = engine_dir / "build.py"
    if build_py.exists():
        try:
            content = build_py.read_text(encoding="utf-8")
            match = re.search(r'"""(.+?)"""', content, re.DOTALL)
            summary = match.group(1).split("\n")[0].strip() if match else "No summary"
            scripts.append(("build.py", summary))
        except Exception:
            scripts.append(("build.py", "No summary"))

    for py_file in sorted(scripts_dir.glob("*.py")):
        try:
            content = py_file.read_text(encoding="utf-8")
            match = re.search(r'"""(.+?)"""', content, re.DOTALL)
            summary = match.group(1).split("\n")[0].strip() if match else "No summary"
            scripts.append((py_file.name, summary))
        except Exception:
            scripts.append((py_file.name, "No summary"))

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

    new_structure = f"""## STRUCTURE
<!-- AUTO-STRUCTURE START -->
{rust_bullets}
{python_bullets}
<!-- AUTO-STRUCTURE END -->"""

    structure_match = re.search(
        r"## STRUCTURE\n<!-- AUTO-STRUCTURE START -->.*?<!-- AUTO-STRUCTURE END -->",
        content,
        re.DOTALL,
    )

    if not structure_match:
        structure_match = re.search(r"## STRUCTURE\n```.*?```", content, re.DOTALL)

    if not structure_match:
        structure_match = re.search(r"## STRUCTURE\n", content)

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
