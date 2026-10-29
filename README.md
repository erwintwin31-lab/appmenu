# Appmenu Lite (Windows) - v2 mit Glass-Design

Dieses Repository enthaelt den Quellcode. Die Setup-.exe entsteht im Build.

## Bauen
- GitHub: Reiter Actions > "Setup bauen" > Run workflow (startet auch automatisch bei jedem Push auf main). Danach unter Artifacts `Appmenu` herunterladen.
- Lokal (Windows mit Adminrechten): Node.js, Rust, C++ Build Tools installieren, Icons erzeugen (`pwsh scripts/make-assets.ps1` und `npx tauri icon src-tauri/app-icon.png -o src-tauri/icons`), dann `build-lokal.bat`.

Ergebnis: Setup-.exe (pro Benutzer, ohne Adminrechte) und `Appmenu-portable.exe`.

Nicht auf Windows getestet. Richtlinien (AppLocker/WDAC) kann Appmenu nicht umgehen.
