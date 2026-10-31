#!/bin/bash
# ab.sh <config-basename> : run TLC and tla-ts on one config, record wall time, peak RSS, state counts.
cd /workspace/chronicler-engine/tmp/tla-ts-run
cfg="$1"
export JAVA=/tmp/statechart-tools/jdk/bin/java TLA2TOOLS=/tmp/statechart-tools/tla2tools.jar
mkdir -p out meta
rm -rf "meta/$cfg"
echo "##### $cfg"
echo -n "TLC     "; ./measure.sh "out/tlc-$cfg.txt" $JAVA -XX:+UseParallelGC -Djava.io.tmpdir=/tmp -cp $TLA2TOOLS tlc2.TLC -metadir "/tmp/tla-ts-run/meta/$cfg" -config "$cfg.cfg" -workers 1 -fp 0 GateSpecOnly.tla
echo -n "tla-ts  "; ./measure.sh "out/tlats-$cfg.txt" ./node_modules/.bin/tla-ts GateSpecOnly.tla -config "$cfg.cfg"
echo "-- TLC summary:";     grep -E "states generated|distinct states|depth of the complete|violated|exited|Finished in" "out/tlc-$cfg.txt" | tail -4
echo "-- tla-ts summary:";  grep -E "states generated|distinct states|depth of the complete|violated|exited" "out/tlats-$cfg.txt" | tail -4
