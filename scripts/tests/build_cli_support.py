"""Shared fixtures for the build.py CLI unit tests: a log handle that survives close,
and a reset for the epilogue's module-level capture state."""

from __future__ import annotations

import io

import build


class _MemLog(io.StringIO):
    """Log handle that survives main()'s close so tests can read it back."""

    def close(self):
        self.flush()


def reset_epilogue_state():
    """Clear every capture class main()'s epilogue prints from.

    The classes are module state that outlives a test. Call this before and
    after a test that reads them back. The caller puts the repo root on
    ``sys.path`` before it imports this module.
    """
    build._NextestSummary.lines = []
    build._NextestSummary.label = ""
    build._NextestSummary.sizes = {}
    build._NextestLeaks.lines = []
    build._CompileErrors.lines = []
