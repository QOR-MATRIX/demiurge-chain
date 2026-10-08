/**
 * QFX backdrop public surface.
 *
 * Re-exports the backdrop selector, preset list and apply helper from Canvas
 * so consumers can import from a single, stable path without pulling in React.
 *
 * Usage:
 *   import { applyBackdrop, currentBackdrop, BACKDROP_PRESETS } from './backdrops';
 */

export type { BackdropId, BackdropPreset } from './Canvas';
export { applyBackdrop, currentBackdrop, BACKDROP_PRESETS } from './Canvas';
