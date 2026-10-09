"""Shared fixture for the build.py CLI unit tests: a log handle that survives close."""

from __future__ import annotations

import io


class _MemLog(io.StringIO):
    """Log handle that survives main()'s close so tests can read it back."""

    def close(self):
        self.flush()
