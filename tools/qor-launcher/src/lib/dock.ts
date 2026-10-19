/**
 * Docking a product's window over part of the launcher's (ADR-094 decision 1, ADR-102 decision 1).
 *
 * QOR Media over the Library and ARQADE over its screen are Qt windows of their own, made children of the launcher's
 * window by the host. This side measures where they sit, in physical pixels of the window, and tells the host whenever
 * that changes; it sends the launcher's colours, and again when the theme changes; and it hides the product when the
 * screen it sat over goes.
 */

/** What a docked product is told, through the host. */
export interface Docked {
  place: (rect: { x: number; y: number; w: number; h: number }) => Promise<void>;
  hide: () => Promise<void>;
  theme: (colors: Record<string, string>) => Promise<void>;
}

const THEME_TOKENS = ['void', 'base', 'surface', 'raised', 'well', 'edge', 'accent', 'accent-bright', 'accent-dim', 'ink', 'ink-body', 'ink-muted', 'ink-faint'];

/** The launcher's theme as `#rrggbb` tokens, read from the page, for a docked product to wear. */
export function themeTokens(): Record<string, string> {
  const style = getComputedStyle(document.documentElement);
  return Object.fromEntries(
    THEME_TOKENS.map((t): [string, string] => [t, style.getPropertyValue(`--${t}`).trim().toLowerCase()]).filter(([, v]) => /^#[0-9a-f]{6}$/.test(v)),
  );
}

/** Keep `to`'s place in step with `el` while it shows, and hide it when it goes. Returns how to stop. */
export function dockOver(el: HTMLElement, to: Docked): () => void {
  let last = '';
  const send = () => {
    const r = el.getBoundingClientRect();
    const d = window.devicePixelRatio || 1;
    const rect = { x: Math.round(r.left * d), y: Math.round(r.top * d), w: Math.round(r.width * d), h: Math.round(r.height * d) };
    const key = JSON.stringify(rect);
    if (key === last || rect.w === 0 || rect.h === 0) return;
    last = key;
    void to.place(rect).catch(() => undefined);
  };
  const observer = new ResizeObserver(send);
  observer.observe(el);
  window.addEventListener('resize', send);
  send();
  void to.theme(themeTokens()).catch(() => undefined);
  // A theme change restyles the page's root: the product follows it.
  const themed = new MutationObserver(() => void to.theme(themeTokens()).catch(() => undefined));
  themed.observe(document.documentElement, { attributes: true, attributeFilter: ['style', 'class', 'data-theme'] });
  return () => {
    observer.disconnect();
    themed.disconnect();
    window.removeEventListener('resize', send);
    void to.hide().catch(() => undefined);
  };
}
