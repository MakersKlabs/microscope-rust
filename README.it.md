# microscope-rust

Applicazione minimale per l'acquisizione da microscopio USB, in stile
**terminale anni '80**, scritta in **Rust**, che si compila in un **unico
binario autoconsistente** per **Linux, Windows e macOS**.

> 🇬🇧 English version: [README.md](README.md)

```
+------------------------------------------------------------+
| MICROSCOPIO       CAMERA [H1600 Cam]  RIS [1920 x 1080]    |
| FPS [30]  RUOTA 90  SPECCHIA-H  SPECCHIA-V  ADATTA FOTO REC |
+------------------------------------------------------------+
|                                                            |
|                 immagine dal microscopio                   |
|                                                            |
+------------------------------------------------------------+
| H1600 Cam | 1920 x 1080 | 29.8 fps | REC 00:42            |
+------------------------------------------------------------+
```

## Perché questa versione

La `microscope-app` originale è Python + Qt + OpenCV: rapidissima da
sviluppare, ma dipende dall'interprete Python e da una pila di pacchetti di
sistema. Questa riscrittura in Rust ribalta il compromesso:

| | microscope-app (Python) | microscope-rust |
|---|---|---|
| Runtime necessario | Python 3 + Qt + OpenCV + PyAV | **nessuno** (binario unico) |
| Dimensione | sorgente / `.deb` ~18 KB + dipendenze | **~5.5 MB** |
| Licenza encoder video | x264 (**GPL**) | OpenH264 (**BSD**) |
| Dipendenze dai pacchetti | molte | autoconsistente |
| Multipiattaforma | con lavoro extra | Linux / Windows / macOS |

Non è un wrapper e non è nemmeno "tutto scritto a mano": usa piccoli crate con
licenze permissive (vedi *Dipendenze*), linkati staticamente.

## Funzioni

Le stesse della versione originale:

- **Selezione telecamera** tra le USB collegate.
- **Selezione risoluzione e frame rate** dalle capacità della telecamera.
- **Rotazione** 0/90/180/270 e **specchiatura** sui due assi, **con memoria per
  ogni telecamera**.
- **Navigazione**: zoom con rotellina, pan con trascinamento sinistro, fit con
  click destro.
- **Foto PNG/JPEG**, ricordando l'ultima cartella.
- **Registrazione video H.264 compressa** (OpenH264 + muxer MP4 in puro Rust),
  ricordando l'ultima cartella.
- **Interfaccia bilingue** (inglese / italiano).
- **Grafica da terminale anni '80**: verde fosforo su nero, monospaziato,
  bordi netti e scanline CRT. L'effetto scanline si puo' disattivare da
  **Impostazioni → Effetti CRT**.

## Tastiera e mouse

| Input | Azione |
|---|---|
| Rotellina | zoom |
| Trascinamento sinistro | pan |
| Click destro | adatta alla finestra |
| `Ctrl+S` | salva foto |
| `Ctrl+R` | avvia/ferma registrazione |
| `F` | adatta |
| `+` / `-` | zoom avanti / indietro |

## Compilazione

Serve una toolchain Rust stabile recente (<https://rustup.rs>). Non servono
pacchetti di sviluppo di sistema: X11, Wayland e OpenGL vengono caricati a
runtime con `dlopen`.

```bash
cargo build --release
# binario: target/release/microscope-rust(.exe)
```

Avvio:

```bash
cargo run --release
```

Diagnostica dell'elenco telecamere (utile quando una camera mostra `NO SIGNAL`):

```bash
./target/release/microscope-rust --list-cameras
```

## Packaging

### Linux
```bash
sh packaging/linux/install.sh       # installa in ~/.local
sh packaging/linux/make-appimage.sh # AppImage autoconsistente
```

### Windows
```powershell
cargo build --release
ISCC packaging\windows\installer.iss   # Inno Setup 6 -> dist\Microscope-Setup-*.exe
```

### macOS
```bash
sh packaging/macos/bundle.sh            # dist/Microscope.app + icon.icns
sh packaging/macos/sign_and_notarize.sh # se i secret di firma sono impostati
```

## Firma e notarizzazione

Il progetto è predisposto per firmare su Windows e firmare + notarizzare su
macOS da GitHub Actions. Vedi [`docs/SIGNING.md`](docs/SIGNING.md) per i secret
necessari:

- Windows: `WINDOWS_CERT_PFX_BASE64`, `WINDOWS_CERT_PASSWORD` (signtool).
- macOS: `MACOS_CERT_P12_BASE64`, `MACOS_CERT_PASSWORD`, `MACOS_SIGN_IDENTITY`,
  `MACOS_TEAM_ID`, `MACOS_NOTARY_APPLE_ID`, `MACOS_NOTARY_PASSWORD`,
  `KEYCHAIN_PASSWORD` (codesign + notarytool + stapler).

Se un secret manca, l'artefatto viene comunque costruito, semplicemente non
firmato.

Il bundle macOS dichiara `NSCameraUsageDescription` e l'entitlement
`com.apple.security.device.camera`, **necessari** perché l'hardened runtime
acceda alla telecamera.

## Configurazione

Le impostazioni sono salvate come JSON nella cartella di configurazione del
sistema:

| OS | Percorso |
|---|---|
| Linux | `~/.config/microscope-rust/settings.json` |
| Windows | `%APPDATA%\MakersKlabs\microscope-rust\config\settings.json` |
| macOS | `~/Library/Application Support/org.MakersKlabs.microscope-rust/settings.json` |

## Dipendenze (tutte permissive)

| Crate | Uso | Licenza |
|---|---|---|
| `eframe` / `egui` | finestra e UI immediate-mode | MIT / Apache-2.0 |
| `nokhwa` | telecamera multipiattaforma (V4L2 / MediaFoundation / AVFoundation) | MIT |
| `image` | decodifica/codifica JPEG e PNG | MIT / Apache-2.0 |
| `openh264` | encoder H.264 (Cisco) | BSD-2-Clause |
| `mp4` | muxer MP4 in puro Rust | MIT |
| `rfd` | finestre di dialogo native | MIT |
| `serde` / `serde_json` | impostazioni | MIT / Apache-2.0 |
| `directories` | cartelle di sistema | MIT / Apache-2.0 |
| `chrono` | timestamp | MIT / Apache-2.0 |

**Nessun codice GPL**: l'app Python originale linkava x264 (GPL); questa
versione usa OpenH264, quindi l'intero progetto può restare MIT.

## Profilo di release

`Cargo.toml` è ottimizzato per il binario più piccolo possibile:

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

## Struttura del progetto

```
src/main.rs        entry point + icona finestra
src/app.rs         applicazione eframe e interfaccia
src/theme.rs       tema CRT anni '80 + scanline
src/i18n.rs        inglese / italiano
src/camera.rs      enumerazione + thread di acquisizione
src/frame.rs       frame + rotazione/specchiatura
src/recorder.rs    registratore OpenH264 + MP4 (con unit test)
src/settings.rs    impostazioni per telecamera
packaging/linux    .desktop, icona, AppImage, script di installazione
packaging/windows  installer Inno Setup + sign.ps1
packaging/macos    Info.plist, entitlements, bundle + notarizzazione
.github/workflows  CI e release (3 OS + firma)
docs/SIGNING.md    guida a firma e notarizzazione
```

## Licenza

MIT — vedi [LICENSE](LICENSE).
