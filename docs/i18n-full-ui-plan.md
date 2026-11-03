# i18n: full app interface

Issue: #64

## Goal

Wire the existing language system into the complete Appmenu interface. Today, only the Language settings area is translated; most interface strings in `src/app.js` and `src/themes.js` remain German.

## Implementation scope

- [ ] Navigation, categories and feature names/descriptions
- [ ] Home, All features, Favorites and category views
- [ ] Search, installation plan, review and job output labels
- [ ] Local files, file check, rights and system status
- [ ] Settings, keyboard shortcuts and toasts
- [ ] Design labels in `src/themes.js`
- [ ] Complete English, German and French catalog parity
- [ ] Preserve user input and raw backend/winget output

## Acceptance

Switching the interface to English must leave no visible German interface text. The web demo and desktop app must behave consistently.
