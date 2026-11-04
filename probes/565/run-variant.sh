#!/bin/zsh
# usage: run-variant.sh <name> <distinct_depth> <distinct_lit> <round>
set -e
name=$1; depth=$2; lit=$3; round=$4
out=/Users/guillem.arias/Documents/gariasf/praxis/pending-artifacts/565/runs/$name.r$round
rm -rf $out; mkdir -p $out
cd /Users/guillem.arias/Documents/gariasf/praxis-565
echo "$(date -u +%FT%TZ) $(uptime | sed 's/.*load/load/') $(pmset -g batt | head -1)" > $out/machine.txt
/usr/bin/time -l -o $out/time.txt env PRAXIS_PROBE_DISTINCT_DEPTH=$depth PRAXIS_PROBE_DISTINCT_LIT=$lit PRAXIS_PROBE_DUMP=$out/ticks.csv ${=EXTRA} \
  ./target/release/praxis bench $out --skip-aliasing --no-publish texture-wall > $out/log.txt 2>&1
