/* Shared glass controls; retain native number input semantics. */
(function () {
'use strict';
const style = document.createElement('style');
style.textContent = `
.am-stepper{position:relative;display:block;min-width:0}
.am-stepper>input[type=number]{box-sizing:border-box;width:100%;padding-right:42px;appearance:textfield;-moz-appearance:textfield}
.am-stepper>input::-webkit-inner-spin-button,.am-stepper>input::-webkit-outer-spin-button{-webkit-appearance:none;margin:0}
.am-step-buttons{position:absolute;right:5px;top:5px;bottom:5px;width:30px;display:grid;grid-template-rows:1fr 1fr;gap:2px}
.am-step-buttons>button{display:grid;place-items:center;padding:0;min-height:18px;width:30px;border:1px solid var(--line);border-radius:5px;background:linear-gradient(135deg,#ffffff18,#ffffff05);color:var(--text);cursor:pointer;backdrop-filter:blur(8px)}
.am-step-buttons>button:hover:not(:disabled){background:var(--asoft);border-color:var(--accent)}
.am-step-buttons>button:focus-visible{outline:2px solid var(--accent);outline-offset:2px}
.am-step-buttons>button:active:not(:disabled){background:var(--sel)}
.am-step-buttons>button:disabled{opacity:.35;cursor:not-allowed}
.am-step-buttons svg{width:12px;height:8px;fill:none;stroke:currentColor;stroke-width:2;pointer-events:none}
#settings-panel-language select,#am-language{box-sizing:border-box;min-height:44px;padding:10px 16px;border:1px solid var(--line);border-radius:12px;background:var(--panel);color:var(--text);font:inherit;color-scheme:dark}
body.light #am-language{color-scheme:light}
body[data-surface=black] .am-step-buttons>button{background:var(--panel);backdrop-filter:none}
@media(prefers-reduced-transparency:reduce){.am-step-buttons>button{background:var(--win);backdrop-filter:none}}
`;
document.head.appendChild(style);
function enhance() {
 document.querySelectorAll('#main input[type="number"]').forEach(input => {
  if (input.parentElement.classList.contains('am-stepper')) return;
  const wrapper = document.createElement('span'); wrapper.className = 'am-stepper';
  input.before(wrapper); wrapper.appendChild(input);
  const buttons = document.createElement('span'); buttons.className = 'am-step-buttons';
  wrapper.appendChild(buttons);
  const name = input.getAttribute('aria-label') || input.labels?.[0]?.textContent.trim() || 'Value';
  const pair = [1,-1].map(direction => {
   const button = document.createElement('button'); button.type = 'button';
   button.setAttribute('aria-label', name + (direction === 1 ? ' + ' : ' − ') + (input.step || '1'));
   button.innerHTML = '<svg viewBox="0 0 12 8" aria-hidden="true"><path d="' + (direction === 1 ? 'M2 6L6 2L10 6' : 'M2 2L6 6L10 2') + '"/></svg>';
   button.addEventListener('click', () => {
    if (input.disabled || input.readOnly) return;
    const old = input.value;
    try { direction === 1 ? input.stepUp() : input.stepDown(); } catch (_) { return; }
    if (input.value !== old) {
     input.dispatchEvent(new Event('input', {bubbles:true}));
     input.dispatchEvent(new Event('change', {bubbles:true}));
    }
    sync();
   });
   buttons.appendChild(button); return button;
  });
  function sync() {
   const value = input.valueAsNumber;
   pair[0].disabled = input.disabled || input.readOnly || (input.max !== '' && value >= Number(input.max));
   pair[1].disabled = input.disabled || input.readOnly || (input.min !== '' && value <= Number(input.min));
  }
  input.addEventListener('input', sync); input.addEventListener('change', sync); sync();
 });
}
const main = document.getElementById('main');
if (main) { new MutationObserver(enhance).observe(main, {childList:true,subtree:true}); enhance(); }
})();
