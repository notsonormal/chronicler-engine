#!/bin/sh
# build.py installs this at one fixed path: cargo hashes the linker's absolute path into every unit
# fingerprint, so a per-checkout `linker` in .cargo/config.toml would stop worktrees reusing each
# other's seeds. `CHRONICLER_NO_LLD=1` opts out. The gcc-ld dir is cached because rustc runs this
# wrapper for every build script and proc macro.
cache="${TMPDIR:-/tmp}/chronicler-gcc-ld-$(id -u)"
gccld="$(cat "$cache" 2>/dev/null)"
if [ ! -x "$gccld/ld.lld" ]; then
    host="$(rustc -vV 2>/dev/null | sed -n 's/^host: //p')"
    gccld="$(rustc --print sysroot 2>/dev/null)/lib/rustlib/$host/bin/gcc-ld"
    [ -x "$gccld/ld.lld" ] && echo "$gccld" > "$cache.$$" && mv "$cache.$$" "$cache"
fi
if [ -x "$gccld/ld.lld" ]; then exec cc -fuse-ld=lld -B"$gccld" "$@"; fi
exec cc "$@"
