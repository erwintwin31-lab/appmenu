# Appmenu

A Swiss-army-knife app menu to find, check and install software safely - built for PCs where you have no admin rights.

![Appmenu themes](docs/screenshots/themes.png)

> The screenshots show the demo mode with simulated data. Nothing is installed in the demo.

## Features

- **Search** programs through winget and install them for your own user account only (`--scope user`, no admin rights needed).
- **Review before install:** package details come straight from the winget manifest, and you confirm every program yourself.
- **File check** for local EXE and MSI files: double file extensions (for example `invoice.pdf.exe`), internet-origin mark, SHA-256 checksum.
- **Rights and policies:** Appmenu shows what your session may do and never tries to bypass device policies.
- **Themes:** light and dark mode plus six accent colors (Glass Blue, Windows Blue, Teal, Violet, Green, Orange).
- **Glass design:** rounded window card, sidebar with categories, pinned favorites and grouped tools.

## Screenshots

| Home (dark) | Home (light) |
|---|---|
| ![Home dark](docs/screenshots/01-home-dark.png) | ![Home light](docs/screenshots/02-home-light.png) |

| Search | Review |
|---|---|
| ![Search](docs/screenshots/03-search.png) | ![Review](docs/screenshots/04-review.png) |

| File check | Rights and policies |
|---|---|
| ![File check](docs/screenshots/05-file-check.png) | ![Rights](docs/screenshots/06-rights.png) |

| Settings |
|---|
| ![Settings](docs/screenshots/07-settings.png) |

## Platforms

| Platform | Status |
|---|---|
| Windows 10 / 11 | Available in this repository (Tauri 2, Rust, WebView2). Per-user installer, no admin rights needed. |
| Linux | Planned. A design preview and a local backend prototype exist, but they are not part of this repository yet. |

## Build

- **GitHub Actions:** open the Actions tab, run the workflow "Setup bauen" (it also runs on every push to `main`) and download the artifact `Appmenu`. It contains the setup `.exe` and `Appmenu-portable.exe`.
- **Local (Windows, with install rights):** install Node.js, Rust and the C++ Build Tools, then run `build-lokal.bat`.

## Limits

- Early version, not tested on many real machines yet.
- Group policies such as AppLocker or WDAC can block unsigned programs. Appmenu does not bypass them.
- Search and install need winget (App Installer) on the device.
