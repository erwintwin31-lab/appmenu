const test = require('node:test');
const assert = require('node:assert/strict');
const {languageView} = require('../src/i18n/language-view.js');

test('language view selects the active language and escapes notices', () => {
  const html = languageView({
    t: key => '<' + key + '>',
    getLocale: () => 'de',
    languages: () => [{code:'en',name:'English'},{code:'de',name:'Deutsch'}],
    notices: () => [{lang:'en',text:'<script>bad</script>'},{lang:'de',text:'Hinweis'}]
  });
  assert.match(html, /value="de" selected/);
  assert.match(html, /lang="en"/);
  assert.match(html, /lang="de"/);
  assert.doesNotMatch(html, /<script>/);
  assert.match(html, /&lt;script&gt;/);
});
