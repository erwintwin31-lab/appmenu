const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const root = path.resolve(__dirname, '..');
const source = fs.readFileSync(path.join(root,'src/themes.js'),'utf8');

function boot(saved = null) {
  const storage = new Map([['am_editor_v4',saved]]), classes = new Set(), vars = new Map(), elements = new Map(), events = {};
  const media = {matches:false,addEventListener:(_,fn) => media.change = fn};
  const element = id => {
    if (!elements.has(id)) elements.set(id,{innerHTML:'',focus(){}});
    return elements.get(id);
  };
  const ctx = {
    console, URL, Blob, setTimeout:() => 0,
    ACC:{glas:['Blau','#0078d4','#60cdff']},
    F:[['search','Suchen','inst','search','Suche','search'],['plan','Plan','inst','box','Plan','plan'],['local','Dateien','dat','file','Dateien','local'],['sig','Prüfen','pruef','shield','Prüfen','local'],['rights','Rechte','sys','lock','Rechte','rights']],
    CATS:{inst:[],dat:[],pruef:[],sys:[]},
    S:{v:'set',acc:'glas',light:false,fav:[],lim:5},
    V:{set:() => '<h1>Einstellungen</h1><h2>Systemstatus</h2><p>Demo</p><h2>Akzentfarbe</h2>Alt',home:() => 'HUB'},
    localStorage:{getItem:k => storage.get(k) ?? null,setItem:(k,v) => storage.set(k,String(v))},
    document:{currentScript:{src:'https://example.invalid/src/themes.js'},readyState:'loading',
      head:{appendChild(){}},createElement:() => ({}),getElementById:element,
      addEventListener:(name,fn) => events[name] = fn,
      body:{dataset:{},classList:{toggle(k,v){v?classes.add(k):classes.delete(k)}},style:{setProperty:(k,v) => vars.set(k,v)}}},
    matchMedia:() => media,
    ic:() => '<svg></svg>',pin:f => f[1],fav(){},toast(){},
  };
  ctx.document.documentElement = {style:{setProperty:(k,v) => vars.set(k,v)}};
  ctx.$ = element; ctx.fx = id => ctx.F.find(f => f[0] === id);
  ctx.theme = () => {const accent = ctx.ACC[ctx.S.acc]; assert.ok(accent); vars.set('--accent',accent[1]); ctx.document.body.classList.toggle('light',ctx.S.light)};
  ctx.nav = () => {element('nav').innerHTML = 'SIDEBAR'};
  ctx.draw = () => {ctx.theme();ctx.nav();ctx.html = ctx.V[ctx.S.v]()};
  ctx.go = value => {ctx.S.v=value;ctx.draw()};
  ctx.window=ctx;
  vm.createContext(ctx);vm.runInContext(source,ctx,{filename:'themes.js'});
  return {ctx,storage,classes,vars,media,events,elements,config:() => JSON.parse(storage.get('am_editor_v4'))};
}

test('settings provide tabs and four working theme choices', () => {
  const {ctx} = boot();
  for (const name of ['design','layout','system']) assert.match(ctx.html,new RegExp('settings-tab-'+name));
  for (const surface of ['glass','black','midnight','frosted-blue']) {
    ctx.edSurface(surface);assert.equal(ctx.document.body.dataset.surface,surface);
  }
});
test('system mode follows the operating-system preference', () => {
  const b=boot();assert.equal(b.config().appearance,'system');
  b.media.matches=true;b.media.change();assert.equal(b.ctx.S.light,false);
  b.media.matches=false;b.media.change();assert.equal(b.ctx.S.light,true);
  b.ctx.edAppearance('dark');b.media.matches=false;b.media.change();assert.equal(b.ctx.S.light,false);
});
test('custom accents work without an undefined ACC entry', () => {
  const b=boot();b.ctx.edColor('#ab1234');assert.equal(b.vars.get('--accent'),'#ab1234');
  b.ctx.edColor('not-a-color');assert.equal(b.vars.get('--accent'),'#ab1234');
});
test('layout, dock and navigation controls update the actual DOM state', () => {
  const b=boot();b.ctx.settingsTab('layout');
  for (const layout of ['compact','normal','hub']) {b.ctx.edPreset(layout);assert.equal(b.ctx.document.body.dataset.layout,layout)}
  b.ctx.edMode('dock');assert.match(b.elements.get('nav').innerHTML,/fd-dock/);
  b.ctx.edTabs('bottom');assert.equal(b.ctx.document.body.dataset.tabs,'bottom');
  b.ctx.edMode('window');assert.equal(b.elements.get('nav').innerHTML,'SIDEBAR');
});
test('corrupt or obsolete storage is migrated and clamped', () => {
  for (const value of [null,'null','[]','invalid json']) assert.doesNotThrow(() => boot(value));
  const b=boot(JSON.stringify({surface:'frostly',light:true,side:9999,gap:-10,accent:'custom',accentColor:'#123456',dock:['search','missing','search']}));
  const c=b.config();assert.equal(c.surface,'frosted-blue');assert.equal(c.appearance,'light');assert.equal(c.side,300);assert.equal(c.gap,4);assert.deepEqual(c.dock,['search']);
});
test('favorites and dock selections survive reload and can be reordered', () => {
  const b=boot();b.ctx.edPin('search');assert.ok(!b.config().favorites.includes('search'));
  b.ctx.edDock('search');assert.ok(!b.config().dock.includes('search'));
  b.ctx.edMove('favorites','rights',-1);assert.equal(b.config().favorites[2],'rights');
  const restored=boot(b.storage.get('am_editor_v4'));assert.deepEqual(restored.config().favorites,b.config().favorites);
});
test('demo decorations are confined to the System tab', () => {
  const b=boot(),before=b.ctx.V.set;
  b.ctx.V.set=() => before()+'DEMO_AI';b.events.DOMContentLoaded();
  assert.ok(!b.ctx.V.set().includes('DEMO_AI'));
  b.ctx.settingsTab('system');assert.ok(b.ctx.V.set().includes('DEMO_AI'));assert.match(b.ctx.html,/Systemstatus/);
});
