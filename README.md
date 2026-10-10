# microscope-rust

A tiny, retro-styled (1980s CRT terminal) USB microscope capture application,
written in **Rust**, that builds as a **single self-contained binary** for
**Linux, Windows and macOS**.

> 🇮🇹 Italian version: [README.it.md](README.it.md)

```
+------------------------------------------------------------+
| MICROSCOPE        CAMERA [H1600 Cam]  RES [1920 x 1080]    |
| FPS [30]  ROTATE 90  MIRROR-H  MIRROR-V   FIT PHOTO REC    |
+------------------------------------------------------------+
|                                                            |
|                 live microscope image                      |
|                                                            |
+------------------------------------------------------------+
| H1600 Cam | 1920 x 1080 | 29.8 fps | REC 00:42            |
+------------------------------------------------------------+
```

## Why this version

The original `microscope-app` is Python + Qt + OpenCV: very fast to develop, but
it depends on the Python interpreter and a stack of system packages. This Rust
rewrite is the opposite trade-off:

| | microscope-app (Python) | microscope-rust |
|---|---|---|
| Runtime needed | Python 3 + Qt + OpenCV + PyAV | **none** (single binary) |
| Binary size | n/a (source) / ~18 KB `.deb` + deps | **~5.5 MB** |
| Video encoder license | x264 (**GPL**) | OpenH264 (**BSD**) |
| Exit code / dependencies | many system packages | self-contained |
| Cross-platform | with effort | Linux / Windows / macOS |

It is **not** a wrapper and it is not "everything written from scratch": it uses
small, permissively licensed crates (see *Dependencies*), statically linked.

## Features

Same features as the original application:

- **Camera selection** among the connected USB cameras.
- **Resolution and frame rate** selection from the camera capabilities,
  **remembered per camera** (with a closest-match fallback when the stored
  resolution is no longer offered).
- **Rotation** 0/90/180/270 and **mirroring** on both axes, **remembered per
  camera**.
- **Navigation**: wheel zoom, left-drag pan, right-click fit.
- **PNG/JPEG photo capture**, remembering the last folder.
- **Compressed H.264 video recording** (OpenH264 + pure-Rust MP4 muxer),
  remembering the last folder.
- **Bilingual interface** (English / Italian).
- **1980s terminal look**: phosphor green on black, monospace, hard bevels and
  CRT scanlines. The scanline overlay can be disabled from **Settings →
  CRT effects**.

## Keyboard and mouse

| Input | Action |
|---|---|
| Wheel | zoom |
| Left drag | pan |
| Right click | fit to window |
| `Ctrl+S` | save photo |
| `Ctrl+R` | start/stop recording |
| `F` | fit |
| `+` / `-` | zoom in / out |

## Build

Requires a recent stable Rust toolchain (<https://rustup.rs>). No system
development packages are needed: X11, Wayland and OpenGL are loaded at runtime
through `dlopen`.

```bash
cargo build --release
# binary: target/release/microscope-rust(.exe)
```

Run it:

```bash
cargo run --release
```

Diagnose the camera list (useful when a camera shows `NO SIGNAL`):

```bash
./target/release/microscope-rust --list-cameras
```

## Packaging

### Linux
```bash
sh packaging/linux/install.sh      # user-local install (~/.local)
sh packaging/linux/make-appimage.sh # self-contained AppImage
```

### Windows
```powershell
cargo build --release
ISCC packaging\windows\installer.iss   # Inno Setup 6 -> dist\Microscope-Setup-*.exe
```

### macOS
```bash
sh packaging/macos/bundle.sh            # dist/Microscope.app + icon.icns
sh packaging/macos/sign_and_notarize.sh # if signing secrets are set
```

## Prebuilt binaries (CI)

Every push to `main`, and every manual run of the **Build artifacts** workflow,
produces ready-to-download packages in the repository's *Actions* tab:

| OS | Artifacts |
|---|---|
| Linux | `Microscope-<ver>-x86_64.AppImage`, `microscope-rust-linux-x86_64.tar.gz` |
| Windows | `Microscope-Setup-<ver>.exe`, `microscope-rust.exe` |
| macOS | `Microscope.dmg`, `Microscope.app` |

Tagged commits (`v*`) are also published on the GitHub Releases page.

## Code signing and notarization

The project is set up to sign on Windows and sign + notarize on macOS from
GitHub Actions. See [`docs/SIGNING.md`](docs/SIGNING.md) for the required
secrets:

- Windows: `WINDOWS_CERT_PFX_BASE64`, `WINDOWS_CERT_PASSWORD` (signtool).
- macOS: `MACOS_CERT_P12_BASE64`, `MACOS_CERT_PASSWORD`, `MACOS_SIGN_IDENTITY`,
  `MACOS_TEAM_ID`, `MACOS_NOTARY_APPLE_ID`, `MACOS_NOTARY_PASSWORD`,
  `KEYCHAIN_PASSWORD` (codesign + notarytool + stapler).

If a secret is missing the artifact is still built, just unsigned.

The macOS bundle declares `NSCameraUsageDescription` and the
`com.apple.security.device.camera` entitlement, which are **required** for the
hardened runtime to access the camera.

## Configuration

Settings are stored as JSON in the platform configuration directory:

| OS | Path |
|---|---|
| Linux | `~/.config/microscope-rust/settings.json` |
| Windows | `%APPDATA%\MakersKlabs\microscope-rust\config\settings.json` |
| macOS | `~/Library/Application Support/org.MakersKlabs.microscope-rust/settings.json` |

## Dependencies (all permissive)

| Crate | Purpose | License |
|---|---|---|
| `eframe` / `egui` | window and immediate-mode UI | MIT / Apache-2.0 |
| `nokhwa` | cross-platform camera (V4L2 / MediaFoundation / AVFoundation) | MIT |
| `image` | JPEG/PNG decode & encode | MIT / Apache-2.0 |
| `openh264` | H.264 encoder (Cisco) | BSD-2-Clause |
| `mp4` | pure-Rust MP4 muxer | MIT |
| `rfd` | native file dialogs | MIT |
| `serde` / `serde_json` | settings | MIT / Apache-2.0 |
| `directories` | platform folders | MIT / Apache-2.0 |
| `chrono` | timestamps | MIT / Apache-2.0 |

**No GPL code**: the original Python app linked x264 (GPL); this version uses
OpenH264, so the whole project can remain MIT.

## Release profile

`Cargo.toml` is tuned for the smallest binary:

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

## Project layout

```
src/main.rs        entry point + window icon
src/app.rs         the eframe application and UI
src/theme.rs       1980s CRT theme + scanlines
src/i18n.rs        English / Italian
src/camera.rs      enumeration + capture thread
src/frame.rs       frame + rotate/mirror
src/recorder.rs    OpenH264 + MP4 recorder (+ unit test)
src/settings.rs    per-camera settings
packaging/linux    .desktop, icon, AppImage, install scripts
packaging/windows  Inno Setup installer + sign.ps1
packaging/macos    Info.plist, entitlements, bundle + notarize
.github/workflows  CI and release (3 OS + signing)
docs/SIGNING.md    signing and notarization guide
```

## License

MIT — see [LICENSE](LICENSE).
