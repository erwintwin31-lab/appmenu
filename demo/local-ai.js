/* Local AI preview: no runtime/model request before an explicit mode choice. */
(() => {
  'use strict';
  const $ = id => document.getElementById(id);
  const MODEL = 'Qwen2.5-0.5B-Instruct-q4f16_1-MLC';
  let mode = null, worker = null, used = 0, busy = false, ready = false, ended = false, timer = null;
  let history = [], activeAnswer = null;
  const status = text => { $('status').textContent = text; };
  const show = (id, on) => { $(id).hidden = !on; };
  function controls() {
    $('start').disabled = !$('agree').checked || !!worker || ended;
    $('question').disabled = busy || ended;
    $('send').disabled = busy || ended || (mode === 'real' && used >= 5);
    $('stop').hidden = mode !== 'real' || !worker;
    $('count').hidden = mode !== 'real';
    if (mode === 'real') $('count').textContent = used + ' / 5 questions used';
  }
  function stop(message) {
    clearTimeout(timer); timer = null;
    if (worker) worker.terminate();
    worker = null; ready = false; busy = false; ended = true;
    history = []; activeAnswer = null;
    status(message); controls();
  }
  function deadline(ms, message) {
    clearTimeout(timer); timer = setTimeout(() => stop(message), ms);
  }
  function message(role, text) {
    const p = document.createElement('p');
    p.textContent = role + ': ' + text; $('log').appendChild(p); return p;
  }
  function openPreview() {
    mode = 'preview'; ended = false; busy = false; ready = true;
    show('choose', false); show('consent', false); show('chat', true);
    $('modeBadge').textContent = 'Preview · simulated answers';
    $('modeBadge').className = 'badge preview';
    status('Preview mode is ready. No AI runtime or model has been requested.');
    controls();
  }
  function openReal() {
    mode = 'real'; show('choose', false); show('consent', true); show('chat', false);
  }
  function back() {
    if (worker) worker.terminate();
    clearTimeout(timer); timer = null;
    worker = null; mode = null; ready = false; busy = false; ended = false;
    history = []; activeAnswer = null; used = 0;
    show('choose', true); show('consent', false); show('chat', false);
    $('agree').checked = false; controls();
  }
  function fakeAnswer(q) {
    const l = q.toLowerCase();
    if (/install|winget|admin|permission/.test(l)) return 'Appmenu uses winget with --scope user in the desktop app, so installations stay in your user profile where possible. It does not bypass device policies. This preview cannot install anything.';
    if (/file|hash|sha|check/.test(l)) return 'In the desktop app, Local Files can inspect an EXE or MSI and calculate a SHA-256 checksum. This browser preview cannot access or inspect files on your device.';
    if (/theme|layout|color|dock/.test(l)) return 'Appmenu offers Glass, Black, Midnight and Frosted Blue surfaces, plus Hub, Normal and Compact layouts. You can change the accent color and dock behavior in Settings.';
    if (/ai|model|language/.test(l)) return 'This is a prepared preview answer, not a running AI model. The real local mode downloads and runs a small model on your device after consent.';
    return 'This preview uses prepared demonstration answers. In the real local mode, a small model would answer on your device, but its response could still be inaccurate.';
  }
  $('preview').addEventListener('click', openPreview);
  $('real').addEventListener('click', openReal);
  $('back').addEventListener('click', back);
  $('agree').addEventListener('change', controls);
  $('start').addEventListener('click', async () => {
    if (!$('agree').checked || worker || ended) return;
    $('start').disabled = true; status('Checking WebGPU support…'); busy = true;
    try {
      if (!window.isSecureContext || !navigator.gpu) throw new Error('WebGPU is unavailable. Use a compatible browser over HTTPS, or use Preview AI.');
      const adapter = await navigator.gpu.requestAdapter();
      if (ended) return;
      if (!adapter || !adapter.features.has('shader-f16')) throw new Error('This model requires a WebGPU adapter with shader-f16 support. Use Preview AI instead.');
      worker = new Worker(new URL('local-ai-worker.js', document.currentScript?.src || new URL('demo/local-ai.js', document.baseURI)), {type:'module'});
      show('consent', false); show('chat', true);
      $('modeBadge').textContent = 'Real local AI'; $('modeBadge').className = 'badge real';
      controls(); status('Downloading and compiling model. First use may take several minutes…');
      deadline(600000, 'Loading timed out. Worker stopped. Reload to try again.');
      const current = worker;
      current.onerror = event => { event.preventDefault(); if (worker === current) stop('AI worker failed. No cloud fallback was used. Reload to retry.'); };
      current.onmessage = ({data}) => {
        if (worker !== current) return;
        if (data.type === 'progress') status(String(data.text));
        if (data.type === 'ready') {
          clearTimeout(timer); ready = true; busy = false;
          status('Local AI ready. No live web access.'); controls();
          deadline(180000, 'Idle timeout. AI worker stopped; cached model files may remain.');
        }
        if (data.type === 'token' && activeAnswer) activeAnswer.textContent += String(data.text);
        if (data.type === 'done') {
          clearTimeout(timer);
          history.push({role:'assistant',content:String(data.text).slice(0,4000)});
          if (activeAnswer) activeAnswer.textContent = 'AI: ' + (data.text || '(No answer generated.)');
          activeAnswer = null; busy = false;
          if (used >= 5) stop('Five-question limit reached. AI worker stopped; downloaded files may remain cached.');
          else { status('Answer generated locally. It may be inaccurate.'); controls(); deadline(180000, 'Idle timeout. AI worker stopped; cached model files may remain.'); }
        }
        if (data.type === 'error') stop('Local AI failed: ' + String(data.message).slice(0,500) + ' No cloud fallback was used.');
      };
      current.postMessage({type:'load',model:MODEL});
    } catch (error) { stop(String(error.message || error) + ' Reload to retry.'); }
  });
  $('form').addEventListener('submit', event => {
    event.preventDefault();
    const q = $('question').value.trim();
    if (!q || q.length > 1200 || busy || ended || (mode === 'real' && (!worker || !ready || used >= 5))) return;
    if (mode === 'real') used++;
    busy = true; history.push({role:'user',content:q});
    message('You',q); activeAnswer = message('AI',''); $('question').value = '';
    if (mode === 'preview') {
      status('Showing a prepared preview answer. No model is running.');
      setTimeout(() => {
        const answer = fakeAnswer(q);
        history.push({role:'assistant',content:answer});
        if (activeAnswer) activeAnswer.textContent = 'AI: ' + answer;
        activeAnswer = null; busy = false; controls();
      }, 500);
    } else {
      status('Generating locally…'); controls();
      deadline(120000, 'Generation timed out. AI worker stopped; this attempted question was counted.');
      worker.postMessage({type:'ask',messages:history.slice(-3)});
    }
  });
  $('stop').addEventListener('click', () => stop('AI worker stopped. Reload to start a new session. Cached files may remain.'));
  window.addEventListener('pagehide', () => { if (worker) worker.terminate(); clearTimeout(timer); });
  controls();
})();
