/**
 * QQ's engine: a scene, a canvas, and the two modes (ADR-082, DIRECTION P3.1).
 *
 * In edit mode the engine draws the scene as the editor holds it, one frame per
 * change and nothing in between, and turns pointer gestures on the viewport into
 * selections and moves, which it reports through the `EditorBridge` (ADR-082
 * decision 3: same process, no IPC). The editor owns the scene; the engine never
 * changes it.
 *
 * In play mode it builds a `World` from the scene and runs it on
 * `requestAnimationFrame`, with the pointer as input. Stop drops the world and
 * the editor's scene is drawn again, exactly as it was.
 *
 * Reduced motion holds play to one still frame: nothing animates, and the
 * editor says so. "Step" advances a held world by a fixed slice, so a creator
 * who asked for less motion can still see what a scene does, at their own pace.
 */

import { KIND, Renderer, fit, type Fit, type Frame, type Instance } from './renderer';
import type { Entity, Scene } from './scene';
import { World, channels, type Body } from './world';

export type Mode = 'edit' | 'play';

/** What the engine tells the editor. The editor decides what to do with it. */
export interface EditorBridge {
  /** A click in edit mode chose this entity, or nothing. */
  select(id: string | null): void;
  /** A drag in edit mode moved this entity here, in scene units. */
  move(id: string, x: number, y: number): void;
  /** While playing: how many particles are live and the frames per second, about four times a second. */
  stats?(particles: number, fps: number): void;
}

/** The size an entity with no shape is drawn as in the editor, so it can still be seen and picked. */
const MARKER = 28;
/** One step of a held world (reduced motion). */
export const STEP_SECONDS = 1 / 4;

function bounds(e: { transform: Entity['transform']; shape?: Entity['shape'] }): { w: number; h: number } {
  const s = e.transform.scale;
  return e.shape ? { w: e.shape.w * s, h: e.shape.h * s } : { w: MARKER, h: MARKER };
}

/** The topmost entity under a point, in scene units. Later entities are drawn over earlier ones. */
export function pick(scene: Scene, x: number, y: number): string | null {
  for (let i = scene.entities.length - 1; i >= 0; i--) {
    const e = scene.entities[i]!;
    const { w, h } = bounds(e);
    // Into the entity's own frame, so a rotated block is picked by its real outline.
    const a = (-e.transform.rotation * Math.PI) / 180;
    const dx = x - e.transform.x;
    const dy = y - e.transform.y;
    const lx = dx * Math.cos(a) - dy * Math.sin(a);
    const ly = dx * Math.sin(a) + dy * Math.cos(a);
    const round = !e.shape || e.shape.kind === 'circle';
    const inside = round
      ? (lx / (w / 2)) ** 2 + (ly / (h / 2)) ** 2 <= 1
      : Math.abs(lx) <= w / 2 && Math.abs(ly) <= h / 2;
    if (inside) return e.id;
  }
  return null;
}

function shapeOf(b: Pick<Body, 'x' | 'y' | 'rotation' | 'scale' | 'shape'>): Instance | null {
  if (!b.shape) return null;
  return {
    x: b.x,
    y: b.y,
    w: b.shape.w * b.scale,
    h: b.shape.h * b.scale,
    rotation: b.rotation,
    colour: channels(b.shape.colour),
    alpha: 1,
    kind: b.shape.kind === 'rect' ? KIND.rect : KIND.circle,
    glow: b.shape.glow,
  };
}

function flat(e: Entity) {
  return { x: e.transform.x, y: e.transform.y, rotation: e.transform.rotation, scale: e.transform.scale, shape: e.shape };
}

export class Engine {
  readonly drawing: boolean;
  private readonly canvas: HTMLCanvasElement;
  private readonly bridge: EditorBridge;
  private readonly renderer: Renderer | null;
  private scene: Scene;
  private selected: string | null = null;
  private letterbox: [number, number, number] = [0, 0, 0];
  private accent: [number, number, number] = [1, 0.42, 0];
  private mode: Mode = 'edit';
  private world: World | null = null;
  private frame = 0;
  private last = 0;
  private pointer: { x: number; y: number } | null = null;
  private dragging: { id: string; dx: number; dy: number; pointerId: number } | null = null;
  private view: Fit | null = null;
  private fps = { frames: 0, since: 0 };
  private readonly resizer: ResizeObserver;

  constructor(canvas: HTMLCanvasElement, scene: Scene, bridge: EditorBridge) {
    this.canvas = canvas;
    this.scene = scene;
    this.bridge = bridge;
    this.renderer = Renderer.create(canvas);
    this.drawing = this.renderer !== null;
    this.readTheme();

    canvas.addEventListener('pointerdown', this.onDown);
    canvas.addEventListener('pointermove', this.onMove);
    canvas.addEventListener('pointerup', this.onUp);
    canvas.addEventListener('pointercancel', this.onUp);
    canvas.addEventListener('pointerleave', this.onLeave);
    this.resizer = new ResizeObserver(() => this.resize());
    this.resizer.observe(canvas);
    this.resize();
  }

  /** The editor's scene changed. Drawn at once in edit mode; a playing world keeps running its own copy. */
  setScene(scene: Scene): void {
    this.scene = scene;
    if (this.mode === 'edit') this.redraw();
  }

  setSelected(id: string | null): void {
    this.selected = id;
    if (this.mode === 'edit') this.redraw();
  }

  /** Theme colours changed: the letterbox and the selection ring follow the launcher's tokens. */
  readTheme(): void {
    const style = getComputedStyle(document.documentElement);
    const hex = (name: string, fallback: [number, number, number]) => {
      const raw = style.getPropertyValue(name).trim();
      return /^#[0-9a-f]{6}$/i.test(raw) ? channels(raw.toLowerCase()) : fallback;
    };
    this.letterbox = hex('--void', [0.02, 0.02, 0.03]);
    this.accent = hex('--accent', [1, 0.42, 0]);
    if (this.mode === 'edit') this.redraw();
  }

  /** Start playing. With `held`, one frame is drawn and nothing moves until `step`. */
  play(held: boolean): void {
    this.stopLoop();
    this.mode = 'play';
    this.dragging = null;
    this.world = new World(this.scene);
    this.fps = { frames: 0, since: performance.now() };
    if (held) {
      this.render();
      this.bridge.stats?.(0, 0);
      return;
    }
    this.last = performance.now();
    this.frame = requestAnimationFrame(this.tick);
  }

  /** Advance a held world by one fixed slice, in steps no larger than the world allows. */
  step(): void {
    if (this.mode !== 'play' || !this.world) return;
    const slice = 1 / 60;
    for (let t = 0; t < STEP_SECONDS - 1e-9; t += slice) this.world.step(slice, this.pointer);
    this.render();
    this.bridge.stats?.(this.world.count, 0);
  }

  /** Back to the editor's scene, exactly as it was before Play. */
  stop(): void {
    this.stopLoop();
    this.mode = 'edit';
    this.world = null;
    this.redraw();
  }

  get playing(): boolean {
    return this.mode === 'play';
  }

  dispose(): void {
    this.stopLoop();
    this.resizer.disconnect();
    const c = this.canvas;
    c.removeEventListener('pointerdown', this.onDown);
    c.removeEventListener('pointermove', this.onMove);
    c.removeEventListener('pointerup', this.onUp);
    c.removeEventListener('pointercancel', this.onUp);
    c.removeEventListener('pointerleave', this.onLeave);
    this.renderer?.dispose();
  }

  // ── drawing ───────────────────────────────────────────────────────────────

  private resize() {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const w = Math.max(1, Math.floor(this.canvas.clientWidth * dpr));
    const h = Math.max(1, Math.floor(this.canvas.clientHeight * dpr));
    if (this.canvas.width !== w || this.canvas.height !== h) {
      this.canvas.width = w;
      this.canvas.height = h;
    }
    this.view = fit(this.canvas.width, this.canvas.height, this.scene.size);
    // A resize clears the canvas; the loop redraws on its own.
    if (this.mode === 'edit' || !this.frame) this.render();
  }

  private redraw() {
    this.view = fit(this.canvas.width, this.canvas.height, this.scene.size);
    this.render();
  }

  private render() {
    const frame: Frame = {
      size: this.scene.size,
      background: channels(this.scene.background),
      letterbox: this.letterbox,
      shapes: [],
      overlay: [],
    };
    if (this.mode === 'play' && this.world) {
      for (const b of this.world.bodies) {
        const s = shapeOf(b);
        if (s) frame.shapes.push(s);
      }
      frame.particles = { data: this.world.particles, count: this.world.count };
    } else {
      for (const e of this.scene.entities) {
        const s = shapeOf(flat(e));
        if (s) frame.shapes.push(s);
        else frame.overlay.push(this.marker(e, 0.5));
      }
      const chosen = this.scene.entities.find((e) => e.id === this.selected);
      if (chosen) frame.overlay.push(this.marker(chosen, 1));
    }
    if (this.renderer) this.view = this.renderer.draw(frame);
  }

  private marker(e: Entity, alpha: number): Instance {
    const { w, h } = bounds(e);
    return {
      x: e.transform.x,
      y: e.transform.y,
      w,
      h,
      rotation: e.transform.rotation,
      colour: this.accent,
      alpha,
      kind: !e.shape || e.shape.kind === 'circle' ? KIND.circleRing : KIND.boxRing,
      glow: 0,
    };
  }

  private tick = (now: number) => {
    if (this.mode !== 'play' || !this.world) return;
    const dt = (now - this.last) / 1000;
    this.last = now;
    this.world.step(dt, this.pointer);
    this.render();
    this.fps.frames += 1;
    if (now - this.fps.since >= 250) {
      this.bridge.stats?.(this.world.count, Math.round((this.fps.frames * 1000) / (now - this.fps.since)));
      this.fps = { frames: 0, since: now };
    }
    this.frame = requestAnimationFrame(this.tick);
  };

  private stopLoop() {
    cancelAnimationFrame(this.frame);
    this.frame = 0;
  }

  // ── input ─────────────────────────────────────────────────────────────────

  /** A pointer event's position in scene units, or null outside the scene's area. */
  private toScene(event: PointerEvent): { x: number; y: number } | null {
    const view = this.view;
    if (!view) return null;
    const rect = this.canvas.getBoundingClientRect();
    const scaleX = this.canvas.width / Math.max(rect.width, 1);
    const scaleY = this.canvas.height / Math.max(rect.height, 1);
    const px = (event.clientX - rect.left) * scaleX - view.x;
    const py = (event.clientY - rect.top) * scaleY - view.y;
    if (px < 0 || py < 0 || px > view.w || py > view.h) return null;
    return { x: px * view.unitsPerPixel - this.scene.size.w / 2, y: py * view.unitsPerPixel - this.scene.size.h / 2 };
  }

  private onDown = (event: PointerEvent) => {
    if (this.mode !== 'edit' || event.button !== 0) return;
    const at = this.toScene(event);
    const id = at ? pick(this.scene, at.x, at.y) : null;
    this.bridge.select(id);
    if (!id || !at) return;
    const e = this.scene.entities.find((n) => n.id === id)!;
    this.dragging = { id, dx: e.transform.x - at.x, dy: e.transform.y - at.y, pointerId: event.pointerId };
    this.canvas.setPointerCapture(event.pointerId);
  };

  private onMove = (event: PointerEvent) => {
    if (this.mode === 'play') {
      this.pointer = this.toScene(event);
      return;
    }
    const drag = this.dragging;
    if (!drag || drag.pointerId !== event.pointerId) return;
    const at = this.toScene(event);
    if (at) this.bridge.move(drag.id, at.x + drag.dx, at.y + drag.dy);
  };

  private onUp = (event: PointerEvent) => {
    if (this.dragging?.pointerId === event.pointerId) {
      this.dragging = null;
      if (this.canvas.hasPointerCapture(event.pointerId)) this.canvas.releasePointerCapture(event.pointerId);
    }
  };

  private onLeave = () => {
    if (this.mode === 'play') this.pointer = null;
  };
}

