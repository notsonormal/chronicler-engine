"""Unit tests for scripts/target_seed.py."""

from __future__ import annotations

import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

try:
    import fcntl
except ImportError:
    fcntl = None

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import target_seed  # noqa: E402

OWN = "aaaaaaaaaaaaaaaa"
DEP = "bbbbbbbbbbbbbbbb"
HTTP_CRATE = "cccccccccccccccc"  # third-party crate that shares a name with a test binary
LOCAL = "dddddddddddddddd"
SHAREABLE = {"serde", "aws-lc-sys", "http"}  # what cargo metadata reports from registry or git


def make_profile_dir(profile_dir: Path, rlibs: int = 2) -> None:
    """Lay out a profile dir like cargo's: one dependency, the workspace crate, a test binary."""
    for rel, body in {
        f".fingerprint/serde-{DEP}/lib-serde": "dep",
        f".fingerprint/chronicler_engine-{OWN}/test-lib": "own",
        f".fingerprint/http-{HTTP_CRATE}/lib-http": "dep",
        f"build/aws-lc-sys-{DEP}/output": "cargo:root=/old/path",
        f"build/chronicler_engine-{OWN}/output": "own",
        f".fingerprint/localdep-{LOCAL}/lib-localdep": "path dep",
        f"build/localdep-{LOCAL}/output": "path dep",
        f"deps/liblocaldep-{LOCAL}.rlib": "path dep",
        f"deps/mystery_file": "no unit hash",
        f"deps/libserde-{DEP}.rlib": "dep",
        f"deps/libserde-{DEP}.rmeta": "dep",
        f"deps/libchronicler_engine-{OWN}.rlib": "own",
        f"deps/http-{OWN}": "own test binary named like a crate",
        f"deps/http-{OWN}.d": "own",
        f"deps/libhttp-{HTTP_CRATE}.rlib": "dep",
        "incremental/chronicler_engine-x/data": "inc",
        "chronicler_engine": "uplifted binary",
    }.items():
        path = profile_dir / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body)
    for index in range(rlibs):
        (profile_dir / "deps" / f"libextra{index}-{DEP}.rlib").write_text("dep")
    os.utime(profile_dir / "deps" / f"libserde-{DEP}.rlib", (1_000_000, 1_000_000))


class CopyTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        self.src = self.root / "src" / "debug"
        self.dst = self.root / "dst" / "debug"
        make_profile_dir(self.src)

    def copied(self) -> set[str]:
        return {str(p.relative_to(self.dst)) for p in self.dst.rglob("*") if p.is_file()}

    def test_workspace_units_are_never_copied(self):
        target_seed.copy_third_party(self.src, self.dst, SHAREABLE)
        files = self.copied()
        self.assertFalse([f for f in files if "chronicler_engine" in f], files)
        self.assertNotIn(f"deps/http-{OWN}", files)  # test binary with the workspace hash
        self.assertNotIn(f"deps/http-{OWN}.d", files)
        self.assertNotIn("incremental/chronicler_engine-x/data", files)
        self.assertNotIn("chronicler_engine", files)

    def test_third_party_crates_including_same_named_ones_are_copied(self):
        target_seed.copy_third_party(self.src, self.dst, SHAREABLE)
        files = self.copied()
        for expected in (
            f".fingerprint/serde-{DEP}/lib-serde",
            f".fingerprint/http-{HTTP_CRATE}/lib-http",
            f"build/aws-lc-sys-{DEP}/output",
            f"deps/libserde-{DEP}.rlib",
            f"deps/libhttp-{HTTP_CRATE}.rlib",
        ):
            self.assertIn(expected, files)

    def test_a_package_outside_the_allow_list_is_never_copied(self):
        target_seed.copy_third_party(self.src, self.dst, SHAREABLE)
        files = self.copied()
        self.assertFalse([f for f in files if "localdep" in f], files)
        self.assertNotIn("deps/mystery_file", files)  # fail closed on what cannot be attributed

    def test_a_package_name_that_extends_an_allowed_one_is_not_allowed(self):
        extra = self.src / ".fingerprint" / f"serde-derive-{LOCAL}" / "lib"
        extra.parent.mkdir(parents=True)
        extra.write_text("not allowed")
        target_seed.copy_third_party(self.src, self.dst, SHAREABLE)
        self.assertNotIn(f".fingerprint/serde-derive-{LOCAL}/lib", self.copied())

    def test_mtimes_are_preserved(self):
        target_seed.copy_third_party(self.src, self.dst, SHAREABLE)
        self.assertEqual(int((self.dst / "deps" / f"libserde-{DEP}.rlib").stat().st_mtime), 1_000_000)


def package(name: str, source: str | None) -> dict:
    return {"name": name, "source": source}


class ShareableFromMetadataTests(unittest.TestCase):
    REGISTRY = "registry+https://github.com/rust-lang/crates.io-index"

    def shareable(self, *packages: dict) -> set[str]:
        return target_seed.shareable_from_metadata({"packages": list(packages)})

    def test_registry_and_git_packages_are_shareable(self):
        got = self.shareable(package("serde", self.REGISTRY), package("fork", "git+https://x/y?rev=1#abc"))
        self.assertEqual(got, {"serde", "fork"})

    def test_workspace_and_path_packages_are_not(self):
        got = self.shareable(package("serde", self.REGISTRY), package("chronicler_engine", None))
        self.assertEqual(got, {"serde"})

    def test_a_patched_name_with_any_unshareable_source_is_excluded(self):
        got = self.shareable(package("serde", self.REGISTRY), package("serde", None))
        self.assertEqual(got, set())

    def test_an_unrecognised_source_kind_is_not_shareable(self):
        self.assertEqual(self.shareable(package("odd", "sparse+https://example/index")), set())


class SignatureTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        (self.root / ".cargo").mkdir()
        (self.root / ".cargo" / "config.toml").write_text("[build]\n")

    def test_signature_is_stable_and_tracks_the_cargo_config(self):
        first = target_seed.build_signature(self.root)
        self.assertEqual(first, target_seed.build_signature(self.root))
        (self.root / ".cargo" / "config.toml").write_text('[target.x86_64-unknown-linux-gnu]\nlinker = "x"\n')
        self.assertNotEqual(first, target_seed.build_signature(self.root))

    def test_signature_tracks_rustflags(self):
        first = target_seed.build_signature(self.root)
        with mock.patch.dict(os.environ, {"RUSTFLAGS": "-D warnings"}):
            self.assertNotEqual(first, target_seed.build_signature(self.root))

    def test_stamp_round_trip(self):
        profile = self.root / "debug"
        profile.mkdir()
        self.assertIsNone(target_seed.read_stamp(profile))
        target_seed.stamp(profile, "abc")
        self.assertEqual(target_seed.read_stamp(profile), "abc")

    def test_stamping_a_missing_dir_is_a_no_op(self):
        target_seed.stamp(self.root / "nope", "abc")
        self.assertFalse((self.root / "nope").exists())


class SeedIfColdTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.root = Path(tmp.name)
        self.src = self.root / "target" / "debug"
        make_profile_dir(self.src, rlibs=3)
        self.target = self.root / "target" / "t1"
        self.messages: list[str] = []
        for name, value in (
            ("shareable_packages", SHAREABLE),
            ("worktree_roots", [self.root]),
        ):
            patcher = mock.patch.object(target_seed, name, return_value=value)
            patcher.start()
            self.addCleanup(patcher.stop)
        os.environ.pop(target_seed.DISABLE_ENV, None)
        target_seed.stamp(self.src, target_seed.build_signature(self.root))

    def seed(self) -> bool:
        return target_seed.seed_if_cold(self.root, self.target, "debug", self.messages.append)

    def test_seeds_a_cold_target_dir_and_leaves_no_staging_dir(self):
        self.assertTrue(self.seed())
        self.assertTrue(target_seed.is_warm(self.target / "debug"))
        self.assertEqual([p.name for p in self.target.iterdir()], ["debug"])

    def test_a_warm_target_dir_is_left_alone(self):
        self.seed()
        marker = self.target / "debug" / "marker"
        marker.write_text("mine")
        self.assertFalse(self.seed())
        self.assertTrue(marker.exists())

    @unittest.skipIf(fcntl is None, "cargo's build lock is an flock")
    def test_a_source_that_cargo_is_building_is_skipped(self):
        lock = self.src / ".cargo-lock"
        lock.write_text("")
        with open(lock, "rb") as builder:
            fcntl.flock(builder, fcntl.LOCK_EX)
            self.assertFalse(self.seed())
        self.assertFalse((self.target / "debug").exists())
        self.assertTrue(any("being built" in m for m in self.messages))
        self.assertTrue(any("every candidate source is busy" in m for m in self.messages))

    def test_the_richer_source_wins(self):
        poor = self.root / "target" / "poor" / "debug"
        make_profile_dir(poor, rlibs=0)
        for rlib in (poor / "deps").glob("*.rlib"):  # a check-only target holds no .rlib
            rlib.unlink()
        ranked = target_seed.candidate_sources(self.root, "debug", self.target / "debug")
        self.assertEqual(ranked, [self.src])

    def test_an_unstamped_source_does_not_qualify_when_a_signature_is_known(self):
        newer = self.root / "target" / "n1" / "debug"
        make_profile_dir(newer, rlibs=0)
        target_seed.stamp(newer, "sig-now")  # self.src carries the real signature, not this one
        ranked = target_seed.candidate_sources(self.root, "debug", self.target / "debug", "sig-now")
        self.assertEqual(ranked, [newer])
        unstamped = self.root / "target" / "raw" / "debug"
        make_profile_dir(unstamped, rlibs=9)
        ranked = target_seed.candidate_sources(self.root, "debug", self.target / "debug", "sig-now")
        self.assertEqual(ranked, [newer])

    def test_a_source_stamped_with_another_signature_is_skipped(self):
        target_seed.stamp(self.src, "sig-old")
        ranked = target_seed.candidate_sources(self.root, "debug", self.target / "debug", "sig-now")
        self.assertEqual(ranked, [])

    def test_without_a_signature_every_warm_dir_qualifies(self):
        target_seed.stamp(self.src, "sig-old")
        ranked = target_seed.candidate_sources(self.root, "debug", self.target / "debug")
        self.assertEqual(ranked, [self.src])

    def test_disabled_by_environment(self):
        with mock.patch.dict(os.environ, {target_seed.DISABLE_ENV: "1"}):
            self.assertFalse(self.seed())
        self.assertFalse((self.target / "debug").exists())

    def test_a_seed_miss_is_silent_when_disabled_by_environment(self):
        target_seed.stamp(self.src, "sig-old")  # a miss that seeding-on would report
        with mock.patch.dict(os.environ, {target_seed.DISABLE_ENV: "1"}):
            self.assertFalse(self.seed())
        self.assertEqual(self.messages, [])
        self.assertFalse(self.seed())  # same miss, seeding enabled: it must be reported
        self.assertTrue(any("no warm sibling with build signature" in m for m in self.messages))

    def test_a_miss_names_the_warm_dirs_rejected_for_their_signature(self):
        target_seed.stamp(self.src, "sig-old")
        self.assertFalse(self.seed())
        miss = [m for m in self.messages if m.startswith("Target seeding: no warm sibling")]
        self.assertEqual(len(miss), 1, self.messages)
        self.assertIn("building dependencies from scratch", miss[0])
        self.assertIn(str(self.src), miss[0])
        self.assertIn("signature sig-old", miss[0])

    def test_a_miss_names_a_check_only_warm_dir(self):
        check_only = self.root / "target" / "check" / "debug"
        make_profile_dir(check_only, rlibs=0)
        for rlib in (check_only / "deps").glob("*.rlib"):  # a check-only target holds no .rlib
            rlib.unlink()
        target_seed.stamp(check_only, target_seed.build_signature(self.root))
        target_seed.stamp(self.src, "sig-old")
        self.assertFalse(self.seed())
        miss = [m for m in self.messages if m.startswith("Target seeding: no warm sibling")][0]
        self.assertIn(f"{check_only} (no compiled libraries)", miss)
        self.assertIn(f"{self.src} (signature sig-old)", miss)

    def test_a_miss_says_when_there_is_no_warm_target_dir_at_all(self):
        for path in sorted(self.src.rglob("*"), reverse=True):
            path.unlink() if path.is_file() else path.rmdir()
        self.assertFalse(self.seed())
        miss = [m for m in self.messages if m.startswith("Target seeding: no warm sibling")]
        self.assertEqual(len(miss), 1, self.messages)
        self.assertIn("no warm target dir in any worktree", miss[0])

    def test_no_candidates_means_a_normal_cold_build(self):
        for path in sorted(self.src.rglob("*"), reverse=True):
            path.unlink() if path.is_file() else path.rmdir()
        self.assertFalse(self.seed())


if __name__ == "__main__":
    unittest.main()
