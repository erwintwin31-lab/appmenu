/* Pure locale state; UI wiring is deliberately separate. */
(function(root){
  'use strict';
  function createI18n(catalogs, storage) {
    const KEY = 'am_language_v1', listeners = new Set();
    const own = (object,key) => Object.prototype.hasOwnProperty.call(object,key);
    const valid = value => typeof value === 'string' && own(catalogs,value);
    let locale = 'en';
    try { const saved = storage?.getItem(KEY); if (valid(saved)) locale = saved; } catch (_) {}
    function t(key,params = {}) {
      const selected = catalogs[locale].messages, english = catalogs.en.messages;
      const text = own(selected,key) ? selected[key] : own(english,key) ? english[key] : key;
      return text.replace(/\{([a-zA-Z0-9_]+)\}/g,(match,name) => own(params,name) ? String(params[name]) : match);
    }
    function setLocale(value) {
      if (!valid(value)) return false;
      if (locale === value) return true;
      locale = value;
      try { storage?.setItem(KEY,value); } catch (_) {}
      for (const listener of listeners) listener(locale);
      return true;
    }
    return Object.freeze({t,setLocale,getLocale:() => locale,
      languages:() => Object.entries(catalogs).map(([code,c]) => ({code,name:c.name,machineTranslated:c.machineTranslated})),
      notices:() => locale === 'en' ? [{lang:'en',text:catalogs.en.messages['language.notice']}] : [{lang:'en',text:catalogs.en.messages['language.notice']},{lang:locale,text:t('language.notice')}],
      subscribe:listener => { listeners.add(listener); return () => listeners.delete(listener); }
    });
  }
  if (typeof module === 'object' && module.exports) module.exports = {createI18n};
  else root.createAppmenuI18n = createI18n;
})(globalThis);
