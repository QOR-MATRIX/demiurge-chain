// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Session management for Demiurge CLI
 * 
 * Persists user preferences, active wallet, and auth state.
 */

import Conf from 'conf';

/**
 * Session state (in-memory)
 */
export interface SessionState {
  /** Active wallet address */
  activeWallet: string | null;
  /** Active wallet private key (stored securely) */
  walletPrivateKey: string | null;
  /** QOR ID (e.g., username#1234) */
  qorId: string | null;
  /** Auth token for QOR Auth */
  authToken: string | null;
  /** Connected RPC endpoint */
  rpcEndpoint: string;
  /** Auth service endpoint */
  authEndpoint: string;
  /** Whether user is authenticated */
  isAuthenticated: boolean;
}

/**
 * Persistent config (stored on disk)
 */
export interface PersistentConfig {
  rpcEndpoint: string;
  authEndpoint: string;
  defaultWalletPath: string | null;
  theme: 'dark' | 'light';
  showAnimations: boolean;
  lastQorId: string | null;
}

const config = new Conf<PersistentConfig>({
  projectName: 'demiurge-cli',
  defaults: {
    rpcEndpoint: 'http://127.0.0.1:9944',
    authEndpoint: 'http://127.0.0.1:8080',
    defaultWalletPath: null,
    theme: 'dark',
    showAnimations: true,
    lastQorId: null,
  },
});

/**
 * Current session state (in-memory only)
 */
export const session: SessionState = {
  activeWallet: null,
  walletPrivateKey: null,
  qorId: null,
  authToken: null,
  rpcEndpoint: config.get('rpcEndpoint'),
  authEndpoint: config.get('authEndpoint'),
  isAuthenticated: false,
};

/**
 * Get persistent config value
 */
export function getConfig<K extends keyof PersistentConfig>(key: K): PersistentConfig[K] {
  return config.get(key);
}

/**
 * Set persistent config value
 */
export function setConfig<K extends keyof PersistentConfig>(key: K, value: PersistentConfig[K]): void {
  config.set(key, value);
}

/**
 * Update session with wallet
 */
export function setWallet(address: string, privateKey: string): void {
  session.activeWallet = address;
  session.walletPrivateKey = privateKey;
}

/**
 * Clear wallet from session
 */
export function clearWallet(): void {
  session.activeWallet = null;
  session.walletPrivateKey = null;
}

/**
 * Set auth credentials
 */
export function setAuth(qorId: string, token: string): void {
  session.qorId = qorId;
  session.authToken = token;
  session.isAuthenticated = true;
  config.set('lastQorId', qorId);
}

/**
 * Clear auth
 */
export function clearAuth(): void {
  session.qorId = null;
  session.authToken = null;
  session.isAuthenticated = false;
}

/**
 * Get session summary for display
 */
export function getSessionSummary(): { wallet?: string; qorId?: string } {
  return {
    wallet: session.activeWallet || undefined,
    qorId: session.qorId || undefined,
  };
}

/**
 * Check if session has wallet
 */
export function hasWallet(): boolean {
  return session.activeWallet !== null;
}

/**
 * Check if session is authenticated
 */
export function isAuthenticated(): boolean {
  return session.isAuthenticated && session.authToken !== null;
}

/**
 * Reset session to defaults
 */
export function resetSession(): void {
  session.activeWallet = null;
  session.walletPrivateKey = null;
  session.qorId = null;
  session.authToken = null;
  session.isAuthenticated = false;
  session.rpcEndpoint = config.get('rpcEndpoint');
  session.authEndpoint = config.get('authEndpoint');
}

/**
 * Get prompt prefix based on session state
 */
export function getPrompt(): string {
  if (session.qorId) {
    return `${session.qorId}> `;
  } else if (session.activeWallet) {
    return `${session.activeWallet.slice(0, 8)}...> `;
  }
  return 'demiurge> ';
}
