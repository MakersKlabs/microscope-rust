# Code signing and notarization

`microscope-rust` is designed to be **signed on Windows** and **signed and
notarized on macOS** directly from GitHub Actions. Everything is driven by
repository secrets: if a secret is missing, the corresponding step is skipped
and the unsigned artifact is still produced.

## Windows (Authenticode)

### What you need
- A **code signing certificate**. Options:
  - an OV/EV certificate exported as a `.pfx` file;
  - **Azure Trusted Signing** (recommended for open source, cheaper).
- The Windows SDK, which provides `signtool.exe` (already present on the
  `windows-latest` GitHub runner).

### Secrets
| Secret | Description |
|---|---|
| `WINDOWS_CERT_PFX_BASE64` | base64 of the `.pfx` file: `base64 -w0 cert.pfx` |
| `WINDOWS_CERT_PASSWORD` | password of the `.pfx` |
| `WINDOWS_TIMESTAMP_URL` | optional, defaults to `http://timestamp.digicert.com` |

### How it works
`packaging/windows/sign.ps1` decodes the certificate, runs
`signtool sign /fd sha256 /tr <timestamp> /td sha256` and verifies the result.
The release workflow signs both `microscope-rust.exe` and the Inno Setup
installer.

### Azure Trusted Signing instead of a PFX
Set up `azure/trusted-signing-action` as an extra step and remove the
`WINDOWS_CERT_PFX_BASE64` secret; the rest of the pipeline is unchanged.

## macOS (codesign + notarization)

### What you need
- An **Apple Developer Program** membership.
- A **Developer ID Application** certificate (`.p12`) and its private key.
- An **app-specific password** for the Apple ID used with `notarytool`.

### Secrets
| Secret | Description |
|---|---|
| `MACOS_CERT_P12_BASE64` | base64 of the Developer ID `.p12` |
| `MACOS_CERT_PASSWORD` | password of the `.p12` |
| `MACOS_SIGN_IDENTITY` | e.g. `Developer ID Application: Name (TEAMID)` |
| `MACOS_TEAM_ID` | 10-character Apple Team ID |
| `MACOS_NOTARY_APPLE_ID` | Apple ID for notarization |
| `MACOS_NOTARY_PASSWORD` | app-specific password |
| `KEYCHAIN_PASSWORD` | any value, used for the temporary keychain |

### How it works
`packaging/macos/sign_and_notarize.sh`:
1. creates a temporary keychain and imports the certificate;
2. `codesign --force --deep --options runtime --entitlements ...`;
3. `xcrun notarytool submit --wait`;
4. `xcrun stapler staple`;
5. `spctl --assess` to verify the result.

### Camera permission (important)
The app accesses the camera, so the bundle **must** declare
`NSCameraUsageDescription` in `Info.plist` (already included) and, because the
hardened runtime is enabled, the
`com.apple.security.device.camera` entitlement
(`packaging/macos/entitlements.plist`). Without them macOS terminates the app
when it opens the camera.

## Building signed artifacts locally

```bash
# Windows (PowerShell)
$env:WINDOWS_CERT_PFX_BASE64 = [Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pfx"))
$env:WINDOWS_CERT_PASSWORD = "password"
pwsh packaging/windows/sign.ps1 -File target/release/microscope-rust.exe

# macOS
export MACOS_CERT_P12_BASE64="$(base64 -i cert.p12)"
export MACOS_CERT_PASSWORD="password"
export MACOS_SIGN_IDENTITY="Developer ID Application: Name (TEAMID)"
export MACOS_TEAM_ID="TEAMID"
export MACOS_NOTARY_APPLE_ID="you@example.com"
export MACOS_NOTARY_PASSWORD="app-specific-password"
export KEYCHAIN_PASSWORD="ci"
sh packaging/macos/bundle.sh
sh packaging/macos/sign_and_notarize.sh
```

## Linux
Native Linux binaries are not signed. Distribution is done through the
self-contained AppImage (or a Flatpak if you prefer), which avoids the
dependency problem entirely.
