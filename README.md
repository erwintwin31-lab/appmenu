# Appmenu

Appmenu is a privacy-first desktop toolbox for finding, checking and installing software safely — especially on PCs where you do not have administrator rights.

It combines a clear app launcher, winget-based software search and installation, local file checks and an experimental local AI assistant in one adaptable interface.

**[Open the live demo](https://erwintwin31-lab.github.io/appmenu/)** — the demo uses simulated data and installs nothing.

![Appmenu themes](docs/screenshots/themes.png)

> The screenshot shows the browser demo with simulated data. It does not install software or download an AI model.

## Current status

Appmenu is an early, actively developed project. The browser demo is functional as a visual and interaction preview; the Windows desktop app is the primary target.

| Area | Current state |
|---|---|
| Browser demo | Available. Search, navigation, themes, layouts and AI preview use simulated data. |
| Windows desktop app | Early version available through Tauri 2, Rust and WebView2. |
| winget search and user installation | Implemented in the desktop app; requires winget on the device. |
| Local file checks | Implemented for EXE and MSI files. |
| Local AI backend | Experimental backend implemented. It is not yet fully connected to the complete desktop AI experience. |
| Linux desktop app | Early preview. Installation features that depend on winget are not available yet. |
| Interface languages | Planned. English is intended as the default and fallback language; German and French are planned next. |

Downloads and automatically created prereleases are considered **untested**. Review release notes before installing them.

## Features

### Search and install

- Search Windows software through the winget package repository.
- Review package details before installation.
- Install selected software for the current user account with `winget install --scope user`.
- Confirm every installation explicitly; Appmenu does not silently install software.
- Limit the number of programs in one installation plan.

### Local file checks

- Find EXE and MSI files in common locations such as Downloads and Desktop.
- Select any file through the system file dialog.
- Detect potentially misleading double file extensions, for example `document.pdf.exe`.
- Show whether Windows marked a file as downloaded from the internet.
- Calculate a SHA-256 checksum.
- Explain whether the current session may start the selected file.

### Rights and policies

- Show the current user, device and elevation state.
- Explain whether search, per-user installation and file execution are available.
- Avoid starting programs that require administrator rights.
- Do not bypass AppLocker, WDAC or other device policies.

### Interface and personalization

- Choose between light, dark and system appearance.
- Select one of four interface themes: Normal, Black, Midnight and Frosted Blue.
- Choose preset or custom accent colors.
- Switch between Hub, Normal and Compact layouts.
- Use a sidebar, bottom tabs or a dock-style navigation mode.
- Pin favorite functions and configure dock entries.
- Export and import layout settings as JSON.

## Themes and layouts

Appmenu separates visual appearance from layout. You can combine a theme with a layout, accent color, density and navigation mode.

| Theme | Character |
|---|---|
| Normal | Clear, calm surface with selectable accent color. |
| Black | High-contrast black or white appearance. |
| Midnight | Deep blue surfaces with softer contrasts. |
| Frosted Blue | Tinted glass look with a cool blue accent. |

| Layout | Purpose |
|---|---|
| Hub | Function groups and a compact overview. |
| Normal | Larger cards and more visible structure. |
| Compact | Space-saving list-oriented layout. |

![Search and AI demo](docs/screenshots/search-ai.png)

> Search and the AI demo are separate concepts: the simulated preview needs no model, while the optional real local AI requires explicit consent and a model download.

## Local AI

Appmenu has two clearly separated AI experiences.

### Demo preview

The browser demo can show prepared, simulated AI answers. This mode:

- does not run an AI model;
- does not download a model;
- does not require WebGPU;
- does not perform live web search;
- does not approve installations.

### Experimental local AI

The desktop backend includes experimental building blocks for local AI:

- Query a local Ollama instance for installed models.
- Recommend a conservative model based on available system memory.
- Validate literal quotations against sources fetched by Appmenu.
- Reject free-form summaries, unknown sources and model-supplied URLs.

This backend is **not yet a fully validated end-user AI feature**. Model quality, speed, memory use, cancellation, error handling and real inference still need testing.

The current implementation does not automatically install models, automatically select models or execute AI output. A model recommendation is provisional and must be confirmed by the user.

![Themes and layouts](docs/screenshots/themes-layouts.png)

> Themes, layouts, accent colors and navigation modes are intended to work independently.

## Roadmap

The roadmap is synchronized with the repository issues. The current major work is tracked in [#56](https://github.com/erwintwin31-lab/appmenu/issues/56).

| Area | Status | Issue |
|---|---|---|
| Language interface: English default, German and French support | Planned | [#56](https://github.com/erwintwin31-lab/appmenu/issues/56) |
| Connect real desktop AI to search and AI Overview | Planned | [#56](https://github.com/erwintwin31-lab/appmenu/issues/56) |
| Unified settings with Design, Layout, System and Language tabs | Planned | [#56](https://github.com/erwintwin31-lab/appmenu/issues/56) |
| Labelled dock navigation | Planned | [#56](https://github.com/erwintwin31-lab/appmenu/issues/56) |
| Separate start page and search page | Planned | [#56](https://github.com/erwintwin31-lab/appmenu/issues/56) |
| Nightly builds and cleanup of old prereleases | Planned | [#24](https://github.com/erwintwin31-lab/appmenu/issues/24) |

Planned features are not guaranteed to be available in current releases.

## Safety boundaries

Appmenu is designed to support careful decisions, not to bypass security boundaries.

- It does not install anything without explicit confirmation.
- It does not run scripts such as `.ps1`, `.bat`, `.cmd`, `.vbs` or `.js`.
- It does not bypass administrator requirements or device policies.
- It does not treat AI output as an installation approval.
- It does not claim that AI responses, package details or file checks replace your own review.

## Build and download

### Download

Automatic prereleases are available on the [Releases page](https://github.com/erwintwin31-lab/appmenu/releases). They are untested and intended for early testing.

### Build on Windows

Run:

```bat
build-lokal.bat
```

Alternatively, start the **Setup bauen** workflow from GitHub Actions.

### Development requirements

- Windows 10 or 11 for the primary desktop experience
- WebView2 runtime
- Rust and Node.js for development
- winget / App Installer for search and installation features
- Ollama only for the optional experimental local AI backend
