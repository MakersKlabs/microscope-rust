#!/bin/sh
# Sign and notarize dist/Microscope.app.
#
# Required environment variables for signing/notarization (GitHub Actions secrets):
#   MACOS_CERT_P12_BASE64   base64 of the Developer ID Application certificate (.p12)
#   MACOS_CERT_PASSWORD     password of the .p12
#   MACOS_SIGN_IDENTITY     e.g. "Developer ID Application: Your Name (TEAMID)"
#   MACOS_TEAM_ID           Apple Team ID
#   MACOS_NOTARY_APPLE_ID   Apple ID used for notarization
#   MACOS_NOTARY_PASSWORD   app-specific password for that Apple ID
#   KEYCHAIN_PASSWORD       temporary keychain password (any value)
#
# If MACOS_CERT_P12_BASE64 is not set, the script exits without signing.
set -eu

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
APP="$ROOT/dist/Microscope.app"

if [ ! -d "$APP" ]; then
    echo "Bundle not found: $APP (run packaging/macos/bundle.sh first)"
    exit 1
fi

if [ -z "${MACOS_CERT_P12_BASE64:-}" ]; then
    echo "MACOS_CERT_P12_BASE64 not set: skipping signing and notarization."
    exit 0
fi

KEYCHAIN="$RUNNER_TEMP/app-signing.keychain-db"
CERT="$RUNNER_TEMP/certificate.p12"
echo "$MACOS_CERT_P12_BASE64" | base64 --decode > "$CERT"

security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
security set-keychain-settings -lut 21600 "$KEYCHAIN"
security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN"
security import "$CERT" -P "$MACOS_CERT_PASSWORD" -A -t cert -f pkcs12 -k "$KEYCHAIN"
security list-keychain -d user -s "$KEYCHAIN" $(security list-keychains -d user | tr -d '"')
security set-key-partition-list -S apple-tool:,apple: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN" >/dev/null

IDENTITY="${MACOS_SIGN_IDENTITY:-Developer ID Application}"

echo "Signing $APP ..."
codesign --force --deep --options runtime \
    --entitlements "$ROOT/packaging/macos/entitlements.plist" \
    --sign "$IDENTITY" "$APP"
codesign --verify --deep --strict --verbose=2 "$APP"

echo "Notarizing ..."
ZIP="$ROOT/dist/Microscope-notarize.zip"
ditto -c -k --keepParent "$APP" "$ZIP"
xcrun notarytool submit "$ZIP" \
    --apple-id "$MACOS_NOTARY_APPLE_ID" \
    --team-id "$MACOS_TEAM_ID" \
    --password "$MACOS_NOTARY_PASSWORD" \
    --wait

xcrun stapler staple "$APP"
spctl --assess --type execute --verbose=2 "$APP"

echo "Signed and notarized: $APP"
