#!/bin/sh
# Cargo linker wrapper: link with the rust-lld inside the rustup toolchain, else plain `cc`.
# build.py installs this at one fixed path and passes it via CARGO_TARGET_<triple>_LINKER. Do not
# set `linker` in .cargo/config.toml: cargo hashes the linker's absolute path into every unit, so a
# per-checkout path would stop worktrees from reusing each other's seeded artifacts.
# `CHRONICLER_NO_LLD=1` makes build.py skip it.
# The gcc-ld dir is cached because rustc runs this wrapper for every build script and proc macro.
cache="${TMPDIR:-/tmp}/chronicler-gcc-ld-$(id -u)"
gccld="$(cat "$cache" 2>/dev/null)"
if [ ! -x "$gccld/ld.lld" ]; then
    host="$(rustc -vV 2>/dev/null | sed -n 's/^host: //p')"
    gccld="$(rustc --print sysroot 2>/dev/null)/lib/rustlib/$host/bin/gcc-ld"
    [ -x "$gccld/ld.lld" ] && echo "$gccld" > "$cache.$$" && mv "$cache.$$" "$cache"
fi
if [ -x "$gccld/ld.lld" ]; then exec cc -fuse-ld=lld -B"$gccld" "$@"; fi
exec cc "$@"
