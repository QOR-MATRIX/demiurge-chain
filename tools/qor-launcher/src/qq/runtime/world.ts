/**
 * QQ's simulation: a scene while it plays (ADR-082, DIRECTION P3.1).
 *
 * Play copies the scene into a world and runs the copy; the scene itself is
 * never touched, so Stop is simply dropping the world. Each step moves what has
 * motion, eases what follows toward the pointer, and runs every emitter's
 * particles. It is deterministic: each entity's random stream is seeded from its
 * id, so the same scene plays the same way every time, which is what a creator
 * comparing two versions needs.
 *
 * Particles are updated on the CPU in one flat array. At P3.1's ceiling of 4,096
 * that costs well under a millisecond; the blueprint's GPU ping-pong is for
 * when a scene needs more.
 */

import type { Emitter, Entity, Follow, Motion, Scene, Shape } from './scene';

export const MAX_PARTICLES = 4096;
/** Floats per particle: x, y, vx, vy, age, life, size, r, g, b. */
export const STRIDE = 10;
/** A long frame (a breakpoint, a hidden tab) is stepped as this much, never more. */
export const MAX_STEP = 1 / 20;

export interface Body {
  id: string;
  x: number;
  y: number;
  rotation: number;
  scale: number;
  shape?: Shape;
  motion?: Motion;
  follow?: Follow;
  emitter?: Emitter;
  /** Particles owed but not yet whole. */
  carry: number;
  random: () => number;
}

/** Mulberry32: small, fast, and the same everywhere. */
function stream(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function seedOf(id: string): number {
  let h = 2166136261;
  for (let i = 0; i < id.length; i++) h = Math.imul(h ^ id.charCodeAt(i), 16777619);
  return h >>> 0;
}

/** `#rrggbb` as three numbers from 0 to 1. */
export function channels(hex: string): [number, number, number] {
  const n = Number.parseInt(hex.slice(1), 16);
  return [((n >> 16) & 255) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255];
}

function body(e: Entity): Body {
  return {
    id: e.id,
    x: e.transform.x,
    y: e.transform.y,
    rotation: e.transform.rotation,
    scale: e.transform.scale,
    shape: e.shape,
    motion: e.motion,
    follow: e.follow,
    emitter: e.emitter,
    carry: 0,
    random: stream(seedOf(e.id)),
  };
}

export class World {
  readonly bodies: Body[];
  readonly particles = new Float32Array(MAX_PARTICLES * STRIDE);
  count = 0;
  time = 0;

  constructor(scene: Scene) {
    this.bodies = scene.entities.map(body);
  }

  /** Advance by `dt` seconds. `pointer` is in scene units, or null when it is outside the viewport. */
  step(dt: number, pointer: { x: number; y: number } | null): void {
    const t = Math.min(Math.max(dt, 0), MAX_STEP);
    if (t === 0) return;
    this.time += t;

    for (const b of this.bodies) {
      if (b.motion) {
        b.x += b.motion.vx * t;
        b.y += b.motion.vy * t;
        b.rotation = (b.rotation + b.motion.spin * t) % 360;
      }
      if (b.follow && pointer && b.follow.strength > 0) {
        // Exponential easing, so the follow feels the same at any frame rate.
        const k = 1 - Math.exp(-b.follow.strength * 14 * t);
        b.x += (pointer.x - b.x) * k;
        b.y += (pointer.y - b.y) * k;
      }
      if (b.emitter && b.emitter.rate > 0) this.emit(b, b.emitter, t);
    }

    const p = this.particles;
    // A little drag, so a burst settles instead of flying off at constant speed.
    const drag = Math.exp(-0.9 * t);
    let i = 0;
    while (i < this.count) {
      const o = i * STRIDE;
      const age = p[o + 4]! + t;
      if (age >= p[o + 5]!) {
        // Dead: the last particle takes its place, so the live ones stay packed.
        this.count -= 1;
        p.copyWithin(o, this.count * STRIDE, this.count * STRIDE + STRIDE);
        continue;
      }
      const vx = p[o + 2]! * drag;
      const vy = p[o + 3]! * drag;
      p[o] = p[o]! + vx * t;
      p[o + 1] = p[o + 1]! + vy * t;
      p[o + 2] = vx;
      p[o + 3] = vy;
      p[o + 4] = age;
      i += 1;
    }
  }

  private emit(b: Body, e: Emitter, t: number): void {
    b.carry += e.rate * t;
    const [r, g, bl] = channels(e.colour);
    while (b.carry >= 1) {
      b.carry -= 1;
      if (this.count >= MAX_PARTICLES) {
        b.carry = 0;
        return;
      }
      // Straight up is -90 degrees on a y-down screen; spread is the whole cone.
      const angle = ((-90 + (b.random() - 0.5) * e.spread) * Math.PI) / 180;
      const speed = e.speed * (0.6 + 0.4 * b.random());
      const o = this.count * STRIDE;
      const p = this.particles;
      p[o] = b.x;
      p[o + 1] = b.y;
      p[o + 2] = Math.cos(angle) * speed;
      p[o + 3] = Math.sin(angle) * speed;
      p[o + 4] = 0;
      p[o + 5] = e.life * (0.7 + 0.3 * b.random());
      p[o + 6] = e.size * (0.7 + 0.6 * b.random());
      p[o + 7] = r;
      p[o + 8] = g;
      p[o + 9] = bl;
      this.count += 1;
    }
  }
}
