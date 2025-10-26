async function injectNav() {
  const placeholder = document.getElementById('sidenav');
  if (!placeholder) return;
  try {
    const res = await fetch('_nav.html');
    if (!res.ok) return;
    const html = await res.text();
    placeholder.innerHTML = html;
  } catch (e) {
    console.error('Nav include failed', e);
  }
}
window.addEventListener('DOMContentLoaded', injectNav);
