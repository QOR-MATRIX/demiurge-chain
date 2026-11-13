// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * QOR ID Wallet Integration Service
 * 
 * Links blockchain addresses to QOR ID accounts and manages wallet operations
 */

import { qorAuth, User } from '@demiurge/qor-sdk';
import { Keyring } from '@polkadot/keyring';
import { KeyringPair } from '@polkadot/keyring/types';
import { u8aToHex } from '@polkadot/util';
import axios from 'axios';
import { generateKeypairFromQorId as generateKeypairFromQorIdWasm } from './wasm-wallet';

// Get default API URL (same logic as qor-sdk)
function getDefaultApiUrl(): string {
  if (process.env.NEXT_PUBLIC_QOR_AUTH_URL) {
    return process.env.NEXT_PUBLIC_QOR_AUTH_URL;
  }
  if (typeof globalThis !== 'undefined' && 'window' in globalThis) {
    const win = (globalThis as any).window;
    if (win?.location?.hostname === 'qorsync.dev') {
      return 'http://127.0.0.1:3000/api/v1';
    }
  }
  return 'http://localhost:8080/api/v1';
}

/**
 * Link blockchain address to QOR ID account via API
 */
async function linkAddressToQorId(address: string): Promise<void> {
  const apiUrl = getDefaultApiUrl();
  const token = qorAuth.getToken();
  
  if (!token) {
    throw new Error('Not authenticated');
  }

  try {
    await axios.post(
      `${apiUrl}/profile/link-wallet`,
      { address },
      {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
      }
    );
  } catch (error: any) {
    if (error.response?.status === 401) {
      throw new Error('Authentication required');
    }
    throw new Error(error.response?.data?.message || 'Failed to link address');
  }
}

/**
 * Generate a deterministic blockchain address from QOR ID
 * 
 * This creates a deterministic address based on the QOR ID, ensuring
 * the same QOR ID always maps to the same blockchain address.
 * 
 * @param qorId QOR ID (e.g., "username#0001")
 * @returns Deterministic blockchain address (SS58 format)
 */
export function generateAddressFromQorId(qorId: string): string {
  try {
    const keyring = new Keyring({ type: 'sr25519', ss58Format: 42 });
    
    // Create a deterministic seed from QOR ID
    // Use double-slash prefix for derivation path (required by Polkadot keyring)
    // Hash the QOR ID to create a valid derivation path
    const sanitizedId = qorId.replace(/[^a-zA-Z0-9]/g, '_');
    const derivationPath = `//${sanitizedId}`;
    
    try {
      // Generate keypair from derivation path
      const pair = keyring.addFromUri(derivationPath, { name: qorId }, 'sr25519');
      return pair.address;
    } catch (error) {
      // Fallback: use a mnemonic-based approach if derivation fails
      console.warn('Derivation path failed, using fallback:', error);
      // Create a deterministic 32-byte seed from QOR ID
      const encoder = new TextEncoder();
      const data = encoder.encode(`DEMIURGE_QOR_ID:${qorId}`);
      // Simple hash-like function for seed generation
      const seed = new Uint8Array(32);
      for (let i = 0; i < data.length; i++) {
        seed[i % 32] ^= data[i];
      }
      const pair = keyring.addFromSeed(seed, { name: qorId }, 'sr25519');
      return pair.address;
    }
  } catch (keyringError) {
    // Ultimate fallback if Polkadot keyring fails entirely
    console.error('Keyring initialization failed, using simple hash fallback:', keyringError);
    return generateSimpleAddress(qorId);
  }
}

/**
 * Generate a simple deterministic address without Polkadot keyring
 * Used as fallback when Polkadot packages fail to load
 */
function generateSimpleAddress(qorId: string): string {
  // Create a deterministic hex string from QOR ID
  const encoder = new TextEncoder();
  const data = encoder.encode(`DEMIURGE_QOR_ID:${qorId}`);
  
  // Simple hash function (FNV-1a variant)
  let hash1 = 2166136261;
  let hash2 = 3671253287;
  
  for (let i = 0; i < data.length; i++) {
    hash1 ^= data[i];
    hash1 = Math.imul(hash1, 16777619);
    hash2 ^= data[data.length - 1 - i];
    hash2 = Math.imul(hash2, 16777619);
  }
  
  // Create a 48-character SS58-like address
  const hex1 = (hash1 >>> 0).toString(16).padStart(8, '0');
  const hex2 = (hash2 >>> 0).toString(16).padStart(8, '0');
  const hex3 = ((hash1 ^ hash2) >>> 0).toString(16).padStart(8, '0');
  const hex4 = ((hash1 + hash2) >>> 0).toString(16).padStart(8, '0');
  const hex5 = ((hash1 * hash2) >>> 0).toString(16).padStart(8, '0');
  const hex6 = (Math.abs(hash1 - hash2) >>> 0).toString(16).padStart(8, '0');
  
  // Format as SS58-like address (starts with 5 for generic substrate)
  return `5${hex1}${hex2}${hex3}${hex4}${hex5}${hex6}`.slice(0, 48);
}

/**
 * Get or create blockchain address for QOR ID account
 * 
 * Checks if user has an on-chain address, generates one if not,
 * and optionally links it to their account.
 * 
 * @param user User object from QOR Auth
 * @param linkToAccount If true, saves address to user's account
 * @returns Blockchain address
 */
export async function getOrCreateAddressForQorId(
  user: User,
  linkToAccount: boolean = true
): Promise<string> {
  // Check if user already has an on-chain address
  const existingAddress = user.on_chain?.address || user.on_chain_address;
  
  if (existingAddress) {
    return existingAddress;
  }
  
  // Generate deterministic address from QOR ID
  const address = generateAddressFromQorId(user.qor_id);
  
  // Optionally link to account (requires API endpoint)
  if (linkToAccount) {
    try {
      // Call QOR Auth API to save address
      await linkAddressToQorId(address);
    } catch (error) {
      console.warn('Failed to link address to account:', error);
      // Continue anyway - address is still valid
    }
  }
  
  return address;
}

/**
 * Get keypair for QOR ID (for signing transactions)
 * 
 * WARNING: This uses a deterministic seed. In production, consider:
 * - Using hardware wallets
 * - Encrypting keys in localStorage
 * - Using session keys for temporary authorization
 * 
 * @param qorId QOR ID (e.g., "username#0001")
 * @returns KeyringPair for signing transactions
 */
export function getKeypairForQorId(qorId: string): KeyringPair {
  const keyring = new Keyring({ type: 'sr25519', ss58Format: 42 });
  
  // Use double-slash prefix for derivation path (required by Polkadot keyring)
  const sanitizedId = qorId.replace(/[^a-zA-Z0-9]/g, '_');
  const derivationPath = `//${sanitizedId}`;
  
  try {
    return keyring.addFromUri(derivationPath, { name: qorId }, 'sr25519');
  } catch (error) {
    // Fallback: use seed-based approach
    const encoder = new TextEncoder();
    const data = encoder.encode(`DEMIURGE_QOR_ID:${qorId}`);
    const seed = new Uint8Array(32);
    for (let i = 0; i < data.length; i++) {
      seed[i % 32] ^= data[i];
    }
    return keyring.addFromSeed(seed, { name: qorId }, 'sr25519');
  }
}

/**
 * Check if QOR ID has a linked blockchain address
 */
export function hasLinkedAddress(user: User): boolean {
  return !!(user.on_chain?.address || user.on_chain_address);
}

/**
 * Format QOR ID for display
 */
export function formatQorId(qorId: string): string {
  return qorId; // Already formatted as "username#0001"
}

/**
 * Extract username from QOR ID
 */
export function getUsernameFromQorId(qorId: string): string {
  const parts = qorId.split('#');
  return parts[0] || qorId;
}

/**
 * Extract discriminator from QOR ID
 */
export function getDiscriminatorFromQorId(qorId: string): string {
  const parts = qorId.split('#');
  return parts[1] || '0000';
}

/**
 * Generate keypair from QOR ID (using WASM wallet)
 * Re-exported from wasm-wallet for convenience
 * 
 * @param qorId QOR ID (e.g., "username#0001")
 * @returns Keypair JSON string
 */
export async function generateKeypairFromQorId(qorId: string): Promise<string> {
  return generateKeypairFromQorIdWasm(qorId);
}

/**
 * Multi-wallet support: Link multiple addresses to one QOR ID
 */

export interface LinkedWallet {
  address: string;
  label?: string;
  isPrimary: boolean;
  createdAt: number;
}

/**
 * Get all linked wallets for a QOR ID
 * 
 * @param qorId QOR ID account
 * @returns Array of linked wallets
 */
export async function getLinkedWallets(qorId: string): Promise<LinkedWallet[]> {
  // TODO: Query QOR Auth API for linked wallets
  // For now, return primary address
  const primaryAddress = generateAddressFromQorId(qorId);
  return [{
    address: primaryAddress,
    label: 'Primary Wallet',
    isPrimary: true,
    createdAt: Date.now(),
  }];
}

/**
 * Add a new wallet address to QOR ID account
 * 
 * @param qorId QOR ID account
 * @param address Blockchain address to link
 * @param label Optional label for the wallet
 */
export async function addLinkedWallet(
  qorId: string,
  address: string,
  label?: string
): Promise<void> {
  // TODO: Call QOR Auth API to add linked wallet
  // await qorAuth.addLinkedWallet(address, label);
}

/**
 * Set a wallet as primary
 * 
 * @param qorId QOR ID account
 * @param address Address to set as primary
 */
export async function setPrimaryWallet(qorId: string, address: string): Promise<void> {
  // TODO: Call QOR Auth API to set primary wallet
  // await qorAuth.setPrimaryWallet(address);
}

/**
 * Remove a linked wallet
 * 
 * @param qorId QOR ID account
 * @param address Address to remove
 */
export async function removeLinkedWallet(qorId: string, address: string): Promise<void> {
  // TODO: Call QOR Auth API to remove linked wallet
  // await qorAuth.removeLinkedWallet(address);
}
