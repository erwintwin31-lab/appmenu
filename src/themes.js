(function(){
var UA=navigator.userAgent||'';
var PLAT=/Windows/i.test(UA)?'windows':(/Linux|X11/i.test(UA)&&!/Android/i.test(UA)?'linux':'windows');
var TH={windows:{def:'blau',list:['glas','blau','tuerkis','violett','gruen','orange']},linux:{def:'cachy',list:['cachy','arch','debian','fedora','ubuntu','suse']}};
var LAY={glass:'Glass',klassisch:'Klassisch'};
var LDEF={windows:'glass',linux:'klassisch'};
Object.assign(ACC,{cachy:['CachyOS','#16b8a6','#4fd8c8'],arch:['Arch Linux','#1793d1','#5cc0f2'],debian:['Debian','#ce0056','#ff6aa2'],fedora:['Fedora','#3c6eb4','#78a8ea'],ubuntu:['Ubuntu','#e95420','#ff8a5c'],suse:['openSUSE','#73ba25','#9be04a']});
var storedAcc=null,storedLay=null;
try{storedAcc=localStorage.getItem('am_a');storedLay=localStorage.getItem('am_ly')}catch(e){}
if(!storedAcc||!ACC[storedAcc])S.acc=TH[PLAT].def;
S.layout=LAY[storedLay]?storedLay:LDEF[PLAT];
window.setAcc=function(k){if(!ACC[k])return;S.acc=k;try{localStorage.setItem('am_a',k)}catch(e){}draw()};
window.setLayout=function(k){if(!LAY[k])return;S.layout=k;try{localStorage.setItem('am_ly',k)}catch(e){}draw()};
var baseTheme=theme;
theme=function(){baseTheme();var b=document.body;b.setAttribute('data-plat',PLAT);b.setAttribute('data-layout',S.layout);var sub=document.querySelector('.sub');if(sub)sub.textContent=PLAT==='linux'?'Linux':'Windows'};
var chip=function(k){var v=ACC[k];return `<button class="chip ${k===S.acc?'on':''}" onclick="setAcc('${k}')"><i style="--c:${v[1]}"></i>${v[0]}</button>`};
var themeSection=function(){var own=TH[PLAT].list,other=TH[PLAT==='linux'?'windows':'linux'].list;return `<h2>Theme</h2><div class="chips">${own.map(chip).join('')}</div><div class="mute" style="margin:14px 0 8px">Weitere Themes</div><div class="chips">${other.map(chip).join('')}</div>`};
var layoutSection=function(){return `<h2>Layout</h2><div class="seg">${Object.keys(LAY).map(function(k){return `<button class="${k===S.layout?'on':''}" onclick="setLayout('${k}')">${LAY[k]}</button>`}).join('')}</div><div class="mute" style="margin-top:8px">Glass: kompakt mit Gruppenkarten. Klassisch: breite Seitenleiste mit Kacheln und Karten.</div>`};
var baseSet=V.set;
V.set=function(){var h=baseSet();var re=/<h2>Akzentfarbe<\/h2><div class="chips">[\s\S]*?<\/div>/;return (re.test(h)?h.replace(re,themeSection()):h+themeSection())+layoutSection()};
var baseHome=V.home;
var homeKlassisch=function(){
 var pins=S.fav.map(function(id){return pin(fx(id))}).join('')||'<span class="mute">Nichts angeheftet.</span>';
 var cards=['search','local','sig','rights'].map(function(id){var f=fx(id);return `<button class="card" onclick="open_('${id}')"><b>${f[1]}</b><small>${f[4]}</small></button>`}).join('');
 return `<div class="sbar" onclick="go('search')">${ic('search',22)}<input placeholder="Programm suchen …" readonly><kbd>Strg K</kbd></div>
<div class="eye">Dein Werkzeugkasten</div><h1>Alles an seinem Platz.</h1><div class="mute">Programme finden, Dateien prüfen und sicher einbinden. Entscheiden tust immer du.</div>
<h2>Angeheftet<a onclick="S.edit=!S.edit;draw()">${S.edit?'Fertig':'Bearbeiten'} ${ic('chev')}</a></h2><div class="tiles ${S.edit?'edit':''}">${pins}</div>
<h2>Entdecken<a onclick="go('all')">Alle anzeigen ${ic('ext')}</a></h2><div class="cards">${cards}</div>`};
V.home=function(){return S.layout==='klassisch'?homeKlassisch():baseHome()};
if(typeof draw==='function')draw();
})();
