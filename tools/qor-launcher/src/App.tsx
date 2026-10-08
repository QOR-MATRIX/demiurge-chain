/**
 * The shell.
 *
 * Two states, and the boundary between them is the whole security model of the
 * launcher: either the Gate is showing, or the shell is. The shell renders once
 * the vault is open. A QOR ID session is not required (ADR-056): a surface that
 * needs one reads `session` and says so when it is absent.
 */

import { useEffect } from 'react';
import { AnimatePresence, MotionConfig, motion } from 'framer-motion';
import { getCurrentWindow } from '@tauri-apps/api/window';

import { Gate } from './components/gate/Gate';
import { Intro } from './qfx/ceremony/Intro';
import { Onboarding } from './components/onboarding/Onboarding';
import { Rail } from './components/chrome/Rail';
import { TitleBar } from './components/chrome/TitleBar';
import type { Motion } from './lib/a11y';
import { selectVaultUnlocked, useQor } from './state/store';
import { QQView } from './qq/editor/QQView';
import { ArqadeView } from './views/ArqadeView';
import { ChainView } from './views/ChainView';
import { GatesView } from './views/GatesView';
import { Horizon } from './views/Horizon';
import { Inventory } from './views/Inventory';
import { Market } from './views/Market';
import { Projects } from './views/Projects';
import { QfxCanvas } from './qfx/Canvas';
import { Nexus } from './views/Nexus';
import { SettingsView } from './views/SettingsView';
import { VaultView } from './views/VaultView';

/** Spoken name for each surface, used as the main landmark's label. */
const SURFACE_LABELS: Record<string, string> = {
  nexus: 'Nexus, all systems',
  vault: 'CGT Vault',
  inventory: 'Inventory and history',
  market: 'Market, what is for sale on chain',
  qq: 'QQ, the QOR Engine',
  arqade: 'ARQADE, games and collectibles',
  library: 'Library',
  social: 'Social',
  mesh: 'Mesh',
  chain: 'Chain',
  gates: 'Release gates',
  settings: 'Settings',
};

/**
 * The in-app Motion setting, as framer-motion understands it.
 *
 * framer-motion animates from JavaScript, so the stylesheet's reduced-motion
 * rules never reach it. Without this it followed only the operating system, and
 * the setting did nothing to page transitions or toasts (roadmap L1.2).
 */
const REDUCED_MOTION: Record<Motion, 'user' | 'always' | 'never'> = {
  system: 'user',
  full: 'never',
  reduced: 'always',
};

export function App() {
  const ready = useQor((s) => s.ready);
  const bootstrap = useQor((s) => s.bootstrap);
  const unlocked = useQor(selectVaultUnlocked);
  const surface = useQor((s) => s.surface);
  const notice = useQor((s) => s.notice);
  const clearNotice = useQor((s) => s.clearNotice);
  const motionSetting = useQor((s) => s.a11y.motion);

  useEffect(() => {
    void bootstrap();
  }, [bootstrap]);

  // The window is created hidden so the first frame the user sees is painted,
  // not a white flash followed by the app. Show it once state has resolved.
  useEffect(() => {
    if (ready) void getCurrentWindow().show();
  }, [ready]);

  // Auto-dismiss the notice. Errors stay long enough to read; successes do not
  // need to linger.
  useEffect(() => {
    if (!notice) return;
    const id = setTimeout(clearNotice, notice.tone === 'bad' ? 8000 : 4000);
    return () => clearTimeout(id);
  }, [notice, clearNotice]);

  // Inside once the vault is open. QOR ID is not a condition: it signs in
  // when it can, and Settings says when it could not (ADR-056).
  const inside = ready && unlocked;

  return (
    <MotionConfig reducedMotion={REDUCED_MOTION[motionSetting]}>
      {/* QFX layer one, and the chrome's guarantee over it. Both are fixed and
          behind everything; neither takes a click. The scrim is the chrome's,
          not the theme's — see src/qfx/contrast.ts. The interface is lifted
          above both by `.qfx-interface`, and has no background of its own: the
          page's base shows when the backdrop is off, and the scrim's when it is
          on. */}
      <QfxCanvas />
      <div className="qfx-scrim" aria-hidden="true" />
      <div className="qfx-interface flex h-full flex-col">
        {/* The first tab stop, so a keyboard user is not walked through the whole
            navigation rail before reaching content. */}
        <a href="#qor-main" className="skip-link no-drag">
          Skip to content
        </a>

        <TitleBar />

        <div className="relative flex min-h-0 flex-1">
          {inside ? (
            <>
              <Rail />
              <main id="qor-main" tabIndex={-1} aria-label={SURFACE_LABELS[surface]} className="min-w-0 flex-1">
                <AnimatePresence mode="wait">
                  <motion.div
                    key={surface}
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    transition={{ duration: 0.16 }}
                    className="h-full"
                  >
                    {surface === 'nexus' && <Nexus />}
                    {surface === 'vault' && <VaultView />}
                    {surface === 'inventory' && <Inventory />}
                    {surface === 'market' && <Market />}
                    {surface === 'projects' && <Projects />}
                    {surface === 'qq' && <QQView />}
                    {surface === 'arqade' && <ArqadeView />}
                    {surface === 'chain' && <ChainView />}
                    {surface === 'gates' && <GatesView />}
                    {surface === 'settings' && <SettingsView />}
                    {(surface === 'library' || surface === 'social' || surface === 'mesh') && (
                      <Horizon surface={surface} />
                    )}
                  </motion.div>
                </AnimatePresence>
              </main>
              <Onboarding />
            </>
          ) : (
            <Gate />
          )}

          <AnimatePresence>
            {notice && (
              <motion.div
                initial={{ opacity: 0, y: 12 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: 12 }}
                transition={{ duration: 0.22, ease: [0.16, 1, 0.3, 1] }}
                aria-hidden="true"
                className={`glass-solid absolute bottom-6 left-1/2 z-50 max-w-lg -translate-x-1/2 px-5 py-3 text-ui ${
                  notice.tone === 'ok'
                    ? 'border-l-2 border-l-ok text-ink'
                    : 'border-l-2 border-l-bad text-bad'
                }`}
              >
                {notice.text}
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
      <Intro />
    </MotionConfig>
  );
}
