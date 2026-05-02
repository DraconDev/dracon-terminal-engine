#!/usr/bin/env bash
# Dracon Terminal Engine — Example Showcase Launcher
#
# Builds all examples and launches the interactive showcase.
# Usage:
#   ./showcase.sh              # Build + launch showcase
#   ./showcase.sh <example>    # Build + run a specific example directly
#
# Examples:
#   ./showcase.sh widget_gallery
#   ./showcase.sh tree_navigator
#   ./showcase.sh log_monitor

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

EXAMPLE="${1:-}"

if [ -n "$EXAMPLE" ]; then
    echo "==> Building and running: $EXAMPLE"
    cargo build --example "$EXAMPLE" 2>&1
    echo ""
    echo "==> Launching $EXAMPLE..."
    cargo run --example "$EXAMPLE"
else
    echo "==> Building all examples..."
    cargo build --examples 2>&1
    echo ""
    echo "==> Launching showcase..."
    cargo run --example showcase
fi
