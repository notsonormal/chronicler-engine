"""Seed a cold cargo target dir with dependency artifacts from a warm sibling checkout.

Only registry and git packages are copied: cargo's artifact hashes ignore absolute paths, so the
workspace crate, path dependencies and ``[patch]`` entries look identical across checkouts with
different code. A half-copied seed is renamed into place, and a source cargo is building is skipped.

``CHRONICLER_NO_SEED=1`` disables seeding.
"""

from __future__ import annotations

import functools
import hashlib
import json
import os
import re
import shutil
import subprocess
from pathlib import Path
from typing import Callable

try:
    import fcntl
except ImportError:  # Windows has no fcntl; seeding is skipped there.
    fcntl = None

DISABLE_ENV = "CHRONICLER_NO_SEED"
_COPIED_DIRS = (".fingerprint", "build", "deps")
_UNIT_HASH = re.compile(r"-([0-9a-f]{16})(?:\.|$)")
_UNIT_DIR = re.compile(r"^(?P<pkg>.+)-(?P<hash>[0-9a-f]{16})$")
_SHAREABLE_SOURCES = ("registry+", "git+")
SIGNATURE_FILE = ".chronicler-build-signature"
# Inputs that change the hash of every unit.
_SIGNATURE_FILES = (".cargo/config.toml", "scripts/lld-linker.sh", "rust-toolchain.toml")
_SIGNATURE_ENV = ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS")
# Coverage builds use instrumentation flags, so none of their hashes match.
_NOT_SEEDS = {"llvm-cov-target", "tmp", "fix"}
_SEEN_LIMIT = 3


def build_signature(repo_root: Path) -> str:
    """Hash of what invalidates every unit at once; ``Cargo.lock`` is deliberately excluded.

    ``rust-toolchain.toml`` stands in for the compiler version: it pins the toolchain, so a
    separate ``rustc -vV`` probe would add nothing to the signature.
    """
    digest = hashlib.sha256()
    for rel in _SIGNATURE_FILES:
        path = repo_root / rel
        digest.update(rel.encode())
        digest.update(path.read_bytes() if path.is_file() else b"-")
    for name in _SIGNATURE_ENV:
        digest.update(f"{name}={os.environ.get(name, '')}".encode())
    return digest.hexdigest()[:16]


def read_stamp(profile_dir: Path) -> str | None:
    try:
        return (profile_dir / SIGNATURE_FILE).read_text().strip() or None
    except OSError:
        return None


def stamp(profile_dir: Path, signature: str) -> None:
    try:
        if profile_dir.is_dir():
            (profile_dir / SIGNATURE_FILE).write_text(signature + "\n")
    except OSError:
        pass


def is_warm(profile_dir: Path) -> bool:
    return (profile_dir / ".fingerprint").is_dir() and any((profile_dir / ".fingerprint").iterdir())


@functools.lru_cache(maxsize=1)
def rustc_host_triple() -> str:
    """The rustc host triple, probed once per process and cached for every caller."""
    out = subprocess.run(["rustc", "-vV"], capture_output=True, text=True).stdout
    match = re.search(r"^host: (\S+)$", out, re.M)
    if not match:
        raise RuntimeError("cannot read the rustc host triple")
    return match.group(1)


def shareable_from_metadata(metadata: dict) -> set[str]:
    """Package names whose every appearance comes from a registry or git source."""
    shareable, other = set(), set()
    for pkg in metadata["packages"]:
        source = pkg.get("source") or ""
        (shareable if source.startswith(_SHAREABLE_SOURCES) else other).add(pkg["name"])
    return shareable - other


def shareable_packages(repo_root: Path) -> set[str]:
    """``--filter-platform`` is what lets ``--offline`` work: without it cargo
    wants other platforms' crates."""
    result = subprocess.run(
        [
            "cargo",
            "metadata",
            "--offline",
            "--locked",
            "--filter-platform",
            rustc_host_triple(),
            "--format-version",
            "1",
        ],
        cwd=repo_root,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "cargo metadata failed")
    return shareable_from_metadata(json.loads(result.stdout))


def worktree_roots(repo_root: Path) -> list[Path]:
    result = subprocess.run(
        ["git", "worktree", "list", "--porcelain"], cwd=repo_root, capture_output=True, text=True
    )
    roots = [Path(line[len("worktree ") :]) for line in result.stdout.splitlines() if line.startswith("worktree ")]
    return roots or [repo_root]


def _warm_candidates(repo_root: Path, profile: str, destination: Path) -> list[tuple[int, float, Path]]:
    found = []
    for root in worktree_roots(repo_root):
        target = root / "target"
        if not target.is_dir():
            continue
        siblings = (p for p in sorted(target.iterdir()) if p.is_dir() and p.name not in _NOT_SEEDS)
        for base in [target, *siblings]:
            profile_dir = base / profile
            if profile_dir.resolve() == destination.resolve() or not is_warm(profile_dir):
                continue
            deps = profile_dir / "deps"
            libs = sum(1 for f in deps.iterdir() if f.suffix == ".rlib") if deps.is_dir() else 0
            found.append((libs, (profile_dir / ".fingerprint").stat().st_mtime, profile_dir))
    found.sort(key=lambda item: item[:2], reverse=True)
    return found


def candidate_sources(
    repo_root: Path, profile: str, destination: Path, signature: str | None = None
) -> list[Path]:
    """Warm profile dirs that qualify as a seed source; a signature restricts them to stamped dirs."""
    return [
        profile_dir
        for libs, _, profile_dir in _warm_candidates(repo_root, profile, destination)
        if libs and (signature is None or read_stamp(profile_dir) == signature)
    ]


def _describe_rejections(repo_root: Path, profile: str, destination: Path) -> str:
    warm = _warm_candidates(repo_root, profile, destination)
    if not warm:
        return "no warm target dir in any worktree"
    seen = []
    for libs, _, profile_dir in warm[:_SEEN_LIMIT]:
        stamp = read_stamp(profile_dir)
        if not libs:  # a cargo check dir carries no .rlib to seed
            why = "no compiled libraries"
        elif stamp:
            why = f"signature {stamp}"
        else:
            why = "unstamped"
        seen.append(f"{profile_dir} ({why})")
    hidden = len(warm) - len(seen)
    if hidden:
        seen.append(f"and {hidden} more")
    return ", ".join(seen)


def _shareable_hashes(source: Path, shareable: set[str]) -> set[str]:
    hashes = set()
    for entry in (source / ".fingerprint").iterdir():
        match = _UNIT_DIR.match(entry.name)
        if match and match["pkg"] in shareable:
            hashes.add(match["hash"])
    return hashes


def _keep(top: str, name: str, shareable: set[str], hashes: set[str]) -> bool:
    if top in (".fingerprint", "build"):
        match = _UNIT_DIR.match(name)
        return bool(match) and match["pkg"] in shareable
    match = _UNIT_HASH.search(name)
    return bool(match) and match.group(1) in hashes


def copy_third_party(source: Path, destination: Path, shareable: set[str]) -> tuple[int, int]:
    hashes = _shareable_hashes(source, shareable)
    files = size = 0
    for top in _COPIED_DIRS:
        src_top = source / top
        if not src_top.is_dir():
            continue
        for root, dirs, names in os.walk(src_top):
            rel = Path(root).relative_to(src_top)
            at_top = rel == Path(".")
            if at_top:
                dirs[:] = [d for d in dirs if _keep(top, d, shareable, hashes)]
            out_dir = destination / top / rel
            out_dir.mkdir(parents=True, exist_ok=True)
            for name in names:
                if at_top and not _keep(top, name, shareable, hashes):
                    continue
                src_file, dst_file = Path(root) / name, out_dir / name
                if src_file.is_symlink():
                    os.symlink(os.readlink(src_file), dst_file)
                else:
                    shutil.copy2(src_file, dst_file)  # copy2 keeps mtimes, which cargo compares
                    size += src_file.stat().st_size
                files += 1
    return files, size


def probe_shared_lock(lock: Path):
    """Open ``lock`` and hold a non-blocking shared flock on it.

    The single lock mode used by every probe: a target-seed copy holds the lock shared, so a
    probe must not mistake it for cargo's exclusive build lock. Returns the open handle (shared
    lock held until ``close()``), or ``None`` when an exclusive holder has the lock. A missing or
    unreadable lock file, or a platform without ``fcntl``, returns a devnull handle so callers can
    always ``close()`` the result.
    """
    if fcntl is None or not lock.exists():
        return open(os.devnull)
    try:
        handle = open(lock, "rb")
    except OSError:
        return open(os.devnull)
    try:
        fcntl.flock(handle, fcntl.LOCK_SH | fcntl.LOCK_NB)
    except BlockingIOError:
        handle.close()
        return None
    return handle


def _source_is_idle(source: Path):
    """Shared flock on the source's .cargo-lock; None if cargo is building there."""
    return probe_shared_lock(source / ".cargo-lock")


def seed_if_cold(
    repo_root: Path,
    target_dir: Path,
    profile: str,
    log: Callable[[str], None] = print,
) -> bool:
    """Seed ``target_dir/<profile>`` from the best warm sibling; True when a seed was applied."""
    if os.environ.get(DISABLE_ENV):
        return False
    destination = target_dir / profile
    if is_warm(destination) or destination.exists() and any(destination.iterdir()):
        return False
    try:
        signature = build_signature(repo_root)
        shareable = shareable_packages(repo_root)
        sources = candidate_sources(repo_root, profile, destination, signature)
    except (OSError, RuntimeError, ValueError) as err:
        log(f"Target seeding skipped: {err}")
        return False

    if not sources:
        try:
            seen = _describe_rejections(repo_root, profile, destination)
        except OSError as err:
            seen = f"unavailable ({err})"
        log(
            f"Target seeding: no warm sibling with build signature {signature};"
            f" building dependencies from scratch. Seen: {seen}"
        )
        return False

    for source in sources:
        guard = _source_is_idle(source)
        if guard is None:
            log(f"Target seeding: {source} is being built right now, trying the next candidate.")
            continue
        staging = target_dir / f"{profile}.seeding-{os.getpid()}"
        try:
            shutil.rmtree(staging, ignore_errors=True)
            log(f"Cold target dir: seeding third-party artifacts from {source} ...")
            files, size = copy_third_party(source, staging, shareable)
            if destination.exists():
                if any(destination.iterdir()):  # a concurrent build seeded or started first
                    shutil.rmtree(staging, ignore_errors=True)
                    return False
                destination.rmdir()
            os.rename(staging, destination)
            log(f"Seeded {files} files ({size / 2**30:.1f} GiB). Only workspace crates will compile.")
            return True
        except OSError as err:
            log(f"Target seeding failed ({err}); building from scratch.")
            shutil.rmtree(staging, ignore_errors=True)
            return False
        finally:
            guard.close()
    log("Target seeding: every candidate source is busy right now; building from scratch.")
    return False
