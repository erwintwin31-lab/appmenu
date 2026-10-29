(function () {
  var TI = window.__TAURI_INTERNALS__;
  if (!(TI && TI.invoke)) return;
  document.documentElement.classList.add('app');
  function inv(cmd) {
    try {
      var r = TI.invoke('plugin:window|' + cmd, { label: 'main' });
      if (r && r.catch) r.catch(function () {});
    } catch (e) {}
  }
  document.querySelectorAll('.wc').forEach(function (el) { el.hidden = false; });
  var min = document.getElementById('wmin');
  var max = document.getElementById('wmax');
  var cls = document.getElementById('wcls');
  if (min) min.onclick = function () { inv('minimize'); };
  if (max) max.onclick = function () { inv('toggle_maximize'); };
  if (cls) cls.onclick = function () { inv('close'); };
  var top = document.getElementById('topbar');
  if (top) {
    top.addEventListener('mousedown', function (e) {
      if (e.button !== 0) return;
      if (e.target.closest && e.target.closest('button')) return;
      if (e.detail === 2) inv('toggle_maximize');
      else inv('start_dragging');
    });
  }
})();
