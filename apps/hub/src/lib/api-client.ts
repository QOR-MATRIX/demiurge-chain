// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Centralized API Client
 * 
 * Handles all HTTP requests with:
 * - Automatic token injection
 * - Token refresh on 401
 * - Retry logic with exponential backoff
 * - Request deduplication
 * - Consistent error handling
 */

import axios, { AxiosError, AxiosInstance, AxiosRequestConfig, InternalAxiosRequestConfig } from 'axios';

// ============================================================================
// Types
// ============================================================================

export interface ApiError {
  message: string;
  code: string;
  statusCode: number;
  details?: unknown;
}

export class AppError extends Error {
  public readonly statusCode: number;
  public readonly code: string;
  public readonly details?: unknown;
  public readonly isNetworkError: boolean;
  public readonly isAuthError: boolean;

  constructor(
    message: string,
    statusCode: number = 500,
    code: string = 'UNKNOWN_ERROR',
    details?: unknown
  ) {
    super(message);
    this.name = 'AppError';
    this.statusCode = statusCode;
    this.code = code;
    this.details = details;
    this.isNetworkError = code === 'NETWORK_ERROR' || statusCode === 0;
    this.isAuthError = statusCode === 401 || code === 'AUTH_EXPIRED';
  }
}

// ============================================================================
// Error Message Mapping
// ============================================================================

const ERROR_MESSAGES: Record<string, string> = {
  NETWORK_ERROR: 'Unable to connect. Please check your internet connection.',
  AUTH_EXPIRED: 'Your session has expired. Please log in again.',
  RATE_LIMITED: 'Too many requests. Please wait a moment.',
  INVALID_ADDRESS: 'The wallet address format is invalid.',
  INVALID_SIGNATURE: 'Transaction signature is invalid.',
  INSUFFICIENT_BALANCE: 'Insufficient balance for this transaction.',
  NOT_FOUND: 'The requested resource was not found.',
  VALIDATION_ERROR: 'Please check your input and try again.',
  SERVER_ERROR: 'Server error. Please try again later.',
  TIMEOUT: 'Request timed out. Please try again.',
};

export function getUserFriendlyError(error: unknown): string {
  if (error instanceof AppError) {
    return ERROR_MESSAGES[error.code] || error.message;
  }

  if (error instanceof Error) {
    if (error.message.includes('fetch') || error.message.includes('network')) {
      return ERROR_MESSAGES.NETWORK_ERROR;
    }
    if (error.message.includes('timeout')) {
      return ERROR_MESSAGES.TIMEOUT;
    }
    // Don't expose raw technical messages
    if (error.message.includes('RPC') || error.message.includes('0x')) {
      return 'Transaction failed. Please try again.';
    }
    return error.message;
  }

  return 'An unexpected error occurred. Please try again.';
}

// ============================================================================
// Request Deduplication
// ============================================================================

const pendingRequests = new Map<string, Promise<unknown>>();

function getRequestKey(config: AxiosRequestConfig): string {
  return `${config.method}-${config.url}-${JSON.stringify(config.params || {})}-${JSON.stringify(config.data || {})}`;
}

// ============================================================================
// Retry Logic
// ============================================================================

interface RetryConfig {
  maxRetries: number;
  baseDelay: number;
  maxDelay: number;
  retryOn: number[];
}

const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxRetries: 3,
  baseDelay: 1000,
  maxDelay: 10000,
  retryOn: [408, 429, 500, 502, 503, 504],
};

async function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function withRetry<T>(
  fn: () => Promise<T>,
  config: RetryConfig = DEFAULT_RETRY_CONFIG
): Promise<T> {
  let lastError: Error | undefined;

  for (let attempt = 0; attempt <= config.maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error as Error;

      if (error instanceof AppError) {
        // Don't retry auth errors or client errors (except rate limiting)
        if (error.isAuthError || (error.statusCode >= 400 && error.statusCode < 500 && error.statusCode !== 429)) {
          throw error;
        }

        // Check if we should retry this status code
        if (!config.retryOn.includes(error.statusCode)) {
          throw error;
        }
      }

      if (attempt < config.maxRetries) {
        const delayMs = Math.min(
          config.baseDelay * Math.pow(2, attempt),
          config.maxDelay
        );
        await delay(delayMs);
      }
    }
  }

  throw lastError;
}

// ============================================================================
// API Client Factory
// ============================================================================

function createApiClient(baseURL: string): AxiosInstance {
  const client = axios.create({
    baseURL,
    timeout: 15000,
    headers: {
      'Content-Type': 'application/json',
    },
  });

  // Request interceptor - add auth token
  client.interceptors.request.use(
    (config: InternalAxiosRequestConfig) => {
      // Add auth token if available
      if (typeof window !== 'undefined') {
        const token = localStorage.getItem('qor_token');
        if (token && config.headers) {
          config.headers.Authorization = `Bearer ${token}`;
        }
      }
      return config;
    },
    (error) => Promise.reject(error)
  );

  // Response interceptor - handle errors
  client.interceptors.response.use(
    (response) => response,
    async (error: AxiosError<{ message?: string; code?: string; error?: string }>) => {
      // Network error
      if (!error.response) {
        throw new AppError(
          'Network connection failed',
          0,
          'NETWORK_ERROR'
        );
      }

      const status = error.response.status;
      const data = error.response.data;
      const message = data?.message || data?.error || error.message;

      // Handle 401 - try to refresh token
      if (status === 401 && typeof window !== 'undefined') {
        const refreshToken = localStorage.getItem('qor_refresh_token');
        
        if (refreshToken && error.config && !error.config.headers?.['X-Retry']) {
          try {
            // Try to refresh
            const refreshResponse = await axios.post(
              `${baseURL}/auth/refresh`,
              { refresh_token: refreshToken }
            );
            
            if (refreshResponse.data?.access_token) {
              localStorage.setItem('qor_token', refreshResponse.data.access_token);
              if (refreshResponse.data.refresh_token) {
                localStorage.setItem('qor_refresh_token', refreshResponse.data.refresh_token);
              }
              
              // Retry original request
              error.config.headers = error.config.headers || {};
              error.config.headers['X-Retry'] = 'true';
              error.config.headers.Authorization = `Bearer ${refreshResponse.data.access_token}`;
              return client.request(error.config);
            }
          } catch (refreshError) {
            // Refresh failed, clear tokens
            localStorage.removeItem('qor_token');
            localStorage.removeItem('qor_refresh_token');
          }
        }
        
        throw new AppError(message, 401, 'AUTH_EXPIRED');
      }

      // Map status codes to error codes
      const codeMap: Record<number, string> = {
        400: 'VALIDATION_ERROR',
        403: 'FORBIDDEN',
        404: 'NOT_FOUND',
        429: 'RATE_LIMITED',
        500: 'SERVER_ERROR',
        502: 'SERVER_ERROR',
        503: 'SERVER_ERROR',
      };

      throw new AppError(
        message,
        status,
        data?.code || codeMap[status] || 'UNKNOWN_ERROR',
        data
      );
    }
  );

  return client;
}

// ============================================================================
// API Client Instance
// ============================================================================

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 
  (typeof window !== 'undefined' && window.location?.hostname === 'localhost'
    ? 'http://localhost:8080/api/v1'
    : 'http://127.0.0.1:3000/api/v1');

const apiClient = createApiClient(API_BASE_URL);

// ============================================================================
// Typed API Methods
// ============================================================================

export const api = {
  /**
   * GET request with automatic retry
   */
  get: async <T>(url: string, config?: AxiosRequestConfig): Promise<T> => {
    return withRetry(async () => {
      const response = await apiClient.get<T>(url, config);
      return response.data;
    });
  },

  /**
   * POST request
   */
  post: async <T>(url: string, data?: unknown, config?: AxiosRequestConfig): Promise<T> => {
    const response = await apiClient.post<T>(url, data, config);
    return response.data;
  },

  /**
   * PUT request
   */
  put: async <T>(url: string, data?: unknown, config?: AxiosRequestConfig): Promise<T> => {
    const response = await apiClient.put<T>(url, data, config);
    return response.data;
  },

  /**
   * PATCH request
   */
  patch: async <T>(url: string, data?: unknown, config?: AxiosRequestConfig): Promise<T> => {
    const response = await apiClient.patch<T>(url, data, config);
    return response.data;
  },

  /**
   * DELETE request
   */
  delete: async <T>(url: string, config?: AxiosRequestConfig): Promise<T> => {
    const response = await apiClient.delete<T>(url, config);
    return response.data;
  },

  /**
   * Deduplicated GET request (prevents multiple identical requests)
   */
  getDeduped: async <T>(url: string, config?: AxiosRequestConfig): Promise<T> => {
    const key = getRequestKey({ method: 'get', url, ...config });
    
    if (pendingRequests.has(key)) {
      return pendingRequests.get(key) as Promise<T>;
    }
    
    const promise = api.get<T>(url, config).finally(() => {
      pendingRequests.delete(key);
    });
    
    pendingRequests.set(key, promise);
    return promise;
  },
};

export default api;
