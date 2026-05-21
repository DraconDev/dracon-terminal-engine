#!/usr/bin/env bash
# bench_showcase.sh - Measure showcase frame render time (release mode)

set -e

cd /home/dracon/Dev/dracon-terminal-engine

echo "Running showcase frame benchmark (release mode)..."

# Run the performance benchmarks in release mode and extract metrics
output=$(cargo test --release --test performance_benchmarks -- --nocapture 2>&1)

# Extract timing metrics - handle both µs and ms formats
# Format examples: "854.796µs", "10.364329ms"
parse_time() {
    local line="$1"
    local ms=$(echo "$line" | grep -oP '[\d.]+(?:ms|µs|us)' | head -1)
    if [[ "$ms" == *"ms" ]]; then
        # Already milliseconds
        echo "$ms" | grep -oP '[\d.]+'
    elif [[ "$ms" == *"µs" ]] || [[ "$ms" == *"us" ]]; then
        # Convert microseconds to milliseconds
        echo "$ms" | grep -oP '[\d.]+' | awk '{print $1/1000}'
    else
        echo "0"
    fi
}

compositor_50=$(echo "$output" | grep "Compositor with 50 planes" | head -1)
compositor_200=$(echo "$output" | grep "Compositor with 200 planes" | head -1)
large_terminal=$(echo "$output" | grep "200x100 terminal render" | head -1)

compositor_50_ms=$(parse_time "$compositor_50")
compositor_200_ms=$(parse_time "$compositor_200")
large_terminal_ms=$(parse_time "$large_terminal")

# Calculate primary metric (frame_us) in microseconds
frame_us=$(python3 -c "print(int($large_terminal_ms * 1000))" 2>/dev/null || echo "0")

echo "METRIC compositor_50_ms=$compositor_50_ms"
echo "METRIC compositor_200_ms=$compositor_200_ms"
echo "METRIC large_terminal_ms=$large_terminal_ms"
echo "METRIC frame_us=$frame_us"