import React from 'react';
import { createRoot } from 'react-dom/client';

// The display face, bundled so it renders offline and on first run (DESIGN_SYSTEM.md §3).
import '@fontsource/barlow-semi-condensed/500.css';
import '@fontsource/barlow-semi-condensed/600.css';
import '@fontsource/barlow-semi-condensed/700.css';
import './styles/qor.css';
import { App } from './App';
import { Boundary } from './components/chrome/Boundary';
import { applyA11y, loadA11y } from './lib/a11y';
import { applyTheme, storedTheme } from './styles/themes';

// The stored theme and accessibility settings, on the document before anything renders. The QFX canvas reads
// `data-ambience` when it mounts: without this it saw no attribute, took the default (live), built a GL context and ran
// a frame or two even for a launcher whose ambience is stored as off. The store applies them again on bootstrap.
applyTheme(storedTheme());
applyA11y(loadA11y());

const host = document.getElementById('qor-root');
if (!host) throw new Error('#qor-root is missing from index.html');

createRoot(host).render(
  <React.StrictMode>
    <Boundary>
      <App />
    </Boundary>
  </React.StrictMode>,
);
