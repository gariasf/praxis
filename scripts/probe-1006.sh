#!/bin/bash
# Usage: scripts/probe-1006.sh <results directory>
# ROUNDS (3), CHOSEN (the variant run again without shader runtime checks),
# QUIET_WAIT_SECONDS (300) and PRAXIS_BINARY (target/release/praxis) are read
# from the environment.
set -u

results=${1:?results directory}
praxis=${PRAXIS_BINARY:-target/release/praxis}
rounds=${ROUNDS:-3}
chosen=${CHOSEN:-town-empty-256}
quiet_wait_seconds=${QUIET_WAIT_SECONDS:-300}

world_camera=4416,16,1920
town_camera=0,16,0

variants=()
for rays in none 8 64 256; do
    variants+=("world-256m-$rays" "town-empty-$rays" "town-crowd-$rays")
done
for rays in 8 64 256; do
    variants+=("world-2000m-$rays")
done
variants+=("world-256m-64-indirect" "town-empty-64-indirect" "town-crowd-64-indirect" "$chosen-unchecked")

run_variant() {
    local name=$1 output=$2
    local scene state rays extra
    IFS=- read -r scene state rays extra <<<"$name"
    local environment=()
    case $scene in
        world) environment+=("PRAXIS_PROBE_CAMERA=$world_camera" "PRAXIS_PROBE_RAY_METRES=${state%m}") ;;
        town) environment+=("PRAXIS_PROBE_CAMERA=$town_camera" "PRAXIS_PROBE_RAY_METRES=256" "PRAXIS_PROBE_MOVING=$state") ;;
    esac
    if [ "$rays" != none ]; then
        environment+=("PRAXIS_PROBE_RAYS_PER_PROBE=$rays")
    fi
    environment+=("PRAXIS_PROBE_BLOCK_FRAMES=50" "RUST_LOG=praxis=info")
    if [ "$extra" != indirect ]; then
        environment+=("PRAXIS_PROBE_DIRECT_DISPATCH=1")
    fi
    if [ "$extra" = unchecked ]; then
        environment+=("PRAXIS_PROBE_UNCHECKED=1")
    fi
    mkdir -p "$output"
    env "${environment[@]}" "$praxis" bench "$output" --skip-aliasing --no-publish "$scene" 2>&1 | tee "$output/log.txt"
}

wait_for_quiet_machine() {
    local waited_seconds=0
    while "$praxis" preflight 2>/dev/null | grep -Eq "^warn +(gpu load|cpu load|processes)"; do
        if [ "$waited_seconds" -ge "$quiet_wait_seconds" ]; then
            echo "$(date +%T) machine still not quiet after ${waited_seconds}s"
            return
        fi
        sleep 5
        waited_seconds=$((waited_seconds + 5))
    done
}

count=${#variants[@]}
for round in $(seq 1 "$rounds"); do
    for position in $(seq 0 $((count - 1))); do
        if [ $((round % 2)) -eq 0 ]; then
            index=$(((count - 1 - position + round) % count))
        else
            index=$(((position + round - 1) % count))
        fi
        name=${variants[$index]}
        if command -v pmset >/dev/null && ! pmset -g batt | grep -q "AC Power"; then
            echo "$(date +%T) on battery: stopped before round $round $name"
            exit 1
        fi
        wait_for_quiet_machine
        echo "$(date +%T) round $round $name"
        run_variant "$name" "$results/round-$round/$name" | grep -oE "probe_ray_pass_alone_us=[0-9.]+|probe_ray_frame_growth_us=[0-9.-]+|probe_ray_pass=[0-9.]+|gpu_us median=[0-9.]+" | tr "\n" " "; echo
    done
done
echo "$(date +%T) turn done"
