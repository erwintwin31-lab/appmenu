# Appmenu Lite (Windows) – v2 mit Glass-Design
Dieses ZIP enthält den Quellcode. Die Setup-.exe entsteht erst beim Build (siehe unten).

## Bauen
- GitHub: Ordner in ein Repository hochladen > Actions > "Setup bauen" > Run workflow > Artifact `Appmenu` laden.
- Lokal (Windows mit Adminrechten): Node.js, Rust, C++ Build Tools installieren, dann `build-lokal.bat`.
Ergebnis: Setup-.exe (pro Benutzer, ohne Adminrechte) und `Appmenu-portable.exe`.

## Änderungen in v2
- Glass-Layout wie in der Vorschau: abgerundete Fensterkarte auf dunklem Hintergrund, auch in der installierten App.
- Standard-Akzent "Glass-Blau" (wie Vorschau), Windows-Blau bleibt wählbar.
- Schrift: Inter, falls installiert, sonst Segoe UI Variable.
- Fensterhintergrund vorab dunkel (kein weißes Aufblitzen), Fenster etwas größer.

Nicht kompiliert und nicht auf Windows getestet. Richtlinien (AppLocker/WDAC) kann Appmenu nicht umgehen.
