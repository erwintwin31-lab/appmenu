/* Open the native CLI in a separate terminal. Never interpret user commands. */
(function () {
  'use strict';
  const button = document.getElementById('appmenu-cli-launch');
  const status = document.getElementById('appmenu-cli-status');
  if (!button || !status) return;
  const native = window.__TAURI_INTERNALS__;
  let available = false;
  let busy = false;
  let reason = 'Terminalverfügbarkeit wird geprüft.';
  function describe(text, reveal) {
    reason = String(text);
    button.title = reason;
    status.textContent = reason;
    status.hidden = !reveal;
  }
  button.addEventListener('click', async function () {
    if (busy) return;
    if (!available) { describe(reason, true); return; }
    busy = true;
    button.disabled = true;
    button.setAttribute('aria-busy', 'true');
    try {
      const result = await native.invoke('open_cli_terminal');
      if (!result || result.started !== true) throw new Error('Terminalstart nicht bestätigt.');
      describe(result.message || 'Terminalstarter gestartet.', true);
    } catch (error) {
      describe(error && error.message ? error.message : String(error), true);
    } finally {
      busy = false;
      button.disabled = false;
      button.setAttribute('aria-busy', 'false');
    }
  });
  document.addEventListener('keydown', function (event) {
    if (event.key === 'Escape') status.hidden = true;
  });
  window.AppmenuTerminalReady = (async function () {
    if (!native || typeof native.invoke !== 'function') {
      describe('Browser-Vorschau: kein nativer Terminalstart verfügbar.', false);
      button.setAttribute('aria-disabled', 'true');
      return;
    }
    try {
      const result = await native.invoke('cli_terminal_status');
      available = Boolean(result && result.available === true);
      describe(result && result.reason ? result.reason : 'Terminalstart nicht verfügbar.', false);
      button.setAttribute('aria-disabled', String(!available));
      const label = document.querySelector('.top .sub');
      if (label && result && ['linux', 'windows', 'macos'].includes(result.platform)) {
        label.textContent = {linux:'Linux', windows:'Windows', macos:'macOS'}[result.platform];
      }
    } catch (error) {
      button.setAttribute('aria-disabled', 'true');
      describe(error && error.message ? error.message : String(error), false);
    }
  })();
})();
