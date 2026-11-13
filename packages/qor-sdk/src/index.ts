// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
import axios, { AxiosInstance } from 'axios';

// Export leveling system
export * from './leveling';

// Export asset management
export * from './assets';

// Default to production server, fallback to localhost for development
function getDefaultApiUrl(): string {
  if (process.env.NEXT_PUBLIC_QOR_AUTH_URL) {
    return process.env.NEXT_PUBLIC_QOR_AUTH_URL;
  }
  // Check if we're in browser environment
  if (typeof globalThis !== 'undefined' && 'window' in globalThis) {
    const win = (globalThis as any).window;
    if (win?.location?.hostname === 'qorsync.dev') {
      return 'http://127.0.0.1:3000/api/v1';
    }
  }
  return 'http://localhost:8080/api/v1';
}

const DEFAULT_API_URL = getDefaultApiUrl();

export interface QorId {
  username: string;
  discriminator: number;
}

export interface User {
  id: string;
  qor_id: string;
  email: string;
  role: 'user' | 'moderator' | 'admin' | 'god';
  created_at?: string;
  updated_at?: string;
  avatar_url?: string | null;
  display_name?: string;
  bio?: string;
  on_chain?: {
    address: string;
    cgt_balance?: string;
  };
  on_chain_address?: string; // Legacy field name
}

// Response from the QOR Auth API
export interface ApiTokenResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  token_type: string;
}

// Convenience interface for SDK users
export interface LoginResponse {
  token: string;
  refresh_token: string;
  user: User;
}

export interface RegisterRequest {
  email?: string; // Optional - if not provided, account is username-only
  password: string;
  username: string;
}

export interface RegisterResponse {
  qor_id: string;
  email_verified: boolean;
  backup_code?: string; // Only for username-only accounts
  email_verification_token?: string; // Only in dev, remove in production
  message: string;
}

export class QorAuthClient {
  private client: AxiosInstance;
  private token: string | null = null;
  private refreshTokenValue: string | null = null;
  private refreshPromise: Promise<void> | null = null;
  private tokenExpiryMs: number = 0;

  constructor(baseURL: string = DEFAULT_API_URL) {
    this.client = axios.create({
      baseURL,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Load stored tokens on init
    this._loadStoredTokens();

    // Add request interceptor to include token and auto-refresh
    this.client.interceptors.request.use(async (config) => {
      // Skip auth for auth endpoints
      const isAuthEndpoint = config.url?.includes('/auth/');
      
      if (!isAuthEndpoint && this.token) {
        // Check if token is about to expire (within 5 minutes)
        const fiveMinutes = 5 * 60 * 1000;
        if (this.tokenExpiryMs && Date.now() > this.tokenExpiryMs - fiveMinutes) {
          // Token expiring soon, try to refresh
          await this._autoRefresh();
        }
        config.headers.Authorization = `Bearer ${this.token}`;
      }
      return config;
    });
  }

  private _loadStoredTokens(): void {
    if (typeof globalThis !== 'undefined' && 'window' in globalThis && 'localStorage' in (globalThis as any).window) {
      const localStorage = (globalThis as any).window.localStorage;
      this.token = localStorage.getItem('qor_token');
      this.refreshTokenValue = localStorage.getItem('qor_refresh_token');
      const expiryStr = localStorage.getItem('qor_token_expiry');
      this.tokenExpiryMs = expiryStr ? parseInt(expiryStr, 10) : 0;
    }
  }

  private async _autoRefresh(): Promise<void> {
    // Prevent multiple simultaneous refreshes
    if (this.refreshPromise) {
      return this.refreshPromise;
    }

    if (!this.refreshTokenValue) {
      return;
    }

    this.refreshPromise = (async () => {
      try {
        console.log('[QOR Auth] Auto-refreshing token...');
        await this.refreshToken(this.refreshTokenValue!);
        console.log('[QOR Auth] Token refreshed successfully');
      } catch (error) {
        console.warn('[QOR Auth] Auto-refresh failed:', error);
        // Don't clear tokens here - let the 401 handler do it
      } finally {
        this.refreshPromise = null;
      }
    })();

    return this.refreshPromise;
  }

  setToken(token: string, refreshToken?: string, expiresIn?: number) {
    this.token = token;
    
    // Calculate and store expiry time (default 1 hour if not provided)
    const expiryMs = Date.now() + ((expiresIn || 3600) * 1000);
    this.tokenExpiryMs = expiryMs;
    
    if (refreshToken) {
      this.refreshTokenValue = refreshToken;
    }
    
    // Store in localStorage for persistence across page refreshes
    if (typeof globalThis !== 'undefined' && 'window' in globalThis && 'localStorage' in (globalThis as any).window) {
      const localStorage = (globalThis as any).window.localStorage;
      const document = (globalThis as any).window.document;
      
      localStorage.setItem('qor_token', token);
      localStorage.setItem('qor_token_expiry', expiryMs.toString());
      if (refreshToken) {
        localStorage.setItem('qor_refresh_token', refreshToken);
      }
      
      // Sync token to cookie for middleware/SSR access
      // Use refresh token expiry (7 days) or token expiry, whichever is longer
      const cookieMaxAge = refreshToken ? 604800 : Math.ceil((expiresIn || 3600));
      if (document) {
        document.cookie = `qor_token=${token}; path=/; max-age=${cookieMaxAge}; SameSite=Lax`;
      }
    }
  }

  getToken(): string | null {
    if (this.token) return this.token;
    if (typeof globalThis !== 'undefined' && 'window' in globalThis && 'localStorage' in (globalThis as any).window) {
      return (globalThis as any).window.localStorage.getItem('qor_token');
    }
    return null;
  }

  clearToken() {
    this.token = null;
    this.refreshTokenValue = null;
    this.tokenExpiryMs = 0;
    
    if (typeof globalThis !== 'undefined' && 'window' in globalThis && 'localStorage' in (globalThis as any).window) {
      const localStorage = (globalThis as any).window.localStorage;
      const document = (globalThis as any).window.document;
      
      localStorage.removeItem('qor_token');
      localStorage.removeItem('qor_refresh_token');
      localStorage.removeItem('qor_token_expiry');
      
      // Clear cookie by setting expired date
      if (document) {
        document.cookie = 'qor_token=; path=/; max-age=0; SameSite=Lax';
      }
    }
  }

  /**
   * Decode JWT token to get user data (without verification)
   * Useful for offline mode when API is unavailable
   */
  getTokenData(): { qor_id: string; user_id: string; role: string; exp: number } | null {
    const token = this.getToken();
    if (!token) return null;
    
    try {
      // JWT format: header.payload.signature
      const parts = token.split('.');
      if (parts.length !== 3) return null;
      
      // Decode base64url payload
      const payload = parts[1];
      const decoded = atob(payload.replace(/-/g, '+').replace(/_/g, '/'));
      const data = JSON.parse(decoded);
      
      return {
        qor_id: data.qor_id || data.sub,
        user_id: data.user_id || data.sub,
        role: data.role || 'user',
        exp: data.exp || 0,
      };
    } catch (e) {
      console.warn('Failed to decode token:', e);
      return null;
    }
  }

  async login(identifier: string, password: string): Promise<LoginResponse> {
    const response = await this.client.post<ApiTokenResponse>('/auth/login', {
      identifier, // Can be email or username
      password,
    });
    
    const { access_token, refresh_token, expires_in } = response.data;
    
    if (access_token) {
      // Store token with refresh token and expiry for session persistence
      this.setToken(access_token, refresh_token, expires_in || 3600);
    }
    
    // Fetch user profile to complete the login response
    let user: User;
    try {
      user = await this.getProfile();
    } catch (e) {
      // If profile fetch fails, create a minimal user object from token
      const tokenData = this.getTokenData();
      user = {
        id: tokenData?.user_id || '',
        qor_id: tokenData?.qor_id || identifier,
        email: '',
        role: (tokenData?.role as User['role']) || 'user',
      };
    }
    
    return {
      token: access_token,
      refresh_token,
      user,
    };
  }

  async register(data: RegisterRequest): Promise<RegisterResponse> {
    try {
      const response = await this.client.post<RegisterResponse>('/auth/register', {
        email: data.email || undefined, // Optional
        username: data.username,
        password: data.password,
      });
      
      // Note: Registration doesn't auto-login, user needs to login after email verification
      return response.data;
    } catch (error: any) {
      if (error.code === 'ERR_NETWORK' || error.message?.includes('Network Error')) {
        throw new Error('QOR Auth service is not available. Please ensure the service is running on port 8080.');
      }
      // Handle API errors - the error format is { error: { code, message } }
      const apiError = error.response?.data?.error;
      if (apiError) {
        // Extract message from error object
        const message = typeof apiError === 'string' ? apiError : apiError.message || apiError.code || 'Registration failed';
        throw new Error(message);
      }
      if (error.response?.data?.message) {
        throw new Error(error.response.data.message);
      }
      throw error;
    }
  }

  async forgotPassword(identifier: string): Promise<{ requires_backup_code?: boolean; reset_token?: string; message: string }> {
    try {
      const response = await this.client.post<{ requires_backup_code?: boolean; reset_token?: string; message: string }>('/auth/forgot-password', {
        identifier,
      });
      return response.data;
    } catch (error: any) {
      if (error.code === 'ERR_NETWORK' || error.message?.includes('Network Error')) {
        throw new Error('QOR Auth service is not available.');
      }
      if (error.response?.data?.message) {
        throw new Error(error.response.data.message);
      }
      throw error;
    }
  }

  async resetPasswordWithBackup(username: string, backupCode: string, newPassword: string): Promise<void> {
    try {
      await this.client.post('/auth/reset-password-backup', {
        username,
        backup_code: backupCode,
        new_password: newPassword,
      });
    } catch (error: any) {
      if (error.response?.data?.message) {
        throw new Error(error.response.data.message);
      }
      throw error;
    }
  }

  async resetPasswordWithToken(token: string, newPassword: string): Promise<void> {
    try {
      await this.client.post('/auth/reset-password', {
        token,
        new_password: newPassword,
      });
    } catch (error: any) {
      if (error.response?.data?.message) {
        throw new Error(error.response.data.message);
      }
      throw error;
    }
  }

  async verifyEmail(token: string): Promise<void> {
    try {
      await this.client.post('/auth/verify-email', { token });
    } catch (error: any) {
      if (error.response?.data?.message) {
        throw new Error(error.response.data.message);
      }
      throw error;
    }
  }

  async getProfile(): Promise<User> {
    // First, get core user data from JWT token (always reliable)
    const tokenData = this.getTokenData();
    
    try {
      const response = await this.client.get<User>('/profile');
      const profileData = response.data;
      
      // Merge profile data with token data, preferring token for core fields
      // This ensures we always have the correct qor_id even if profile API returns placeholder
      return {
        ...profileData,
        id: tokenData?.user_id || profileData.id,
        qor_id: tokenData?.qor_id || profileData.qor_id,
        role: (tokenData?.role as User['role']) || profileData.role || 'user',
      };
    } catch (error: any) {
      // Handle network errors gracefully - fall back to token data
      if (error.code === 'ERR_NETWORK' || error.message?.includes('Network Error')) {
        // If we have token data, use it instead of throwing
        if (tokenData) {
          return {
            id: tokenData.user_id,
            qor_id: tokenData.qor_id,
            email: '',
            role: (tokenData.role as User['role']) || 'user',
          };
        }
        throw new Error('QOR Auth service is not available. Please ensure the service is running on port 8080.');
      }
      throw error;
    }
  }

  async refreshToken(refreshToken: string): Promise<LoginResponse> {
    const response = await this.client.post<ApiTokenResponse>('/auth/refresh', {
      refresh_token: refreshToken,
    });
    
    const { access_token, refresh_token: newRefreshToken, expires_in } = response.data;
    
    if (access_token) {
      // Store new token with updated refresh token and expiry
      this.setToken(access_token, newRefreshToken, expires_in || 3600);
    }
    
    // Fetch updated user profile
    let user: User;
    try {
      user = await this.getProfile();
    } catch (e) {
      const tokenData = this.getTokenData();
      user = {
        id: tokenData?.user_id || '',
        qor_id: tokenData?.qor_id || '',
        email: '',
        role: (tokenData?.role as User['role']) || 'user',
      };
    }
    
    return {
      token: access_token,
      refresh_token: newRefreshToken,
      user,
    };
  }

  async logout(): Promise<void> {
    try {
      await this.client.post('/auth/logout');
    } finally {
      this.clearToken();
    }
  }

  async isGod(): Promise<boolean> {
    try {
      const profile = await this.getProfile();
      return profile.role === 'god';
    } catch {
      return false;
    }
  }

  async isAdmin(): Promise<boolean> {
    try {
      const profile = await this.getProfile();
      return profile.role === 'admin' || profile.role === 'god';
    } catch {
      return false;
    }
  }

  isAuthenticated(): boolean {
    const token = this.getToken();
    if (!token) return false;
    
    // Check if token is expired
    if (this.tokenExpiryMs && Date.now() > this.tokenExpiryMs) {
      // Token expired - try to refresh if we have a refresh token
      if (this.refreshTokenValue) {
        // Don't await - just return true and let the interceptor handle refresh
        return true;
      }
      return false;
    }
    
    return true;
  }

  /**
   * Check if token will expire within the given timeframe
   */
  isTokenExpiringSoon(withinMs: number = 5 * 60 * 1000): boolean {
    if (!this.tokenExpiryMs) return false;
    return Date.now() > this.tokenExpiryMs - withinMs;
  }

  /**
   * Get the stored refresh token
   */
  getRefreshToken(): string | null {
    return this.refreshTokenValue;
  }

  /**
   * Manually trigger token refresh
   */
  async manualRefresh(): Promise<boolean> {
    if (!this.refreshTokenValue) return false;
    
    try {
      await this.refreshToken(this.refreshTokenValue);
      return true;
    } catch (error) {
      console.error('[QOR Auth] Manual refresh failed:', error);
      return false;
    }
  }

  /**
   * Update user profile
   */
  async updateProfile(data: { display_name?: string; bio?: string }): Promise<User> {
    try {
      const response = await this.client.put<User>('/profile', data);
      return response.data;
    } catch (error: any) {
      if (error.code === 'ERR_NETWORK' || error.message?.includes('Network Error')) {
        throw new Error('QOR Auth service is not available.');
      }
      if (error.response?.data?.message) {
        throw new Error(error.response.data.message);
      }
      throw error;
    }
  }

  /**
   * Change user PIN
   */
  async changePin(currentPin: string, newPin: string): Promise<void> {
    try {
      await this.client.post('/profile/change-pin', {
        current_pin: currentPin,
        new_pin: newPin,
      });
    } catch (error: any) {
      if (error.code === 'ERR_NETWORK' || error.message?.includes('Network Error')) {
        throw new Error('QOR Auth service is not available.');
      }
      if (error.response?.data?.message) {
        throw new Error(error.response.data.message);
      }
      throw error;
    }
  }

  async checkUsername(username: string): Promise<{ available: boolean; username: string }> {
    try {
      const response = await this.client.post<{ available: boolean; username: string }>('/auth/check-username', {
        username,
      });
      return response.data;
    } catch (error: any) {
      // If service is not available, assume username is available (for offline mode)
      if (error.code === 'ERR_NETWORK' || error.message?.includes('Network Error')) {
        return { available: true, username: username.toLowerCase() };
      }
      throw error;
    }
  }

  async checkEmail(email: string): Promise<{ available: boolean; email: string; reason?: string }> {
    try {
      const response = await this.client.post<{ available: boolean; email: string; reason?: string }>('/auth/check-email', {
        email,
      });
      return response.data;
    } catch (error: any) {
      // If service is not available, assume email is available (for offline mode)
      if (error.code === 'ERR_NETWORK' || error.message?.includes('Network Error')) {
        return { available: true, email: email.toLowerCase() };
      }
      throw error;
    }
  }

  /**
   * Upload avatar image and mint as DRC-369 NFT
   * 
   * @param file Image file to upload
   * @param qorId User's QOR ID for the NFT metadata
   * @returns Avatar URL
   */
  async uploadAvatar(file: File, qorId: string): Promise<string> {
    try {
      const formData = new FormData();
      formData.append('avatar', file);
      formData.append('qor_id', qorId);

      const response = await this.client.post<{ avatar_url: string; asset_uuid?: string }>(
        '/profile/avatar',
        formData,
        {
          headers: {
            'Content-Type': 'multipart/form-data',
          },
        }
      );

      return response.data.avatar_url;
    } catch (error: any) {
      if (error.code === 'ERR_NETWORK' || error.message?.includes('Network Error')) {
        throw new Error('QOR Auth service is not available. Please ensure the service is running on port 8080.');
      }
      if (error.response?.data?.message) {
        throw new Error(error.response.data.message);
      }
      if (error.response?.data?.error) {
        throw new Error(error.response.data.error);
      }
      throw error;
    }
  }
}

// Export singleton instance
export const qorAuth = new QorAuthClient();

// Export default for convenience
export default qorAuth;
