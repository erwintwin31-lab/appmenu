const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const read = name => JSON.parse(fs.readFileSync(path.join(__dirname, '../src-tauri', name), 'utf8'));
const base = read('tauri.conf.json');
const windows = read('tauri.windows.conf.json');
const linux = read('tauri.linux.conf.json');
function mergePatch(target, patch) {
  if (patch === null || typeof patch !== 'object' || Array.isArray(patch)) return patch;
  const result = target && typeof target === 'object' && !Array.isArray(target) ? {...target} : {};
  for (const [key, value] of Object.entries(patch)) {
    if (value === null) delete result[key];
    else result[key] = mergePatch(result[key], value);
  }
  return result;
}
test('platform-neutral base retains shared native settings', () => {
  assert.equal(base.productName, 'Appmenu');
  assert.equal(base.identifier, 'de.appmenu.app');
  assert.equal(base.build.frontendDist, '../src');
  assert.equal(base.app.withGlobalTauri, false);
  assert.equal(base.app.security.csp, null);
  assert.equal(base.bundle.active, true);
  assert.equal(Object.hasOwn(base.bundle, 'targets'), false);
  assert.equal(Object.hasOwn(base.bundle, 'windows'), false);
  assert.deepEqual(base.app.windows, [{label: 'main', title: 'Appmenu', width: 1240, height: 860, minWidth: 760, minHeight: 560, center: true, decorations: false, transparent: true, shadow: false}]);
});
test('platform overlays preserve shared settings and select their bundle formats', () => {
  for (const overlay of [windows, linux]) {
    assert.equal(Object.hasOwn(overlay, 'app'), false);
    const effective = mergePatch(base, overlay);
    assert.deepEqual(effective.app, base.app);
    assert.deepEqual(effective.build, base.build);
    assert.deepEqual(effective.bundle.icon, base.bundle.icon);
    assert.equal(effective.identifier, base.identifier);
    assert.equal(effective.version, base.version);
  }
  assert.deepEqual(mergePatch(base, windows).bundle.targets, ['nsis']);
  const effectiveLinux = mergePatch(base, linux);
  assert.deepEqual(effectiveLinux.bundle.targets, ['deb', 'appimage']);
  assert.equal(Object.hasOwn(effectiveLinux.bundle, 'windows'), false);
});
test('Windows installer behavior remains unchanged', () => {
  assert.deepEqual(windows.bundle.windows, {
    webviewInstallMode: {type: 'downloadBootstrapper', silent: true},
    nsis: {installMode: 'currentUser', languages: ['German'], displayLanguageSelector: false, sidebarImage: 'installer/sidebar.bmp', headerImage: 'installer/header.bmp'}
  });
});
