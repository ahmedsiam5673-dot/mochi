/* Small decorative cap overlay for the user's orange-and-white mascot theme.
   It does not replace or intercept the app's canvas, hit areas, or agent logic. */
function addPeachyCap() {
  const canvas = document.querySelector<HTMLCanvasElement>('#bot-canvas');
  if (!canvas || !canvas.parentElement) return;
  const parent = canvas.parentElement as HTMLElement;
  let cap = parent.querySelector<HTMLElement>(':scope > .peachy-cap-overlay');
  if (!cap) {
    cap = document.createElement('div');
    cap.className = 'peachy-cap-overlay';
    cap.setAttribute('aria-hidden', 'true');
    parent.appendChild(cap);
  }
  const left = canvas.offsetLeft;
  const top = canvas.offsetTop;
  const width = canvas.clientWidth || canvas.width;
  const height = canvas.clientHeight || canvas.height;
  cap.style.left = `${left + width * 0.235}px`;
  cap.style.top = `${top + height * 0.205}px`;
  cap.style.width = `${width * 0.53}px`;
  cap.style.height = `${height * 0.245}px`;
  cap.style.zIndex = '3';
}
function bootPeachyBranding() {
  const run = () => requestAnimationFrame(addPeachyCap);
  run();
  const root = document.getElementById('root');
  if (root) new MutationObserver(run).observe(root, { childList: true, subtree: true });
  const canvas = document.querySelector<HTMLCanvasElement>('#bot-canvas');
  if (canvas && typeof ResizeObserver !== 'undefined') new ResizeObserver(run).observe(canvas);
  window.addEventListener('resize', run);
}
if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', bootPeachyBranding, { once: true });
else bootPeachyBranding();
