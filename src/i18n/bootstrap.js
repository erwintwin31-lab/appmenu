/* Load after catalogs.js and core.js, before frontend renderers. */
(function (root) {
  'use strict';
  const headerLabels = {
    en: ['Toggle light/dark mode', 'Minimize', 'Maximize'],
    de: ['Hell/Dunkel umschalten', 'Minimieren', 'Maximieren'],
    fr: ['Basculer entre les modes clair et sombre', 'Réduire', 'Agrandir']
  };
  function initializeLocale(target, catalogs, createI18n) {
    let storage;
    try { storage = target.localStorage; } catch (_) {}
    const i18n = createI18n(catalogs, storage);
    const updateLanguage = locale => {
      const doc = target.document;
      if (!doc) return;
      if (doc.documentElement) doc.documentElement.lang = locale;
      if (typeof doc.getElementById !== 'function') return;
      const [theme, minimize, maximize] = Object.hasOwn(headerLabels, locale) ? headerLabels[locale] : headerLabels.en;
      const labels = { th: theme, st: i18n.t('nav.settings'), wmin: minimize, wmax: maximize, wcls: i18n.t('action.close') };
      for (const [id, text] of Object.entries(labels)) {
        const button = doc.getElementById(id);
        if (!button) continue;
        button.setAttribute('title', text);
        button.setAttribute('aria-label', text);
      }
    };
    updateLanguage(i18n.getLocale());
    i18n.subscribe(updateLanguage);
    target.AppmenuI18n = i18n;
    return i18n;
  }
  if (typeof module === 'object' && module.exports) module.exports = {initializeLocale};
  else initializeLocale(root, root.AppmenuCatalogs, root.createAppmenuI18n);
})(globalThis);
