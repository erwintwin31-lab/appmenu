/* Appmenu: eigenständige Dock-Navigation mit automatischem Ausblenden */
(() => {
  const enableDock = () => {
    if (document.body.dataset.mode !== 'dock' && typeof window.edSet === 'function') {
      window.edSet('mode', 'dock');
    }
  };

  const setupAutoHide = () => {
    const win = document.querySelector('.win');
    const nav = document.getElementById('nav');
    if (!win || !nav) return;

    let hideTimer = null;

    const show = () => {
      clearTimeout(hideTimer);
      document.body.classList.remove('dock-hidden');
    };

    const scheduleHide = () => {
      clearTimeout(hideTimer);
      hideTimer = setTimeout(() => document.body.classList.add('dock-hidden'), 900);
    };

    document.addEventListener('mousemove', (event) => {
      const rect = win.getBoundingClientRect();
      const nearBottom = event.clientY >= rect.bottom - 56 && event.clientY <= rect.bottom;
      const overNav = nav.contains(event.target);

      if (nearBottom || overNav) show();
      else scheduleHide();
    });

    nav.addEventListener('mouseenter', show);
    nav.addEventListener('mouseleave', scheduleHide);
    scheduleHide();
  };

  const start = () => {
    enableDock();
    setupAutoHide();
  };

  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start);
  else start();
})();
