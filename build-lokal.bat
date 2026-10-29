@echo off
rem Nur auf einem Rechner MIT Internet und Installationsrechten (Node.js + Rust + C++ Build Tools vorher installieren).
call npm install || goto :fail
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
