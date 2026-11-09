"""The staleness warning of the verify skills, and the command that records the hashes.

Each verify skill (``.agents/skills/verify-*/``) holds a ``coverage.toml`` that lists the Rust
functions its TLA+ model describes. Each listed function can carry a hash of its text, recorded
when an agent last confirmed that the TLA+ model follows the code::

    { name = "GameWriter::stop", hash = "0123456789abcdef" },

``check`` hashes each listed function as it is on disk and compares. A function warns when its
hash differs, when it has no hash, or when it is not found (renamed, moved, deleted) or not unique.
An edit elsewhere in a covered file gives no warning. ``record`` writes the hashes of the
functions as they are now: the maintain pass runs it after the TLA+ model follows the code.
Neither mode reads git, so uncommitted work counts.

A function is keyed as ``coverage.toml`` names it: ``Type::fn`` for a function of ``impl Type``,
``Trait for Type::fn`` for a function of ``impl Trait for Type``, ``Trait for Type`` for that whole
trait impl block (``Drop for JobWriter``), and ``fn`` for a free function. The hashed text of a
function runs from the start of the line that holds the ``fn`` keyword to the closing brace of
its body. The hashed text of a trait impl block runs from the start of its ``impl`` line. Attributes
and doc comments above it are not part of it. Comments, strings and char literals are blanked
before the brace count, so a brace inside one cannot move the bounds.

Usage::

    python scripts/verify_coverage.py check
    python scripts/verify_coverage.py record [--function NAME ...]
"""

from __future__ import annotations

import argparse
import hashlib
import re
import sys
import tomllib
from pathlib import Path
from typing import NamedTuple

COVERAGE_GLOB = ".agents/skills/verify-*/coverage.toml"
NAME_LIMIT = 4
HASH_LENGTH = 16
MAINTAIN_SKILL = "/maintain-formal-verification-skill"
RECORD_COMMAND = "python scripts/verify_coverage.py record"

_RAW_STRING = re.compile(r'b?r(#*)"')
_ITEM = re.compile(r"\{|\bimpl\b|\bfn\s+(\w+)")
_MEMBER = re.compile(r"\{|\bfn\s+(\w+)")
_ENTRY_LINE = re.compile(
    r'^(?P<indent>\s*)(?:"(?P<plain>[^"]+)"'
    r'|\{\s*name\s*=\s*"(?P<named>[^"]+)"(?:\s*,\s*hash\s*=\s*"[0-9a-f]+")?\s*\})\s*,?\s*$'
)
_PATH_LINE = re.compile(r'^\s*path\s*=\s*"(?P<path>[^"]+)"')


class Entry(NamedTuple):
    name: str
    hash: str | None


class SkillStatus(NamedTuple):
    skill: str
    changed: list[str]
    missing: list[str]
    unhashed: list[str]
    unreadable: str | None = None

    @property
    def stale(self) -> bool:
        return bool(self.changed or self.missing or self.unhashed or self.unreadable)


def hash_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()[:HASH_LENGTH]


def _char_literal_end(src: str, i: int) -> int:
    """0 when the quote starts a lifetime."""
    if src.startswith("\\", i + 1):
        j = src.find("'", i + 3)
        return j + 1 if j >= 0 else 0
    if i + 2 < len(src) and src[i + 2] == "'":
        return i + 3
    return 0


def blank_literals(src: str) -> str:
    """The result has the length of the source, and each newline stays, so an index of one is an index of the other."""
    out = list(src)
    n = len(src)

    def blank(a: int, b: int) -> None:
        for k in range(a, b):
            if out[k] != "\n":
                out[k] = " "

    i = 0
    while i < n:
        c = src[i]
        raw = _RAW_STRING.match(src, i) if c in "br" and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == "_")) else None
        if src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            blank(i, j)
            i = j
        elif src.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif src.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            blank(i, j)
            i = j
        elif raw:
            close = '"' + raw.group(1)
            j = src.find(close, raw.end())
            j = n if j < 0 else j + len(close)
            blank(i, j)
            i = j
        elif c == '"':
            j = i + 1
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            j = min(j + 1, n)
            blank(i, j)
            i = j
        elif c == "'" and (j := _char_literal_end(src, i)):
            blank(i, j)
            i = j
        else:
            i += 1
    return "".join(out)


def _match_brace(s: str, open_at: int) -> int:
    depth = 0
    for j in range(open_at, len(s)):
        if s[j] == "{":
            depth += 1
        elif s[j] == "}":
            depth -= 1
            if depth == 0:
                return j
    return len(s) - 1


def _body_start(s: str, i: int) -> int | None:
    """None when a ``;`` ends the item first. A ``;`` inside brackets, as in ``[u8; 4]``, does not count."""
    depth = 0
    for j in range(i, len(s)):
        c = s[j]
        if c in "([":
            depth += 1
        elif c in ")]":
            depth -= 1
        elif depth == 0 and c == "{":
            return j
        elif depth == 0 and c == ";":
            return None
    return None


def _strip_generics(text: str) -> str:
    """The text must start with the ``<`` of the group."""
    depth = 0
    for j, c in enumerate(text):
        if c == "<":
            depth += 1
        elif c == ">" and text[j - 1] != "-":
            depth -= 1
            if depth == 0:
                return text[j + 1 :].strip()
    return text


def _base_name(type_text: str) -> str:
    return type_text.split("<", 1)[0].strip().rsplit("::", 1)[-1].strip()


def _impl_key(header: str) -> str:
    h = " ".join(header.split())
    if h.startswith("<"):
        h = _strip_generics(h)
    h = re.split(r"\swhere\b", h, maxsplit=1)[0]
    parts = re.split(r"\bfor\b", h, maxsplit=1)
    if len(parts) == 2:
        return f"{_base_name(parts[0])} for {_base_name(parts[1])}"
    return _base_name(h)


def find_functions(source: str) -> dict[str, list[str]]:
    """Finds only items at depth 0. A key that has two texts is ambiguous."""
    s = blank_literals(source)
    found: dict[str, list[str]] = {}

    def add(key: str, fn_at: int, body_open: int) -> int:
        end = _match_brace(s, body_open)
        found.setdefault(key, []).append(source[s.rfind("\n", 0, fn_at) + 1 : end + 1])
        return end

    pos = 0
    while m := _ITEM.search(s, pos):
        if m.group() == "{":
            pos = _match_brace(s, m.start()) + 1
            continue
        body = _body_start(s, m.end())
        if body is None:
            pos = m.end()
        elif m.group(1):
            pos = add(m.group(1), m.start(), body) + 1
        else:
            key = _impl_key(s[m.end() : body])
            close = _match_brace(s, body)
            p = body + 1
            while inner := _MEMBER.search(s, p, close):
                if inner.group() == "{":
                    p = _match_brace(s, inner.start()) + 1
                    continue
                fn_body = _body_start(s, inner.end())
                if fn_body is None:
                    p = inner.end()
                else:
                    p = add(f"{key}::{inner.group(1)}", inner.start(), fn_body) + 1
            if " for " in key:
                found.setdefault(key, []).append(source[s.rfind("\n", 0, m.start()) + 1 : close + 1])
            pos = close + 1
    return found


def _load(coverage: Path) -> list[tuple[str, list[Entry]]]:
    data = tomllib.loads(coverage.read_text(encoding="utf-8"))
    files = []
    for item in data.get("files", []):
        entries = []
        for f in item["functions"]:
            entries.append(Entry(f, None) if isinstance(f, str) else Entry(f["name"], f.get("hash")))
        files.append((item["path"], entries))
    return files


def _coverage_files(root: Path) -> list[Path]:
    return sorted(root.glob(COVERAGE_GLOB))


def _source_functions(root: Path, rel: str) -> dict[str, list[str]] | None:
    path = root / rel
    if not path.is_file():
        return None
    return find_functions(path.read_text(encoding="utf-8"))


def check_skill(root: Path, coverage: Path) -> SkillStatus:
    skill = coverage.parent.name
    try:
        files = _load(coverage)
    except Exception as e:  # A malformed coverage.toml is one notice, not a crash.
        return SkillStatus(skill, [], [], [], unreadable=f"coverage.toml cannot be read ({type(e).__name__}: {e})")
    changed, missing, unhashed = [], [], []
    for rel, entries in files:
        functions = _source_functions(root, rel)
        for entry in entries:
            texts = (functions or {}).get(entry.name, [])
            if len(texts) != 1:
                missing.append(entry.name)
            elif entry.hash is None:
                unhashed.append(entry.name)
            elif hash_text(texts[0]) != entry.hash:
                changed.append(entry.name)
    return SkillStatus(skill, changed, missing, unhashed)


def check(root: Path) -> list[SkillStatus]:
    return [check_skill(root, c) for c in _coverage_files(root)]


def _names(names: list[str]) -> str:
    shown = ", ".join(names[:NAME_LIMIT])
    return shown + (f" and {len(names) - NAME_LIMIT} more" if len(names) > NAME_LIMIT else "")


def warning_lines(root: Path) -> list[str]:
    lines = []
    for status in check(root):
        if not status.stale:
            continue
        if status.unreadable:
            lines.append(f"WARNING {status.skill}: {status.unreadable}")
            continue
        parts = [
            f"{label}: {_names(names)}"
            for label, names in (
                ("changed", status.changed),
                ("not found or not unique", status.missing),
                ("no hash", status.unhashed),
            )
            if names
        ]
        lines.append(f"WARNING {status.skill}: the TLA+ model may not match the code ({'; '.join(parts)}).")
    if lines:
        lines.append(f"The agent that changed a listed function runs {MAINTAIN_SKILL} before it ends its task.")
        lines.append(f"Then record the new hashes: {RECORD_COMMAND}")
    return lines


def record(root: Path, only: set[str] | None = None) -> list[str]:
    """Returns a note for each function not recorded: not found, or not unique."""
    notes = []
    for coverage in _coverage_files(root):
        cache: dict[str, dict[str, list[str]] | None] = {}
        out, path, in_functions, changed, seen = [], None, False, False, set()
        for line in coverage.read_text(encoding="utf-8").splitlines(keepends=True):
            stripped = line.strip()
            if m := _PATH_LINE.match(line):
                path = m.group("path")
            if re.match(r"functions\s*=\s*\[\s*$", stripped):
                in_functions = True
            elif in_functions and stripped.startswith("]"):
                in_functions = False
            entry = _ENTRY_LINE.match(line) if in_functions and path else None
            name = entry and (entry.group("plain") or entry.group("named"))
            if not name or (only is not None and name not in only):
                out.append(line)
                continue
            seen.add((path, name))
            if path not in cache:
                cache[path] = _source_functions(root, path)
            texts = (cache[path] or {}).get(name, [])
            if len(texts) != 1:
                notes.append(f"{name}: not recorded, {'not found' if not texts else 'not unique'} in {path}")
                out.append(line)
                continue
            new = f'{entry.group("indent")}{{ name = "{name}", hash = "{hash_text(texts[0])}" }},\n'
            changed = changed or new != line
            out.append(new)
        for rel, entries in _load(coverage):
            for entry in entries:
                if (rel, entry.name) not in seen and (only is None or entry.name in only):
                    notes.append(f"{entry.name}: not recorded, write it on its own line of a multi-line functions list in {rel}")
        if changed:
            coverage.write_text("".join(out), encoding="utf-8")
    return notes


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = parser.add_subparsers(dest="mode", required=True)
    sub.add_parser("check", help="Print the staleness warning, if any. Always exits 0.")
    rec = sub.add_parser("record", help="Record the hash of each listed function.")
    rec.add_argument("--function", action="append", dest="functions", help="Record only this function (repeatable).")
    args = parser.parse_args(argv)
    root = Path(__file__).resolve().parents[1]
    if args.mode == "check":
        lines = warning_lines(root)
        print("\n".join(lines) if lines else "verify coverage: every listed function matches its recorded hash.")
        return 0
    notes = record(root, set(args.functions) if args.functions else None)
    print("\n".join(notes) if notes else "verify coverage: hashes recorded.")
    return 1 if notes else 0


if __name__ == "__main__":
    sys.exit(main())
