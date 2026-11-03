const test = require('node:test');
const assert = require('node:assert/strict');
const catalogs = require('../src/i18n/catalogs.js');
const {createI18n} = require('../src/i18n/core.js');

test('complete catalogs use English fallback and interpolation', () => {
  const keys = Object.keys(catalogs.en.messages);
  for (const code of ['de','fr']) assert.deepEqual(Object.keys(catalogs[code].messages).sort(), keys.slice().sort());
  const i18n = createI18n(catalogs, {getItem: () => null, setItem: () => {}});
  assert.equal(i18n.getLocale(), 'en');
  assert.equal(i18n.t('settings.title'), 'Settings');
  assert.equal(i18n.t('missing.key'), 'missing.key');
  i18n.setLocale('de');
  assert.equal(i18n.t('settings.title'), 'Einstellungen');
  assert.equal(i18n.t('search.selected', {count:2,limit:5}), '2 / 5 gewählt');
  assert.equal(i18n.t('toast.limit', {limit:5}), 'Limit erreicht: höchstens 5 Programme');
  assert.equal(i18n.t('missing.key'), 'missing.key');
  i18n.setLocale('fr');
  assert.equal(i18n.t('settings.title'), 'Paramètres');
});
