; Hooks for the Windows installer (tauri.bundle.conf.json, bundle.windows.nsis.installerHooks).
;
; QOR Media, ARQADE and the Qontrol helper need the Microsoft Visual C++ runtime (VCRUNTIME140.dll, MSVCP140.dll), which
; a fresh Windows does not always have; the launcher itself links its runtime statically. windeployqt ships Microsoft's
; own runtime installer beside QOR Media and beside ARQADE, so one of them is run here, quietly: QOR Media's, or
; ARQADE's when QOR Media's is not there. It installs nothing when the same or a newer runtime is already there, and
; the installer carries on whatever it returns: the launcher runs without it.

!macro NSIS_HOOK_POSTINSTALL
  IfFileExists "$INSTDIR\qor-media\vc_redist.x64.exe" 0 qor_runtime_from_arqade
    DetailPrint "Installing the Microsoft Visual C++ runtime QOR Media and ARQADE need"
    ExecWait '"$INSTDIR\qor-media\vc_redist.x64.exe" /install /quiet /norestart'
    Goto qor_runtime_done
  qor_runtime_from_arqade:
  IfFileExists "$INSTDIR\arqade\vc_redist.x64.exe" 0 qor_runtime_done
    DetailPrint "Installing the Microsoft Visual C++ runtime ARQADE needs"
    ExecWait '"$INSTDIR\arqade\vc_redist.x64.exe" /install /quiet /norestart'
  qor_runtime_done:
!macroend
