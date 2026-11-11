"""Shared link code of the work records: find relative links and rewrite them through a table of moves.

A link is a relative markdown link `[text](target)`, an image link
`![alt](target)` or a reference definition `[label]: target`. Links inside
fenced code and HTML comments are not links: a template placeholder such as
`(link)` sits in a comment, and code shows text, not a reference.

A **move table** maps old absolute paths (files or folders) to new ones. A
path under a moved folder moves with it; the longest matching entry wins.
"""

from __future__ import annotations

import os
import re
from collections.abc import Iterator, Mapping
from pathlib import Path
from typing import NamedTuple
from urllib.parse import quote, unquote

from validate_docs import FENCE_DELIMITER, MARKDOWN_LINK, URI_SCHEMES

IMAGE_LINK = re.compile(r"!\[[^\]]*\]\(([^)]+)\)")
REFERENCE_DEFINITION = re.compile(r"^ {0,3}\[[^\]]+\]:[ \t]+(\S+)")
HTML_COMMENT_OPEN = "<!--"
HTML_COMMENT_CLOSE = "-->"

# The status folders of `work/issues/`. The folder of a one-off issue is its status.
STATUS_FOLDERS = ("pending", "ready", "in-progress", "closed")

BACKTICK_SPAN = re.compile(r"`([^`\n]+)`")
# The lookbehind keeps a match from starting inside another path, such as `../work/x`.
RECORD_PATH_TOKEN = re.compile(r"(?<![\w/.-])(work(?:/[^\s`'\")\],;:]*)?)")


class Link(NamedTuple):
    """One relative link: the target text and its span in the whole text."""

    target: str
    start: int
    end: int
    line: int


class SplitTarget(NamedTuple):
    """A link target cut into the path and the `#anchor`/`?query` tail."""

    path: str
    suffix: str


def split_target(target: str) -> SplitTarget:
    """The path part may be empty."""
    cut = len(target)
    for mark in ("#", "?"):
        index = target.find(mark)
        if index != -1:
            cut = min(cut, index)
    return SplitTarget(target[:cut], target[cut:])


def is_relative_target(target: str) -> bool:
    if not target or target.startswith(("#", "/", "<")):
        return False
    if target.startswith(URI_SCHEMES):
        return False
    return re.match(r"^[A-Za-z][A-Za-z0-9+.-]*:", target) is None


def _visible_spans(line: str, in_comment: bool) -> tuple[list[tuple[int, int]], bool]:
    """The spans of `line` that sit outside HTML comments, and whether a comment is open at its end."""
    spans: list[tuple[int, int]] = []
    position = 0
    while position <= len(line):
        if in_comment:
            close = line.find(HTML_COMMENT_CLOSE, position)
            if close == -1:
                return spans, True
            position = close + len(HTML_COMMENT_CLOSE)
            in_comment = False
            continue
        opening = line.find(HTML_COMMENT_OPEN, position)
        if opening == -1:
            spans.append((position, len(line)))
            return spans, False
        spans.append((position, opening))
        position = opening + len(HTML_COMMENT_OPEN)
        in_comment = True
    return spans, in_comment


def _link_target(raw: str, raw_start: int) -> tuple[str, int, int]:
    """A title after a space is not part of the target."""
    stripped = raw.lstrip()
    lead = len(raw) - len(stripped)
    token = stripped.split(" ", 1)[0].split("\t", 1)[0]
    start = raw_start + lead
    return token, start, start + len(token)


class TextLine(NamedTuple):
    """One line outside fenced code: its number, its offset in the whole text, and the spans outside HTML comments."""

    number: int
    offset: int
    text: str
    visible: list[tuple[int, int]]


def lines_outside_fences(text: str) -> Iterator[TextLine]:
    """Every line of `text` that is not fenced code, with the spans of it outside HTML comments.

    A fence delimiter inside an HTML comment does not open or close a fence.
    """
    in_fence = False
    in_comment = False
    offset = 0
    for lineno, line in enumerate(text.splitlines(keepends=True), start=1):
        line_start = offset
        offset += len(line)
        if not in_comment and FENCE_DELIMITER.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        spans, in_comment = _visible_spans(line, in_comment)
        yield TextLine(lineno, line_start, line, spans)


def iter_links(text: str) -> Iterator[Link]:
    """Every relative link of `text` outside fenced code and HTML comments, in text order."""
    for lineno, line_start, line, spans in lines_outside_fences(text):
        found: list[Link] = []
        for span_start, span_end in spans:
            segment = line[span_start:span_end]
            for pattern in (MARKDOWN_LINK, IMAGE_LINK):
                for match in pattern.finditer(segment):
                    token, start, end = _link_target(match.group(1), match.start(1))
                    found.append(
                        Link(token, line_start + span_start + start, line_start + span_start + end, lineno)
                    )
            if span_start == 0:
                definition = REFERENCE_DEFINITION.match(segment)
                if definition:
                    found.append(
                        Link(
                            definition.group(1),
                            line_start + definition.start(1),
                            line_start + definition.end(1),
                            lineno,
                        )
                    )
        found.sort(key=lambda link: link.start)
        yield from (link for link in found if is_relative_target(link.target))


def resolve_target(file: Path, target: str) -> Path | None:
    """The absolute path a relative link of `file` names, or None for an anchor-only link."""
    path = split_target(target).path
    if not path:
        return None
    return Path(os.path.normpath(file.parent / unquote(path)))


def link_resolves(file: Path, target: str) -> bool:
    """True when the path of the link exists on disk (an anchor-only link always resolves)."""
    resolved = resolve_target(file, target)
    return resolved is None or resolved.exists()


def map_path(path: Path, moves: Mapping[Path, Path]) -> Path:
    """Where `path` lives after the moves; the longest moved prefix wins."""
    for candidate in (path, *path.parents):
        if candidate in moves:
            return moves[candidate] / path.relative_to(candidate)
    return path


def _relative_link(target_path: Path, from_folder: Path, keep_trailing_slash: bool, encode: bool) -> str:
    text = os.path.relpath(target_path, from_folder).replace(os.sep, "/")
    if keep_trailing_slash and not text.endswith("/"):
        text += "/"
    return quote(text, safe="/._-~+()&=,!@$'*") if encode else text


def rewrite_link_target(file: Path, target: str, moves: Mapping[Path, Path]) -> str:
    """The text of `target` for `file` after the moves.

    The target resolves against the old place of `file` and is written
    relative to its new place. An unresolvable target, or one that still names
    the same path, is returned byte for byte.
    """
    old_target = resolve_target(file, target)
    if old_target is None:
        return target
    if not old_target.exists():
        return target
    new_file = map_path(file, moves)
    new_target = map_path(old_target, moves)
    split = split_target(target)
    if Path(os.path.normpath(new_file.parent / unquote(split.path))) == new_target:
        return target
    rewritten = _relative_link(new_target, new_file.parent, split.path.endswith("/"), "%" in split.path)
    return rewritten + split.suffix


def rewrite_links_in_text(file: Path, text: str, moves: Mapping[Path, Path]) -> str:
    pieces: list[str] = []
    cursor = 0
    for link in iter_links(text):
        replacement = rewrite_link_target(file, link.target, moves)
        if replacement == link.target:
            continue
        pieces.append(text[cursor : link.start])
        pieces.append(replacement)
        cursor = link.end
    pieces.append(text[cursor:])
    return "".join(pieces)


def _rewrite_path_token(token: str, moves: Mapping[Path, Path], root: Path) -> str:
    trailing = "/" if token.endswith("/") else ""
    old = Path(os.path.normpath(root / token))
    new = map_path(old, moves)
    if new == old:
        return token
    text = new.relative_to(root).as_posix()
    return text + trailing if not text.endswith("/") else text


def rewrite_path_tokens(
    text: str, moves: Mapping[Path, Path], root: Path, token_pattern: re.Pattern[str] = RECORD_PATH_TOKEN
) -> str:
    """Every repo path token of `text` (`work/…` by default) rewritten through the moves.

    A token whose prefix moved changes even when the full path no longer
    exists, so a citation of a deleted file still names the new folder. A
    token with no moved prefix, such as a bare `work/` or a placeholder
    `work/<map>/`, stays. `token_pattern` finds the tokens; group 1 is the path.
    """
    return token_pattern.sub(lambda match: _rewrite_path_token(match.group(1), moves, root), text)


def rewrite_backticked(
    text: str, moves: Mapping[Path, Path], root: Path, token_pattern: re.Pattern[str] = RECORD_PATH_TOKEN
) -> str:
    """Repo path tokens inside backticked spans outside fenced code, rewritten through the moves."""
    lines = {line.number: line for line in lines_outside_fences(text)}
    out: list[str] = []
    for lineno, line in enumerate(text.splitlines(keepends=True), start=1):
        if lineno not in lines:
            out.append(line)
            continue
        out.append(
            BACKTICK_SPAN.sub(
                lambda match: "`" + rewrite_path_tokens(match.group(1), moves, root, token_pattern) + "`",
                line,
            )
        )
    return "".join(out)


def citing_files(root: Path) -> list[Path]:
    """The files that cite `work/…` paths: `AGENTS.md`, `docs/**/*.md` and every file under `.agents/skills/`."""
    files: set[Path] = set()
    agents = root / "AGENTS.md"
    if agents.is_file():
        files.add(agents)
    files.update((root / "docs").rglob("*.md"))
    files.update(path for path in (root / ".agents" / "skills").rglob("*") if path.is_file())
    return sorted(files)
