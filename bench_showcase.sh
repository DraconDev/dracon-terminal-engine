#!/usr/bin/env bash
# bench_showcase.sh - Measure showcase frame render time (release mode, stable)

cd /home/dracon/Dev/dracon-terminal-engine

echo "Running showcase frame benchmark (release mode, 5 iterations)..."

# Warm-up run first
cargo test --release --test performance_benchmarks benchmark_large_terminal_200x100 -- --nocapture 2>/dev/null

# Run 5 iterations and collect times using Python for reliable parsing
python3 << 'EOF'
import subprocess
import re

times = []
for i in range(5):
    result = subprocess.run(
        ["cargo", "test", "--release", "--test", "performance_benchmarks", 
         "benchmark_large_terminal_200x100", "--", "--nocapture"],
        capture_output=True, text=True, cwd="/home/dracon/Dev/dracon-terminal-engine"
    )
    output = result.stdout
    # Find "200x100 terminal render: X.XXXms"
    match = re.search(r'200x100 terminal render: ([\d.]+)', output)
    if match:
        time_us = int(float(match.group(1)) * 1000)
        times.append(time_us)
        print(f"  Run {i+1}: {time_us}µs")

avg_us = sum(times) // len(times) if times else 0

# Get compositor metrics
result = subprocess.run(
    ["cargo", "test", "--release", "--test", "performance_benchmarks", "--", "--nocapture"],
    capture_output=True, text=True, cwd="/home/dracon/Dev/dracon-terminal-engine"
)
output = result.stdout

comp_50 = re.search(r'Compositor with 50 planes: ([\d.]+)', output)
comp_200 = re.search(r'Compositor with 200 planes: ([\d.]+)', output)
comp_50_val = float(comp_50.group(1)) / 1000 if comp_50 else 0
comp_200_val = float(comp_200.group(1)) / 1000 if comp_200 else 0

print(f"=== Benchmark Results ===")
print(f"METRIC compositor_50_ms={comp_50_val:.3f}")
print(f"METRIC compositor_200_ms={comp_200_val:.3f}")
print(f"METRIC large_terminal_ms={avg_us/1000:.3f}")
print(f"METRIC frame_us={avg_us}")
EOF