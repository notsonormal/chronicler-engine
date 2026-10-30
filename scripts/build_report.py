"""Readers for the structured output of cargo, nextest and rustfmt, used by ``build.py``.

``build.py`` runs cargo and nextest with machine-readable output and reads these formats:

- the nextest JUnit report (``target/nextest/<profile>/junit.xml`` under the workspace root;
  nextest ignores ``CARGO_TARGET_DIR`` for it): the failing tests
  with the first line of their message, the duration of every test, and the per-check lines
  that a stub runner prints (nextest keeps the output of passing tests there, see
  ``store-success-output`` in ``.config/nextest.toml``);
- the cargo JSON message stream (``--message-format json``): the compile errors, and the
  rendered compiler output that goes to the build log;
- the nextest ``Summary`` line: the test counts. The JUnit report has no count of ignored
  tests and does not mark a test that leaked a process, so the Summary line and the
  ``LEAK``/``FLAKY`` result lines stay the source for both;
- the ``cargo fmt --check -- -l`` output: one path per unformatted file; and the text of a
  rustfmt syntax error (``error:`` header and ``-->`` location), because rustfmt has no JSON mode.

Every function is pure: it takes text or a path and returns plain values, so the tests feed
it recorded output from ``scripts/tests/fixtures/build_output/``.
"""

from __future__ import annotations

import json
import os
import re
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import NamedTuple

MESSAGE_LIMIT = 200

# nextest Summary line, e.g. "Summary [1.0s] 5 tests run: 3 passed (1 leaky), 1 failed, 1 skipped".
# A run that stopped early reads "2/5 tests run": 2 tests reached, 5 selected.
_SUMMARY_RE = re.compile(r"^\s*Summary \[[^\]]*\]\s+(\d+)(?:/(\d+))? tests? run:(.*)$")
# A LEAK or FLAKY result line, e.g. "LEAK [ 0.208s] (4/5) probe::leak leaks_a_child".
# The progress counter is optional.
_LEAK_RE = re.compile(r"^\s*(LEAK|FLAKY)\s*\[[^\]]*\]\s*(?:\([^)]*\))?\s*(\S.*)$")
# One line of a stub runner's per-check summary (`StubRunner::finish` in
# tests/browser/stub/support.rs), e.g. "  0.902s  [OK]  check_x".
_STUB_CHECK_RE = re.compile(r"^\s*([0-9]+(?:\.[0-9]+)?)s\s+\[(OK|FAIL)\]\s+(check_\w+)\s*$")


class Counts(NamedTuple):
    """The numbers of one nextest ``Summary`` line."""

    passed: int
    failed: int
    skipped: int
    leaky: int
    flaky: int
    timed_out: int
    reached: int
    selected: int

    @property
    def size(self) -> int:
        """The size of the test set: the selected tests plus the ignored ones."""
        return self.selected + self.skipped

    @property
    def line(self) -> str:
        """The epilogue one-liner. ``failed`` always shows; other zero counts are left out."""
        text = f"nextest: {self.passed} passed, {self.failed} failed"
        for count, word in (
            (self.skipped, "skipped"),
            (self.leaky, "leaky"),
            (self.flaky, "flaky"),
            (self.timed_out, "timed out"),
            (self.selected - self.reached, "not run"),
        ):
            if count > 0:
                text += f", {count} {word}"
        return text


def parse_summary(text: str) -> Counts | None:
    """The counts of the last nextest Summary line in ``text``, or None when it has none."""
    found = None
    for raw in text.splitlines():
        match = _SUMMARY_RE.match(raw)
        if match is None:
            continue
        reached = int(match.group(1))
        rest = match.group(3)

        def count(word: str) -> int:
            hit = re.search(rf"(\d+) {word}", rest)
            return int(hit.group(1)) if hit else 0

        found = Counts(
            passed=count("passed"),
            failed=count("failed"),
            skipped=count("skipped"),
            leaky=count("leaky"),
            flaky=count("flaky"),
            timed_out=count("timed out"),
            reached=reached,
            selected=int(match.group(2)) if match.group(2) else reached,
        )
    return found


def find_leaks(text: str) -> list[tuple[str, str]]:
    """The ``(status, test name)`` of each LEAK or FLAKY result line, once each.

    nextest prints such a line live and may print it again in its final list.
    """
    leaks: list[tuple[str, str]] = []
    for raw in text.splitlines():
        match = _LEAK_RE.match(raw)
        if match and (entry := (match.group(1), match.group(2).strip())) not in leaks:
            leaks.append(entry)
    return leaks


class JunitReport(NamedTuple):
    """What ``read_junit`` takes from one JUnit file."""

    failures: list[tuple[str, str]]  # (test name, first message line)
    times: list[tuple[float, str]]  # (seconds, test name), every test
    stub_checks: list[tuple[float, str, str, str]]  # (seconds, runner test, check, OK|FAIL)


def _failure_message(failure: ET.Element) -> str:
    """The first message line of a failed test.

    A panic body reads ``thread 'x' panicked at file:L:C:`` and then the message, so the message
    is the second line. A failure without a body, such as a timeout, gives its ``message`` or
    ``type`` attribute.
    """
    lines = [line.strip() for line in (failure.text or "").splitlines() if line.strip()]
    if lines and "panicked at " in lines[0] and len(lines) > 1:
        return lines[1][:MESSAGE_LIMIT]
    if lines:
        return lines[0][:MESSAGE_LIMIT]
    return (failure.get("message") or failure.get("type") or "")[:MESSAGE_LIMIT]


def read_junit(path: Path) -> JunitReport | None:
    """Read one nextest JUnit file, or None when it is missing or cut short.

    A test is named like nextest names it: ``<binary> <test>``.
    """
    try:
        root = ET.parse(path).getroot()
    except (OSError, ET.ParseError):
        return None
    failures, times, stub_checks = [], [], []
    for case in root.iter("testcase"):
        name = f"{case.get('classname', '')} {case.get('name', '')}".strip()
        times.append((float(case.get("time", "0")), name))
        failure = case.find("failure")
        if failure is None:
            failure = case.find("error")
        if failure is not None:
            failures.append((name, _failure_message(failure)))
        for stream in ("system-err", "system-out"):
            for line in (case.findtext(stream) or "").splitlines():
                check = _STUB_CHECK_RE.match(line)
                if check:
                    stub_checks.append(
                        (float(check.group(1)), name, check.group(3), check.group(2))
                    )
    return JunitReport(failures, times, stub_checks)


class CargoOutput(NamedTuple):
    """The cargo JSON stream of one command, split for the log and for the epilogue."""

    text: str  # rendered compiler output and the plain lines, in order, for the log
    errors: list[str]  # the compile errors, each once, in first-seen order


def _error_line(message: dict) -> str:
    """One epilogue line for a compile error, e.g. ``error[E0308]: mismatched types  (src/lib.rs:6:5)``."""
    rendered = message.get("rendered") or message.get("message") or ""
    line = (rendered.splitlines() or [""])[0].strip()
    for span in message.get("spans", []):
        if span.get("is_primary"):
            return f"{line}  ({span['file_name']}:{span['line_start']}:{span['column_start']})"
    return line


def read_cargo_stream(stdout: str) -> CargoOutput:
    """Split the stdout of a cargo command run with a JSON message format.

    A ``compiler-message`` gives its rendered text to the log, and an error one also gives a
    line to ``errors``. A line of another JSON reason is dropped. A line that is not JSON
    stays in the log as it is. cargo reports one error once for each target that has it
    (``--all-targets``), so a repeated error line is kept once.
    """
    text: list[str] = []
    errors: list[str] = []
    for raw in stdout.splitlines():
        try:
            record = json.loads(raw) if raw.startswith("{") else None
        except json.JSONDecodeError:
            record = None
        if not isinstance(record, dict) or "reason" not in record:
            text.append(raw)
            continue
        message = record.get("message")
        if record["reason"] != "compiler-message" or not isinstance(message, dict):
            continue
        text.append((message.get("rendered") or message.get("message") or "").rstrip("\n"))
        if message.get("level") == "error":
            line = _error_line(message)
            if line not in errors:
                errors.append(line)
    return CargoOutput("\n".join(text), errors)


def unformatted_files(output: str) -> list[str]:
    """The paths in ``cargo fmt --check -- -l`` output, relative to the working directory."""
    files: list[str] = []
    for raw in output.splitlines():
        path = raw.strip()
        if not path.endswith(".rs"):
            continue
        try:
            path = os.path.relpath(path)
        except ValueError:  # another drive on Windows
            pass
        if path not in files:
            files.append(path)
    return files


def text_errors(output: str) -> list[str]:
    """The ``error:`` headers of plain rustfmt output, each with its ``-->`` location.

    rustfmt has no JSON mode, so a syntax error reads like a compiler error in text:
    ``error: <message>`` and, below it, `` --> path:line:col``. The path is made relative to
    the working directory, as cargo's own spans are.
    """
    errors: list[str] = []
    lines = output.splitlines()
    for i, raw in enumerate(lines):
        if not raw.startswith("error"):
            continue
        line = raw.strip()[:MESSAGE_LIMIT]
        for follow in lines[i + 1 : i + 4]:
            stripped = follow.strip()
            if stripped.startswith("--> "):
                path, sep, position = stripped[4:].partition(":")
                try:
                    path = os.path.relpath(path)
                except ValueError:  # another drive on Windows
                    pass
                line += f"  ({path}{sep}{position})"
                break
            if not stripped:
                break
        if line not in errors:
            errors.append(line)
    return errors


def tail(text: str, count: int = 3) -> list[str]:
    """The last ``count`` non-empty lines of ``text``."""
    lines = [line.strip()[:MESSAGE_LIMIT] for line in text.splitlines() if line.strip()]
    return lines[-count:]
