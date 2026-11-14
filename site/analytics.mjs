// SPDX-License-Identifier: Apache-2.0
import { inject } from './vendor/vercel-analytics.mjs';

const origin = 'https://qorsync.dev';

// This is a single public page. Never forward paths, queries, fragments or
// custom events supplied by a visitor or a future authenticated surface.
export function beforeSend(event) {
  if (event?.type !== 'pageview') return null;
  try {
    const url = new URL(event.url);
    if (url.origin !== origin || url.pathname !== '/' || url.username || url.password) return null;
    return { type: 'pageview', url: `${origin}/` };
  } catch {
    return null;
  }
}

if (typeof window !== 'undefined'
    && window.location.origin === origin
    && window.location.pathname === '/'
    && window.navigator.doNotTrack !== '1'
    && window.navigator.doNotTrack !== 'yes'
    && !window.navigator.globalPrivacyControl) {
  inject({
    mode: 'production',
    debug: false,
    scriptSrc: '/_vercel/insights/script.js',
    endpoint: '/_vercel/insights',
    beforeSend,
  });
}
