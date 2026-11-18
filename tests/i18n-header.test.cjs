const test = require('node:test');
const assert = require('node:assert/strict');
const catalogs = require('../src/i18n/catalogs.js');
const {createI18n} = require('../src/i18n/core.js');
const {initializeLocale} = require('../src/i18n/bootstrap.js');

test('header labels update on language change, even without native window controls', () => {
  const attrs = {};
  const document = {
    documentElement: {},
    getElementById(id) {
      if (!['st', 'th'].includes(id)) return null;
      return {setAttribute(name, value) { attrs[`${id}.${name}`] = value; }};
    }
  };
  const i18n = initializeLocale({document}, catalogs, createI18n);
  assert.equal(attrs['st.title'], 'Settings');
  i18n.setLocale('de');
  assert.equal(attrs['st.aria-label'], 'Einstellungen');
  i18n.setLocale('fr');
  assert.equal(attrs['st.title'], 'Paramètres');
  assert.equal(attrs['th.aria-label'], 'Basculer entre les modes clair et sombre');
  assert.equal(document.documentElement.lang, 'fr');
});
