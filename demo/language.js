(function(root){
'use strict';
function enhance(){
  const main=document.getElementById('main');
  if(!main||!root.appmenuLanguageView)return;
  const heading=main.querySelector('h1');
  if(!heading||heading.textContent.trim()!=='Einstellungen')return;
  if(document.getElementById('am-language'))return;
  const section=document.createElement('div');
  section.innerHTML='<h2>Sprache</h2>'+root.appmenuLanguageView();
  const limit=[...main.querySelectorAll('h2')].find(h=>h.textContent.trim()==='Programmlimit');
  if(limit&&limit.parentElement===main)main.insertBefore(section,limit);
  else main.appendChild(section);
}
function start(){
  const main=document.getElementById('main');
  if(!main)return;
  new MutationObserver(enhance).observe(main,{childList:true});
  enhance();
}
if(root.AppmenuI18n)start();
else{let n=0;const wait=()=>{if(root.AppmenuI18n||n++>100)start();else setTimeout(wait,20)};wait()}
})(globalThis);
