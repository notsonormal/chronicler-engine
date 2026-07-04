"""Generate an auto-updating index for chronicler_engine/docs/AGENTS.md.

Scans all .md files in the docs directory, extracts H1 titles, and regenerates
the indexed section of AGENTS.md between AUTO-INDEX markers.
"""

import argparse
import sys
from datetime import UTC, datetime
from pathlib import Path

MARKER_START = "<!-- AUTO-INDEX START -->"
MARKER_END = "<!-- AUTO-INDEX END -->"


def extract_h1(md_path: Path) -> str | None:
    """Extract the first H1 heading from a markdown file."""
    try:
        with md_path.open("r", encoding="utf-8") as f:
            for line in f:
                stripped = line.strip()
                if stripped.startswith("# "):
                    return stripped[2:].strip()
    except OSError:
        pass
    return None


def discover_docs(docs_dir: Path) -> dict[str, list[tuple[str, str]]]:
    """Return a dict mapping relative dir -> list of (filename, h1_title)."""
    groups: dict[str, list[tuple[str, str]]] = {}
    for md_path in sorted(docs_dir.rglob("*.md")):
        if md_path.name.lower() == "agents.md":
            continue
        rel = md_path.relative_to(docs_dir)
        parent = str(rel.parent).replace("\\", "/")
        if parent == ".":
            parent = "(root)"
        h1 = extract_h1(md_path)
        title = h1 if h1 else md_path.stem.replace("_", " ").replace("-", " ").title()
        groups.setdefault(parent, []).append((rel.as_posix(), title))
    return groups


def generate_index(docs_dir: Path) -> str:
    """Generate the markdown index block."""
    groups = discover_docs(docs_dir)
    lines: list[str] = []
    lines.append("")
    lines.append(f"*Index last generated: {datetime.now(UTC).strftime('%Y-%m-%d %H:%M %Z')}*")
    lines.append("")

    dir_order = sorted(groups.keys(), key=lambda d: (d != "(root)", d))

    for directory in dir_order:
        files = sorted(groups[directory], key=lambda t: t[0].lower())
        if directory == "(root)":
            lines.append("### Root files")
        else:
            lines.append(f"### `docs/{directory}/`")
        lines.append("")
        for rel_path, title in files:
            lines.append(f"- [{title}](./{rel_path})")
        lines.append("")

    return "\n".join(lines) + "\n"


def _strip_timestamp(index_block: str) -> str:
    """Remove the timestamp line from an index block for comparison."""
    lines = index_block.splitlines()
    filtered = [line for line in lines if not line.strip().startswith("*Index last generated:")]
    return "\n".join(filtered)


def regenerate(readme_path: Path, docs_dir: Path) -> bool:
    """Regenerate README.md index. Returns True if file was written."""
    if not readme_path.exists():
        raise FileNotFoundError(f"README not found: {readme_path}")

    original = readme_path.read_text(encoding="utf-8")
    start_idx = original.find(MARKER_START)
    end_idx = original.find(MARKER_END)

    if start_idx == -1 or end_idx == -1:
        raise ValueError(
            f"Could not find AUTO-INDEX markers in {readme_path}. "
            f"Ensure both {MARKER_START!r} and {MARKER_END!r} are present."
        )

    before = original[: start_idx + len(MARKER_START)]
    after = original[end_idx:]
    old_index = original[start_idx + len(MARKER_START) : end_idx]
    new_index = generate_index(docs_dir)

    # Only write if the actual file listing changed, not just the timestamp.
    if _strip_timestamp(old_index) == _strip_timestamp(new_index):
        return False

    new_content = before + new_index + after
    readme_path.write_text(new_content, encoding="utf-8")
    return True


def check_only(readme_path: Path, docs_dir: Path) -> int:
    """Return 0 if index is up-to-date, 1 if stale."""
    try:
        if regenerate(readme_path, docs_dir):
            print("Index is stale. Run this script without --check to regenerate.")
            return 1
        print("Index is up-to-date.")
        return 0
    except (FileNotFoundError, ValueError) as e:
        print(f"Error: {e}")
        return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate docs index for README.md")
    parser.add_argument(
        "--docs-dir",
        type=Path,
        default=Path(__file__).resolve().parent.parent / "docs",
        help="Path to the docs directory (default: chronicler_engine/docs)",
    )
    parser.add_argument(
        "--readme",
        type=Path,
        default=None,
        help="Path to AGENTS.md (default: <docs-dir>/AGENTS.md)",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Exit with non-zero status if index is stale (do not write)",
    )
    args = parser.parse_args()

    docs_dir = args.docs_dir.resolve()
    readme_path = (args.readme or docs_dir / "AGENTS.md").resolve()

    if args.check:
        return check_only(readme_path, docs_dir)

    try:
        changed = regenerate(readme_path, docs_dir)
        if changed:
            print(f"Updated index in {readme_path}")
        else:
            print(f"Index already up-to-date in {readme_path}")
        return 0
    except (FileNotFoundError, ValueError) as e:
        print(f"Error: {e}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
