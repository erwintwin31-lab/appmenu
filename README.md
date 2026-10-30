# Appmenu

A Swiss-army-knife app menu to find, check and install software safely - built for PCs where you have no admin rights.

**[Try the live demo](https://erwintwin31-lab.github.io/appmenu/)** - runs in your browser with simulated data.

![Appmenu themes](docs/screenshots/themes.png)

> The screenshot shows the demo mode with simulated data. Nothing is installed in the demo.

## Features

- **Search** programs through winget and install them for your own user account only (`--scope user`, no admin rights needed).
- **Review before install:** package details come from the winget manifest, and you confirm every program yourself.
- **File check** for local EXE and MSI files: double file extensions, internet-origin mark, SHA-256 checksum.
- **Rights and policies:** shows what your session may do and never tries to bypass device policies.
- **Themes and layouts:** light and dark mode, accent colors per platform, layouts Glass and Klassisch.

## Platforms

| Platform | Status |
|---|---|
| Windows 10 / 11 | Available (Tauri 2, Rust, WebView2). Per-user installer, no admin rights needed. Themes: Glass, Blue, Teal, Violet, Green, Orange. |
| Linux | Early preview with AppImage and deb builds. Themes: CachyOS, Arch, Debian, Fedora, Ubuntu, openSUSE. The install functions (winget) are not available on Linux yet. |

## Download and build

- **Download:** automatic prereleases are on the Releases page. They are untested.
- **Build:** run the workflow "Setup bauen" in GitHub Actions, or run `build-lokal.bat` locally on Windows.

## Limits

- Early version, not tested on many real machines yet.
- Group policies such as AppLocker or WDAC can block unsigned programs. Appmenu does not bypass them.
- Search and install need winget (App Installer) on the device.
