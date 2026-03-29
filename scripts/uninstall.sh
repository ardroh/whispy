#!/bin/bash
set -euo pipefail

APP_NAME="Whispy"
BUNDLE_ID="com.whispy.app"
APP_DIR="$HOME/Applications/${APP_NAME}.app"
LAUNCH_AGENT="$HOME/Library/LaunchAgents/${BUNDLE_ID}.plist"

echo "==> Stopping Whispy..."
launchctl bootout "gui/$(id -u)/${BUNDLE_ID}" 2>/dev/null || true

echo "==> Resetting microphone & accessibility privacy decisions for ${BUNDLE_ID}..."
tccutil reset Microphone "${BUNDLE_ID}" 2>/dev/null || true
tccutil reset Accessibility "${BUNDLE_ID}" 2>/dev/null || true

echo "==> Removing launch agent..."
rm -f "${LAUNCH_AGENT}"

echo "==> Removing app bundle..."
rm -rf "${APP_DIR}"

WHISPY_SUPPORT="$HOME/Library/Application Support/whispy"
if [[ -f "${WHISPY_SUPPORT}/.permissions_intro_shown" ]]; then
    echo "==> Removing first-run permissions marker..."
    rm -f "${WHISPY_SUPPORT}/.permissions_intro_shown"
fi

echo "==> Done! Whispy has been uninstalled."
echo "   Config is still at: ~/Library/Application Support/whispy/"
echo "   Remove it manually if you want a clean uninstall."
