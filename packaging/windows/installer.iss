; Inno Setup script for the Windows installer.
; Build the release binary first: cargo build --release
; Then compile this script with Inno Setup 6 (ISCC.exe installer.iss).

#define MyAppName "Microscope"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "MakersKlabs"
#define MyAppURL "https://github.com/MakersKlabs/makers_tools"
#define MyAppExeName "microscope-rust.exe"

[Setup]
AppId={{7C2B5A1E-3E4D-4B7A-9C1F-MICROSCOPERUST}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputBaseFilename=Microscope-Setup-{#MyAppVersion}
OutputDir=..\..\dist
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
SetupIconFile=..\..\assets\icon.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Files]
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional icons:"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch {#MyAppName}"; Flags: nowait postinstall skipifsilent
