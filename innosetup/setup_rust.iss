#define MyAppName "SteelMouse"

[Setup]
AppName={#MyAppName}
AppVersion=2.4.0
DefaultDirName={userpf}\{#MyAppName}
DefaultGroupName={#MyAppName}
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
UninstallDisplayIcon={app}\steelmouse.exe
OutputDir=.
OutputBaseFilename=SteelMouse_Rust_Setup
Compression=lzma2/ultra64
SolidCompression=no
SetupIconFile=..\steelmouse_python\images\logo.ico

[Files]
Source: "..\target\x86_64-pc-windows-msvc\release\steelmouse.exe"; DestDir: "{app}"; Flags: skipifsourcedoesntexist ignoreversion
Source: "..\target\release\steelmouse.exe"; DestDir: "{app}"; Flags: skipifsourcedoesntexist ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\steelmouse.exe"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{userstartup}\{#MyAppName}"; Filename: "{app}\steelmouse.exe"; Tasks: autostart

[Run]
Filename: "{app}\steelmouse.exe"; Description: "Launch the application"; Flags: nowait postinstall skipifsilent

[Tasks]
Name: "autostart"; Description: "Start the application when Windows starts"; GroupDescription: "Additional tasks"; Flags: checkedonce
