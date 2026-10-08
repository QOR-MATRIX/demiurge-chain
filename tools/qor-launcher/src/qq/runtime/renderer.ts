/**
 * QQ's 2D renderer, on WebGL2 (ADR-082, blueprint layer one).
 *
 * Everything is one instanced quad. A shape's edge and its glow are computed in
 * the fragment shader from a signed distance, so a circle is round and a glow is
 * smooth at any size, with no textures and no tessellation. Shapes are drawn
 * with ordinary blending and particles additively, so a cloud of them builds
 * light the way sparks do. The scene keeps its aspect: it is fitted into the
 * canvas and the margin is drawn in the letterbox colour.
 *
 * `create` returns null when WebGL2 is not available. That is not an error: the
 * editor still edits and saves, and says it cannot draw.
 */

import { STRIDE } from './world';

/** Floats per instance: cx, cy, w, h, rotation (radians), r, g, b, a, kind, glow. */
const FLOATS = 11;

export const KIND = { rect: 0, circle: 1, particle: 2, boxRing: 3, circleRing: 4 } as const;

export interface Instance {
  x: number;
  y: number;
  w: number;
  h: number;
  /** Degrees, clockwise. */
  rotation: number;
  colour: [number, number, number];
  alpha: number;
  kind: number;
  glow: number;
}

export interface Frame {
  size: { w: number; h: number };
  background: [number, number, number];
  letterbox: [number, number, number];
  shapes: Instance[];
  /** The world's particle array and how many are live; drawn after the shapes, additively. */
  particles?: { data: Float32Array; count: number };
  /** Drawn last, over everything: the editor's selection and markers. */
  overlay: Instance[];
}

/** Where the scene sits in the canvas, in device pixels, and how many scene units one device pixel is. */
export interface Fit {
  x: number;
  y: number;
  w: number;
  h: number;
  unitsPerPixel: number;
}

export function fit(canvasW: number, canvasH: number, size: { w: number; h: number }): Fit {
  const scale = Math.min(canvasW / size.w, canvasH / size.h);
  const w = Math.max(1, Math.floor(size.w * scale));
  const h = Math.max(1, Math.floor(size.h * scale));
  return { x: Math.floor((canvasW - w) / 2), y: Math.floor((canvasH - h) / 2), w, h, unitsPerPixel: size.w / w };
}

const VERTEX = `#version 300 es
layout(location = 0) in vec2 a_corner;
layout(location = 1) in vec4 a_rect;
layout(location = 2) in float a_rotation;
layout(location = 3) in vec4 a_colour;
layout(location = 4) in vec2 a_style;

uniform vec2 u_half;
uniform float u_px;

out vec2 v_local;
out vec2 v_half;
out vec4 v_colour;
out vec2 v_style;

void main() {
  vec2 half_size = max(a_rect.zw * 0.5, vec2(u_px));
  // Room around the shape for its glow and for the antialiased edge.
  float pad = 2.0 * u_px + a_style.y * max(half_size.x, half_size.y) * 1.6 + (a_style.x > 2.5 ? 6.0 * u_px : 0.0);
  vec2 local = a_corner * (half_size + pad);
  float c = cos(a_rotation);
  float s = sin(a_rotation);
  vec2 world = a_rect.xy + vec2(local.x * c - local.y * s, local.x * s + local.y * c);
  gl_Position = vec4(world.x / u_half.x, -world.y / u_half.y, 0.0, 1.0);
  v_local = local;
  v_half = half_size;
  v_colour = a_colour;
  v_style = a_style;
}`;

const FRAGMENT = `#version 300 es
precision highp float;

in vec2 v_local;
in vec2 v_half;
in vec4 v_colour;
in vec2 v_style;

uniform float u_px;

out vec4 outColour;

float box(vec2 p, vec2 b, float r) {
  vec2 q = abs(p) - b + r;
  return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

float ellipse(vec2 p, vec2 b) {
  return (length(p / b) - 1.0) * min(b.x, b.y);
}

void main() {
  int kind = int(v_style.x + 0.5);
  float glowAmount = v_style.y;
  float alpha;

  if (kind == 2) {
    // A particle: a soft dot, brightest at its centre.
    float t = clamp(1.0 - length(v_local / v_half), 0.0, 1.0);
    alpha = t * t;
  } else {
    bool round = kind == 1 || kind == 4;
    float d = round ? ellipse(v_local, v_half) : box(v_local, v_half, min(min(v_half.x, v_half.y) * 0.2, 6.0 * u_px + 2.0));
    if (kind >= 3) {
      // A marker ring just outside the shape: the editor's, never the game's.
      float ring = abs(d - 4.0 * u_px);
      alpha = 1.0 - smoothstep(0.6 * u_px, 1.6 * u_px, ring);
    } else {
      float fill = 1.0 - smoothstep(-u_px, u_px, d);
      float reach = max(glowAmount * max(v_half.x, v_half.y) * 0.55, u_px);
      float glow = glowAmount > 0.0 ? glowAmount * 0.85 * exp(-max(d, 0.0) / reach) : 0.0;
      alpha = fill + glow * (1.0 - fill);
    }
  }

  float a = clamp(alpha * v_colour.a, 0.0, 1.0);
  outColour = vec4(v_colour.rgb * a, a);
}`;

export class Renderer {
  private readonly gl: WebGL2RenderingContext;
  private readonly program: WebGLProgram;
  private readonly vao: WebGLVertexArrayObject;
  private readonly corners: WebGLBuffer;
  private readonly instances: WebGLBuffer;
  private readonly uHalf: WebGLUniformLocation | null;
  private readonly uPx: WebGLUniformLocation | null;
  private scratch = new Float32Array(FLOATS * 256);

  private constructor(gl: WebGL2RenderingContext, program: WebGLProgram) {
    this.gl = gl;
    this.program = program;
    this.uHalf = gl.getUniformLocation(program, 'u_half');
    this.uPx = gl.getUniformLocation(program, 'u_px');

    this.vao = gl.createVertexArray()!;
    gl.bindVertexArray(this.vao);

    this.corners = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.corners);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);

    this.instances = gl.createBuffer()!;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.instances);
    const bytes = FLOATS * 4;
    const attribute = (location: number, size: number, offset: number) => {
      gl.enableVertexAttribArray(location);
      gl.vertexAttribPointer(location, size, gl.FLOAT, false, bytes, offset * 4);
      gl.vertexAttribDivisor(location, 1);
    };
    attribute(1, 4, 0);
    attribute(2, 1, 4);
    attribute(3, 4, 5);
    attribute(4, 2, 9);
    gl.bindVertexArray(null);
  }

  static create(canvas: HTMLCanvasElement): Renderer | null {
    const gl = canvas.getContext('webgl2', {
      alpha: false,
      antialias: false,
      premultipliedAlpha: true,
      preserveDrawingBuffer: false,
    });
    if (!gl) return null;
    const compile = (type: number, source: string) => {
      const shader = gl.createShader(type)!;
      gl.shaderSource(shader, source);
      gl.compileShader(shader);
      if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        console.error('QQ shader:', gl.getShaderInfoLog(shader));
        gl.deleteShader(shader);
        return null;
      }
      return shader;
    };
    const vs = compile(gl.VERTEX_SHADER, VERTEX);
    const fs = compile(gl.FRAGMENT_SHADER, FRAGMENT);
    if (!vs || !fs) return null;
    const program = gl.createProgram()!;
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    gl.deleteShader(vs);
    gl.deleteShader(fs);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      gl.deleteProgram(program);
      return null;
    }
    return new Renderer(gl, program);
  }

  draw(frame: Frame): Fit {
    const gl = this.gl;
    const canvas = gl.canvas as HTMLCanvasElement;
    const view = fit(canvas.width, canvas.height, frame.size);

    gl.disable(gl.SCISSOR_TEST);
    gl.viewport(0, 0, canvas.width, canvas.height);
    gl.clearColor(frame.letterbox[0], frame.letterbox[1], frame.letterbox[2], 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    // GL counts rows from the bottom.
    const glY = canvas.height - view.y - view.h;
    gl.enable(gl.SCISSOR_TEST);
    gl.scissor(view.x, glY, view.w, view.h);
    gl.viewport(view.x, glY, view.w, view.h);
    gl.clearColor(frame.background[0], frame.background[1], frame.background[2], 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.useProgram(this.program);
    gl.uniform2f(this.uHalf, frame.size.w / 2, frame.size.h / 2);
    gl.uniform1f(this.uPx, view.unitsPerPixel);
    gl.bindVertexArray(this.vao);
    gl.enable(gl.BLEND);

    gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);
    this.instanced(frame.shapes);

    if (frame.particles && frame.particles.count > 0) {
      gl.blendFunc(gl.ONE, gl.ONE);
      this.particles(frame.particles.data, frame.particles.count);
    }

    gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);
    this.instanced(frame.overlay);

    gl.bindVertexArray(null);
    gl.disable(gl.SCISSOR_TEST);
    return view;
  }

  private room(count: number): Float32Array {
    if (this.scratch.length < count * FLOATS) {
      this.scratch = new Float32Array(Math.max(count, this.scratch.length / FLOATS * 2) * FLOATS);
    }
    return this.scratch;
  }

  private upload(data: Float32Array, count: number) {
    const gl = this.gl;
    gl.bindBuffer(gl.ARRAY_BUFFER, this.instances);
    gl.bufferData(gl.ARRAY_BUFFER, data.subarray(0, count * FLOATS), gl.DYNAMIC_DRAW);
    gl.drawArraysInstanced(gl.TRIANGLE_STRIP, 0, 4, count);
  }

  private instanced(list: Instance[]) {
    if (list.length === 0) return;
    const data = this.room(list.length);
    list.forEach((s, i) => {
      const o = i * FLOATS;
      data[o] = s.x;
      data[o + 1] = s.y;
      data[o + 2] = s.w;
      data[o + 3] = s.h;
      data[o + 4] = (s.rotation * Math.PI) / 180;
      data[o + 5] = s.colour[0];
      data[o + 6] = s.colour[1];
      data[o + 7] = s.colour[2];
      data[o + 8] = s.alpha;
      data[o + 9] = s.kind;
      data[o + 10] = s.glow;
    });
    this.upload(data, list.length);
  }

  private particles(source: Float32Array, count: number) {
    const data = this.room(count);
    for (let i = 0; i < count; i++) {
      const p = i * STRIDE;
      const o = i * FLOATS;
      const size = source[p + 6]!;
      data[o] = source[p]!;
      data[o + 1] = source[p + 1]!;
      data[o + 2] = size;
      data[o + 3] = size;
      data[o + 4] = 0;
      data[o + 5] = source[p + 7]!;
      data[o + 6] = source[p + 8]!;
      data[o + 7] = source[p + 9]!;
      // Fades out over its life.
      data[o + 8] = 1 - source[p + 4]! / source[p + 5]!;
      data[o + 9] = KIND.particle;
      data[o + 10] = 0;
    }
    this.upload(data, count);
  }

  dispose() {
    const gl = this.gl;
    gl.deleteBuffer(this.corners);
    gl.deleteBuffer(this.instances);
    gl.deleteVertexArray(this.vao);
    gl.deleteProgram(this.program);
  }
}
