#!/usr/bin/env bash
# bench_showcase.sh - Measure showcase frame render time (release mode, stable)

set -e

cd /home/dracon/Dev/dracon-terminal-engine

echo "Running showcase frame benchmark (release mode, 5 iterations)..."

# Warm-up run first
cargo test --release --test performance_benchmarks benchmark_large_terminal_200x100 -- --nocapture 2>/dev/null

# Run multiple iterations and collect times (in microseconds)
total_us=0
count=0
for i in {1..5}; do
    output=$(cargo test --release --test performance_benchmarks benchmark_large_terminal_200x100 -- --nocapture 2>&1)
    # Get only the first matching time (from current iteration)
    time_us=$(echo "$output" | grep "200x100 terminal render:" | head -1 | grep -oP '[\d.]+' | awk '{print int($1 * 1000)}')
    total_us=$((total_us + time_us))
    count=$((count + 1))
done

# Calculate average in microseconds
avg_us=$((total_us / count))

# Also get compositor metrics
output=$(cargo test --release --test performance_benchmarks -- --nocapture 2>&1)
comp_50=$(echo "$output" | grep "Compositor with 50 planes:" | head -1 | grep -oP '[\d.]+' | head -1)
comp_200=$(echo "$output" | grep "Compositor with 200 planes:" | head -1 | grep -oP '[\d.]+' | head -1)
large_ms=$(echo "$avg_us" | awk '{printf "%.3f", $1 / 1000}')

echo "METRIC compositor_50_ms=$comp_50"
echo "METRIC compositor_200_ms=$comp_200"
echo "METRIC large_terminal_ms=$large_ms"
echo "METRIC frame_us=$avg_us"