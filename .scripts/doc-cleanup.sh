#!/bin/bash
# Dracon Terminal Engine — Documentation Cleanup Script
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Dracon Terminal Engine — Doc Cleanup ==="

# Phase 1: Archive superseded docs
echo ""
echo "Phase 1: Archiving superseded docs to .archive/..."
mkdir -p "$PROJECT_DIR/.archive"

for f in \
    "$PROJECT_DIR/ENRICHMENT.md" \
    "$PROJECT_DIR/AUDIT_REPORT.md" \
    "$PROJECT_DIR/RESEARCH.md" \
    "$PROJECT_DIR/MANUAL_TESTING_REPORT.md" \
    "$PROJECT_DIR/MIGRATION.md" \
    "$PROJECT_DIR/plans/" \
    "$PROJECT_DIR/.ralph/" \

; do
    if [ -e "$f" ]; then
        echo "  Moving $f → .archive/"
        mv "$f" "$PROJECT_DIR/.archive/"
    fi
done

# Phase 2: Delete truly temporary/garbage files
echo ""
echo "Phase 2: Deleting temporary/obsolete files..."
for f in \
    "$PROJECT_DIR/autoresearch.md" \
    "$PROJECT_DIR/autoresearch.ideas.md" \
    "$PROJECT_DIR/todo.md" \
    "$PROJECT_DIR/note.md" \
    "$PROJECT_DIR/TASKS.md" \
; do
    if [ -f "$f" ]; then
        echo "  Deleting $f"
        rm "$f"
    fi
done

echo ""
echo "=== Cleanup Complete ==="
echo "Remaining docs in root:"
ls -lh "$PROJECT_DIR"/*.md 2>/dev/null | awk '{print "  " $5, $NF}' | sed 's|.*/||'
