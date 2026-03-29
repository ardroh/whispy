#!/bin/bash
set -euo pipefail

APP_NAME="Whispy"
BUNDLE_ID="com.whispy.app"
APP_DIR="$HOME/Applications/${APP_NAME}.app"
PLIST_LABEL="${BUNDLE_ID}"
LAUNCH_AGENT="$HOME/Library/LaunchAgents/${PLIST_LABEL}.plist"

echo "==> Building Whispy in release mode..."
cargo build --release

echo "==> Creating app bundle at ${APP_DIR}..."
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

cp target/release/whispy "${APP_DIR}/Contents/MacOS/whispy"
cp Info.plist "${APP_DIR}/Contents/Info.plist"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
if [[ -f "${REPO_ROOT}/assets/AppIcon.icns" ]]; then
    cp "${REPO_ROOT}/assets/AppIcon.icns" "${APP_DIR}/Contents/Resources/AppIcon.icns"
fi

echo "==> Registering to start at login..."
# Unload existing agent if present
launchctl bootout "gui/$(id -u)/${PLIST_LABEL}" 2>/dev/null || true

cat > "${LAUNCH_AGENT}" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${PLIST_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>${APP_DIR}/Contents/MacOS/whispy</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>StandardOutPath</key>
    <string>${HOME}/Library/Logs/whispy.log</string>
    <key>StandardErrorPath</key>
    <string>${HOME}/Library/Logs/whispy.log</string>
</dict>
</plist>
EOF

launchctl bootstrap "gui/$(id -u)" "${LAUNCH_AGENT}"

echo ""
echo "==> Done!"
echo "   App installed to: ${APP_DIR}"
echo "   Launch agent:     ${LAUNCH_AGENT}"
echo "   Logs:             ~/Library/Logs/whispy.log"
echo ""
echo "   Whispy will start automatically at login."
echo "   On first launch, macOS will ask for Microphone and Accessibility (for paste)."
echo "   You can re-check anytime from the menu bar: Check Permissions..."
echo "   To start now:  open '${APP_DIR}'"
echo "   To stop:       launchctl bootout gui/$(id -u)/${PLIST_LABEL}"
echo "   To uninstall:  bash scripts/uninstall.sh"
