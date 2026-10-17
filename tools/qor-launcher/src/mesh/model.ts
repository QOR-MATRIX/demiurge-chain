/**
 * The Mesh, the interface's side (ADR-095; docs/blueprints/mesh.md; L10.1). The host holds the node, its key and the
 * store; this file only asks it, through the `mesh_*` commands.
 */

import { invoke } from '@tauri-apps/api/core';

export interface MeshSettings {
  seeding: boolean;
  diskBytes: number;
  uploadBytesPerSecond: number;
}

export interface MeshStatus {
  settings: MeshSettings;
  /** This launcher's node, as peers dial it; null while nothing runs. */
  node: string | null;
  held: number;
  usedBytes: number;
  servedBytes: number;
}

export interface Held {
  root: string;
  size: number;
  kind: 'own' | 'fetched';
}

export const mesh = {
  status: () => invoke<MeshStatus>('mesh_status'),
  held: () => invoke<Held[]>('mesh_held'),
  configure: (settings: MeshSettings) => invoke<MeshStatus>('mesh_configure', { settings }),
  ticket: (root: string) => invoke<string>('mesh_ticket', { root }),
  fetch: (ticket: string) => invoke<{ root: string; size: number }>('mesh_fetch', { ticket }),
};

const GIB = 1024 ** 3;
const MB = 1000 ** 2;

/** What the person may give: choices, not suggestions of what is right. */
export const DISK_CHOICES = [5, 20, 50, 100, 250].map((g) => g * GIB);
export const UPLOAD_CHOICES = [1, 5, 10, 25, 50].map((m) => m * MB);

export function bytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const units = ['KiB', 'MiB', 'GiB', 'TiB'];
  let v = n / 1024;
  let u = 0;
  while (v >= 1024 && u < units.length - 1) {
    v /= 1024;
    u += 1;
  }
  return `${v >= 100 ? v.toFixed(0) : v.toFixed(1)} ${units[u]}`;
}

export function rate(n: number): string {
  return `${(n / MB).toLocaleString(undefined, { maximumFractionDigits: 1 })} MB/s`;
}

export function short(root: string): string {
  return root.length > 16 ? `${root.slice(0, 8)}…${root.slice(-8)}` : root;
}
