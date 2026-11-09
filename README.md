# Appmenu

A modular Swiss-army-knife desktop app for finding software, reviewing downloads and making everyday tasks easier. Built with Tauri 2, Rust and a web-based interface.

[Try the live demo](https://erwintwin31-lab.github.io/appmenu/) · [Downloads](https://github.com/erwintwin31-lab/appmenu/releases) · [Contributing](CONTRIBUTING.md)

> Early development: builds are experimental. The browser demo uses simulated data and does not install software.

## Features

- Find Windows apps through winget, select multiple programs and review package details before installation.
- Install for your own account using `--scope user`, when supported by the package. No attempt to bypass administrator requirements or device policies.
- Inspect local EXE/MSI files: SHA-256 checksums, suspicious double extensions and internet-origin marks. These checks are not a malware verdict.
- View session permissions and available actions. Personalize the interface with themes, accent colors, light/dark mode and layout options.
- Explore AI Overview and follow-up chat previews. The existing overview/chat uses simulated responses; the separate local AI preview runs an experimental model in the browser after consent.

## Preview

Explore the interface in the [interactive demo](https://erwintwin31-lab.github.io/appmenu/). Demo content is simulated, not evidence of working native integrations.

The [local AI preview](https://erwintwin31-lab.github.io/appmenu/local-ai.html) requires a compatible WebGPU browser and downloads runtime/model files after consent. It has no cloud-inference fallback, does not retrieve live sources and may produce incorrect answers. See [limitations and testing notes](docs/local-ai-demo.md).

## Install

### Windows

1. Open [Releases](https://github.com/erwintwin31-lab/appmenu/releases) and select a Windows setup `.exe` from the desired release or prerelease.
2. Run the installer normally and follow the wizard. It is configured to install for the current user.
3. Launch Appmenu from the Start menu. Software search and installation require winget (App Installer).

Not every package supports per-user installation. Device policies can also block Appmenu or an installer; Appmenu does not bypass these restrictions.

### Linux

Experimental AppImage and `.deb` builds are provided when published in Releases. The Windows winget installation backend is not available on Linux. macOS is not currently a supported build target.

### Build from source

Install Node.js/npm, Rust and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform, then run:

```sh
git clone https://github.com/erwintwin31-lab/appmenu.git
cd appmenu
npm ci
npm run check
npm test
npm run dev
```

Use `npm run build` to package the app. Existing GitHub Actions workflows also handle builds and generated assets; native packaging still needs to be tested on each target platform.

## Roadmap

Planned work, not a list of completed features:

- Integrate a real AI Overview and follow-up chat with clearer sources, privacy controls and validation.
- Expand Linux software sources and installation backends.
- Improve localization, accessibility and real-device testing.
- Add more focused quality-of-life modules while keeping the interface lightweight.

Found a bug or have an idea? Open an [issue](https://github.com/erwintwin31-lab/appmenu/issues). For security concerns, see [SECURITY.md](SECURITY.md).
