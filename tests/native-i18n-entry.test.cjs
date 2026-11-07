const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const root = path.resolve(__dirname, '..');
test('native entry loads i18n dependencies before app.js', () => {
  const html = fs.readFileSync(path.join(root, 'src/index.html'), 'utf8');
  const scripts = [...html.matchAll(/<script\b[^>]*\bsrc=["']([^"']+)["'][^>]*>/g)].map(m => m[1]);
  const dependencies = ['i18n/catalogs.js', 'i18n/core.js', 'i18n/bootstrap.js', 'i18n/language-view.js', 'app.js'];
  let previous = -1;
  for (const name of dependencies) {
    assert.equal(scripts.filter(s => s === name).length, 1, name);
    const index = scripts.indexOf(name);
    assert.ok(index > previous, name + ' load order');
    assert.ok(fs.existsSync(path.join(root, 'src', name)), name + ' exists');
    previous = index;
  }
  const storage = new Map();
  const context = vm.createContext({
    document: {documentElement: {lang: ''}},
    localStorage: {getItem: key => storage.get(key) ?? null, setItem: (key, value) => storage.set(key, String(value))}
  });
  for (const name of dependencies.slice(0, -1)) {
    vm.runInContext(fs.readFileSync(path.join(root, 'src', name), 'utf8'), context, {filename: name});
  }
  assert.ok(context.AppmenuI18n);
  assert.notEqual(context.AppmenuI18n.t('home.title'), 'home.title');
  assert.notEqual(context.AppmenuI18n.t('nav.home'), 'nav.home');
  assert.match(context.appmenuLanguageView(), /<select id="am-language"/);
  assert.match(context.appmenuLanguageView(), /<option /);
  assert.equal(context.document.documentElement.lang, context.AppmenuI18n.getLocale());
});
