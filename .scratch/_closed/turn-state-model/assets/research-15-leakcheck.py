"""Leak check for a clean-room specification: search inputs for `src/` identifiers.

Builds a list of distinctive code identifiers from `src/` and counts where each
input document names one. Interface names the player or an author can see
(data-file JSON keys) are subtracted, so they do not count as leaks.

Run it from the repository root, so that `src/` and `data/` resolve.

Usage: python .scratch/turn-state-model/assets/research-15-leakcheck.py FILE [FILE ...]
"""

import json
import pathlib
import re
import sys

ROOT = pathlib.Path.cwd()
SRC = ROOT / "src"
DATA = ROOT / "data"

DECL = re.compile(
    r"\b(?:fn|struct|enum|trait|type|const|static|mod)\s+([A-Za-z_][A-Za-z0-9_]*)"
)
FIELD = re.compile(r"^\s*(?:pub(?:\([a-z]+\))?\s+)?([a-z][a-z0-9]*_[a-z0-9_]+)\s*:", re.M)
VARIANT = re.compile(r"^\s+([A-Z][a-z0-9]+(?:[A-Z][a-z0-9]+)+)\s*[,({]", re.M)
CAMEL_MULTI = re.compile(r"^[A-Z][a-z0-9]+(?:[A-Z][a-z0-9]*)+$")
SNAKE = re.compile(r"^[a-z][a-z0-9]*(?:_[a-z0-9]+)+$")


def code_identifiers() -> set[str]:
    names: set[str] = set()
    for path in SRC.rglob("*.rs"):
        text = path.read_text(encoding="utf-8", errors="replace")
        names.update(DECL.findall(text))
        names.update(FIELD.findall(text))
        names.update(VARIANT.findall(text))
        names.add(path.stem)
    keep = set()
    for name in names:
        if SNAKE.match(name) and len(name) >= 6:
            keep.add(name)
        elif CAMEL_MULTI.match(name):
            keep.add(name)
    return keep


def data_keys() -> set[str]:
    keys: set[str] = set()

    def walk(node):
        if isinstance(node, dict):
            for key, value in node.items():
                keys.add(key)
                walk(value)
        elif isinstance(node, list):
            for item in node:
                walk(item)

    for path in DATA.rglob("*.json"):
        try:
            walk(json.loads(path.read_text(encoding="utf-8")))
        except (ValueError, UnicodeDecodeError):
            continue
    return keys


def main(files: list[str]) -> None:
    if not SRC.is_dir():
        sys.exit("run from the repository root: src/ not found")
    identifiers = code_identifiers() - data_keys()
    pattern = re.compile(
        r"(?<![A-Za-z0-9_])(" + "|".join(sorted(map(re.escape, identifiers), key=len, reverse=True)) + r")(?![A-Za-z0-9_])"
    )
    path_pattern = re.compile(r"(?<![/\w])src/[A-Za-z0-9_/]+(?:\.rs)?|\b[a-z_]+\.rs\b")
    print(f"# identifiers in list: {len(identifiers)}")
    print("file\twords\tdistinct_ids\tid_hits\tsrc_paths\tid_hits_per_1k_words\texamples")
    for name in files:
        text = pathlib.Path(name).read_text(encoding="utf-8", errors="replace")
        words = max(len(text.split()), 1)
        hits = pattern.findall(text)
        paths = path_pattern.findall(text)
        distinct = sorted(set(hits), key=lambda h: -hits.count(h))
        rate = 1000 * len(hits) / words
        print(f"{name}\t{words}\t{len(distinct)}\t{len(hits)}\t{len(paths)}\t{rate:.1f}\t{', '.join(distinct[:8])}")


if __name__ == "__main__":
    main(sys.argv[1:])
