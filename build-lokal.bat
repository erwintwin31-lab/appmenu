@echo off
rem Nur auf einem Rechner MIT Internet und Installationsrechten (Node.js + Rust + C++ Build Tools vorher installieren).
call npm install || goto :fail
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\make-assets.ps1 || goto :fail
call npx tauri icon src-tauri\app-icon.png -o src-tauri\icons || goto :fail
call npx tauri build || goto :fail
echo.
echo Fertig. Setup: src-tauri\target\release\bundle\nsis\
echo Portable:  src-tauri\target\release\appmenu.exe
pause
exit /b 0
:fail
echo Build fehlgeschlagen.
pause
exit /b 1
