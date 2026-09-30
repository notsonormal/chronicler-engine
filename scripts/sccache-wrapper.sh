#!/bin/sh
# Cargo rustc-wrapper: use sccache when it is installed, otherwise run the
# compiler cargo passed us. Override the binary with SCCACHE_BIN=/path/to/sccache.
#
# CARGO_TARGET_DIR is dropped for the sccache child because sccache hashes its
# environment: build.py sets that variable to a different path per agent, which
# would make every worktree miss the whole cache. rustc does not need it — cargo
# passes --out-dir and -L explicitly.
SCCACHE_BIN="${SCCACHE_BIN:-$HOME/.local/bin/sccache}"
if [ -x "$SCCACHE_BIN" ]; then exec env -u CARGO_TARGET_DIR "$SCCACHE_BIN" "$@"; fi
if command -v sccache >/dev/null 2>&1; then exec env -u CARGO_TARGET_DIR sccache "$@"; fi
exec "$@"
