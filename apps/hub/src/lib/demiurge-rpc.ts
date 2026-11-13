// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Demiurge Custom Blockchain RPC Client
 * 
 * Connects to the Demiurge custom blockchain node via JSON-RPC 2.0
 * Replaces Polkadot API for our custom consensus engine
 */

export interface RpcRequest {
  jsonrpc: '2.0';
  id: number | string;
  method: string;
  params?: any[];
}

export interface RpcResponse<T = any> {
  jsonrpc: '2.0';
  id: number | string;
  result?: T;
  error?: {
    code: number;
    message: string;
    data?: any;
  };
}

export interface BlockInfo {
  hash: string;
  number: number;
  parentHash: string;
  stateRoot: string;
  extrinsicsRoot: string;
  timestamp: number;
  transactions: TransactionInfo[];
}

export interface TransactionInfo {
  hash: string;
  from: string;
  to?: string;
  amount?: string;
  nonce: number;
  status: 'pending' | 'inBlock' | 'finalized' | 'failed';
}

export interface ValidatorInfo {
  account: string;
  stake: string;
  commission: number;
  active: boolean;
  publicKey: string;
}

export interface StakingPoolInfo {
  validator: string;
  totalStake: string;
  nominators: Array<{
    account: string;
    stake: string;
    era: number;
  }>;
  commission: number;
}

export interface EraInfo {
  era: number;
  blockNumber: number;
  totalRewards: string;
  transactionFees: string;
  validators: ValidatorInfo[];
}

export interface EnergyInfo {
  current: number;
  max: number;
  regenerationRate: number;
  lastUpdate: number;
}

export interface ConsensusStatus {
  currentEra: number;
  blockNumber: number;
  validators: number;
  totalStake: string;
  transactionFees: string;
}

export class DemiurgeRpcClient {
  private rpcUrl: string;
  private requestId = 0;

  constructor(rpcUrl: string = 'http://127.0.0.1:9944') {
    this.rpcUrl = rpcUrl;
  }

  /**
   * Make JSON-RPC request
   */
  private async request<T>(method: string, params?: any[]): Promise<T> {
    const id = ++this.requestId;
    const request: RpcRequest = {
      jsonrpc: '2.0',
      id,
      method,
      params: params || [],
    };

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 5000); // 5 second timeout

      const response = await fetch(this.rpcUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data: RpcResponse<T> = await response.json();

      if (data.error) {
        throw new Error(`RPC error: ${data.error.message} (code: ${data.error.code})`);
      }

      return data.result as T;
    } catch (error: any) {
      // Handle network errors gracefully (blockchain may not be running)
      const isNetworkError = 
        error.name === 'AbortError' || 
        error.message?.includes('fetch') || 
        error.message?.includes('Failed to fetch') ||
        error.message?.includes('NetworkError') ||
        error.code === 'ERR_NETWORK';

      if (isNetworkError) {
        // Create a custom error that components can check for
        // Don't log network errors - they're expected when blockchain is offline
        const networkError = new Error('Blockchain RPC unavailable');
        (networkError as any).isNetworkError = true;
        throw networkError;
      }
      
      // For other errors, log and rethrow
      console.error(`RPC request failed for ${method}:`, error);
      throw error;
    }
  }

  /**
   * Get chain health status
   */
  async getHealth(): Promise<{
    connected: boolean;
    blockNumber: number;
    blockTime: number;
    finality: number;
  }> {
    const raw = await this.request<{
      connected: boolean;
      block_number: number;
      block_time: number;
      finality: number;
    }>('chain_getHealth');
    
    // Transform snake_case to camelCase
    return {
      connected: raw.connected,
      blockNumber: raw.block_number,
      blockTime: raw.block_time,
      finality: raw.finality,
    };
  }

  /**
   * Get latest block number
   */
  async getBlockNumber(): Promise<number> {
    return this.request('chain_getBlockNumber');
  }

  /**
   * Get block by number
   */
  async getBlock(blockNumber: number): Promise<BlockInfo | null> {
    return this.request('chain_getBlock', [blockNumber]);
  }

  /**
   * Get latest block
   */
  async getLatestBlock(): Promise<BlockInfo> {
    return this.request('chain_getLatestBlock');
  }

  /**
   * Get balance for an account
   */
  async getBalance(address: string): Promise<string> {
    return this.request('balances_getBalance', [address]);
  }

  /**
   * Get the next expected request nonce for an account.
   *
   * State-changing calls bind this into the signed payload so a captured
   * request cannot be replayed.
   */
  async getNonce(address: string): Promise<number> {
    return this.request('account_getNonce', [address]);
  }

  /**
   * Transfer CGT tokens.
   *
   * The signature must cover the canonical payload built by
   * `payloads.transfer(...)` from @demiurge/sdk - the node rebuilds the same
   * bytes and verifies them against `fromAddress`. Fetch `nonce` with
   * `getNonce()` immediately before signing.
   */
  async transfer(
    fromAddress: string,
    toAddress: string,
    amount: string,
    nonce: number,
    signature: string
  ): Promise<string> {
    return this.request('balances_transfer', [fromAddress, toAddress, amount, nonce, signature]);
  }

  /**
   * Claim starter CGT bonus (for new users)
   * This is NOT a public crypto faucet - it's platform onboarding credits
   */
  async claimStarterBonus(address: string): Promise<{
    success: boolean;
    amount: string;
    message: string;
  }> {
    return this.request('balances_claimStarter', [address]);
  }

  /**
   * Check if user has already claimed their starter bonus
   */
  async hasClaimedStarter(address: string): Promise<boolean> {
    try {
      return await this.request('balances_hasClaimedStarter', [address]);
    } catch {
      // If RPC fails, assume not claimed to allow retry
      return false;
    }
  }

  /**
   * Get energy for an account
   */
  async getEnergy(address: string): Promise<EnergyInfo> {
    const raw = await this.request<{
      current: number;
      max: number;
      regeneration_rate: number;
      last_update: number;
    }>('energy_getEnergy', [address]);
    
    return {
      current: raw.current,
      max: raw.max,
      regenerationRate: raw.regeneration_rate,
      lastUpdate: raw.last_update,
    };
  }

  /**
   * Get current era information
   */
  async getCurrentEra(): Promise<EraInfo> {
    const raw = await this.request<{
      era: number;
      block_number: number;
      total_rewards: string;
      transaction_fees: string;
      validators: Array<{
        account: string;
        stake: string;
        commission: number;
        active: boolean;
        public_key: string;
      }>;
    }>('consensus_getCurrentEra');
    
    return {
      era: raw.era,
      blockNumber: raw.block_number,
      totalRewards: raw.total_rewards,
      transactionFees: raw.transaction_fees,
      validators: raw.validators.map(v => ({
        account: v.account,
        stake: v.stake,
        commission: v.commission,
        active: v.active,
        publicKey: v.public_key,
      })),
    };
  }

  /**
   * Get validator set
   */
  async getValidators(): Promise<ValidatorInfo[]> {
    const raw = await this.request<Array<{
      account: string;
      stake: string;
      commission: number;
      active: boolean;
      public_key: string;
    }>>('consensus_getValidators');
    
    return raw.map(v => ({
      account: v.account,
      stake: v.stake,
      commission: v.commission,
      active: v.active,
      publicKey: v.public_key,
    }));
  }

  /**
   * Get validator by account
   */
  async getValidator(account: string): Promise<ValidatorInfo | null> {
    const raw = await this.request<{
      account: string;
      stake: string;
      commission: number;
      active: boolean;
      public_key: string;
    } | null>('consensus_getValidator', [account]);
    
    if (!raw) return null;
    
    return {
      account: raw.account,
      stake: raw.stake,
      commission: raw.commission,
      active: raw.active,
      publicKey: raw.public_key,
    };
  }

  /**
   * Get staking pool for a validator
   */
  async getStakingPool(validator: string): Promise<StakingPoolInfo | null> {
    const raw = await this.request<{
      validator: string;
      total_stake: string;
      nominators: Array<{
        account: string;
        stake: string;
        era: number;
      }>;
      commission: number;
    } | null>('consensus_getStakingPool', [validator]);
    
    if (!raw) return null;
    
    return {
      validator: raw.validator,
      totalStake: raw.total_stake,
      nominators: raw.nominators,
      commission: raw.commission,
    };
  }

  /**
   * Register as a validator
   */
  async registerValidator(
    account: string,
    stake: string,
    commission: number,
    sessionKey: string,
    signature: string
  ): Promise<{ success: boolean; message: string }> {
    return this.request('consensus_registerValidator', [account, stake, commission, sessionKey, signature]);
  }

  /**
   * Nominate a validator
   */
  async nominateValidator(
    nominator: string,
    validator: string,
    amount: string,
    signature: string
  ): Promise<string> {
    return this.request('consensus_nominateValidator', [nominator, validator, amount, signature]);
  }

  /**
   * Get active session keys for an account
   */
  async getSessionKeys(account: string): Promise<Array<{
    sessionKey: string;
    expiryBlock: number;
  }>> {
    return this.request('sessionKeys_getActiveKeys', [account]);
  }

  /**
   * Authorize a session key
   */
  async authorizeSessionKey(
    primaryAccount: string,
    sessionKey: string,
    duration: number,
    signature: string
  ): Promise<string> {
    return this.request('sessionKeys_authorize', [primaryAccount, sessionKey, duration, signature]);
  }

  /**
   * Get transaction by hash
   */
  async getTransaction(hash: string): Promise<TransactionInfo | null> {
    return this.request('chain_getTransaction', [hash]);
  }

  /**
   * Get transaction history for an account
   */
  async getTransactionHistory(address: string, limit: number = 50): Promise<TransactionInfo[]> {
    return this.request('chain_getTransactionHistory', [address, limit]);
  }

  /**
   * Submit a signed transaction
   */
  async submitTransaction(
    transaction: {
      from: string;
      to?: string;
      amount?: string;
      nonce: number;
      data?: any;
      signature: string;
    }
  ): Promise<string> {
    return this.request('chain_submitTransaction', [transaction]);
  }

  /**
   * Get consensus status
   */
  async getConsensusStatus(): Promise<ConsensusStatus> {
    const raw = await this.request<{
      current_era: number;
      block_number: number;
      validators: number;
      total_stake: string;
      transaction_fees: string;
    }>('consensus_getStatus');
    
    // Transform snake_case to camelCase
    return {
      currentEra: raw.current_era,
      blockNumber: raw.block_number,
      validators: raw.validators,
      totalStake: raw.total_stake,
      transactionFees: raw.transaction_fees,
    };
  }

  /**
   * Get chain info (combined data for dashboard)
   */
  async getChainInfo(): Promise<{
    blockHeight: number;
    currentEra: number;
    validators: number;
    totalStake: string;
    connected: boolean;
  }> {
    try {
      const [status, health] = await Promise.all([
        this.getConsensusStatus(),
        this.getHealth(),
      ]);
      
      return {
        blockHeight: status.blockNumber,
        currentEra: status.currentEra,
        validators: status.validators,
        totalStake: status.totalStake,
        connected: health.connected,
      };
    } catch (error) {
      // Return mock data if blockchain is unavailable
      return {
        blockHeight: 0,
        currentEra: 0,
        validators: 0,
        totalStake: '0',
        connected: false,
      };
    }
  }

  /**
   * Get user's NFT assets (DRC-369)
   */
  async getUserNFTs(address: string): Promise<Array<{
    id: string;
    name: string;
    description: string;
    image: string;
    collection?: string;
    attributes: Array<{ trait_type: string; value: string }>;
    royalty: number;
    owner: string;
    mintedAt: number;
  }>> {
    try {
      return await this.request('drc369_getAssets', [address]);
    } catch (error) {
      console.warn('Failed to fetch NFTs:', error);
      return [];
    }
  }

  /**
   * Mint a new DRC-369 NFT
   */
  async mintNFT(
    creator: string,
    metadataUri: string,
    royaltyBps: number,
    signature: string
  ): Promise<{
    success: boolean;
    tokenId?: string;
    txHash?: string;
    error?: string;
  }> {
    return this.request('drc369_mint', [creator, metadataUri, royaltyBps, signature]);
  }

  /**
   * Get recent chain events for feed
   */
  async getRecentEvents(limit: number = 10): Promise<Array<{
    type: 'block' | 'transaction' | 'nft_mint' | 'stake' | 'reward';
    data: any;
    timestamp: number;
    blockNumber: number;
  }>> {
    try {
      return await this.request('chain_getRecentEvents', [limit]);
    } catch (error) {
      console.warn('Failed to fetch events:', error);
      return [];
    }
  }

  /**
   * Get user stats (level, XP, achievements, karma)
   */
  async getUserStats(userId: string): Promise<{
    level: number;
    xp: number;
    achievements: number;
    karma: number;
  } | null> {
    try {
      return await this.request('user_getStats', [userId]);
    } catch (error) {
      console.warn('Failed to fetch user stats:', error);
      return null;
    }
  }

  /**
   * Get user activity history
   */
  async getUserActivity(userId: string, limit: number = 10): Promise<Array<{
    id: string;
    type: string;
    description: string;
    timestamp: number;
    reward?: number;
  }>> {
    try {
      return await this.request('user_getActivity', [userId, limit]);
    } catch (error) {
      console.warn('Failed to fetch user activity:', error);
      return [];
    }
  }

  /**
   * Get marketplace listings
   */
  async getMarketplaceListings(filter?: string): Promise<any[]> {
    try {
      return await this.request('marketplace_getListings', [filter]);
    } catch (error) {
      console.warn('Failed to fetch marketplace listings:', error);
      return [];
    }
  }

  /**
   * Toggle energy sponsorship for a developer address
   */
  async setEnergySponsorship(
    developerAddress: string,
    enabled: boolean,
    signature: string
  ): Promise<{ success: boolean; enabled: boolean }> {
    return this.request('energy_setSponsorship', [developerAddress, enabled, signature]);
  }

  /**
   * Get historical era data
   */
  async getHistoricalEras(count: number = 10): Promise<Array<{
    era: number;
    total_rewards: string;
    transaction_fees: string;
    block_number: number;
    validator_count: number;
  }>> {
    try {
      return await this.request('consensus_getHistoricalEras', [count]);
    } catch (error) {
      console.warn('Failed to fetch historical eras:', error);
      return [];
    }
  }

  /**
   * Get sponsorship history for a developer address
   */
  async getSponsorshipHistory(developerAddress: string): Promise<Array<{
    id: string;
    userAddress: string;
    energyCost: number;
    timestamp: number;
    status: 'success' | 'failed';
  }>> {
    try {
      return await this.request('energy_getSponsorshipHistory', [developerAddress]);
    } catch (error) {
      console.warn('Failed to fetch sponsorship history:', error);
      return [];
    }
  }

  /**
   * Get network historical data for analytics
   */
  async getNetworkHistory(timeRange: '1h' | '24h' | '7d' | '30d'): Promise<Array<{
    timestamp: number;
    blockNumber: number;
    transactionVolume: number;
    activeValidators: number;
    totalStake: number;
  }>> {
    try {
      return await this.request('analytics_getNetworkHistory', [timeRange]);
    } catch (error) {
      console.warn('Failed to fetch network history:', error);
      return [];
    }
  }
}

// Export RPC URL configuration
// Default to qorsync.dev for production, fallback to localhost for development
export const RPC_URL = process.env.NEXT_PUBLIC_DEMIURGE_RPC_URL || 
  (typeof window !== 'undefined' && window.location?.hostname === 'localhost' 
    ? 'http://localhost:9944' 
    : 'http://127.0.0.1:9944');

// Export singleton instance
export const demiurgeRpc = new DemiurgeRpcClient(RPC_URL);
