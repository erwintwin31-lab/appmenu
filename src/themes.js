/* Appmenu settings: shared by browser demo and desktop frontend. */
(function () {
  'use strict';
  const KEY = 'am_editor_v4';
  const skins = { glass: 'Normal', black: 'Black', midnight: 'Midnight', 'frosted-blue': 'Frosted Blue' };
  const layouts = { hub: 'Hub', normal: 'Normal', compact: 'Kompakt' };
  const navigation = { top: 'Seitenleiste', bottom: 'Reiter unten' };
  const modes = { window: 'Fenster', dock: 'Dock' };
  const accents = {
    glas: ['Blau', '#0078d4', '#60cdff'],
    tuerkis: ['Türkis', '#16b8a6', '#4fd8c8'],
    violett: ['Violett', '#7c5cff', '#b5a3ff'],
    gruen: ['Grün', '#2f9e44', '#7be08f'],
    orange: ['Orange', '#e95420', '#ff9a6c'],
    rosa: ['Rosa', '#cf3676', '#ff9dc8']
  };
  Object.assign(ACC, accents);
  const limits = { wr: [0,36], cr: [0,32], br: [0,24], ir: [0,24], side: [140,300], pad: [8,48], gap: [4,32], cols: [1,6], hh: [48,100], ps: [18,48], pp: [4,32], opacity: [45,100], blur: [0,40] };
  const defaults = {
    surface: 'glass', layout: 'hub', tabs: 'top', mode: 'window', appearance: 'system',
    rounded: true, wr: 20, cr: 14, br: 10, ir: 12, side: 200, pad: 26, gap: 14, cols: 4, hh: 64, ps: 26, pp: 18,
    opacity: 94, blur: 20, logo: true, subtitle: true, footer: true, icons: false, light: false,
    accent: 'glas', accentColor: '#0078d4', favorites: ['search','plan','local','sig','rights'],
    groups: ['inst','dat','pruef','sys'], dock: ['search','plan','local','sig','rights']
  };
  const own = (obj, key) => Object.prototype.hasOwnProperty.call(obj, key);
  const isHex = v => typeof v === 'string' && /^#[0-9a-f]{6}$/i.test(v);
  const unique = (value, allowed, fallback) => Array.isArray(value)
    ? Array.from(new Set(value.filter(id => typeof id === 'string' && allowed(id)))) : fallback.slice();
  function validate(value) {
    const v = value && typeof value === 'object' && !Array.isArray(value) ? value : {};
    const n = Object.assign({}, defaults);
    for (const key of Object.keys(limits)) {
      if (Number.isFinite(v[key])) n[key] = Math.round(Math.max(limits[key][0], Math.min(limits[key][1], v[key])));
    }
    for (const key of ['rounded','logo','subtitle','footer','icons','light']) {
      if (typeof v[key] === 'boolean') n[key] = v[key];
    }
    for (const [key, choices] of [['surface',skins],['layout',layouts],['tabs',navigation],['mode',modes]]) {
      if (own(choices, v[key])) n[key] = v[key];
    }
    if (v.surface === 'frostly') n.surface = 'frosted-blue';
    if (['system','light','dark'].includes(v.appearance)) n.appearance = v.appearance;
    else if (typeof v.light === 'boolean') n.appearance = v.light ? 'light' : 'dark';
    if (own(ACC, v.accent)) n.accent = v.accent;
    else if (v.accent === 'custom') n.accent = 'custom';
    if (isHex(v.accentColor)) n.accentColor = v.accentColor;
    const isFunction = id => F.some(f => f[0] === id);
    n.favorites = unique(v.favorites, isFunction, defaults.favorites);
    n.dock = unique(v.dock, isFunction, defaults.dock);
    n.groups = unique(v.groups, id => own(CATS, id), defaults.groups);
    return n;
  }
  let config;
  try { config = validate(JSON.parse(localStorage.getItem(KEY))); } catch (_) { config = validate(null); }
  const media = matchMedia('(prefers-color-scheme: dark)');
  const light = () => config.appearance === 'system' ? !media.matches : config.appearance === 'light';
  const save = () => { try { localStorage.setItem(KEY, JSON.stringify(config)); } catch (_) {} };
  const e = value => String(value).replace(/[&<>"']/g, c => ({ '&':'&amp;', '<':'&lt;', '>':'&gt;', '"':'&quot;', "'":'&#39;' }[c]));
  let tab = 'design';
  S.acc = config.accent; S.light = light(); S.fav = config.favorites.slice();

  // Resolve the stylesheet relative to this script, both at / and at /src/.
  const script = document.currentScript;
  if (script && script.src && !document.getElementById('settings-style')) {
    const link = document.createElement('link'); link.id = 'settings-style'; link.rel = 'stylesheet';
    link.href = new URL('./settings.css', script.src).href; document.head.appendChild(link);
  }
  const baseTheme = theme;
  theme = function () {
    S.light = light();
    if (config.accent === 'custom') ACC.custom = ['Eigene Farbe', config.accentColor, config.accentColor];
    S.acc = config.accent;
    baseTheme();
    const body = document.body;
    body.dataset.surface = config.surface; body.dataset.layout = config.layout;
    body.dataset.tabs = config.tabs; body.dataset.mode = config.mode; body.dataset.rounded = String(config.rounded);
    for (const key of ['subtitle','footer','icons']) body.classList.toggle('ed-' + key, key === 'icons' ? config[key] : !config[key]);
    body.classList.toggle('ed-logo-hidden', !config.logo);
    for (const key of Object.keys(limits)) body.style.setProperty('--ed-' + key, config[key] + (key === 'cols' ? '' : key === 'opacity' ? '%' : 'px'));
    config.light = S.light;
  };
  const baseDraw = draw;
  draw = function () { baseDraw(); config.favorites = S.fav.slice(); save(); };
  const refresh = () => { config = validate(config); S.fav = config.favorites.slice(); draw(); };
  window.edSet = function (key, value) {
    if (!own(defaults, key)) return;
    if (key === 'light') config.appearance = value ? 'light' : 'dark';
    else config[key] = value;
    refresh();
  };
  window.edPreset = function (name) {
    if (!own(layouts, name)) return;
    Object.assign(config, name === 'compact' ? {side:168,pad:16,gap:6,hh:52,pp:8} : name === 'normal' ? {side:248,pad:30,gap:16,hh:64,pp:20} : {side:200,pad:26,gap:14,hh:64,pp:18});
    config.layout = name; refresh();
  };
  window.setLayout = window.edPreset;
  window.edSurface = value => window.edSet('surface', value);
  window.edTabs = value => window.edSet('tabs', value);
  window.edMode = value => window.edSet('mode', value);
  window.setAcc = value => window.edSet('accent', value);
  window.edColor = function (value) { if (!isHex(value)) return; config.accent = 'custom'; config.accentColor = value; refresh(); };
  window.edCorners = function (name) {
    const options = {eckig:[0,0,0,0],klein:[6,6,4,4],mittel:[12,12,8,8],gross:[20,20,14,14],rund:[28,28,18,18]};
    if (!own(options, name)) return;
    ['wr','cr','br','ir'].forEach((key,i) => config[key] = options[name][i]); refresh();
  };
  window.edPin = function (id) {
    if (!F.some(f => f[0] === id)) return;
    const index = config.favorites.indexOf(id);
    if (index < 0) config.favorites.push(id); else config.favorites.splice(index, 1);
    refresh();
  };
  fav = window.edPin;
  window.edDock = function (id) {
    if (!F.some(f => f[0] === id)) return;
    const index = config.dock.indexOf(id);
    if (index < 0) config.dock.push(id); else config.dock.splice(index, 1);
    refresh();
  };
  window.edMove = function (key, id, delta) {
    if (!['favorites','groups','dock'].includes(key) || ![-1,1].includes(delta)) return;
    const a = config[key], i = a.indexOf(id), j = i + delta;
    if (i < 0 || j < 0 || j >= a.length) return;
    [a[i],a[j]] = [a[j],a[i]]; refresh();
  };
  window.edReset = function () { config = validate(null); refresh(); };
  window.edExport = function () {
    const url = URL.createObjectURL(new Blob([JSON.stringify(config,null,2)], {type:'application/json'}));
    const a = document.createElement('a'); a.href = url; a.download = 'appmenu-layout.json'; a.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  };
  window.edImport = function () {
    const input = document.createElement('input'); input.type = 'file'; input.accept = '.json';
    input.onchange = async () => {
      try {
        if (!input.files[0] || input.files[0].size > 50000) throw new Error('size');
        config = validate(JSON.parse(await input.files[0].text())); refresh();
      } catch (_) { toast('Ungültige Layout-Datei'); }
    };
    input.click();
  };
  window.settingsTab = function (value) {
    if (!['design','layout','system'].includes(value)) return;
    tab = value; draw(); document.getElementById('settings-tab-' + tab)?.focus();
  };
  const onSystemChange = () => { if (config.appearance === 'system') draw(); };
  if (media.addEventListener) media.addEventListener('change', onSystemChange);
  else if (media.addListener) media.addListener(onSystemChange);
  const toggle = document.getElementById('th');
  if (toggle) toggle.onclick = () => { config.appearance = light() ? 'dark' : 'light'; refresh(); };

  function choices(object, current, handler) {
    return '<div class="ed-options">' + Object.entries(object).map(([key,label]) =>
      `<button type="button" class="ed-choice ${current === key ? 'active' : ''}" aria-pressed="${current === key}" onclick="${handler}('${key}')">${label}</button>`).join('') + '</div>';
  }
  function checkbox(key, label) {
    return `<label class="ed-check"><input type="checkbox" ${config[key] ? 'checked' : ''} onchange="edSet('${key}',this.checked)"> ${label}</label>`;
  }
  function number(key, label) {
    return `<label class="ed-num"><span>${label}</span><input type="number" min="${limits[key][0]}" max="${limits[key][1]}" value="${config[key]}" onchange="edSet('${key}',Number(this.value))"></label>`;
  }
  function functionList(key) {
    const handler = key === 'dock' ? 'edDock' : 'edPin';
    const order = config[key].concat(F.map(f => f[0]).filter(id => !config[key].includes(id)));
    return order.map(id => {
      const f = fx(id), selected = config[key].includes(id), index = config[key].indexOf(id);
      return `<div class="ed-item"><button type="button" aria-pressed="${selected}" onclick="${handler}('${id}')">${selected ? '✓' : '＋'} ${e(f[1])}</button><span class="sp"></span><button type="button" aria-label="${e(f[1])} nach vorne" ${index <= 0 ? 'disabled' : ''} onclick="edMove('${key}','${id}',-1)">←</button><button type="button" aria-label="${e(f[1])} nach hinten" ${!selected || index === config[key].length - 1 ? 'disabled' : ''} onclick="edMove('${key}','${id}',1)">→</button></div>`;
    }).join('');
  }
  function designView() {
    const descriptions = {glass:'Klar und ruhig, Akzentfarbe wählbar',black:'Schwarz oder Weiß, mit starkem Kontrast',midnight:'Tiefes Blau und sanfte Flächen','frosted-blue':'Getöntes Glas mit kühlem Blau'};
    return '<h2>Theme</h2><div class="theme-grid">' + Object.entries(skins).map(([key,label]) =>
      `<button type="button" class="theme-card ${config.surface === key ? 'selected' : ''}" aria-pressed="${config.surface === key}" onclick="edSurface('${key}')"><span class="theme-preview preview-${key}" aria-hidden="true"><span></span><span></span></span><strong>${label}</strong><small>${descriptions[key]}</small>${config.surface === key ? '<span class="theme-check" aria-hidden="true">✓</span>' : ''}</button>`).join('') +
      '</div><h2>Modus</h2>' + choices({system:'System',light:'Hell',dark:'Dunkel'}, config.appearance, 'edAppearance') +
      '<p class="mute">System folgt dem Farbschema deines Betriebssystems. Theme und Fenstermodus sind unabhängig.</p><h2>Akzentfarbe</h2><div class="chips">' +
      Object.entries(accents).map(([key,value]) => `<button type="button" class="chip ${config.accent === key ? 'on' : ''}" aria-pressed="${config.accent === key}" onclick="setAcc('${key}')"><i style="--c:${value[1]}" aria-hidden="true"></i>${value[0]}</button>`).join('') +
      `</div><label class="ed-color"><span>Eigene Akzentfarbe</span><input type="color" value="${config.accentColor}" onchange="edColor(this.value)"><input aria-label="Akzentfarbe als HEX" class="ed-hex" value="${config.accentColor}" maxlength="7" onchange="edColor(this.value)"></label>` +
      '<h2>Glas-Effekt</h2><p class="mute">Die Browser-Unschärfe betrifft nur den Seitenhintergrund, nicht den Desktop.</p><div class="ed-grid">' + number('opacity','Deckkraft (%)') + number('blur','Unschärfe') + '</div>';
  }
  function layoutView() {
    return '<h2>Layout</h2>' + choices(layouts, config.layout, 'edPreset') +
      '<p class="mute">Hub zeigt Funktionsgruppen. Normal zeigt größere Karten, Kompakt eine platzsparende Liste.</p><h2>Fenstermodus</h2>' + choices(modes, config.mode, 'edMode') +
      '<h2>Navigation im Fenstermodus</h2>' + choices(navigation, config.tabs, 'edTabs') +
      (config.mode === 'dock' ? '<p class="mute">Im Dock-Modus ersetzt die Leiste unten die normale Navigation. Die Reiterposition gilt wieder im Fenstermodus.</p>' : '') +
      '<h2>Bereiche</h2>' + checkbox('logo','Logo anzeigen') + checkbox('subtitle','Plattformtext anzeigen') + checkbox('footer','Fußzeile anzeigen') + checkbox('icons','Navigation nur mit Icons') +
      '<div class="ed-grid">' + number('side','Seitenleiste') + number('hh','Kopfzeile') + number('pad','Inhaltsabstand') + number('gap','Kartenabstand') + number('cols','Kartenspalten') + number('ps','Icongröße') + number('pp','Kachelabstand') +
      '</div><h2>Ecken</h2>' + checkbox('rounded','Abgerundete Ecken aktiv') + choices({eckig:'Eckig',klein:'Klein',mittel:'Mittel',gross:'Groß',rund:'Rund'}, '', 'edCorners') +
      '<div class="ed-grid">' + number('wr','Fenster') + number('cr','Karten') + number('br','Buttons') + number('ir','Eingaben') + '</div><h2>Angeheftete Funktionen</h2>' + functionList('favorites') +
      '<h2>Dock-Funktionen</h2>' + functionList('dock') + '<div class="bar"><button class="btn" onclick="go(\'home\')">Layout ansehen</button><button class="btn g2" onclick="edExport()">Export</button><button class="btn g2" onclick="edImport()">Import</button><button class="btn g2" onclick="edReset()">Zurücksetzen</button></div>';
  }
  window.edAppearance = value => window.edSet('appearance', value);
  const baseSettings = V.set;
  function settingsView() {
    const tabs = {design:'Design',layout:'Layout',system:'System'};
    const header = '<h1>Einstellungen</h1><div class="settings-tabs" role="tablist" aria-label="Einstellungsbereich">' + Object.entries(tabs).map(([key,label]) =>
      `<button type="button" id="settings-tab-${key}" role="tab" aria-selected="${tab === key}" aria-controls="settings-panel-${key}" tabindex="${tab === key ? 0 : -1}" onclick="settingsTab('${key}')">${label}</button>`).join('') + '</div>';
    let content;
    if (tab === 'design') content = designView();
    else if (tab === 'layout') content = layoutView();
    else {
      const info = baseSettings();
      content = info.slice(info.indexOf('</h1>') + 5).split('<h2>Akzentfarbe</h2>')[0];
      content += '<h2>Programmlimit</h2><div class="seg"><button aria-label="Programmlimit senken" onclick="setLim(-1)">−</button><button disabled>' + S.lim + '</button><button aria-label="Programmlimit erhöhen" onclick="setLim(1)">+</button></div><p class="mute">Gilt für ausgewählte Programme, nicht für deren Abhängigkeiten.</p>';
    }
    return header + `<section role="tabpanel" id="settings-panel-${tab}" aria-labelledby="settings-tab-${tab}">${content}</section>`;
  }
  V.set = settingsView;
  // demo.js appends AI settings to V.set; keep those on the System tab only.
  function finishSettings() {
    const decorated = V.set;
    if (decorated !== settingsView) V.set = () => tab === 'system' ? decorated() : settingsView();
    draw();
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', finishSettings, {once:true});
  else finishSettings();
  document.addEventListener('keydown', event => {
    if (!event.target.closest?.('.settings-tabs')) return;
    const keys = ['design','layout','system'], current = keys.indexOf(tab);
    const index = event.key === 'ArrowRight' ? (current+1)%3 : event.key === 'ArrowLeft' ? (current+2)%3 : event.key === 'Home' ? 0 : event.key === 'End' ? 2 : -1;
    if (index < 0) return;
    event.preventDefault(); window.settingsTab(keys[index]);
  });

  const baseNav = nav;
  nav = function () {
    if (config.mode !== 'dock') return baseNav();
    const items = config.dock.map(id => { const f = fx(id); return `<button class="fd-item ${S.v === f[5] ? 'on' : ''}" onclick="open_('${id}')" aria-label="${e(f[1])}" title="${e(f[1])}">${ic(f[3])}<span>${e(f[1])}</span></button>`; }).join('');
    $('nav').innerHTML = '<div class="fd-dock"><button class="fd-item" onclick="go(\'home\')" aria-label="Übersicht" title="Übersicht">' + ic('home') + '<span>Übersicht</span></button>' + items + '<div class="fd-sep"></div><button class="fd-item fd-settings" onclick="go(\'set\')" aria-label="Einstellungen" title="Einstellungen">' + ic('sl') + '<span>Einstellungen</span></button></div>';
  };
  const baseHome = V.home;
  V.home = function () {
    if (config.layout === 'hub') return baseHome();
    return '<div class="sbar" onclick="go(\'search\')">' + ic('search') + '<input placeholder="Programm suchen …" readonly><kbd>Strg K</kbd></div><div class="eye">Dein Werkzeugkasten</div><h1>Alles an seinem Platz.</h1><h2>Angeheftet</h2><div class="ed-pins">' + S.fav.map(id => pin(fx(id))).join('') + '</div><h2>Funktionen</h2><div class="ed-cards">' + F.map(f => `<button class="ed-tool" onclick="open_('${f[0]}')">${ic(f[3])}<span><b>${e(f[1])}</b><small>${e(f[4])}</small></span></button>`).join('') + '</div>';
  };
  draw();
})();
