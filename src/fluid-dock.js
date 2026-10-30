/* Appmenu · Fluid Glass Dock */
(() => {
  'use strict';

  const setupDock = () => {
    const win = document.querySelector('.win');
    const nav = document.getElementById('nav');
    if (!win || !nav) return;

    let hideTimer = null;

    const isDockMode = () => document.body.dataset.mode === 'dock';

    const show = () => {
      if (!isDockMode()) return;
      clearTimeout(hideTimer);
      document.body.classList.remove('dock-hidden');
    };

    const scheduleHide = () => {
      if (!isDockMode()) {
        document.body.classList.remove('dock-hidden');
        return;
      }
      clearTimeout(hideTimer);
      hideTimer = setTimeout(() => {
        if (isDockMode()) document.body.classList.add('dock-hidden');
      }, 900);
    };

    document.addEventListener('mousemove', (event) => {
      if (!isDockMode()) {
        document.body.classList.remove('dock-hidden');
        return;
      }
      const rect = win.getBoundingClientRect();
      const nearBottom = event.clientY >= rect.bottom - 64 && event.clientY <= rect.bottom;
      const overNav = nav.contains(event.target);
      if (nearBottom || overNav) show();
      else scheduleHide();
    });

    nav.addEventListener('mouseenter', show);
    nav.addEventListener('mouseleave', scheduleHide);
    scheduleHide();
  };

  const start = () => setupDock();
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start);
  else start();
})();
