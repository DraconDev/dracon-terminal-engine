#!/usr/bin/env bash
# bench_showcase.sh - Measure showcase frame render time (release mode, stable)

set -e

cd /home/dracon/Dev/dracon-terminal-engine

echo "Running showcase frame benchmark (release mode, 5 iterations)..."

# Warm-up run first
cargo test --release --test performance_benchmarks benchmark_large_terminal_200x100 -- --nocapture 2>/dev/null

# Run multiple iterations and collect times
declare -a times
for i in {1..5}; do
    output=$(cargo test --release --test performance_benchmarks benchmark_large_terminal_200x100 -- --nocapture 2>&1)
    time=$(echo "$output" | grep "200x100 terminal render" | grep -oP '[\d.]+')
    times+=("$time")
done

# Calculate average
total=0
for t in "${times[@]}"; do
    total=$(python3 -c "print($total + $t)")
done
avg=$(python3 -c "print(round($total / ${#times[@]}, 3))")

# Also get compositor metrics
output=$(cargo test --release --test performance_benchmarks -- --nocapture 2>&1)
comp_50=$(echo "$output" | grep "Compositor with 50 planes" | grep -oP '[\d.]+' | head -1)
comp_200=$(echo "$output" | grep "Compositor with 200 planes" | grep -oP '[\d.]+' | head -1)

# Convert to ms and calculate frame_us
frame_us=$(python3 -c "print(int($avg * 1000))")

echo "METRIC compositor_50_ms=$comp_50"
echo "METRIC compositor_200_ms=$comp_200"
echo "METRIC large_terminal_ms=$avg"
echo "METRIC frame_us=$frame_us"