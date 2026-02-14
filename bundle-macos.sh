#!/usr/bin/env bash
# Creates Wallpicker.app in the project root from a release build.
set -euo pipefail

APP_NAME="Wallpicker"
BUNDLE="${APP_NAME}.app"
BINARY_NAME="wallpicker"

# Build release binary
echo "Building release binary..."
cargo build --release

BINARY="target/release/${BINARY_NAME}"

if [ ! -f "${BINARY}" ]; then
    echo "Error: binary not found at ${BINARY}" >&2
    exit 1
fi

# Clean and create bundle structure
rm -rf "${BUNDLE}"
mkdir -p "${BUNDLE}/Contents/MacOS"
mkdir -p "${BUNDLE}/Contents/Resources"

# Copy binary
cp "${BINARY}" "${BUNDLE}/Contents/MacOS/${BINARY_NAME}"

# Write Info.plist
VERSION=$(cargo metadata --no-deps --format-version 1 \
    | python3 -c "import sys,json; pkgs=json.load(sys.stdin)['packages']; \
      print(next(p['version'] for p in pkgs if p['name']=='wallpicker'))")

cat > "${BUNDLE}/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.wallpicker.app</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleExecutable</key>
    <string>${BINARY_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>LSUIElement</key>
    <false/>
</dict>
</plist>
PLIST

echo "Bundle created: ${BUNDLE}"
echo "Run with: open ${BUNDLE}"
echo "Or install to /Applications: cp -r ${BUNDLE} /Applications/"
