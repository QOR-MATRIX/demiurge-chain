/**
 * The Mesh, the interface's side (ADR-095, ADR-097; docs/blueprints/mesh.md; L10.1 to L10.4). The host holds the node,
 * its key, the store and the Mesh grant; this file only asks it, through the `mesh_*` commands. Nothing here signs.
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
  paid: PaidStatus;
}

/** Paid work (ADR-097): what this PC holds for escrows, and what the chain paid for it. */
export interface PaidStatus {
  /** The account the Mesh signs its own work for; null until the person allows it. */
  account: string | null;
  holdings: { escrow: number; root: string; size: number; answered: number | null }[];
  pendingReceipts: number;
  paidEmbers: string;
  paidCgt: string;
  payments: { kind: 'storage' | 'serving'; escrow: number | null; paidEmbers: string; paidCgt: string; blockHash: string }[];
}

/** What an uploader asks of an escrow. Every price is theirs, in embers as a decimal string. */
export interface EscrowTerms {
  root: string;
  size: number;
  copies: number;
  storagePriceEmbers: string;
  servingPricePerMibEmbers: string;
  servingCapPerPeriod: number;
  depositEmbers: string;
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
  fetchRoot: (root: string) => invoke<{ root: string; size: number }>('mesh_fetch_root', { root }),
  allowPaid: () => invoke<PaidStatus>('mesh_allow_paid'),
  openEscrow: (terms: EscrowTerms) => invoke<number>('mesh_open_escrow', { terms }),
};

/**
 * An amount typed in CGT, as a whole number of embers (18 decimals), exactly: no floating point touches it, and more
 * than 18 decimals is refused rather than cut (AGENTS.md §5). Null when it is not an amount.
 */
export function toEmbers(cgt: string): string | null {
  const m = /^(\d+)(?:\.(\d{1,18}))?$/.exec(cgt.trim());
  if (!m) return null;
  const embers = BigInt(m[1] ?? '0') * 10n ** 18n + BigInt((m[2] ?? '').padEnd(18, '0') || '0');
  return embers.toString();
}

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
