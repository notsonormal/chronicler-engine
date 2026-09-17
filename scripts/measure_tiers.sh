#!/usr/bin/env bash
# Measure per-test wall time for the tier-2 stub tests against their tier-3
# engine-backed counterparts. Runs each test directly through the browser test
# binary (no nextest retry, no parallelism) and prints the median of N runs.
#
# Usage: scripts/measure_tiers.sh [runs-per-test]
set -u

RUNS="${1:-3}"
BIN="$(ls -t target/debug/deps/browser-* 2>/dev/null | grep -v '\.d$' | head -1)"
if [ -z "$BIN" ]; then
  echo "no browser test binary found under target/debug/deps — build first" >&2
  exit 1
fi

printf '%-58s %8s %8s\n' "test" "median" "runs"
printf '%-58s %8s %8s\n' "----" "------" "----"

for test in \
  "tier2::test_slash_menu_opens_on_slash_tier2" \
  "slash_menu::test_slash_menu_opens_on_slash" \
  "tier2::test_error_toast_on_action_failure_tier2" \
  "dashboard::test_error_toast_on_action_failure"
do
  times=()
  ok=0
  for _ in $(seq 1 "$RUNS"); do
    start=$(date +%s.%N)
    if "$BIN" --exact "$test" >/dev/null 2>&1; then
      ok=$((ok + 1))
    fi
    end=$(date +%s.%N)
    times+=("$(echo "$end - $start" | bc)")
  done
  median=$(printf '%s\n' "${times[@]}" | sort -n | awk '{a[NR]=$1} END {print (NR%2==1) ? a[(NR+1)/2] : (a[NR/2]+a[NR/2+1])/2}')
  printf '%-58s %7.2fs %5d/%d\n' "$test" "$median" "$ok" "$RUNS"
done
