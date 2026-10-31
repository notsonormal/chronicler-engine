#!/bin/bash
# Run a command, report wall time and peak RSS (VmHWM) of the process tree.
out="$1"; shift
start=$(date +%s.%N)
"$@" > "$out" 2>&1 &
pid=$!
peak=0
while kill -0 $pid 2>/dev/null; do
  for p in $pid $(pgrep -P $pid 2>/dev/null); do
    if [ -r /proc/$p/status ]; then
      hwm=$(awk '/VmHWM/{print $2}' /proc/$p/status 2>/dev/null)
      [ -n "$hwm" ] && [ "$hwm" -gt "$peak" ] && peak=$hwm
    fi
  done
  sleep 0.2
done
wait $pid; rc=$?
end=$(date +%s.%N)
python3 -c "print('WALL=%.2fs PEAK_RSS=%dMB EXIT=%d' % (float('$end')-float('$start'), $((peak/1024)), $rc))"
