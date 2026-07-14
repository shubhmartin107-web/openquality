#!/bin/bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/.."

echo "Installing Python dependencies..."
pip install pandas numpy -q 2>/dev/null || true

echo ""
echo "Running OpenQuality benchmark suite..."
echo ""

PYTHONPATH="$SCRIPT_DIR/../python/src" python "$SCRIPT_DIR/injected_anomalies.py"
