#!/usr/bin/env bash
# Stress the worlds posture flow without retry masking.
#
# Runs the tier-3 wiring test directly through the test binary
# (`--exact --nocapture`) N times, keeping the loop independent of nextest's
# retry configuration and capturing each run log. Any lost interaction — the
# ticket-03 legacy signature — fails the run and is counted.
#
# Usage: scripts/stress_posture.sh [runs]
#
# Respects CARGO_TARGET_DIR (default `target`), matching build.py's
# `--target-dir`. Both the binary lookup and the harness's own engine-binary
# resolution read that variable, so under a concurrent build the script grades
# the tree the agent actually built rather than a stale `target/debug`.
set -u

RUNS="${1:-50}"
LOG_DIR="tmp/posture_stress"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"
export CARGO_TARGET_DIR="$TARGET_DIR"
BIN="$(ls -t "$TARGET_DIR"/debug/deps/browser-* 2>/dev/null | grep -v '\.d$' | head -1)"

if [ -z "$BIN" ]; then
  echo "no browser test binary found under $TARGET_DIR/debug/deps — build first" >&2
  exit 1
fi

mkdir -p "$LOG_DIR"
rm -f "$LOG_DIR"/*.log "$LOG_DIR"/summary.txt

pass=0
fail=0
lost_interaction=0
declare -a failures=()

for i in $(seq 1 "$RUNS"); do
  run_log="$LOG_DIR/run_$(printf '%03d' "$i").log"
  start=$(date +%s.%N)
  "$BIN" --exact worlds::test_world_posture_change_autosaves_server_state --nocapture \
    >"$run_log" 2>&1
  code=$?
  end=$(date +%s.%N)
  elapsed=$(echo "$end - $start" | bc)

  if [ "$code" -eq 0 ]; then
    pass=$((pass + 1))
    status="PASS"
  else
    fail=$((fail + 1))
    status="FAIL"
    failures+=("run $i")
    # Legacy signature: the gate never saw the posture span settle.
    if grep -q "never settled '#world-posture-status'" "$run_log"; then
      lost_interaction=$((lost_interaction + 1))
      status="FAIL(lost-interaction)"
    fi
  fi

  printf 'run %3d/%d  %-22s  %6.2fs\n' "$i" "$RUNS" "$status" "$elapsed"
  printf 'run %3d  %-22s  %6.2fs\n' "$i" "$status" "$elapsed" >>"$LOG_DIR/summary.txt"
done

cat >"$LOG_DIR/verdict.txt" <<EOF
runs:               $RUNS
pass:               $pass
fail:               $fail
lost interaction:   $lost_interaction
failed runs:        ${failures[*]:-none}
EOF

echo
cat "$LOG_DIR/verdict.txt"
