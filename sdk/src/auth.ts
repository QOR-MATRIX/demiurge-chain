// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * DemiurgeAuth - Authentication for Demiurge-Cloud
 * 
 * Supports both password-based and keypair-based authentication.
 * Keypair auth uses Ed25519 signatures for a Nostr-style experience.
 */

import { Wallet } from './wallet';
import { bytesToHex, hexToBytes } from './utils';

/**
 * Auth session containing tokens
 */
export interface AuthSession {
  accessToken: string;
  refreshToken: string;
  expiresAt: Date;
  qorId?: string;
  pubkey?: string;
}

/**
 * Challenge response from the server
 */
export interface ChallengeResponse {
  challenge: string;
  expires_at: string;
}

/**
 * Registration result
 */
export interface RegistrationResult {
  qorId: string;
  userId: string;
  pubkey: string;
  onChainAddress: string;
  authMethod: string;
}

/**
 * Auth client options
 */
export interface DemiurgeAuthOptions {
  /** Auth service endpoint */
  authEndpoint?: string;
  /** Request timeout in ms */
  timeout?: number;
}

/**
 * DemiurgeAuth - Unified authentication client
 * 
 * @example
 * ```typescript
 * // Keypair-based login
 * const auth = new DemiurgeAuth({ authEndpoint: 'http://127.0.0.1:8080' });
 * const wallet = Wallet.generate();
 * const session = await auth.loginWithKeypair(wallet);
 * 
 * // Password-based login
 * const session = await auth.loginWithPassword('username', 'password');
 * ```
 */
export class DemiurgeAuth {
  private authEndpoint: string;
  private timeout: number;
  private currentSession: AuthSession | null = null;

  constructor(options: DemiurgeAuthOptions = {}) {
    this.authEndpoint = options.authEndpoint || 'http://127.0.0.1:8080';
    this.timeout = options.timeout || 30000;
  }

  /**
   * Get a challenge for keypair authentication
   */
  async getChallenge(pubkey: string): Promise<ChallengeResponse> {
    const response = await this.fetch(
      `/api/v1/auth/challenge?pubkey=${encodeURIComponent(pubkey)}`,
      { method: 'GET' }
    );
    return response as ChallengeResponse;
  }

  /**
   * Login with keypair (signature-based)
   * 
   * This is the Nostr-style authentication flow:
   * 1. Request a challenge from the server
   * 2. Sign the challenge with your private key
   * 3. Submit the signature to authenticate
   */
  async loginWithKeypair(wallet: Wallet, deviceId?: string): Promise<AuthSession> {
    const pubkey = wallet.publicKey();
    
    // Step 1: Get challenge
    const { challenge, expires_at } = await this.getChallenge(pubkey);
    
    // Step 2: Sign challenge
    const messageBytes = new TextEncoder().encode(challenge);
    const signatureBytes = wallet.sign(messageBytes);
    const signature = bytesToHex(signatureBytes);
    
    // Step 3: Submit login
    const response = await this.fetch('/api/v1/auth/keypair-login', {
      method: 'POST',
      body: JSON.stringify({
        pubkey,
        challenge,
        signature,
        device_id: deviceId,
      }),
    });
    
    const tokens = response as { access_token: string; refresh_token: string; expires_in: number };
    
    this.currentSession = {
      accessToken: tokens.access_token,
      refreshToken: tokens.refresh_token,
      expiresAt: new Date(Date.now() + tokens.expires_in * 1000),
      pubkey,
    };
    
    return this.currentSession;
  }

  /**
   * Register with keypair (creates new account)
   * 
   * This creates a new QOR ID linked to your keypair.
   * You can optionally provide a username, or one will be auto-generated.
   */
  async registerWithKeypair(
    wallet: Wallet,
    username?: string
  ): Promise<RegistrationResult> {
    const pubkey = wallet.publicKey();
    
    // Get challenge
    const { challenge } = await this.getChallenge(pubkey);
    
    // Sign challenge
    const messageBytes = new TextEncoder().encode(challenge);
    const signatureBytes = wallet.sign(messageBytes);
    const signature = bytesToHex(signatureBytes);
    
    // Submit registration
    const response = await this.fetch('/api/v1/auth/keypair-register', {
      method: 'POST',
      body: JSON.stringify({
        pubkey,
        username,
        challenge,
        signature,
      }),
    });
    
    const result = response as {
      qor_id: string;
      user_id: string;
      pubkey: string;
      on_chain_address: string;
      auth_method: string;
    };
    
    return {
      qorId: result.qor_id,
      userId: result.user_id,
      pubkey: result.pubkey,
      onChainAddress: result.on_chain_address,
      authMethod: result.auth_method,
    };
  }

  /**
   * Login with password (traditional)
   */
  async loginWithPassword(
    identifier: string,
    password: string,
    deviceId?: string
  ): Promise<AuthSession> {
    const response = await this.fetch('/api/v1/auth/login', {
      method: 'POST',
      body: JSON.stringify({
        identifier,
        password,
        device_id: deviceId,
      }),
    });
    
    const tokens = response as { access_token: string; refresh_token: string; expires_in: number };
    
    this.currentSession = {
      accessToken: tokens.access_token,
      refreshToken: tokens.refresh_token,
      expiresAt: new Date(Date.now() + tokens.expires_in * 1000),
    };
    
    return this.currentSession;
  }

  /**
   * Register with password (traditional)
   */
  async registerWithPassword(
    username: string,
    password: string,
    email?: string
  ): Promise<{ qorId: string; backupCode?: string }> {
    const response = await this.fetch('/api/v1/auth/register', {
      method: 'POST',
      body: JSON.stringify({
        username,
        password,
        email,
      }),
    });
    
    const result = response as { qor_id: string; backup_code?: string };
    
    return {
      qorId: result.qor_id,
      backupCode: result.backup_code,
    };
  }

  /**
   * Refresh the current session
   */
  async refresh(): Promise<AuthSession> {
    if (!this.currentSession) {
      throw new Error('No active session to refresh');
    }
    
    const response = await this.fetch('/api/v1/auth/refresh', {
      method: 'POST',
      body: JSON.stringify({
        refresh_token: this.currentSession.refreshToken,
      }),
    });
    
    const tokens = response as { access_token: string; refresh_token: string; expires_in: number };
    
    this.currentSession = {
      ...this.currentSession,
      accessToken: tokens.access_token,
      refreshToken: tokens.refresh_token,
      expiresAt: new Date(Date.now() + tokens.expires_in * 1000),
    };
    
    return this.currentSession;
  }

  /**
   * Logout and invalidate current session
   */
  async logout(): Promise<void> {
    if (this.currentSession) {
      await this.fetch('/api/v1/auth/logout', {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${this.currentSession.accessToken}`,
        },
      }).catch(() => {
        // Ignore logout errors, just clear local session
      });
    }
    this.currentSession = null;
  }

  /**
   * Get current session
   */
  getSession(): AuthSession | null {
    return this.currentSession;
  }

  /**
   * Check if session is valid and not expired
   */
  isAuthenticated(): boolean {
    if (!this.currentSession) return false;
    return this.currentSession.expiresAt > new Date();
  }

  /**
   * Get access token for API calls
   */
  getAccessToken(): string | null {
    return this.currentSession?.accessToken || null;
  }

  /**
   * Link keypair to existing account (requires active session)
   */
  async linkKeypair(wallet: Wallet): Promise<void> {
    if (!this.currentSession) {
      throw new Error('Must be authenticated to link keypair');
    }
    
    const pubkey = wallet.publicKey();
    const { challenge } = await this.getChallenge(pubkey);
    
    const messageBytes = new TextEncoder().encode(challenge);
    const signatureBytes = wallet.sign(messageBytes);
    const signature = bytesToHex(signatureBytes);
    
    await this.fetch('/api/v1/auth/link-keypair', {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${this.currentSession.accessToken}`,
      },
      body: JSON.stringify({
        pubkey,
        challenge,
        signature,
      }),
    });
  }

  /**
   * Internal fetch helper
   */
  private async fetch(
    path: string,
    options: RequestInit
  ): Promise<unknown> {
    const url = `${this.authEndpoint}${path}`;
    
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);
    
    try {
      const response = await fetch(url, {
        ...options,
        signal: controller.signal,
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
      });
      
      clearTimeout(timeoutId);
      
      if (!response.ok) {
        const error = await response.json().catch(() => ({ message: 'Unknown error' })) as { message?: string };
        throw new Error(error.message || `HTTP ${response.status}`);
      }
      
      return response.json();
    } catch (err) {
      clearTimeout(timeoutId);
      throw err;
    }
  }
}

/**
 * Create a quick wallet for testing/development
 */
export function createTestWallet(): { wallet: Wallet; pubkey: string; privateKey: string } {
  const wallet = Wallet.generate();
  return {
    wallet,
    pubkey: wallet.publicKey(),
    privateKey: wallet.exportPrivateKey(),
  };
}
