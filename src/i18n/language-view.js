(function(root){
'use strict';
function languageView(i18n){
 const e=value=>String(value).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 return '<label for="am-language">'+e(i18n.t('language.label'))+'</label><select id="am-language" onchange="appmenuSetLanguage(this.value)">'+i18n.languages().map(l=>'<option value="'+e(l.code)+'"'+(l.code===i18n.getLocale()?' selected':'')+'>'+e(l.name)+'</option>').join('')+'</select><p>'+e(i18n.t('language.original'))+'</p>'+i18n.notices().map(n=>'<p class="info" lang="'+e(n.lang)+'">'+e(n.text)+'</p>').join('');
}
if(typeof module==='object'&&module.exports)module.exports={languageView};
else{
 root.appmenuLanguageView=()=>languageView(root.AppmenuI18n);
 root.appmenuSetLanguage=value=>{
  if(root.AppmenuI18n.setLocale(value)){
   draw();
   document.getElementById('am-language')?.focus();
  }
 };
}
})(globalThis);
