; Dictymus — per-machine Windows installer, one native package per architecture.
;
; Driven by `cargo xtask dist`, which passes:
;   /DVERSION=<display>  /DVERSIONNUM=<x.y.z.0>  /DSTAGING=<dir>
;   /DOUTDIR=<dir>  /DARCH=<x64|arm64>  /DICONFILE=<dictymus.ico>
; STAGING contains windows\<rust arch>\dictymus.exe and MicrosoftEdgeWebView2Setup.exe.

#ifndef VERSION
	#define VERSION "0.0.0"
#endif
#ifndef VERSIONNUM
	#define VERSIONNUM "0.0.0.0"
#endif
#ifndef STAGING
	#error STAGING is required (cargo xtask dist passes /DSTAGING=<dir>)
#endif
#ifndef OUTDIR
	#define OUTDIR "."
#endif
#ifndef ARCH
	#define ARCH "x64"
#endif
#if ARCH == "arm64"
	#define SRCDIR STAGING + "\windows\aarch64"
	#define ARCH_ALLOWED "arm64"
#else
	#define SRCDIR STAGING + "\windows\x86_64"
	#define ARCH_ALLOWED "x64compatible"
#endif

[Setup]
	AppName=Dictymus
	AppVersion={#VERSION}
	AppPublisher=Project Didymus
	AppPublisherURL=https://github.com/ProjectDidymus/dictymus
	AppSupportURL=https://github.com/ProjectDidymus/dictymus/issues
	AppUpdatesURL=https://github.com/ProjectDidymus/dictymus/releases
	DefaultDirName={autopf}\Dictymus
	DisableProgramGroupPage=yes
	DisableDirPage=no
	OutputDir={#OUTDIR}
	OutputBaseFilename=dictymus_setup-{#ARCH}
	Compression=lzma2
	SolidCompression=yes
	WizardStyle=modern
	UninstallDisplayIcon={app}\dictymus.exe
	ArchitecturesAllowed={#ARCH_ALLOWED}
	ArchitecturesInstallIn64BitMode={#ARCH_ALLOWED}
	VersionInfoVersion={#VERSIONNUM}
	ChangesAssociations=yes
#ifdef ICONFILE
	SetupIconFile={#ICONFILE}
#endif

[Languages]
	Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
	Source: "{#SRCDIR}\dictymus.exe"; DestDir: "{app}"; Flags: ignoreversion
	Source: "{#STAGING}\MicrosoftEdgeWebView2Setup.exe"; DestDir: "{tmp}"; Check: not WebView2Present

[Tasks]
	Name: "startmenuicon"; Description: "Start Menu Shortcut"
	Name: "desktopicon"; Description: "Desktop Shortcut"; Flags: unchecked
	Name: "assoc_ifo"; Description: "Open StarDict dictionaries (*.ifo) with Dictymus by default"
	Name: "assoc_mdx"; Description: "Open MDict dictionaries (*.mdx) with Dictymus by default"

[Icons]
	Name: "{autoprograms}\Dictymus"; Filename: "{app}\dictymus.exe"; Tasks: startmenuicon
	Name: "{autodesktop}\Dictymus"; Filename: "{app}\dictymus.exe"; Tasks: desktopicon

[Registry]
	; Shared ProgID for both dictionary types.
	Root: HKCR; Subkey: "Dictymus.Dictionary"; ValueType: string; ValueName: ""; ValueData: "Dictymus dictionary"; Flags: uninsdeletekey
	Root: HKCR; Subkey: "Dictymus.Dictionary\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\dictymus.exe,0"
	Root: HKCR; Subkey: "Dictymus.Dictionary\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\dictymus.exe"" ""%1"""
	Root: HKCR; Subkey: ".ifo"; ValueType: string; ValueName: ""; ValueData: "Dictymus.Dictionary"; Flags: uninsdeletevalue uninsdeletekeyifempty; Tasks: assoc_ifo
	Root: HKCR; Subkey: ".ifo\OpenWithProgids"; ValueType: string; ValueName: "Dictymus.Dictionary"; ValueData: ""; Flags: uninsdeletevalue uninsdeletekeyifempty; Tasks: assoc_ifo
	Root: HKCR; Subkey: ".mdx"; ValueType: string; ValueName: ""; ValueData: "Dictymus.Dictionary"; Flags: uninsdeletevalue uninsdeletekeyifempty; Tasks: assoc_mdx
	Root: HKCR; Subkey: ".mdx\OpenWithProgids"; ValueType: string; ValueName: "Dictymus.Dictionary"; ValueData: ""; Flags: uninsdeletevalue uninsdeletekeyifempty; Tasks: assoc_mdx

[Code]
const
	// WebView2 Evergreen detection keys. The WOW6432Node path is the documented
	// per-machine location on 64-bit Windows when read from the 64-bit view.
	WV2KeyMachine = 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}';
	WV2KeyUser = 'Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}';

function WebView2Present: Boolean;
var
	Version: string;
begin
	if not RegQueryStringValue(HKLM64, WV2KeyMachine, 'pv', Version) then
		if not RegQueryStringValue(HKCU, WV2KeyUser, 'pv', Version) then
			Version := '';
	Result := (Version <> '') and (Version <> '0.0.0.0');
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
	ResultCode: Integer;
begin
	if CurStep <> ssPostInstall then Exit;
	if WebView2Present then Exit;
	if not Exec(ExpandConstant('{tmp}\MicrosoftEdgeWebView2Setup.exe'), '/silent /install', '', SW_HIDE, ewWaitUntilTerminated, ResultCode) then
		ResultCode := -1;
	if ResultCode <> 0 then
		SuppressibleMsgBox(
			'Dictymus was installed, but the Microsoft Edge WebView2 Runtime could not be installed (error '
			+ IntToStr(ResultCode) + ').' #13#10
			+ 'Dictionary articles will not display until it is installed from:' #13#10
			+ 'https://developer.microsoft.com/microsoft-edge/webview2/',
			mbError, MB_OK, IDOK);
end;

[Run]
	Filename: "{app}\dictymus.exe"; Description: "Launch Dictymus"; Flags: nowait postinstall skipifsilent
