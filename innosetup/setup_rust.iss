#define MyAppName "SteelMouse"

[Setup]
AppName={#MyAppName}
AppVersion=2.0.5
DefaultDirName={commonpf}\{#MyAppName}
DefaultGroupName={#MyAppName}
UninstallDisplayIcon={app}\steelmouse.exe
OutputDir=.
OutputBaseFilename=SteelMouse_Rust_Setup
Compression=lzma
SolidCompression=yes
SetupIconFile=..\images\logo.ico

[Files]
Source: "..\steelmouse_rust\target\x86_64-pc-windows-msvc\release\steelmouse.exe"; DestDir: "{app}"; Flags: skipifsourcedoesntexist ignoreversion
Source: "..\steelmouse_rust\target\release\steelmouse.exe"; DestDir: "{app}"; Flags: skipifsourcedoesntexist ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\steelmouse.exe"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{commonstartup}\{#MyAppName}"; Filename: "{app}\steelmouse.exe"; Tasks: autostart

[Run]
Filename: "{app}\steelmouse.exe"; Description: "Launch the application"; Flags: nowait postinstall skipifsilent

[Tasks]
Name: "autostart"; Description: "Start the application when Windows starts"; GroupDescription: "Additional tasks"; Flags: checkedonce
