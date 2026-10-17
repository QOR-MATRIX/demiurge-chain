; Hooks for the Windows installer (tauri.bundle.conf.json, bundle.windows.nsis.installerHooks).
;
; QOR Media and the Qontrol helper need the Microsoft Visual C++ runtime (VCRUNTIME140.dll, MSVCP140.dll), which a fresh
; Windows does not always have; the launcher itself links its runtime statically. windeployqt ships Microsoft's own
; runtime installer beside QOR Media, so it is run here, quietly. It installs nothing when the same or a newer runtime is
; already there, and the installer carries on whatever it returns: the launcher runs without it.

!macro NSIS_HOOK_POSTINSTALL
  IfFileExists "$INSTDIR\qor-media\vc_redist.x64.exe" 0 +3
    DetailPrint "Installing the Microsoft Visual C++ runtime QOR Media needs"
    ExecWait '"$INSTDIR\qor-media\vc_redist.x64.exe" /install /quiet /norestart'
!macroend
