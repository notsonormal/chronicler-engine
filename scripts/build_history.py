"""The build-history journal of ``build.py``: one record per run, and the tree each run covered.

``logs/build_history.txt`` is append-only. Each record is one pipe-delimited line with the
columns of ``HEADER``: timestamp, duration, exit code, the arguments, the tree identity, and
the size of each test tier. ``build.py`` compares the size of a tier with the newest record of
that tier, so a test that vanishes shows in the epilogue.
"""

from __future__ import annotations

import hashlib
import subprocess
import time
from pathlib import Path
from typing import NamedTuple

try:
    import fcntl
except ImportError:  # Windows has no fcntl; journal locking is skipped there.
    fcntl = None

HISTORY_FILE = Path("logs/build_history.txt")
HEADER = "timestamp | duration_s | exit_code | args | tree | tests"
MAX_LINES = 1000

# Test tiers in journal column order. A tier is one fixed nextest command, so a step run and
# the gate's run of the same tier share one baseline.
TIERS = ("guardrails", "integration", "browser")


class TierRecord(NamedTuple):
    """A tier's newest test-set size in the journal."""

    timestamp: str
    tree: str
    size: int


def tree_identity(repo: Path) -> str:
    """Name the tree a run covers: the short HEAD, plus a content digest when dirty.

    The digest hashes ``git diff HEAD`` and every untracked, non-ignored path
    with its blob id, so two dirty trees share a name only when their content
    matches. Returns ``unknown`` when git fails (no git, not a repo, no HEAD).
    """

    def git(*argv: str, stdin: bytes | None = None) -> bytes:
        return subprocess.run(
            ["git", "-C", str(repo), *argv],
            input=stdin,
            capture_output=True,
            check=True,
        ).stdout

    try:
        head = git("rev-parse", "--short=10", "HEAD").decode().strip()
        diff = git("diff", "HEAD", "--binary", "--no-ext-diff", "--no-textconv")
        untracked = sorted(
            p
            for p in git("ls-files", "--others", "--exclude-standard", "-z").split(b"\0")
            if p
        )
        if not diff and not untracked:
            return head
        digest = hashlib.sha256(diff)
        for path in untracked:
            digest.update(b"\0" + path)
        hashable = [p for p in untracked if b"\n" not in p]
        if hashable:
            # --stdin-paths reads one path per line. A file removed by a
            # concurrent agent makes it fail; the path list still names the tree.
            try:
                blobs = git(
                    "hash-object", "--stdin-paths", stdin=b"\n".join(hashable) + b"\n"
                )
            except subprocess.CalledProcessError:
                blobs = b"unhashed"
            digest.update(b"\0" + blobs)
        return f"{head}+{digest.hexdigest()[:8]}"
    except (OSError, subprocess.SubprocessError, UnicodeDecodeError):
        return "unknown"


def tests_column(sizes: dict[str, int]) -> str:
    """Render the ``tests`` column from a tier -> size map, e.g. ``guardrails=163 integration=1565``.

    Tiers appear in ``TIERS`` order; ``-`` when no tier recorded a size.
    """
    return " ".join(f"{t}={sizes[t]}" for t in TIERS if t in sizes) or "-"


def last_tier_sizes(path: Path) -> dict[str, TierRecord]:
    """Return tier -> ``TierRecord`` from each tier's newest journal record.

    Read under a shared lock. Records older than the ``tree`` and
    ``tests`` columns, and ``-`` values, name no tier and are skipped. A missing
    journal gives an empty result.
    """
    try:
        with open(path, encoding="utf-8") as fh:
            # Shared lock: append() truncates and rewrites under LOCK_EX.
            if fcntl is not None:
                fcntl.flock(fh, fcntl.LOCK_SH)
            text = fh.read()
    except FileNotFoundError:
        return {}
    newest: dict[str, TierRecord] = {}
    for raw in text.splitlines()[1:]:
        parts = raw.split(" | ")
        if len(parts) < 6 or parts[5].strip() == "-":
            continue
        timestamp, tree = parts[0].strip(), parts[4].strip()
        for token in parts[5].split():
            tier, _, value = token.partition("=")
            if tier in TIERS and value.isdigit():
                newest[tier] = TierRecord(timestamp, tree, int(value))
    return newest


def size_suffix(size: int, baseline: TierRecord | None) -> str:
    """The epilogue text that compares a tier's size with its newest record."""
    if baseline is None:
        return f"tests: {size} (no earlier record)"
    where = f"{baseline.timestamp}, tree {baseline.tree}"
    if size == baseline.size:
        return f"tests: {size} (same as {where})"
    return f"tests: {size} ({size - baseline.size:+d} vs {where})"


def append(
    path: Path,
    args_str: str,
    duration_sec: float,
    exit_code: int,
    tree: str,
    tests: str,
) -> None:
    """Append one run record to the journal, then trim.

    One pipe-delimited line per run, columns per ``HEADER``, newest
    last so ``tail`` shows recent runs. An older header in line 1 is replaced
    in place; older records keep their shorter column set. The trim keeps the header plus the
    newest records, rewritten in place while still holding the lock — flock
    keeps concurrent agents (shared ``logs/``) from interleaving appends or
    racing the rewrite. Callers treat any failure as ignorable: the journal
    must never break a build.
    """
    args_str = args_str.replace("|", "/").replace("\n", " ").strip()
    line = (
        f"{time.strftime('%Y-%m-%dT%H:%M:%S')} | {duration_sec:.1f} | "
        f"{exit_code} | {args_str} | {tree} | {tests}\n"
    )
    with open(path, "a+", encoding="utf-8") as fh:
        if fcntl is not None:
            fcntl.flock(fh, fcntl.LOCK_EX)
        try:
            fh.seek(0)
            lines = fh.readlines()
            if lines and lines[0].startswith("timestamp |"):
                lines[0] = HEADER + "\n"
            else:
                lines.insert(0, HEADER + "\n")
            lines.append(line)
            if len(lines) > MAX_LINES:
                lines = lines[:1] + lines[-(MAX_LINES - 1) :]
            # "a+" writes land at EOF (O_APPEND), but truncate empties the
            # file first, so the rewrite starts back at the top.
            fh.seek(0)
            fh.truncate()
            fh.writelines(lines)
            fh.flush()
        finally:
            if fcntl is not None:
                fcntl.flock(fh, fcntl.LOCK_UN)
