/* Web demo only: keep simulated search and real inference visibly separate. */
(() => {
  'use strict';
  if (typeof MOCK === 'undefined' || !MOCK) return;
  let dialog = null, opener = null;
  function close() {
    if (!dialog) return;
    const frame = dialog.querySelector('iframe');
    if (frame) { frame.src = 'about:blank'; frame.remove(); }
    dialog.close(); dialog.remove(); dialog = null;
    if (opener && opener.isConnected) opener.focus();
  }
  window.demoAIReal = function () {
    if (dialog) return;
    opener = document.activeElement;
    dialog = document.createElement('dialog');
    dialog.setAttribute('aria-label', 'Real local AI: consent and five-question trial');
    dialog.style.cssText = 'width:min(900px,94vw);height:88vh;max-height:94vh;padding:16px;border:1px solid #536580;border-radius:16px;background:#101827;color:#edf2ff';
    const button = document.createElement('button');
    button.type = 'button'; button.className = 'btn';
    button.textContent = 'Close local AI and return to simulated preview';
    button.addEventListener('click',close);
    const frame = document.createElement('iframe');
    frame.title = 'Local AI trial: explicit consent required before model download';
    frame.src = 'local-ai.html';
    frame.style.cssText = 'display:block;width:100%;height:calc(100% - 58px);border:0;margin-top:12px';
    dialog.append(button,frame); document.body.appendChild(dialog);
    dialog.addEventListener('cancel', event => { event.preventDefault(); close(); });
    dialog.showModal(); button.focus();
  };
  window.demoAIPreview = function () {
    close();
    go('search');
    document.getElementById('qi')?.focus();
  };
  const oldSearchView = V.search;
  V.search = function () {
    const template = document.createElement('template');
    template.innerHTML = oldSearchView();
    template.content.querySelectorAll('.glass-search-tools select, .glass-search-tools small').forEach(el => el.remove());
    return `<section class="info" aria-label="AI demo modes"><h2 style="margin-top:0">Choose an AI demo mode</h2><p><button class="btn g2" onclick="demoAIPreview()" aria-pressed="true">Preview AI · simulated</button> <button class="btn" onclick="demoAIReal()">Real local AI · optional download</button></p><p role="note">Current search: prepared, simulated answers — no AI model is running. No model download or WebGPU is needed for this preview. Local theme commands are rule-based, not AI-generated.</p><p>Real local AI requires compatible WebGPU, explicit consent and a model download. It uses device memory and GPU resources and allows five attempted questions. Opening the consent screen alone does not download a model. Neither mode has live web search; answers are not installation approval.</p></section>` + template.innerHTML;
  };
  function route() { if (location.hash === '#search') go('search'); }
  window.addEventListener('hashchange',route);
  window.addEventListener('pagehide',close);
  route(); draw();
})();
