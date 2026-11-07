# Native platform configuration

- `src-tauri/tauri.conf.json`: shared app identity, version, frontend, window, security and icons.
- `src-tauri/tauri.windows.conf.json`: NSIS target and Windows installer settings.
- `src-tauri/tauri.linux.conf.json`: deb and AppImage targets.

Tauri automatically merges the matching platform file into the base configuration. Objects merge by key; arrays are replaced, not extended. Keep the complete shared `app.windows` array in the base unless a platform genuinely needs a complete replacement.

Run `npm run build` on the corresponding platform. Linux requires the native dependencies used in CI. Asset generation remains unchanged; use the existing workflows to generate icons and Windows installer images. Existing CI, release and prerelease commands remain compatible; their explicit Linux `--bundles deb,appimage` selection matches the Linux configuration.

Run `npm test` for configuration regression tests. These check configuration structure and modeled JSON Merge Patch results; actual Tauri parsing and packaging must also pass the Linux and Windows CI builds. No successful build or manual native test is implied by these files alone.

This change does not implement Linux terminal actions, alter the frontend platform label, or change translations.

Tauri documentation: https://v2.tauri.app/develop/configuration-files/
