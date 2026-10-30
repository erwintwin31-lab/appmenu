/* Appmenu: erweiterbarer, simulierter KI-Chat */
(() => {
  const MAIN = document.getElementById('main');
  if (!MAIN) return;

  let lastQuestion = '';
  let chat = null;

  const isAiOverview = (node) => {
    if (!(node instanceof HTMLElement)) return false;
    const text = (node.textContent || '').toLowerCase();
    const cls = node.className || '';
    return cls.includes('ai') || cls.includes('overview') || text.includes('ki-übersicht') || text.includes('ki übersicht');
  };

  const addMoreButton = (container) => {
    if (container.querySelector('.frost-more')) return;
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'frost-more';
    button.textContent = 'Mehr dazu';
    button.addEventListener('click', () => openChat());
    container.appendChild(button);
  };

  const scanForOverview = () => {
    MAIN.querySelectorAll('*').forEach((node) => {
      if (isAiOverview(node) && !node.querySelector('.frost-more')) addMoreButton(node);
    });
  };

  const buildChat = () => {
    if (chat) return chat;
    chat = document.createElement('section');
    chat.className = 'frost-chat';
    chat.setAttribute('aria-label', 'Erweiterter KI-Chat');
    chat.innerHTML = `
      <div class="frost-chat-top">
        <button class="frost-nav" data-action="back" title="Zurück zur Suche" aria-label="Zurück zur Suche">←</button>
        <button class="frost-nav" data-action="forward" title="Chat erneut öffnen" aria-label="Chat erneut öffnen" disabled>→</button>
        <div class="frost-chat-logo">appmenu</div>
      </div>
      <div class="frost-chat-log" aria-live="polite"></div>
      <form class="frost-chat-form">
        <input class="frost-chat-input" type="text" placeholder="Anschlussfrage stellen …" aria-label="Anschlussfrage" />
        <button class="frost-chat-send" type="submit">Senden</button>
      </form>
      <p class="frost-chat-note">Simulierte Demo-Oberfläche – keine echte KI-Modellanbindung.</p>
    `;

    const log = chat.querySelector('.frost-chat-log');
    const form = chat.querySelector('.frost-chat-form');
    const input = chat.querySelector('.frost-chat-input');
    const back = chat.querySelector('[data-action="back"]');
    const forward = chat.querySelector('[data-action="forward"]');

    const message = (text, kind = 'ai') => {
      const item = document.createElement('div');
      item.className = `frost-msg ${kind}`;
      item.textContent = text;
      log.appendChild(item);
      log.scrollTop = log.scrollHeight;
    };

    back.addEventListener('click', () => {
      chat.classList.remove('open');
      document.body.classList.remove('frost-chat-open');
      forward.disabled = false;
    });

    forward.addEventListener('click', () => openChat());

    form.addEventListener('submit', (event) => {
      event.preventDefault();
      const value = input.value.trim();
      if (!value) return;
      message(value, 'user');
      input.value = '';
      setTimeout(() => message('Diese Web-Demo kann Anschlussfragen nur simulieren. In der späteren Appmenu-Version würde hier die passende KI-Antwort erscheinen.'), 350);
    });

    document.body.appendChild(chat);
    return chat;
  };

  const openChat = () => {
    const panel = buildChat();
    const log = panel.querySelector('.frost-chat-log');
    if (!log.children.length && lastQuestion) {
      const intro = document.createElement('div');
      intro.className = 'frost-msg user';
      intro.textContent = lastQuestion;
      log.appendChild(intro);
      const answer = document.createElement('div');
      answer.className = 'frost-msg';
      answer.textContent = 'Hier kannst du die KI-Übersicht vertiefen. Diese Demo simuliert die weiteren Antworten.';
      log.appendChild(answer);
    }
    panel.classList.add('open');
    document.body.classList.add('frost-chat-open');
    panel.querySelector('.frost-chat-input').focus();
  };

  const captureQuestion = () => {
    const field = document.querySelector('#q, input[type="search"], input[name="q"]');
    if (field && field.value.trim()) lastQuestion = field.value.trim();
  };

  const observer = new MutationObserver(scanForOverview);
  observer.observe(MAIN, { childList: true, subtree: true });
  document.addEventListener('input', captureQuestion, true);
  document.addEventListener('submit', captureQuestion, true);
  scanForOverview();
})();
