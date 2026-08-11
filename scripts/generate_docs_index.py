"""Generate an auto-updating index for docs/AGENTS.md.

Scans .md files in `docs/diataxis/` only (other docs subfolders are
intentionally excluded: external_applications/, plans/, specs/ live under
docs/ alongside diataxis/ and are not part of the discoverable documentation),
extracts H1 titles, and regenerates the indexed section of AGENTS.md between
AUTO-INDEX markers.
"""

import argparse
import sys
from datetime import UTC, datetime
from pathlib import Path

MARKER_START = "<!-- AUTO-INDEX START -->"
MARKER_END = "<!-- AUTO-INDEX END -->"

DOCS_DIR: Path = Path(__file__).resolve().parent.parent / "docs"
README_PATH: Path = DOCS_DIR / "AGENTS.md"

# Files in docs/ excluded from the auto-index because they don't fit the
# "documentation discoverable by type" pattern (e.g. release logs, migration
# journals — reference material, not discovery material). Silent skip.
EXCLUDED_FROM_NAV: set[str] = {
    "CHANGELOG.md",
}


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


def discover_docs() -> dict[str, list[tuple[str, str]]]:
    """Return a dict mapping relative dir -> list of (filename, h1_title)."""
    groups: dict[str, list[tuple[str, str]]] = {}
    for md_path in sorted(DOCS_DIR.rglob("*.md")):
        if md_path.name.lower() == "agents.md":
            continue
        if md_path.name in EXCLUDED_FROM_NAV:
            continue
        rel = md_path.relative_to(DOCS_DIR)
        # Index scope: only docs/diataxis/. Other top-level dirs
        # (external_applications/, plans/, specs/, plus any newly added
        # root-level docs) are not part of the auto-generated catalogue.
        if not rel.parts or rel.parts[0] != "diataxis":
            continue
        parent = str(rel.parent).replace("\\", "/")
        if parent == ".":
            parent = "(root)"
        h1 = extract_h1(md_path)
        title = h1 if h1 else md_path.stem.replace("_", " ").replace("-", " ").title()
        groups.setdefault(parent, []).append((rel.as_posix(), title))
    return groups


def generate_index() -> str:
    """Generate the markdown index block."""
    groups = discover_docs()
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


def regenerate() -> bool:
    """Regenerate AGENTS.md index. Returns True if file was written."""
    if not README_PATH.exists():
        raise FileNotFoundError(f"README not found: {README_PATH}")

    original = README_PATH.read_text(encoding="utf-8")
    start_idx = original.find(MARKER_START)
    end_idx = original.find(MARKER_END)

    if start_idx == -1 or end_idx == -1:
        raise ValueError(
            f"Could not find AUTO-INDEX markers in {README_PATH}. "
            f"Ensure both {MARKER_START!r} and {MARKER_END!r} are present."
        )

    before = original[: start_idx + len(MARKER_START)]
    after = original[end_idx:]
    old_index = original[start_idx + len(MARKER_START) : end_idx]
    new_index = generate_index()

    # Only write if the actual file listing changed, not just the timestamp.
    if _strip_timestamp(old_index) == _strip_timestamp(new_index):
        return False

    new_content = before + new_index + after
    README_PATH.write_text(new_content, encoding="utf-8")
    return True


def check_only() -> int:
    """Return 0 if index is up-to-date, 1 if stale."""
    try:
        if regenerate():
            print("Index is stale. Run this script without --check to regenerate.")
            return 1
        print("Index is up-to-date.")
        return 0
    except (FileNotFoundError, ValueError) as e:
        print(f"Error: {e}")
        return 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate docs index for AGENTS.md")
    parser.add_argument(
        "--check",
        action="store_true",
        help="Exit with non-zero status if index is stale (do not write)",
    )
    args = parser.parse_args()

    if args.check:
        return check_only()

    try:
        changed = regenerate()
        if changed:
            print(f"Updated index in {README_PATH}")
        else:
            print(f"Index already up-to-date in {README_PATH}")
        return 0
    except (FileNotFoundError, ValueError) as e:
        print(f"Error: {e}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
