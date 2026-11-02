/* Search presentation; settings remain owned by themes.js. */
(function () {
  'use strict';
  const KEY = 'am_search_ui_v1';
  const models = {'lokal-klein':'Lokal · Klein','lokal-mittel':'Lokal · Mittel',cloud:'Cloud · optional'};
  let size = 'large', undo = null;
  try { const value = localStorage.getItem(KEY); if (['large','compact'].includes(value)) size = value; } catch (_) {}
  function ai() { try { return JSON.parse(localStorage.getItem('am_ai_v1') || '{}') || {}; } catch (_) { return {}; } }
  function resize(value) { size = value; try { localStorage.setItem(KEY, value); } catch (_) {} }
  function color(value) { edColor(value); }
  window.glassSize = function () { S.q = document.getElementById('qi')?.value || S.q; resize(size === 'large' ? 'compact' : 'large'); draw(); document.getElementById('qi')?.focus(); };
  window.glassModel = function (value) {
    if (!Object.prototype.hasOwnProperty.call(models, value)) return;
    if (typeof window.edAI === 'function') return window.edAI('model', value);
    const settings = ai(); settings.model = value;
    try { localStorage.setItem('am_ai_v1', JSON.stringify(settings)); } catch (_) {}
    draw();
  };
  const examples = ['Was kann Appmenu?','Wie prüfe ich eine Datei?','Mach das Theme Frosted Blue'];
  window.glassExample = function (index) { if (!examples[index]) return; S.q = examples[index]; draw(); document.getElementById('qi')?.focus(); };
  function snapshot() { return {surface:document.body.dataset.surface,mode:document.body.dataset.mode,accent:S.acc,color:document.documentElement.style.getPropertyValue('--accent').trim(),size}; }
  window.glassUndo = function () {
    if (!undo) return;
    const old = undo; undo = null;
    edSet('surface', old.surface); edSet('mode', old.mode); resize(old.size);
    if (old.accent === 'custom' && /^#[0-9a-f]{6}$/i.test(old.color)) color(old.color);
    else if (ACC[old.accent]) setAcc(old.accent);
    S.commandNotice = 'Änderung rückgängig gemacht.'; draw();
  };
  function command(q) {
    const text = q.toLowerCase().trim().replace(/[.!?]+$/, '');
    if (!/^(mach(?:e)?|setz(?:e)?|stell(?:e)?|wechsel(?:e)?|aktiviere|schalte|große suche|grosse suche|kompakte suche|dock einschalten|dock ausschalten)\b/u.test(text)) return null;
    const colors = {blau:'#0078d4',türkis:'#16b8a6',tuerkis:'#16b8a6',violett:'#7c5cff',rot:'#e05264',grün:'#2f9e44',gruen:'#2f9e44',orange:'#e95420'};
    if (/\b(theme|farbe|highlight|akzent|akzentfarbe)\b/u.test(text)) {
      for (const [name,value] of Object.entries(colors)) {
        if (new RegExp('\\b' + name + '\\b', 'u').test(text)) return {run:() => color(value),text:'Highlight-Farbe auf ' + name + ' gesetzt.'};
      }
    }
    for (const [word,value] of [['midnight','midnight'],['frosted blue','frosted-blue'],['frosted-blue','frosted-blue'],['frostly','frosted-blue'],['glass','glass'],['normal','glass'],['black','black']]) {
      if (text.includes(word)) return {run:() => edSet('surface',value),text:'Design ' + value + ' aktiviert.'};
    }
    if (text.includes('suche') && /kompakt|compact/.test(text)) return {run:() => resize('compact'),text:'Kompakte Suche aktiviert.'};
    if (text.includes('suche') && /groß|gross|large/.test(text)) return {run:() => resize('large'),text:'Große Suche aktiviert.'};
    if (text.includes('dock') && /aus|deaktiviere/.test(text)) return {run:() => edSet('mode','window'),text:'Fenstermodus aktiviert.'};
    if (text.includes('dock') && /ein|aktiviere/.test(text)) return {run:() => edSet('mode','dock'),text:'Dock aktiviert.'};
    return null;
  }
  const oldSearch = doSearch;
  doSearch = async function () {
    const q = (document.getElementById('qi')?.value || S.q || '').trim(), hit = command(q);
    if (!hit) return oldSearch.apply(this, arguments);
    undo = snapshot(); S.q = q; hit.run(); S.commandNotice = hit.text; draw();
  };
  window.glassEnter = function (event) { if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) { event.preventDefault(); doSearch(); } };
  const oldView = V.search;
  V.search = function () {
    const html = oldView(), end = html.indexOf('</div>'); if (end < 0) return html;
    const a = ai(), model = Object.prototype.hasOwnProperty.call(models,a.model) ? a.model : 'lokal-klein';
    return '<section class="glass-search" data-size="' + size + '"><div class="glass-search-heading"><h1>Was möchtest du erledigen?</h1><p>Suchen, Fragen stellen oder Appmenu anpassen.</p></div><div class="glass-composer"><textarea id="qi" aria-label="Suche oder lokalen Befehl eingeben" rows="' + (size === 'large' ? 4 : 1) + '" placeholder="Programm suchen oder Frage stellen …" oninput="S.q=this.value" onkeydown="glassEnter(event)">' + esc(S.q) + '</textarea><div class="glass-search-tools"><select aria-label="Modell auswählen – Vorschau" onchange="glassModel(this.value)">' + Object.entries(models).map(([key,label]) => '<option value="' + key + '" ' + (key === model ? 'selected' : '') + '>' + label + '</option>').join('') + '</select><small>Modellwahl · ' + (MOCK ? 'Demo' : 'Vorschau') + '</small><button class="glass-size" onclick="glassSize()" aria-label="Suchansicht wechseln">' + (size === 'large' ? '↗' : '↙') + '</button><button class="btn" onclick="doSearch()">Suchen</button></div></div><div class="glass-examples">' + examples.map((q,i) => '<button onclick="glassExample(' + i + ')">' + esc(q) + '</button>').join('') + '</div></section>' +
      (S.commandNotice ? '<div class="info" role="status">' + esc(S.commandNotice) + (undo ? ' <button class="btn g2" onclick="glassUndo()">Rückgängig</button>' : '') + '</div>' : '') + html.slice(end + 6);
  };
  const oldNav = nav;
  nav = function () { oldNav(); document.querySelectorAll('.fd-item').forEach(button => { const name = button.querySelector('span')?.textContent; if (name) { button.setAttribute('aria-label',name); button.title = name; } if (button.classList.contains('on')) button.setAttribute('aria-current','page'); }); };
  let tick = 0;
  const hints = ['Programm suchen …','Mach das Theme Frosted Blue','Wechsle zu Midnight','Mach die Suche kompakt'];
  const timer = setInterval(() => {
    const input = document.getElementById('qi');
    if (!input || input.value || input === document.activeElement || document.hidden || matchMedia('(prefers-reduced-motion: reduce)').matches) return;
    const text = hints[Math.floor(tick / 45) % hints.length]; input.placeholder = text.slice(0, Math.min(tick % 45,text.length)) || ' '; tick++;
  },110);
  window.addEventListener('pagehide', () => clearInterval(timer), {once:true});
  draw();
})();
