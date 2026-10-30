#!/bin/sh
# Cargo rustc-wrapper: use sccache when it is installed, otherwise run the
# compiler cargo passed us. Override the binary with SCCACHE_BIN=/path/to/sccache.
#
# sccache is not in the repo. Install it by unpacking the prebuilt
# `sccache-v<version>-x86_64-unknown-linux-musl.tar.gz` that the mozilla/sccache releases publish
# onto `~/.local/bin` (v0.19.0, 2026-10-30). A missing binary costs cache hits, not correctness:
# builds stay green and slower, which is the only symptom.
#
# CARGO_TARGET_DIR is dropped for the sccache child because sccache hashes its
# environment: build.py sets that variable to a different path per agent, which
# would make every worktree miss the whole cache. rustc does not need it — cargo
# passes --out-dir and -L explicitly.
SCCACHE_BIN="${SCCACHE_BIN:-$HOME/.local/bin/sccache}"
if [ -x "$SCCACHE_BIN" ]; then exec env -u CARGO_TARGET_DIR "$SCCACHE_BIN" "$@"; fi
if command -v sccache >/dev/null 2>&1; then exec env -u CARGO_TARGET_DIR sccache "$@"; fi
exec "$@"
