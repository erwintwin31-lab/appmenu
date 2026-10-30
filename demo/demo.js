(function(){
var NL=String.fromCharCode(10);
var sleep=function(ms){return new Promise(function(r){setTimeout(r,ms)})};
var DB={
 '7zip.7zip':{nm:'7-Zip',ver:'24.08',pub:'Igor Pavlov',lic:'LGPL',home:'https://www.7-zip.org/',desc:'ein freier Datei-Packer mit hoher Kompression für ZIP, 7z, TAR und weitere Formate',kw:['7zip','7-zip','zip','packer','archiv','archive','entpack']},
 'VideoLAN.VLC':{nm:'VLC media player',ver:'3.0.21',pub:'VideoLAN',lic:'GPL',home:'https://www.videolan.org/',desc:'ein freier Medienplayer, der die meisten Audio- und Videoformate ohne zusätzliche Codecs abspielt',kw:['vlc','video','player','film','musik','media']},
 'Notepad++.Notepad++':{nm:'Notepad++',ver:'8.7',pub:'Notepad++ Team',lic:'GPL',home:'https://notepad-plus-plus.org/',desc:'ein freier Quelltext-Editor mit Tabs, Syntaxhervorhebung und Plugins',kw:['notepad','editor','text','code']},
 'Mozilla.Firefox':{nm:'Mozilla Firefox',ver:'131.0',pub:'Mozilla',lic:'MPL-2.0',home:'https://www.mozilla.org/firefox/',desc:'ein freier Webbrowser der Mozilla Foundation',kw:['firefox','browser','internet','web']},
 'Git.Git':{nm:'Git',ver:'2.47.0',pub:'The Git Development Community',lic:'GPL-2.0',home:'https://git-scm.com/',desc:'ein verteiltes Versionsverwaltungssystem für Quelltext',kw:['git','version','github']},
 'Microsoft.VisualStudioCode':{nm:'Visual Studio Code',ver:'1.95.0',pub:'Microsoft',lic:'Proprietär (kostenlos)',home:'https://code.visualstudio.com/',desc:'ein erweiterbarer Code-Editor von Microsoft',kw:['vscode','visual studio','code','editor','entwick']},
 'Python.Python.3.12':{nm:'Python 3.12',ver:'3.12.7',pub:'Python Software Foundation',lic:'PSF-Lizenz',home:'https://www.python.org/',desc:'die Programmiersprache Python in Version 3.12',kw:['python','programmier','skript']},
 'OBSProject.OBSStudio':{nm:'OBS Studio',ver:'30.2.3',pub:'OBS Project',lic:'GPL-2.0',home:'https://obsproject.com/',desc:'eine freie Software zum Aufnehmen und Streamen von Video',kw:['obs','stream','aufnahme','record','twitch']},
 'Audacity.Audacity':{nm:'Audacity',ver:'3.7.0',pub:'Audacity Team',lic:'GPL-3.0',home:'https://www.audacityteam.org/',desc:'ein freier Audio-Editor zum Aufnehmen und Bearbeiten von Ton',kw:['audacity','audio','ton','sound','mikrofon']},
 'GIMP.GIMP':{nm:'GIMP',ver:'2.10.38',pub:'GIMP Team',lic:'GPL-3.0',home:'https://www.gimp.org/',desc:'ein freies Bildbearbeitungsprogramm',kw:['gimp','bild','foto','grafik','image']}
};
MOCKDB.length=0;
Object.keys(DB).forEach(function(id){MOCKDB.push({id:id,nm:DB[id].nm,ver:DB[id].ver})});
var FAQ=[
 {re:/admin|rechte|berechtig|benutzerkonto|scope/i,a:'Appmenu installiert Programme mit winget und dem Schalter --scope user. Dadurch landet die Software nur in deinem Benutzerprofil und braucht keine Administratorrechte. Gibt es für ein Paket keine Benutzerinstallation, meldet winget das, und Appmenu weicht nicht auf eine Systeminstallation aus. Geräterichtlinien wie AppLocker umgeht Appmenu nicht.',s:['Appmenu · Rechte und Richtlinien','winget install --scope user']},
 {re:/sha|hash|pruefsumme|prüfsumme|checksum/i,a:'Eine SHA-256-Prüfsumme ist ein digitaler Fingerabdruck einer Datei. Stimmt sie mit dem Wert des Herstellers überein, wurde die Datei beim Download nicht verändert. In Appmenu öffnest du dazu Lokale Dateien, wählst eine EXE oder MSI und klickst auf SHA-256 berechnen.',s:['Appmenu · Dateiprüfung']},
 {re:/doppelt|endung|extension|pdf[.]exe/i,a:'Eine doppelte Dateiendung wie rechnung.pdf.exe tarnt ein Programm als Dokument. Appmenu markiert solche Dateien in der Dateiprüfung und warnt dich, bevor etwas gestartet wird.',s:['Appmenu · Dateiprüfung']},
 {re:/winget/i,a:'winget ist der Paketmanager von Windows (App-Installer). Appmenu nutzt ihn zum Suchen und Installieren. Auf Geräten ohne winget bleibt die Suche leer, die Dateiprüfung funktioniert weiter.',s:['winget','Appmenu · Einstellungen']},
 {re:/sicher|vertrau|virus|malware/i,a:'Appmenu zeigt dir Herkunft, die Angaben aus dem winget-Manifest und Warnungen bei verdächtigen Dateien. Diese Hinweise ersetzen nicht deine eigene Prüfung: Du bestätigst jede Installation selbst.',s:['Appmenu · Quellen prüfen']},
 {re:/linux|arch|debian|ubuntu|fedora|suse/i,a:'Appmenu gibt es als frühe Vorschau auch für Linux (AppImage und deb). Die Installationsfunktionen über winget sind dort noch nicht verfügbar.',s:['Appmenu · Plattformen']},
 {re:/theme|layout|farbe|design|dunkel/i,a:'Unter Einstellungen wählst du Hell oder Dunkel, ein Farb-Theme und das Layout: Glass (kompakt mit Gruppenkarten) oder Klassisch (breite Seitenleiste mit Kacheln und Karten).',s:['Appmenu · Einstellungen']}
];
function findPk(l){var hits=[];Object.keys(DB).forEach(function(id){var d=DB[id],ok=(id+' '+d.nm).toLowerCase().indexOf(l)>=0;d.kw.forEach(function(k){if(l.indexOf(k)>=0||(k.indexOf(l)>=0&&l.length>=3))ok=true});if(ok)hits.push(id)});return hits}
function answer(q){
 var l=q.toLowerCase();
 for(var i=0;i<FAQ.length;i++){if(FAQ[i].re.test(l))return{text:FAQ[i].a,src:FAQ[i].s}}
 var h=findPk(l);
 if(h.length){
  var parts=h.slice(0,3).map(function(id){var d=DB[id];return d.nm+' ist '+d.desc+'. Herausgeber: '+d.pub+', Lizenz: '+d.lic+'.'});
  var t=parts.join(' ')+' Appmenu zeigt dir vor der Installation die Angaben aus dem winget-Manifest. Ob ein Paket für dein Benutzerkonto installiert werden kann, meldet winget; Appmenu weicht nie auf eine Systeminstallation aus.';
  return{text:t,src:h.slice(0,3).map(function(id){return 'winget-Manifest · '+id}).concat(['Appmenu · Rechte und Richtlinien'])}
 }
 return{text:'Zu '+q+' gibt es im Demo-Katalog keine Treffer. Der Demo-Katalog enthält zehn Programme, zum Beispiel VLC, 7-Zip, Firefox und Git. In der echten App durchsucht Appmenu den vollständigen winget-Katalog.',src:['Appmenu · Demo-Katalog']}
}
var esc2=function(s){return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;')};
var SPARK=`<svg viewBox='0 0 24 24'><path d='M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z'/><path d='M19 16l.8 2.2L22 19l-2.2.8L19 22l-.8-2.2L16 19l2.2-.8z'/></svg>`;
function aiBlock(){
 var a=S.ai;if(!a||!a.q)return '';
 var head=`<div class='ai-h'>${SPARK}<span>KI-Übersicht</span><span class='ai-badge'>Demo</span></div>`;
 if(a.phase==='think')return `<div class='ai' id='ai'>${head}<div class='ai-think'><i></i><i></i><i></i><span>Quellen werden gelesen ...</span></div><div class='ai-sk'></div><div class='ai-sk s2'></div><div class='ai-sk s3'></div></div>`;
 var txt=esc2(a.words.slice(0,a.n).join(' '));
 var done=a.phase==='done';
 var src=done?`<div class='ai-src'>${a.src.map(function(s){return `<span class='ai-chip'>${esc2(s)}</span>`}).join('')}</div><div class='ai-ft'><span>KI-Übersichten können Fehler enthalten und sind nie eine Installationsfreigabe.</span><span class='ai-fb'><button onclick='aiFb(1)'>Hilfreich</button><button onclick='aiFb(0)'>Nicht hilfreich</button></span></div>`:'';
 return `<div class='ai' id='ai'>${head}<div class='ai-txt'>${txt}${done?'':`<span class='ai-caret'></span>`}</div>${src}</div>`
}
window.aiFb=function(){toast('Danke für dein Feedback (Demo)')};
var T=null;
function paint(){var el=document.getElementById('ai');if(el)el.outerHTML=aiBlock()}
function startAI(q){
 if(T)clearInterval(T);
 var a=answer(q),words=a.text.split(' ');
 S.ai={q:q,phase:'think',words:words,n:0,src:a.src};
 setTimeout(function(){
  if(!S.ai||S.ai.q!==q)return;
  S.ai.phase='type';paint();
  T=setInterval(function(){
   if(!S.ai||S.ai.q!==q){clearInterval(T);return}
   S.ai.n=Math.min(words.length,S.ai.n+2);
   if(S.ai.n>=words.length){S.ai.phase='done';clearInterval(T)}
   paint();
  },60);
 },700);
}
var baseSearch=V.search;
V.search=function(){
 var h=baseSearch();
 if(!S.ai||!S.ai.q)return h;
 var m='<kbd>Enter</kbd></div>',i=h.indexOf(m);
 if(i<0)return h+aiBlock();
 i+=m.length;
 return h.slice(0,i)+aiBlock()+h.slice(i);
};
var baseDoSearch=doSearch;
window.doSearch=function(){
 var el=document.getElementById('qi'),q=(el?el.value:S.q).trim();
 if(q.length>=2)startAI(q);
 return baseDoSearch.apply(this,arguments);
};
var baseMock=mock,JOBS={};
function manifest(id){
 var d=DB[id];if(!d)return 'Keine Details gefunden.';
 return ['Gefunden '+d.nm+' ['+id+']','Version: '+d.ver,'Herausgeber: '+d.pub,'Lizenz: '+d.lic,'Homepage: '+d.home,'Beschreibung: '+d.desc,'Quelle: winget (Demo-Daten)'].join(NL);
}
function jobState(jid){
 var j=JOBS[jid];if(!j)return{lines:[],done:true,ok:false};
 var steps=Math.floor((Date.now()-j.t)/900),lines=[],used=0,done=true;
 j.ids.forEach(function(id){
  var d=DB[id]||{nm:id,ver:''};
  var seq=['== '+id+' ==','Gefunden '+d.nm+' ['+id+'] Version '+d.ver,'Der Installer wird heruntergeladen ...','Installer-Hash erfolgreich überprüft','Installation für das Benutzerkonto wird gestartet ...','Erfolgreich installiert (Demo)'];
  seq.forEach(function(s){if(used<steps)lines.push(s);else done=false;used++});
 });
 if(done)lines.push('Demo: Es wurde nichts wirklich installiert.');
 return{lines:lines,done:done,ok:done};
}
mock=async function(p,b){
 if(p==='detail'){await sleep(350);return{text:manifest(b.id)}}
 if(p==='install'){var id='demo'+Date.now();JOBS[id]={t:Date.now(),ids:b.ids.slice()};return{job:id}}
 if(p==='job'){return jobState(b.id)}
 return baseMock(p,b);
};
var bn=document.createElement('div');
bn.className='demo-banner';
bn.innerHTML=`<span>Demo · simulierte Daten, es wird nichts installiert</span><a href='https://github.com/erwintwin31-lab/appmenu/releases' target='_blank' rel='noopener'>Download</a>`;
document.body.appendChild(bn);
})();
