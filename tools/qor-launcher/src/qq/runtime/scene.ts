/**
 * QQ's scene: what a creator authors, saves as a `.qq.json` file and versions
 * with Qontrol (ADR-082, DIRECTION P3.1).
 *
 * The format is deliberately small and plain. A scene is a size, a background
 * and a list of entities; an entity is an id, a name, a transform and whichever
 * of the optional components it has. Nothing is binary and nothing is private
 * to the engine, so a scene can be read, diffed and merged by a person.
 *
 * `serialiseScene` writes the one canonical form: keys in a fixed order, numbers
 * rounded to what the editor can set, one component per line. Two saves of the
 * same scene are the same bytes, and a change to one value is a one-line diff in
 * Projects. `parseScene` is the only way in: it checks every field and refuses
 * anything it does not understand, so the runtime never meets a value it has to
 * guess about. The host checks the outer shape again before it writes
 * (`src-tauri/src/qq.rs`).
 */

import { QQ_PALETTE } from '../../styles/themes';

export const FORMAT = 1;
export const MAX_ENTITIES = 4096;

export type ShapeKind = 'rect' | 'circle';

export interface Transform {
  x: number;
  y: number;
  /** Degrees, clockwise. */
  rotation: number;
  scale: number;
}

/** A filled shape, drawn as a signed-distance field so its edge and glow are exact at any size. */
export interface Shape {
  kind: ShapeKind;
  w: number;
  h: number;
  colour: string;
  /** 0 is a hard edge; 1 is the widest halo. */
  glow: number;
}

/** Constant movement while playing. */
export interface Motion {
  vx: number;
  vy: number;
  /** Degrees per second. */
  spin: number;
}

/** Eases toward the pointer while playing: realtime input. 0 ignores it; 1 is the tightest follow. */
export interface Follow {
  strength: number;
}

/** Particles while playing, from the entity's position. */
export interface Emitter {
  /** Particles a second. */
  rate: number;
  /** Seconds each particle lives. */
  life: number;
  speed: number;
  /** Degrees either side of straight up (`360` is every direction). */
  spread: number;
  size: number;
  colour: string;
}

export interface Entity {
  id: string;
  name: string;
  transform: Transform;
  shape?: Shape;
  motion?: Motion;
  follow?: Follow;
  emitter?: Emitter;
}

export interface Scene {
  qq: typeof FORMAT;
  name: string;
  size: { w: number; h: number };
  background: string;
  entities: Entity[];
}

export type ComponentKey = 'shape' | 'motion' | 'follow' | 'emitter';
export const COMPONENTS: ComponentKey[] = ['shape', 'motion', 'follow', 'emitter'];

/** Every number the editor can set, with its range and the step it is kept to. */
export const LIMITS = {
  x: { min: -8192, max: 8192, step: 0.1 },
  y: { min: -8192, max: 8192, step: 0.1 },
  rotation: { min: -360, max: 360, step: 0.1 },
  scale: { min: 0.05, max: 20, step: 0.01 },
  w: { min: 1, max: 4096, step: 0.1 },
  h: { min: 1, max: 4096, step: 0.1 },
  glow: { min: 0, max: 1, step: 0.01 },
  vx: { min: -4000, max: 4000, step: 0.1 },
  vy: { min: -4000, max: 4000, step: 0.1 },
  spin: { min: -1440, max: 1440, step: 0.1 },
  strength: { min: 0, max: 1, step: 0.01 },
  rate: { min: 0, max: 2000, step: 1 },
  life: { min: 0.05, max: 10, step: 0.01 },
  speed: { min: 0, max: 4000, step: 0.1 },
  spread: { min: 0, max: 360, step: 1 },
  size: { min: 0.5, max: 256, step: 0.1 },
} as const;

export type NumberField = keyof typeof LIMITS;

const COLOUR = /^#[0-9a-f]{6}$/;
const ID = /^[a-z0-9-]{1,32}$/;

/** Keep a number to its field's range and step, so what is saved is what the inspector shows. */
export function keep(field: NumberField, value: number): number {
  const { min, max, step } = LIMITS[field];
  const clamped = Math.min(max, Math.max(min, value));
  const decimals = step >= 1 ? 0 : Math.round(-Math.log10(step));
  const rounded = Number((Math.round(clamped / step) * step).toFixed(decimals));
  return Object.is(rounded, -0) ? 0 : rounded;
}

/** A colour as the format keeps it: `#rrggbb`, lowercase. `null` when it is not one. */
export function colourOf(value: unknown): string | null {
  if (typeof value !== 'string') return null;
  const lower = value.trim().toLowerCase();
  return COLOUR.test(lower) ? lower : null;
}

export class SceneError extends Error {}

function fail(message: string): never {
  throw new SceneError(message);
}

function record(value: unknown, where: string): Record<string, unknown> {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) fail(`${where} is not an object`);
  return value as Record<string, unknown>;
}

function only(rec: Record<string, unknown>, keys: readonly string[], where: string) {
  for (const key of Object.keys(rec)) if (!keys.includes(key)) fail(`${where} has an unknown field "${key}"`);
}

function num(rec: Record<string, unknown>, field: NumberField, where: string): number {
  const value = rec[field];
  if (typeof value !== 'number' || !Number.isFinite(value)) fail(`${where}.${field} is not a number`);
  const { min, max } = LIMITS[field];
  if (value < min || value > max) fail(`${where}.${field} is outside ${min} to ${max}`);
  return keep(field, value);
}

function colour(rec: Record<string, unknown>, where: string): string {
  return colourOf(rec.colour) ?? fail(`${where}.colour is not a #rrggbb colour`);
}

function text(value: unknown, where: string, max: number): string {
  if (typeof value !== 'string' || value.length > max) fail(`${where} is not text of at most ${max} characters`);
  return value;
}

function parseEntity(value: unknown, index: number): Entity {
  const where = `entity ${index + 1}`;
  const e = record(value, where);
  only(e, ['id', 'name', 'transform', ...COMPONENTS], where);
  const id = typeof e.id === 'string' && ID.test(e.id) ? e.id : fail(`${where}.id is not lowercase letters, digits and dashes`);
  const t = record(e.transform, `${where}.transform`);
  only(t, ['x', 'y', 'rotation', 'scale'], `${where}.transform`);
  const entity: Entity = {
    id,
    name: text(e.name, `${where}.name`, 64),
    transform: {
      x: num(t, 'x', `${where}.transform`),
      y: num(t, 'y', `${where}.transform`),
      rotation: num(t, 'rotation', `${where}.transform`),
      scale: num(t, 'scale', `${where}.transform`),
    },
  };
  if (e.shape !== undefined) {
    const s = record(e.shape, `${where}.shape`);
    only(s, ['kind', 'w', 'h', 'colour', 'glow'], `${where}.shape`);
    if (s.kind !== 'rect' && s.kind !== 'circle') fail(`${where}.shape.kind is not rect or circle`);
    entity.shape = {
      kind: s.kind,
      w: num(s, 'w', `${where}.shape`),
      h: num(s, 'h', `${where}.shape`),
      colour: colour(s, `${where}.shape`),
      glow: num(s, 'glow', `${where}.shape`),
    };
  }
  if (e.motion !== undefined) {
    const m = record(e.motion, `${where}.motion`);
    only(m, ['vx', 'vy', 'spin'], `${where}.motion`);
    entity.motion = { vx: num(m, 'vx', `${where}.motion`), vy: num(m, 'vy', `${where}.motion`), spin: num(m, 'spin', `${where}.motion`) };
  }
  if (e.follow !== undefined) {
    const f = record(e.follow, `${where}.follow`);
    only(f, ['strength'], `${where}.follow`);
    entity.follow = { strength: num(f, 'strength', `${where}.follow`) };
  }
  if (e.emitter !== undefined) {
    const m = record(e.emitter, `${where}.emitter`);
    only(m, ['rate', 'life', 'speed', 'spread', 'size', 'colour'], `${where}.emitter`);
    entity.emitter = {
      rate: num(m, 'rate', `${where}.emitter`),
      life: num(m, 'life', `${where}.emitter`),
      speed: num(m, 'speed', `${where}.emitter`),
      spread: num(m, 'spread', `${where}.emitter`),
      size: num(m, 'size', `${where}.emitter`),
      colour: colour(m, `${where}.emitter`),
    };
  }
  return entity;
}

/** Read a scene from its file's text, or throw a `SceneError` that says what is wrong. */
export function parseScene(source: string): Scene {
  let value: unknown;
  try {
    value = JSON.parse(source);
  } catch {
    fail('the file is not JSON');
  }
  const s = record(value, 'the scene');
  only(s, ['qq', 'name', 'size', 'background', 'entities'], 'the scene');
  if (s.qq !== FORMAT) fail(`this QQ reads scenes of format ${FORMAT}`);
  const size = record(s.size, 'the scene size');
  only(size, ['w', 'h'], 'the scene size');
  const side = (v: unknown, which: string) =>
    typeof v === 'number' && Number.isInteger(v) && v >= 64 && v <= 4096 ? v : fail(`the scene ${which} is not a whole number from 64 to 4096`);
  if (!Array.isArray(s.entities)) fail('the scene has no list of entities');
  if (s.entities.length > MAX_ENTITIES) fail(`a scene has at most ${MAX_ENTITIES} entities`);
  const entities = s.entities.map(parseEntity);
  const ids = new Set<string>();
  for (const e of entities) {
    if (ids.has(e.id)) fail(`two entities share the id ${e.id}`);
    ids.add(e.id);
  }
  return {
    qq: FORMAT,
    name: text(s.name, 'the scene name', 64),
    size: { w: side(size.w, 'width'), h: side(size.h, 'height') },
    background: colourOf(s.background) ?? fail('the scene background is not a #rrggbb colour'),
    entities,
  };
}

/** One component as one line: `{ "a": 1, "b": 2 }` with keys in the given order. */
function line(value: object, keys: readonly string[]): string {
  const rec = value as Record<string, unknown>;
  return `{ ${keys.map((k) => `${JSON.stringify(k)}: ${JSON.stringify(rec[k])}`).join(', ')} }`;
}

/** The canonical text of a scene: the same scene is always the same bytes. */
export function serialiseScene(scene: Scene): string {
  const entity = (e: Entity) => {
    const parts = [
      `      "id": ${JSON.stringify(e.id)}`,
      `      "name": ${JSON.stringify(e.name)}`,
      `      "transform": ${line(e.transform, ['x', 'y', 'rotation', 'scale'])}`,
    ];
    if (e.shape) parts.push(`      "shape": ${line(e.shape, ['kind', 'w', 'h', 'colour', 'glow'])}`);
    if (e.motion) parts.push(`      "motion": ${line(e.motion, ['vx', 'vy', 'spin'])}`);
    if (e.follow) parts.push(`      "follow": ${line(e.follow, ['strength'])}`);
    if (e.emitter) parts.push(`      "emitter": ${line(e.emitter, ['rate', 'life', 'speed', 'spread', 'size', 'colour'])}`);
    return `    {\n${parts.join(',\n')}\n    }`;
  };
  const entities = scene.entities.length ? `[\n${scene.entities.map(entity).join(',\n')}\n  ]` : '[]';
  return (
    `{\n` +
    `  "qq": ${FORMAT},\n` +
    `  "name": ${JSON.stringify(scene.name)},\n` +
    `  "size": ${line(scene.size, ['w', 'h'])},\n` +
    `  "background": ${JSON.stringify(scene.background)},\n` +
    `  "entities": ${entities}\n` +
    `}\n`
  );
}

/** The next free id: `e` and the lowest number not taken. */
export function nextId(scene: Scene): string {
  const taken = new Set(scene.entities.map((e) => e.id));
  for (let n = 1; ; n++) if (!taken.has(`e${n}`)) return `e${n}`;
}

/** What "Add" puts in the scene, at the centre. */
export function newEntity(scene: Scene, kind: 'rect' | 'circle' | 'emitter'): Entity {
  const id = nextId(scene);
  const transform = { x: 0, y: 0, rotation: 0, scale: 1 };
  if (kind === 'emitter') {
    return {
      id,
      name: 'Emitter',
      transform,
      emitter: { rate: 80, life: 1.4, speed: 140, spread: 40, size: 5, colour: QQ_PALETTE.spark },
    };
  }
  return {
    id,
    name: kind === 'rect' ? 'Block' : 'Orb',
    transform,
    shape: kind === 'rect'
      ? { kind, w: 140, h: 36, colour: QQ_PALETTE.ice, glow: 0.3 }
      : { kind, w: 72, h: 72, colour: QQ_PALETTE.ember, glow: 0.6 },
  };
}

/** A component with the values "Add component" gives it. */
export function defaultComponent<K extends ComponentKey>(key: K): NonNullable<Entity[K]> {
  const defaults: { [P in ComponentKey]: NonNullable<Entity[P]> } = {
    shape: { kind: 'circle', w: 64, h: 64, colour: QQ_PALETTE.ember, glow: 0.5 },
    motion: { vx: 0, vy: 0, spin: 45 },
    follow: { strength: 0.5 },
    emitter: { rate: 60, life: 1.2, speed: 120, spread: 360, size: 4, colour: QQ_PALETTE.glint },
  };
  return structuredClone(defaults[key]);
}

/** The scene a new QQ file starts as: something moving and lit, so the first Play shows what the engine does. */
export function starterScene(name = 'first-light'): Scene {
  return {
    qq: FORMAT,
    name,
    size: { w: 960, h: 540 },
    background: QQ_PALETTE.background,
    entities: [
      {
        id: 'e1',
        name: 'Core',
        transform: { x: 0, y: 0, rotation: 0, scale: 1 },
        shape: { kind: 'circle', w: 84, h: 84, colour: QQ_PALETTE.ember, glow: 0.7 },
        follow: { strength: 0.35 },
        emitter: { rate: 90, life: 1.6, speed: 110, spread: 360, size: 4, colour: QQ_PALETTE.spark },
      },
      {
        id: 'e2',
        name: 'Ring',
        transform: { x: -260, y: 0, rotation: 0, scale: 1 },
        shape: { kind: 'rect', w: 120, h: 18, colour: QQ_PALETTE.ice, glow: 0.4 },
        motion: { vx: 0, vy: 0, spin: 90 },
      },
      {
        id: 'e3',
        name: 'Ground',
        transform: { x: 0, y: 220, rotation: 0, scale: 1 },
        shape: { kind: 'rect', w: 760, h: 10, colour: QQ_PALETTE.slate, glow: 0 },
      },
    ],
  };
}
