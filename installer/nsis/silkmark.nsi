; SilkMark NSIS Installer Script
; Build with: makensis -DAPP_VERSION=<ver> -DSTAGE_DIR=<win-path> -DOUT_FILE=<win-path> silkmark.nsi
;
; The staging directory (STAGE_DIR) must follow the standard GTK-on-Windows
; layout so the bundled libraries resolve their data directories relative to
; the binary location:
;
;   <STAGE_DIR>\
;       bin\silkmark.exe        (+ the whole DLL closure: gtk-4, glib, curl, ...)
;       lib\gdk-pixbuf-2.0\<ver>\loaders\*.dll + loaders.cache
;       share\glib-2.0\schemas\gschemas.compiled
;       share\icons\Adwaita\... , share\icons\hicolor\...
;
; gdk-pixbuf derives libdir from the libgdk_pixbuf DLL as <bindir>/../lib, and
; GTK derives datadir as <bindir>/../share, so the bin/lib/share split above
; keeps everything relocatable once installed to $PROGRAMFILES64\SilkMark.

Unicode True

;--------------------------------
; General

!define APP_NAME        "SilkMark"
!ifndef APP_VERSION
!define APP_VERSION     "0.0.0"
!endif
!define APP_PUBLISHER   "SilkMark"
!define APP_EXE         "silkmark.exe"
!define APP_BIN_SUBDIR  "bin"
!define INSTALL_DIR     "$PROGRAMFILES64\${APP_NAME}"
!define REG_UNINSTALL   "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
!define REG_APP         "Software\${APP_PUBLISHER}\${APP_NAME}"
!ifndef STAGE_DIR
!error "STAGE_DIR must be defined (a Windows path to the staging directory)"
!endif
!ifndef OUT_FILE
!define OUT_FILE        "silkmark-${APP_VERSION}-win-x64-setup.exe"
!endif

;--------------------------------
; Includes

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "FileFunc.nsh"

;--------------------------------
; Installer metadata

Name          "${APP_NAME} ${APP_VERSION}"
OutFile       "${OUT_FILE}"
InstallDir    "${INSTALL_DIR}"
InstallDirRegKey HKLM "${REG_APP}" "InstallDir"
RequestExecutionLevel admin
BrandingText  "${APP_NAME} ${APP_VERSION}"

;--------------------------------
; MUI Settings

!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN          "$INSTDIR\${APP_BIN_SUBDIR}\${APP_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT     "Launch ${APP_NAME}"

;--------------------------------
; Pages

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

;--------------------------------
; Languages

!insertmacro MUI_LANGUAGE "English"

;--------------------------------
; Installer sections

Section "Main Application" SecMain
    SectionIn RO  ; required section

    SetOutPath "$INSTDIR"
    File /r "${STAGE_DIR}\*.*"

    ; Write registry keys
    WriteRegStr HKLM "${REG_APP}" "InstallDir" "$INSTDIR"
    WriteRegStr HKLM "${REG_APP}" "Version"    "${APP_VERSION}"

    ; Write uninstaller registry entries (Add/Remove Programs)
    WriteRegStr   HKLM "${REG_UNINSTALL}" "DisplayName"     "${APP_NAME}"
    WriteRegStr   HKLM "${REG_UNINSTALL}" "DisplayVersion"  "${APP_VERSION}"
    WriteRegStr   HKLM "${REG_UNINSTALL}" "Publisher"       "${APP_PUBLISHER}"
    WriteRegStr   HKLM "${REG_UNINSTALL}" "InstallLocation" "$INSTDIR"
    WriteRegStr   HKLM "${REG_UNINSTALL}" "DisplayIcon"     "$INSTDIR\${APP_BIN_SUBDIR}\${APP_EXE}"
    WriteRegStr   HKLM "${REG_UNINSTALL}" "UninstallString" '"$INSTDIR\uninstall.exe"'
    WriteRegStr   HKLM "${REG_UNINSTALL}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
    WriteRegDWORD HKLM "${REG_UNINSTALL}" "NoModify"        1
    WriteRegDWORD HKLM "${REG_UNINSTALL}" "NoRepair"        1
    WriteRegStr   HKLM "${REG_UNINSTALL}" "URLInfoAbout"    "https://github.com/hg2ecz/silkmark"

    ; Estimate install size (in KB)
    ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
    IntFmt $0 "0x%08X" $0
    WriteRegDWORD HKLM "${REG_UNINSTALL}" "EstimatedSize" "$0"

    ; Write uninstaller executable
    WriteUninstaller "$INSTDIR\uninstall.exe"

    ; Start Menu shortcut
    CreateDirectory "$SMPROGRAMS\${APP_NAME}"
    CreateShortcut  "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" \
                    "$INSTDIR\${APP_BIN_SUBDIR}\${APP_EXE}" "" \
                    "$INSTDIR\${APP_BIN_SUBDIR}\${APP_EXE}" 0

    CreateShortcut  "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk" \
                    "$INSTDIR\uninstall.exe"

SectionEnd

;--------------------------------
; Optional: Desktop shortcut

Section "Desktop Shortcut" SecDesktop
    CreateShortcut "$DESKTOP\${APP_NAME}.lnk" \
                   "$INSTDIR\${APP_BIN_SUBDIR}\${APP_EXE}" "" \
                   "$INSTDIR\${APP_BIN_SUBDIR}\${APP_EXE}" 0
SectionEnd

;--------------------------------
; File association for .md / .markdown

Section -FileAssoc
    WriteRegStr HKCR ".md"       "" "SilkMark.md"
    WriteRegStr HKCR ".markdown" "" "SilkMark.md"
    WriteRegStr HKCR "SilkMark.md"                     "" "SilkMark Markdown"
    WriteRegStr HKCR "SilkMark.md\DefaultIcon"         "" "$INSTDIR\${APP_BIN_SUBDIR}\${APP_EXE},0"
    WriteRegStr HKCR "SilkMark.md\shell\open\command"   "" '"$INSTDIR\${APP_BIN_SUBDIR}\${APP_EXE}" "%1"'
SectionEnd

;--------------------------------
; Section descriptions

LangString DESC_SecMain    ${LANG_ENGLISH} "Installs ${APP_NAME} and all required GTK libraries."
LangString DESC_SecDesktop ${LANG_ENGLISH} "Creates a shortcut on the Desktop."

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${SecMain}    $(DESC_SecMain)
    !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} $(DESC_SecDesktop)
!insertmacro MUI_FUNCTION_DESCRIPTION_END

;--------------------------------
; Uninstaller

Section "Uninstall"

    ; Kill running instance before removing files
    ExecWait 'taskkill /F /IM "${APP_EXE}"' $0

    ; Remove installed files
    RMDir /r "$INSTDIR"

    ; Remove shortcuts
    RMDir /r "$SMPROGRAMS\${APP_NAME}"
    Delete   "$DESKTOP\${APP_NAME}.lnk"

    ; Remove registry keys
    DeleteRegKey HKCR "SilkMark.md"
    DeleteRegKey HKCR ".md"
    DeleteRegKey HKCR ".markdown"
    DeleteRegKey HKLM "${REG_UNINSTALL}"
    DeleteRegKey HKLM "${REG_APP}"
    DeleteRegKey /ifempty HKLM "Software\${APP_PUBLISHER}"

SectionEnd
