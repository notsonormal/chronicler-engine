"""Generate tests/AGENTS.md structure index from module summaries."""

from __future__ import annotations

import re
from collections import defaultdict
from pathlib import Path


def extract_module_info(filepath: Path) -> str | None:
    """Extract summary text from a test file.

    Accepts both shapes:
      - `//! <summary>` on line 1 (no DOC anchor)
      - `//! [DOC: <path>]` on line 1 + `//! <summary>` on line 2
    Returns the summary text, or None if the file does not have a valid header.
    """
    try:
        lines = filepath.read_text(encoding="utf-8").splitlines()
    except Exception:
        return None

    if not lines:
        return None

    first = next((ln for ln in lines if ln.strip()), None)
    if first is None or not first.lstrip().startswith("//!"):
        return None

    first_text = first.lstrip()[3:].lstrip()

    if first_text.startswith("[DOC:"):
        idx = lines.index(first)
        for nxt in lines[idx + 1 :]:
            if not nxt.strip():
                continue
            if nxt.lstrip().startswith("//!"):
                return nxt.lstrip()[3:].lstrip()
            return None
        return None

    return first_text


def build_bullet_structure(tests_dir: Path) -> str:
    """Build a bullet tree grouped by test binary (top-level subdir of `tests/`)."""
    tree: dict = {}
    top_level_files: list[tuple[str, str]] = []

    for rs_file in sorted(tests_dir.rglob("*.rs")):
        rel_path = rs_file.relative_to(tests_dir)
        parts = [str(p) for p in rel_path.parts]
        if len(parts) == 0:
            continue

        info = extract_module_info(rs_file)
        if info is None:
            continue

        if len(parts) == 1:
            top_level_files.append((parts[0], info))
            continue

        node = tree
        for segment in parts[:-1]:
            node = node.setdefault(segment, {})
        node.setdefault("__files__", []).append((parts[-1], info))

    lines: list[str] = []

    if top_level_files:
        for name, summary in sorted(top_level_files):
            lines.append(f"- `{name}` — {summary}")

    for top_dir in sorted(tree.keys()):
        lines.append(f"- **{top_dir}/**")
        _render_node(lines, tree[top_dir], indent=2)

    return "\n".join(lines)


def _render_node(lines: list[str], node: dict, indent: int) -> None:
    """Recursively render a tree node: file leaves first (sorted), then subdirs."""
    pad = "  " * indent
    for name, summary in sorted(node.get("__files__", [])):
        lines.append(f"{pad}- `{name}` — {summary}")
    for subdir in sorted(k for k in node.keys() if k != "__files__"):
        lines.append(f"{pad}- **{subdir}/**")
        _render_node(lines, node[subdir], indent + 1)


def main() -> int:
    engine_dir = Path(__file__).parent.parent
    tests_dir = engine_dir / "tests"
    target = engine_dir / "tests" / "AGENTS.md"

    if not tests_dir.exists():
        print(f"tests/ not found: {tests_dir}")
        return 1

    bullets = build_bullet_structure(tests_dir)
    new_body = f"""## STRUCTURE
<!-- AUTO-STRUCTURE-TESTS START -->
{bullets}
<!-- AUTO-STRUCTURE-TESTS END -->"""

    if target.exists():
        content = target.read_text(encoding="utf-8")
        pattern = re.compile(
            r"## STRUCTURE\n<!-- AUTO-STRUCTURE-TESTS START -->.*?<!-- AUTO-STRUCTURE-TESTS END -->",
            re.DOTALL,
        )
        if pattern.search(content):
            new_content = pattern.sub(new_body, content)
        else:
            sep = "\n" if not content.endswith("\n") else ""
            new_content = content + sep + "\n" + new_body + "\n"
    else:
        new_content = new_body + "\n"

    target.write_text(new_content, encoding="utf-8")
    print(f"  Updated STRUCTURE section in {target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())