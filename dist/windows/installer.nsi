; Dictymus — per-machine Windows installer, fat x64+ARM64 package.
;
; Driven by `cargo xtask dist`, which passes:
;   -DVERSION=<display>  -DVERSIONNUM=<x.y.z.0>  -DSTAGING=<dir>
;   -DOUTFILE=<setup.exe>  -DICONFILE=<dictymus.ico>
;   [-DSINGLEARCH=<x86_64|aarch64>]   ; local single-arch test builds
; STAGING contains windows\<arch>\dictymus.exe and MicrosoftEdgeWebView2Setup.exe.

Unicode true
SetCompressor /SOLID LZMA
ManifestDPIAware true

!include "MUI2.nsh"
!include "x64.nsh"
!include "LogicLib.nsh"
!include "FileFunc.nsh"

!ifndef VERSION
  !define VERSION "0.0.0"
!endif
!ifndef VERSIONNUM
  !define VERSIONNUM "0.0.0.0"
!endif
!ifndef STAGING
  !error "STAGING is required (cargo xtask dist passes -DSTAGING=<dir>)"
!endif
!ifndef OUTFILE
  !define OUTFILE "Dictymus-Setup.exe"
!endif

!define EXE "dictymus.exe"
!define PROGID "Dictymus.Dictionary"
!define ARP "Software\Microsoft\Windows\CurrentVersion\Uninstall\Dictymus"
; WebView2 Evergreen detection keys. The WOW6432Node path is the documented
; per-machine location on 64-bit Windows, also when read from the 64-bit
; registry view.
!define WV2_GUID "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
!define WV2_HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\${WV2_GUID}"
!define WV2_HKCU "Software\Microsoft\EdgeUpdate\Clients\${WV2_GUID}"

Name "Dictymus ${VERSION}"
OutFile "${OUTFILE}"
RequestExecutionLevel admin
InstallDir "$PROGRAMFILES64\Dictymus"

VIProductVersion "${VERSIONNUM}"
VIAddVersionKey "ProductName" "Dictymus"
VIAddVersionKey "FileDescription" "Dictymus installer"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "Project Didymus"

!ifdef ICONFILE
  !define MUI_ICON "${ICONFILE}"
  !define MUI_UNICON "${ICONFILE}"
!endif

!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Function .onInit
  ; The installer stub is 32-bit; target the 64-bit registry view and
  ; all-users shell folders throughout.
  SetRegView 64
  SetShellVarContext all
  InitPluginsDir
FunctionEnd

Function un.onInit
  SetRegView 64
  SetShellVarContext all
FunctionEnd

; Sets $0 to 1 when the WebView2 Evergreen runtime is present.
Function DetectWebView2
  ReadRegStr $0 HKLM "${WV2_HKLM}" "pv"
  ${If} $0 == ""
    ReadRegStr $0 HKCU "${WV2_HKCU}" "pv"
  ${EndIf}
  ${If} $0 != ""
  ${AndIf} $0 != "0.0.0.0"
    StrCpy $0 1
  ${Else}
    StrCpy $0 0
  ${EndIf}
FunctionEnd

Section "Dictymus (required)" SecApp
  SectionIn RO
  SetOutPath "$INSTDIR"
  ; Both arch exes are packed; only the native one is extracted.
!ifdef SINGLEARCH
  File "/oname=${EXE}" "${STAGING}\windows\${SINGLEARCH}\${EXE}"
!else
  ${If} ${IsNativeARM64}
    File "/oname=${EXE}" "${STAGING}\windows\aarch64\${EXE}"
  ${Else}
    File "/oname=${EXE}" "${STAGING}\windows\x86_64\${EXE}"
  ${EndIf}
!endif

  ; WebView2 Evergreen runtime: install only when absent.
  Call DetectWebView2
  ${If} $0 = 0
    DetailPrint "Installing the Microsoft Edge WebView2 Runtime..."
    File "/oname=$PLUGINSDIR\MicrosoftEdgeWebView2Setup.exe" "${STAGING}\MicrosoftEdgeWebView2Setup.exe"
    ExecWait '"$PLUGINSDIR\MicrosoftEdgeWebView2Setup.exe" /silent /install' $0
    ${If} $0 <> 0
      DetailPrint "WebView2 bootstrapper exited with $0"
      MessageBox MB_OK|MB_ICONEXCLAMATION \
        "Dictymus was installed, but the Microsoft Edge WebView2 Runtime could not be installed (error $0).$\r$\nDictionary articles will not display until it is installed from:$\r$\nhttps://developer.microsoft.com/microsoft-edge/webview2/" /SD IDOK
    ${EndIf}
  ${Else}
    DetailPrint "WebView2 Runtime already present."
  ${EndIf}

  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "${ARP}" "DisplayName" "Dictymus"
  WriteRegStr HKLM "${ARP}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "${ARP}" "DisplayIcon" "$INSTDIR\${EXE},0"
  WriteRegStr HKLM "${ARP}" "Publisher" "Project Didymus"
  WriteRegStr HKLM "${ARP}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKLM "${ARP}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKLM "${ARP}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegDWORD HKLM "${ARP}" "NoModify" 1
  WriteRegDWORD HKLM "${ARP}" "NoRepair" 1
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  WriteRegDWORD HKLM "${ARP}" "EstimatedSize" $0
SectionEnd

Section "Start Menu shortcut" SecStartMenu
  CreateShortcut "$SMPROGRAMS\Dictymus.lnk" "$INSTDIR\${EXE}"
SectionEnd

Section /o "Desktop shortcut" SecDesktop
  CreateShortcut "$DESKTOP\Dictymus.lnk" "$INSTDIR\${EXE}"
SectionEnd

; Shared ProgID for both dictionary types.
!macro WriteProgId
  WriteRegStr HKLM "Software\Classes\${PROGID}" "" "Dictymus dictionary"
  WriteRegStr HKLM "Software\Classes\${PROGID}\DefaultIcon" "" '"$INSTDIR\${EXE}",0'
  WriteRegStr HKLM "Software\Classes\${PROGID}\shell\open\command" "" '"$INSTDIR\${EXE}" "%1"'
!macroend

; Register ${ext}: default handler plus an "Open with" entry.
!macro Associate ext
  !insertmacro WriteProgId
  WriteRegStr HKLM "Software\Classes\${ext}" "" "${PROGID}"
  WriteRegStr HKLM "Software\Classes\${ext}\OpenWithProgids" "${PROGID}" ""
!macroend

SectionGroup "File associations" SecGrpAssoc
  Section "StarDict (.ifo)" SecAssocIfo
    !insertmacro Associate ".ifo"
  SectionEnd
  Section "MDict (.mdx)" SecAssocMdx
    !insertmacro Associate ".mdx"
  SectionEnd
SectionGroupEnd

Function .onInstSuccess
  ; SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, 0, 0)
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
FunctionEnd

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecApp} "The Dictymus application (required)."
  !insertmacro MUI_DESCRIPTION_TEXT ${SecStartMenu} "Add a Dictymus shortcut to the Start Menu for all users."
  !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} "Add a Dictymus shortcut to the desktop."
  !insertmacro MUI_DESCRIPTION_TEXT ${SecGrpAssoc} "Make Dictymus the default application for dictionary files."
  !insertmacro MUI_DESCRIPTION_TEXT ${SecAssocIfo} "Open StarDict dictionaries (.ifo) with Dictymus by default."
  !insertmacro MUI_DESCRIPTION_TEXT ${SecAssocMdx} "Open MDict dictionaries (.mdx) with Dictymus by default."
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; Undo an association only where it still points at this install.
!macro UnAssociate ext
  ReadRegStr $0 HKLM "Software\Classes\${ext}" ""
  ${If} $0 == "${PROGID}"
    DeleteRegValue HKLM "Software\Classes\${ext}" ""
  ${EndIf}
  DeleteRegValue HKLM "Software\Classes\${ext}\OpenWithProgids" "${PROGID}"
  DeleteRegKey /ifempty HKLM "Software\Classes\${ext}\OpenWithProgids"
  DeleteRegKey /ifempty HKLM "Software\Classes\${ext}"
!macroend

Section "Uninstall"
  Delete "$INSTDIR\${EXE}"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  Delete "$SMPROGRAMS\Dictymus.lnk"
  Delete "$DESKTOP\Dictymus.lnk"

  !insertmacro UnAssociate ".ifo"
  !insertmacro UnAssociate ".mdx"
  DeleteRegKey HKLM "Software\Classes\${PROGID}"
  DeleteRegKey HKLM "${ARP}"
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
  ; User config in %APPDATA%\dictymus stays; the font cache in %TEMP%\dictymus
  ; is per-user and the app re-extracts it.
SectionEnd
