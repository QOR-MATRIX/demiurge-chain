// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * DRC-369 Client
 * 
 * Main client for interacting with DRC-369 NFTs on the Demiurge blockchain.
 * Includes CVP security awareness and real-time event streaming.
 */

import axios, { AxiosInstance, AxiosError } from 'axios';
import { EventEmitter } from 'eventemitter3';
import {
  Drc369Nft,
  NftId,
  Address,
  TxHash,
  MintParams,
  TransferParams,
  UpdateStateParams,
  AddXpParams,
  AddResourceParams,
  NestParams,
  DelegateParams,
  TxResult,
  PaginatedResponse,
  NftQueryFilters,
  Drc369Event,
  Drc369EventType,
  CvpStatus,
  Permission,
  NftResource,
  calculateLevel,
} from './types';

// =============================================================================
// CLIENT CONFIGURATION
// =============================================================================

export interface Drc369ClientConfig {
  /** Base URL for the DRC-369 API */
  baseUrl?: string;
  
  /** Authentication token */
  token?: string;
  
  /** WebSocket URL for real-time events */
  wsUrl?: string;
  
  /** Request timeout in ms */
  timeout?: number;
  
  /** Whether to auto-reconnect WebSocket */
  autoReconnect?: boolean;
  
  /** Custom headers */
  headers?: Record<string, string>;
}

const DEFAULT_CONFIG: Required<Drc369ClientConfig> = {
  baseUrl: 'http://127.0.0.1:3000/api/v1/drc369',
  token: '',
  wsUrl: 'http://127.0.0.1:3000/ws/drc369',
  timeout: 30000,
  autoReconnect: true,
  headers: {},
};

// =============================================================================
// CLIENT IMPLEMENTATION
// =============================================================================

export interface Drc369ClientEvents {
  'connected': () => void;
  'disconnected': () => void;
  'error': (error: Error) => void;
  'event': (event: Drc369Event) => void;
  'mint': (event: Drc369Event) => void;
  'transfer': (event: Drc369Event) => void;
  'burn': (event: Drc369Event) => void;
  'xp_added': (event: Drc369Event) => void;
  'level_up': (event: Drc369Event) => void;
  'state_updated': (event: Drc369Event) => void;
  'cvp_mutation': (event: Drc369Event) => void;
  'threat_detected': (event: Drc369Event) => void;
}

export class Drc369Client extends EventEmitter<Drc369ClientEvents> {
  private http: AxiosInstance;
  private ws: WebSocket | null = null;
  private config: Required<Drc369ClientConfig>;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  private subscriptions: Set<string> = new Set();

  constructor(config: Drc369ClientConfig = {}) {
    super();
    
    this.config = { ...DEFAULT_CONFIG, ...config };
    
    // Initialize HTTP client
    this.http = axios.create({
      baseURL: this.config.baseUrl,
      timeout: this.config.timeout,
      headers: {
        'Content-Type': 'application/json',
        ...this.config.headers,
      },
    });

    // Add auth interceptor
    this.http.interceptors.request.use((cfg) => {
      if (this.config.token) {
        cfg.headers.Authorization = `Bearer ${this.config.token}`;
      }
      return cfg;
    });

    // Add error interceptor
    this.http.interceptors.response.use(
      (response) => response,
      (error: AxiosError) => {
        const message = (error.response?.data as any)?.error || error.message;
        return Promise.reject(new Error(message));
      }
    );
  }

  // ===========================================================================
  // AUTHENTICATION
  // ===========================================================================

  /**
   * Set authentication token
   */
  setToken(token: string): void {
    this.config.token = token;
  }

  /**
   * Clear authentication token
   */
  clearToken(): void {
    this.config.token = '';
  }

  /**
   * Check if client is authenticated
   */
  isAuthenticated(): boolean {
    return !!this.config.token;
  }

  // ===========================================================================
  // NFT OPERATIONS
  // ===========================================================================

  /**
   * Mint a new DRC-369 NFT
   */
  async mint(params: MintParams): Promise<TxResult<Drc369Nft>> {
    const response = await this.http.post<TxResult<Drc369Nft>>('/nfts', params);
    return response.data;
  }

  /**
   * Get an NFT by ID
   */
  async getNft(nftId: NftId): Promise<Drc369Nft> {
    const response = await this.http.get<Drc369Nft>(`/nfts/${nftId}`);
    return this.enrichNft(response.data);
  }

  /**
   * List NFTs with optional filters
   */
  async listNfts(filters: NftQueryFilters = {}): Promise<PaginatedResponse<Drc369Nft>> {
    const response = await this.http.get<PaginatedResponse<Drc369Nft>>('/nfts', {
      params: filters,
    });
    return {
      ...response.data,
      items: response.data.items.map(nft => this.enrichNft(nft)),
    };
  }

  /**
   * Get NFTs owned by an address
   */
  async getNftsByOwner(owner: Address, filters: Omit<NftQueryFilters, 'owner'> = {}): Promise<PaginatedResponse<Drc369Nft>> {
    return this.listNfts({ ...filters, owner });
  }

  /**
   * Get NFTs created by an address
   */
  async getNftsByCreator(creator: Address, filters: Omit<NftQueryFilters, 'creator'> = {}): Promise<PaginatedResponse<Drc369Nft>> {
    return this.listNfts({ ...filters, creator });
  }

  /**
   * Transfer an NFT to another address
   */
  async transfer(params: TransferParams): Promise<TxResult> {
    const response = await this.http.post<TxResult>(`/nfts/${params.nftId}/transfer`, {
      to: params.to,
      memo: params.memo,
    });
    return response.data;
  }

  /**
   * Burn (destroy) an NFT
   */
  async burn(nftId: NftId): Promise<TxResult> {
    const response = await this.http.delete<TxResult>(`/nfts/${nftId}`);
    return response.data;
  }

  // ===========================================================================
  // STATE OPERATIONS
  // ===========================================================================

  /**
   * Update custom state on an NFT
   */
  async updateState(params: UpdateStateParams): Promise<TxResult> {
    const response = await this.http.patch<TxResult>(`/nfts/${params.nftId}/state`, {
      key: params.key,
      value: params.value,
    });
    return response.data;
  }

  /**
   * Batch update multiple state keys
   */
  async batchUpdateState(nftId: NftId, updates: Record<string, string>): Promise<TxResult> {
    const response = await this.http.patch<TxResult>(`/nfts/${nftId}/state/batch`, {
      updates,
    });
    return response.data;
  }

  /**
   * Get custom state value
   */
  async getState(nftId: NftId, key: string): Promise<string | null> {
    const nft = await this.getNft(nftId);
    return nft.customState[key] ?? null;
  }

  // ===========================================================================
  // XP & LEVELING
  // ===========================================================================

  /**
   * Add XP to an NFT
   */
  async addXp(params: AddXpParams): Promise<TxResult<{ newXp: number; newLevel: number; leveledUp: boolean }>> {
    const response = await this.http.post<TxResult<{ newXp: number; newLevel: number; leveledUp: boolean }>>(
      `/nfts/${params.nftId}/xp`,
      {
        amount: params.amount,
        reason: params.reason,
      }
    );
    return response.data;
  }

  /**
   * Get XP history for an NFT
   */
  async getXpHistory(nftId: NftId, limit = 50): Promise<Array<{ amount: number; reason?: string; timestamp: number }>> {
    const response = await this.http.get<Array<{ amount: number; reason?: string; timestamp: number }>>(
      `/nfts/${nftId}/xp/history`,
      { params: { limit } }
    );
    return response.data;
  }

  // ===========================================================================
  // SOULBOUND
  // ===========================================================================

  /**
   * Make an NFT soulbound (irreversible)
   */
  async setSoulbound(nftId: NftId): Promise<TxResult> {
    const response = await this.http.post<TxResult>(`/nfts/${nftId}/soulbound`);
    return response.data;
  }

  // ===========================================================================
  // RESOURCES
  // ===========================================================================

  /**
   * Add a resource to an NFT
   */
  async addResource(params: AddResourceParams): Promise<TxResult<NftResource>> {
    const response = await this.http.post<TxResult<NftResource>>(
      `/nfts/${params.nftId}/resources`,
      params.resource
    );
    return response.data;
  }

  /**
   * Remove a resource from an NFT
   */
  async removeResource(nftId: NftId, resourceId: string): Promise<TxResult> {
    const response = await this.http.delete<TxResult>(`/nfts/${nftId}/resources/${resourceId}`);
    return response.data;
  }

  /**
   * Update resource priority
   */
  async updateResourcePriority(nftId: NftId, resourceId: string, priority: number): Promise<TxResult> {
    const response = await this.http.patch<TxResult>(`/nfts/${nftId}/resources/${resourceId}`, {
      priority,
    });
    return response.data;
  }

  // ===========================================================================
  // NESTING & COMPOSABILITY
  // ===========================================================================

  /**
   * Nest an NFT inside another
   */
  async nest(params: NestParams): Promise<TxResult> {
    const response = await this.http.post<TxResult>(`/nfts/${params.childId}/nest`, {
      parentId: params.parentId,
      slotName: params.slotName,
    });
    return response.data;
  }

  /**
   * Unnest an NFT from its parent
   */
  async unnest(childId: NftId): Promise<TxResult> {
    const response = await this.http.post<TxResult>(`/nfts/${childId}/unnest`);
    return response.data;
  }

  /**
   * Equip an NFT into a slot
   */
  async equip(parentId: NftId, childId: NftId, slotName: string): Promise<TxResult> {
    const response = await this.http.post<TxResult>(`/nfts/${parentId}/equip`, {
      childId,
      slotName,
    });
    return response.data;
  }

  /**
   * Unequip an NFT from a slot
   */
  async unequip(parentId: NftId, slotName: string): Promise<TxResult> {
    const response = await this.http.post<TxResult>(`/nfts/${parentId}/unequip`, {
      slotName,
    });
    return response.data;
  }

  /**
   * Get children of an NFT
   */
  async getChildren(parentId: NftId): Promise<Drc369Nft[]> {
    const response = await this.http.get<Drc369Nft[]>(`/nfts/${parentId}/children`);
    return response.data.map(nft => this.enrichNft(nft));
  }

  // ===========================================================================
  // DELEGATION
  // ===========================================================================

  /**
   * Delegate permissions on an NFT
   */
  async delegate(params: DelegateParams): Promise<TxResult> {
    const response = await this.http.post<TxResult>(`/nfts/${params.nftId}/delegate`, {
      delegate: params.delegate,
      permissions: params.permissions,
      expiresAt: params.expiresAt ?? 0,
      canSubDelegate: params.canSubDelegate ?? false,
    });
    return response.data;
  }

  /**
   * Revoke delegation
   */
  async revokeDelegation(nftId: NftId, delegate: Address): Promise<TxResult> {
    const response = await this.http.delete<TxResult>(`/nfts/${nftId}/delegate/${delegate}`);
    return response.data;
  }

  /**
   * Check if an address has permission on an NFT
   */
  async hasPermission(nftId: NftId, address: Address, permission: Permission): Promise<boolean> {
    const response = await this.http.get<{ hasPermission: boolean }>(
      `/nfts/${nftId}/permission`,
      { params: { address, permission } }
    );
    return response.data.hasPermission;
  }

  // ===========================================================================
  // CVP STATUS
  // ===========================================================================

  /**
   * Get CVP protection status for an NFT
   */
  async getCvpStatus(nftId: NftId): Promise<CvpStatus> {
    const response = await this.http.get<CvpStatus>(`/nfts/${nftId}/cvp`);
    return response.data;
  }

  /**
   * Get CVP system-wide status
   */
  async getSystemCvpStatus(): Promise<{
    currentEpoch: number;
    epochBlocksRemaining: number;
    totalMutations: number;
    activeThreats: number;
    proofSystem: string;
  }> {
    const response = await this.http.get<{
      currentEpoch: number;
      epochBlocksRemaining: number;
      totalMutations: number;
      activeThreats: number;
      proofSystem: string;
    }>('/cvp/status');
    return response.data;
  }

  /**
   * Get recent threats detected by CVP
   */
  async getRecentThreats(limit = 10): Promise<Array<{
    type: string;
    severity: string;
    description: string;
    targetNftId?: string;
    timestamp: number;
  }>> {
    const response = await this.http.get<Array<{
      type: string;
      severity: string;
      description: string;
      targetNftId?: string;
      timestamp: number;
    }>>('/cvp/threats', { params: { limit } });
    return response.data;
  }

  // ===========================================================================
  // WEBSOCKET EVENTS
  // ===========================================================================

  /**
   * Connect to WebSocket for real-time events
   */
  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return;
    }

    try {
      const url = new URL(this.config.wsUrl);
      if (this.config.token) {
        url.searchParams.set('token', this.config.token);
      }

      this.ws = new WebSocket(url.toString());

      this.ws.onopen = () => {
        this.reconnectAttempts = 0;
        this.emit('connected');
        
        // Resubscribe to previous subscriptions
        for (const sub of this.subscriptions) {
          this.ws?.send(JSON.stringify({ type: 'subscribe', topic: sub }));
        }
      };

      this.ws.onclose = () => {
        this.emit('disconnected');
        
        if (this.config.autoReconnect && this.reconnectAttempts < this.maxReconnectAttempts) {
          this.reconnectAttempts++;
          setTimeout(() => this.connect(), this.reconnectDelay * this.reconnectAttempts);
        }
      };

      this.ws.onerror = (error) => {
        this.emit('error', new Error('WebSocket error'));
      };

      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data) as Drc369Event;
          this.emit('event', data);
          this.emit(data.type, data);
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e);
        }
      };
    } catch (error) {
      this.emit('error', error as Error);
    }
  }

  /**
   * Disconnect WebSocket
   */
  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /**
   * Subscribe to events for a specific NFT
   */
  subscribeToNft(nftId: NftId): void {
    const topic = `nft:${nftId}`;
    this.subscriptions.add(topic);
    
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'subscribe', topic }));
    }
  }

  /**
   * Unsubscribe from NFT events
   */
  unsubscribeFromNft(nftId: NftId): void {
    const topic = `nft:${nftId}`;
    this.subscriptions.delete(topic);
    
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'unsubscribe', topic }));
    }
  }

  /**
   * Subscribe to all events for an address (as owner)
   */
  subscribeToAddress(address: Address): void {
    const topic = `address:${address}`;
    this.subscriptions.add(topic);
    
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'subscribe', topic }));
    }
  }

  /**
   * Subscribe to CVP system events
   */
  subscribeToCvp(): void {
    const topic = 'cvp:system';
    this.subscriptions.add(topic);
    
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'subscribe', topic }));
    }
  }

  // ===========================================================================
  // UTILITIES
  // ===========================================================================

  /**
   * Enrich NFT with calculated fields
   */
  private enrichNft(nft: Drc369Nft): Drc369Nft {
    return {
      ...nft,
      level: calculateLevel(nft.xp),
    };
  }

  /**
   * Upload file and get URI
   */
  async uploadFile(file: File | Blob, filename?: string): Promise<string> {
    const formData = new FormData();
    formData.append('file', file, filename);

    const response = await this.http.post<{ uri: string }>('/upload', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });

    return response.data.uri;
  }

  /**
   * Get transaction status
   */
  async getTxStatus(txHash: TxHash): Promise<{
    status: 'pending' | 'confirmed' | 'failed';
    blockNumber?: number;
    error?: string;
  }> {
    const response = await this.http.get<{
      status: 'pending' | 'confirmed' | 'failed';
      blockNumber?: number;
      error?: string;
    }>(`/tx/${txHash}`);
    return response.data;
  }

  /**
   * Wait for transaction confirmation
   */
  async waitForTx(txHash: TxHash, timeoutMs = 60000): Promise<void> {
    const startTime = Date.now();
    
    while (Date.now() - startTime < timeoutMs) {
      const status = await this.getTxStatus(txHash);
      
      if (status.status === 'confirmed') {
        return;
      }
      
      if (status.status === 'failed') {
        throw new Error(status.error || 'Transaction failed');
      }
      
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    
    throw new Error('Transaction confirmation timeout');
  }
}

// Export singleton instance
export const drc369 = new Drc369Client();

export default drc369;
