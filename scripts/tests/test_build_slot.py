"""Unit tests for scripts/build_slot.py."""

from __future__ import annotations

import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import build_slot  # noqa: E402

_HOLDER = """
import sys, time
sys.path.insert(0, {scripts!r})
import build_slot
with build_slot.hold("holder-label", lambda m: None) as got:
    print("ready" if got else "failed", flush=True)
    time.sleep(60)
"""


class IsHeavyTests(unittest.TestCase):
    def test_cargo_compile_and_test_commands_are_heavy(self):
        for cmd in (
            "cargo check --all-targets --all-features",
            "cargo clippy --all-targets --all-features -- -D warnings",
            "cargo nextest run --no-fail-fast --test architecture",
            "cargo test --lib",
            "cargo llvm-cov nextest",
            "cargo build --release",
        ):
            self.assertTrue(build_slot.is_heavy(cmd), cmd)

    def test_light_steps_are_not_heavy(self):
        for cmd in ("cargo fmt", "python scripts/validate_docs.py", "", "python build.py"):
            self.assertFalse(build_slot.is_heavy(cmd), cmd)


@unittest.skipIf(build_slot.fcntl is None, "the slot needs fcntl")
class SlotTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.lock = Path(self.tmp.name) / "slot.lock"
        env = {build_slot.LOCK_ENV: str(self.lock)}
        patcher = mock.patch.dict(os.environ, env)
        patcher.start()
        self.addCleanup(patcher.stop)
        for key in (build_slot.HELD_ENV, build_slot.DISABLE_ENV, build_slot.WAIT_ENV):
            os.environ.pop(key, None)
        self.messages: list[str] = []

    def _spawn_holder(self) -> subprocess.Popen:
        proc = subprocess.Popen(
            [sys.executable, "-c", _HOLDER.format(scripts=str(SCRIPTS))],
            stdout=subprocess.PIPE,
            text=True,
            env=os.environ.copy(),
        )
        self.addCleanup(lambda: (proc.kill(), proc.wait(), proc.stdout.close()))
        self.assertEqual(proc.stdout.readline().strip(), "ready")
        return proc

    def test_uncontended_acquire_and_release(self):
        with build_slot.hold("x", self.messages.append) as got:
            self.assertTrue(got)
            self.assertEqual(os.environ.get(build_slot.HELD_ENV), "1")
        self.assertNotIn(build_slot.HELD_ENV, os.environ)
        self.assertEqual(self.messages, [])
        with build_slot.hold("y", self.messages.append) as got:
            self.assertTrue(got)

    def test_second_holder_waits_and_names_the_first(self):
        proc = self._spawn_holder()
        try:
            os.environ[build_slot.WAIT_ENV] = "1.2"
            with mock.patch.object(build_slot, "POLL_SECS", 0.05):
                with build_slot.hold("waiter", self.messages.append) as got:
                    self.assertFalse(got)  # wait limit passed, ran without the slot
        finally:
            proc.kill()
        joined = "\n".join(self.messages)
        self.assertIn("Waiting for the build slot", joined)
        self.assertIn("holder-label", joined)
        self.assertIn("running without it", joined)

    def test_kill_9_of_the_holder_frees_the_slot(self):
        proc = self._spawn_holder()
        proc.send_signal(signal.SIGKILL)
        proc.wait()
        started = time.monotonic()
        os.environ[build_slot.WAIT_ENV] = "5"
        with build_slot.hold("after-crash", self.messages.append) as got:
            self.assertTrue(got)
        self.assertLess(time.monotonic() - started, 3)
        self.assertFalse([m for m in self.messages if "WARNING" in m])

    def test_waiter_gets_the_slot_when_holder_exits(self):
        proc = self._spawn_holder()
        os.environ[build_slot.WAIT_ENV] = "10"

        def holder_exits_during_the_first_wait(_secs):
            proc.terminate()  # only reached after the waiter found the slot taken
            proc.wait()

        with mock.patch.object(build_slot.time, "sleep", side_effect=holder_exits_during_the_first_wait):
            with build_slot.hold("waiter", self.messages.append) as got:
                self.assertTrue(got)
        self.assertTrue(any("acquired after" in m for m in self.messages))

    def test_disabled_by_environment(self):
        os.environ[build_slot.DISABLE_ENV] = "0"
        with build_slot.hold("x", self.messages.append) as got:
            self.assertFalse(got)
        self.assertFalse(self.lock.exists())

    def test_nested_build_does_not_wait_on_its_parent(self):
        os.environ[build_slot.HELD_ENV] = "1"
        with build_slot.hold("child", self.messages.append) as got:
            self.assertFalse(got)

    def test_unwritable_lock_path_fails_open(self):
        os.environ[build_slot.LOCK_ENV] = str(Path(self.tmp.name) / "missing-dir" / "slot.lock")
        with build_slot.hold("x", self.messages.append) as got:
            self.assertFalse(got)

    def test_maybe_hold_skips_light_commands(self):
        with build_slot.maybe_hold("cargo fmt", "fmt", self.messages.append) as got:
            self.assertFalse(got)
        self.assertFalse(self.lock.exists())
        with build_slot.maybe_hold("cargo check", "check", self.messages.append) as got:
            self.assertTrue(got)


if __name__ == "__main__":
    unittest.main()
