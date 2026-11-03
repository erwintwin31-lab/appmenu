const test = require('node:test');
const assert = require('node:assert/strict');
const catalogs = require('../src/i18n/catalogs.js');
const {createI18n} = require('../src/i18n/core.js');
const {initializeLocale} = require('../src/i18n/bootstrap.js');

test('frontend defaults to English and exposes the shared locale instance', () => {
  const target = {document:{documentElement:{lang:'de'}}};
  const instance = initializeLocale(target, catalogs, createI18n);
  assert.equal(target.document.documentElement.lang, 'en');
  assert.equal(target.AppmenuI18n, instance);
});

test('saved locale is restored and document language follows changes', () => {
  const saved = new Map([['am_language_v1','fr']]);
  const target = {document:{documentElement:{}},localStorage:{getItem:k=>saved.get(k),setItem:(k,v)=>saved.set(k,v)}};
  const instance = initializeLocale(target, catalogs, createI18n);
  assert.equal(target.document.documentElement.lang, 'fr');
  assert.equal(instance.setLocale('de'), true);
  assert.equal(target.document.documentElement.lang, 'de');
  assert.equal(saved.get('am_language_v1'), 'de');
  assert.equal(instance.setLocale('invalid'), false);
  assert.equal(target.document.documentElement.lang, 'de');
});

test('blocked localStorage access does not prevent frontend initialization', () => {
  const target = {document:{documentElement:{}}};
  Object.defineProperty(target, 'localStorage', {get(){throw new Error('blocked');}});
  const instance = initializeLocale(target, catalogs, createI18n);
  assert.equal(instance.getLocale(), 'en');
  instance.setLocale('fr');
  assert.equal(target.document.documentElement.lang, 'fr');
});
