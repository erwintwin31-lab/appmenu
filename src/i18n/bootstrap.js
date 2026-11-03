/* Load after catalogs.js and core.js, before frontend renderers. */
(function (root) {
  'use strict';
  function initializeLocale(target, catalogs, createI18n) {
    let storage;
    try { storage = target.localStorage; } catch (_) {}
    const i18n = createI18n(catalogs, storage);
    const updateLanguage = locale => {
      if (target.document && target.document.documentElement) {
        target.document.documentElement.lang = locale;
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
