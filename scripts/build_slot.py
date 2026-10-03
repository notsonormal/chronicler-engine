"""Machine-wide build slot: one cargo compile or test step at a time across all checkouts.

Overlapping builds overrun the container's memory cap. The ``flock`` is released by the kernel when
the holder dies. It fails open: a lock error, or ``CHRONICLER_BUILD_SLOT_WAIT`` seconds (default
1800, per step), runs the step anyway. ``CHRONICLER_BUILD_SLOT=0`` disables it.
"""

from __future__ import annotations

import contextlib
import json
import os
import tempfile
import time
from pathlib import Path
from typing import Callable, Iterator

try:
    import fcntl
except ImportError:  # Windows has no fcntl; the slot is a no-op there.
    fcntl = None

HELD_ENV = "CHRONICLER_BUILD_SLOT_HELD"
DISABLE_ENV = "CHRONICLER_BUILD_SLOT"
WAIT_ENV = "CHRONICLER_BUILD_SLOT_WAIT"
LOCK_ENV = "CHRONICLER_BUILD_SLOT_LOCK"

DEFAULT_WAIT_SECS = 1800.0
POLL_SECS = 0.5
STATUS_EVERY_SECS = 30.0

_LIGHT_CARGO_SUBCOMMANDS = {"fmt"}


def is_heavy(cmd: str) -> bool:
    parts = cmd.split()
    if not parts or parts[0] != "cargo":
        return False
    return len(parts) < 2 or parts[1] not in _LIGHT_CARGO_SUBCOMMANDS


def lock_path() -> Path:
    """One lock file per user, shared by every worktree."""
    override = os.environ.get(LOCK_ENV)
    if override:
        return Path(override)
    return Path(tempfile.gettempdir()) / f"chronicler-engine-build-slot-{os.getuid()}.lock"


def _holder_path(lock: Path) -> Path:
    return lock.with_suffix(".holder")


def _read_holder(lock: Path) -> str:
    try:
        info = json.loads(_holder_path(lock).read_text())
        age = int(time.time() - info["since"])
        return f"{info['label']!r} in {info['cwd']} (pid {info['pid']}, running {age}s)"
    except Exception:
        return ""


def _write_holder(lock: Path, label: str) -> None:
    info = {"pid": os.getpid(), "cwd": os.getcwd(), "label": label, "since": time.time()}
    with contextlib.suppress(OSError):
        _holder_path(lock).write_text(json.dumps(info))


def _wait_limit() -> float:
    try:
        return float(os.environ.get(WAIT_ENV, DEFAULT_WAIT_SECS))
    except ValueError:
        return DEFAULT_WAIT_SECS


def _enabled() -> bool:
    if fcntl is None:
        return False
    if os.environ.get(DISABLE_ENV, "").strip().lower() in {"0", "off", "false", "no"}:
        return False
    # A child started inside a held slot must not wait on its own parent.
    return not os.environ.get(HELD_ENV)


def _acquire(fd: int, lock: Path, echo: Callable[[str], None]) -> bool:
    limit = _wait_limit()
    start = time.monotonic()
    next_status = 0.0
    while True:
        try:
            fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            if next_status:
                echo(f"Build slot acquired after {int(time.monotonic() - start)}s.")
            return True
        except BlockingIOError:
            pass
        waited = time.monotonic() - start
        if waited >= limit:
            echo(f"WARNING: waited {int(waited)}s for the build slot; running without it.")
            return False
        if waited >= next_status:
            holder = _read_holder(lock)
            suffix = f" Held by {holder}." if holder else ""
            echo(f"Waiting for the build slot (another build is running; waited {int(waited)}s).{suffix}")
            next_status = waited + STATUS_EVERY_SECS
        time.sleep(POLL_SECS)


@contextlib.contextmanager
def hold(label: str, echo: Callable[[str], None] = print) -> Iterator[bool]:
    if not _enabled():
        yield False
        return

    lock = lock_path()
    try:
        fd = os.open(lock, os.O_RDWR | os.O_CREAT, 0o666)
    except OSError:
        yield False
        return

    try:
        acquired = _acquire(fd, lock, echo)
        if acquired:
            _write_holder(lock, label)
        os.environ[HELD_ENV] = "1"
        try:
            yield acquired
        finally:
            os.environ.pop(HELD_ENV, None)
            if acquired:
                with contextlib.suppress(OSError):
                    fcntl.flock(fd, fcntl.LOCK_UN)
    finally:
        os.close(fd)


def maybe_hold(cmd: str, label: str, echo: Callable[[str], None] = print):
    if is_heavy(cmd):
        return hold(label, echo)
    return contextlib.nullcontext(False)
