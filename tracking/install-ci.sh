#!/bin/bash
# Install the Atlas nightly CI launchd job
# Run once: ./tracking/install-ci.sh
set -euo pipefail

PROJECT_ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
PLIST_SRC="$PROJECT_ROOT/tracking/com.atlas.nightly-ci.plist"
PLIST_DEST="$HOME/Library/LaunchAgents/com.atlas.nightly-ci.plist"

# Unload existing if already installed
if launchctl list | grep -q "com.atlas.nightly-ci" 2>/dev/null; then
    echo "Unloading existing com.atlas.nightly-ci..."
    launchctl unload "$PLIST_DEST" 2>/dev/null || true
fi

cp "$PLIST_SRC" "$PLIST_DEST"
launchctl load "$PLIST_DEST"

echo ""
echo "Atlas nightly CI installed. Runs at 2am daily."
echo ""
echo "Commands:"
echo "  Run now:      launchctl start com.atlas.nightly-ci"
echo "  Check status: launchctl list | grep atlas"
echo "  View logs:    tail -f $PROJECT_ROOT/tracking/ci-runner.log"
echo "  Uninstall:    launchctl unload $PLIST_DEST && rm $PLIST_DEST"
echo "  On-demand:    atlas-track run-ci"
echo ""
