/* Local AI preview: no remote inference and no runtime import before consent. */
(() => {
  'use strict';
  const $ = id => document.getElementById(id);
  const MODEL = 'Qwen2.5-0.5B-Instruct-q4f16_1-MLC';
  let worker = null, used = 0, busy = false, ready = false, ended = false, timer = null;
  let history = [], activeAnswer = null;
  const status = text => { $('status').textContent = text; };
  function controls() {
    $('start').disabled = !$('agree').checked || !!worker || ended;
    $('question').disabled = !ready || busy || ended;
    $('send').disabled = !ready || busy || ended || used >= 5;
    $('stop').disabled = !worker;
    $('count').textContent = used + ' / 5 questions used';
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
  $('agree').addEventListener('change', controls);
  $('start').addEventListener('click', async () => {
    if (!$('agree').checked || worker || ended) return;
    $('start').disabled = true; status('Checking WebGPU support…');
    busy = true;
    try {
      if (!window.isSecureContext || !navigator.gpu) throw new Error('WebGPU is unavailable. Use a compatible browser over HTTPS.');
      const adapter = await navigator.gpu.requestAdapter();
      if (ended) return;
      if (!adapter || !adapter.features.has('shader-f16')) throw new Error('This model requires a WebGPU adapter with shader-f16 support.');
      worker = new Worker(new URL('local-ai-worker.js', document.currentScript?.src || new URL('demo/local-ai.js', document.baseURI)), {type:'module'});
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
    if (!q || q.length > 1200 || !worker || !ready || busy || used >= 5 || ended) return;
    used++; busy = true; history.push({role:'user',content:q});
    message('You',q); activeAnswer = message('AI',''); $('question').value = '';
    status('Generating locally…'); controls();
    deadline(120000, 'Generation timed out. AI worker stopped; this attempted question was counted.');
    worker.postMessage({type:'ask',messages:history.slice(-3)});
  });
  $('stop').addEventListener('click', () => stop('AI worker stopped. Reload to start a new session. Cached files may remain.'));
  window.addEventListener('pagehide', () => { if (worker) worker.terminate(); clearTimeout(timer); });
  controls();
})();
