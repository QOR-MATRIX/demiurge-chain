// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Blockchain Integration Client
 * 
 * Connects to the Demiurge Substrate node for CGT and DRC-369 operations
 */

import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { KeyringPair } from '@polkadot/keyring/types';

// Suppress WebSocket disconnection errors - they're expected when node is offline
// Initialize error suppression only in browser environment
let errorSuppressionInitialized = false;

function initializeErrorSuppression() {
  if (errorSuppressionInitialized) {
    return;
  }
  if (typeof window === 'undefined') {
    return;
  }
  if (typeof console === 'undefined' || !console.error) {
    return;
  }

  try {
    const originalConsoleError = console.error.bind(console);
    console.error = function(...args: any[]) {
      // Convert all args to string for checking
      let message = '';
      for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        if (typeof arg === 'string') {
          message += arg + ' ';
        } else if (typeof arg === 'object' && arg !== null) {
          try {
            message += JSON.stringify(arg) + ' ';
          } catch {
            message += String(arg) + ' ';
          }
        } else {
          message += String(arg) + ' ';
        }
      }
      
      // Suppress expected disconnection errors from Polkadot API-WS
      if (message.includes('API-WS') && 
          message.includes('disconnected') && 
          (message.includes('1006') || message.includes('Abnormal Closure'))) {
        return; // Silently ignore expected disconnection errors
      }
      
      originalConsoleError.apply(console, args);
    };
    errorSuppressionInitialized = true;
  } catch (error) {
    // Silently fail if we can't intercept console.error
  }
}

// Initialize on module load if in browser
if (typeof window !== 'undefined') {
  initializeErrorSuppression();
}

// Default to production RPC server, fallback to localhost for development
function getDefaultWsUrl(): string {
  if (process.env.NEXT_PUBLIC_BLOCKCHAIN_WS_URL) {
    return process.env.NEXT_PUBLIC_BLOCKCHAIN_WS_URL;
  }
  // Use qorsync.dev for production, localhost for development
  if (typeof window !== 'undefined' && window.location && window.location.hostname === 'localhost') {
    return 'ws://localhost:9944';
  }
  // Use secure WebSocket through qorsync.dev
  return 'ws://127.0.0.1:9944';
}

const DEFAULT_WS_URL = getDefaultWsUrl();

export class BlockchainClient {
  private api: ApiPromise | null = null;
  private keyring: Keyring;
  private wsUrl: string;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000; // Start with 1 second

  constructor(wsUrl: string = DEFAULT_WS_URL) {
    this.wsUrl = wsUrl;
    this.keyring = new Keyring({ type: 'sr25519' });
  }

  /**
   * Connect to the blockchain node with retry logic
   */
  async connect(): Promise<void> {
    if (this.api && this.api.isConnected) {
      return; // Already connected
    }

    // Reset API if disconnected
    if (this.api && !this.api.isConnected) {
      try {
        await this.api.disconnect();
      } catch {
        // Ignore disconnect errors
      }
      this.api = null;
    }

    try {
      const provider = new WsProvider(this.wsUrl);
      
      // Set up error handlers
      provider.on('error', (error: Error) => {
        // Log connection errors for debugging
        console.warn('[Blockchain] WebSocket error:', error.message);
      });

      provider.on('disconnected', () => {
        console.warn('[Blockchain] WebSocket disconnected');
        this.api = null;
      });

      provider.on('connected', () => {
        // WebSocket connected
      });

      // Create API instance
      this.api = await ApiPromise.create({ provider });
      
      // Wait for API to be ready with timeout
      await Promise.race([
        this.api.isReady,
        new Promise((_, reject) => 
          setTimeout(() => reject(new Error('Connection timeout after 10 seconds')), 10000)
        )
      ]);
      
      // Verify connection is actually established
      if (!this.api.isConnected) {
        throw new Error('API created but not connected');
      }
      
      // Reset reconnect attempts on successful connection
      this.reconnectAttempts = 0;
      this.reconnectDelay = 1000;
      
    } catch (error: any) {
      // Log connection failures for debugging
      console.warn('[Blockchain] Failed to connect:', error.message || error);
      
      // Clean up failed connection
      if (this.api) {
        try {
          await this.api.disconnect();
        } catch {
          // Ignore cleanup errors
        }
        this.api = null;
      }
      
      // Don't throw - allow the app to work without blockchain connection
      // The UI will show the disconnected banner
    }
  }

  /**
   * Get CGT balance for an account
   * 
   * CGT uses pallet-balances for storage, so we query the balances pallet.
   * Balance is stored as u128 with 2 decimals (CGT_UNIT = 100, 100 Sparks = 1 CGT).
   */
  async getCGTBalance(address: string): Promise<string> {
    if (!this.api) {
      await this.connect();
    }

    try {
      // Convert address to AccountId format
      const accountId = this.api!.createType('AccountId32', address);
      
      // Query balance from system account (pallet-balances stores balances here)
      // Format: { data: { free: Balance, reserved: Balance, ... }, ... }
      const accountInfo = await this.api!.query.system.account(accountId);
      
      // Type assertion: accountInfo is AccountInfo which has data property
      const accountInfoTyped = accountInfo as any;
      if (accountInfoTyped && accountInfoTyped.data && accountInfoTyped.data.free) {
        // Extract free balance (available CGT)
        const freeBalance = accountInfoTyped.data.free.toString();
        return freeBalance;
      }
      
      return '0';
    } catch (error) {
      console.error('Failed to query CGT balance:', error);
      // Return 0 on error (account might not exist or node not connected)
      return '0';
    }
  }

  /**
   * Format CGT balance for display (with 2 decimals)
   * 
   * @param balance Raw balance string (in smallest units, e.g., "100" = 1 CGT, 100 Sparks = 1 CGT)
   * @returns Formatted balance string (e.g., "1.00")
   */
  formatCGTBalance(balance: string): string {
    const CGT_UNIT = 100; // 100 Sparks = 1 CGT
    const balanceNum = BigInt(balance);
    const whole = balanceNum / BigInt(CGT_UNIT);
    const fractional = balanceNum % BigInt(CGT_UNIT);
    
    // Format fractional part with leading zeros
    const fractionalStr = fractional.toString().padStart(2, '0');
    
    return `${whole}.${fractionalStr}`;
  }

  /**
   * Transfer CGT tokens
   * 
   * @param fromPair Keyring pair of sender (must be unlocked)
   * @param toAddress Destination account address
   * @param amount Amount in smallest units (e.g., "100000000" = 1 CGT)
   * @returns Transaction hash
   */
  async transferCGT(
    fromPair: KeyringPair,
    toAddress: string,
    amount: string
  ): Promise<string> {
    if (!this.api) {
      await this.connect();
    }

    try {
      // Convert amount to Compact<Balance>
      const amountCompact = this.api!.createType('Compact<Balance>', amount);
      
      // Convert toAddress to AccountId
      const toAccountId = this.api!.createType('AccountId32', toAddress);
      
      // Build transfer extrinsic using pallet-balances
      const transfer = this.api!.tx.balances.transferKeepAlive(toAccountId, amountCompact);
      
      // Sign and submit transaction
      return new Promise((resolve, reject) => {
        transfer.signAndSend(fromPair, ({ status, txHash }) => {
          if (status.isInBlock || status.isFinalized) {
            resolve(txHash.toString());
          } else if ((status as any).isError) {
            reject(new Error('Transaction failed'));
          }
        }).catch(reject);
      });
    } catch (error) {
      console.error('Failed to transfer CGT:', error);
      throw error;
    }
  }

  /**
   * Transfer CGT tokens using WASM wallet signing
   * 
   * @param keypairJson WASM keypair JSON string
   * @param fromAddress Sender's address (for verification)
   * @param toAddress Destination account address
   * @param amount Amount in smallest units (e.g., "100" = 1 CGT)
   * @param signMessage Function to sign message with WASM
   * @returns Transaction hash
   */
  async transferCGTWithWasm(
    keypairJson: string,
    fromAddress: string,
    toAddress: string,
    amount: string,
    signMessage: (keypairJson: string, message: Uint8Array) => Promise<string>
  ): Promise<string> {
    if (!this.api) {
      await this.connect();
    }

    if (!this.api) {
      throw new Error('Blockchain not connected');
    }

    try {
      // Convert amount to Compact<Balance>
      const amountCompact = this.api.createType('Compact<Balance>', amount);
      
      // Convert toAddress to AccountId
      const toAccountId = this.api.createType('AccountId32', toAddress);
      
      // Build transfer extrinsic
      const transfer = this.api.tx.balances.transferKeepAlive(toAccountId, amountCompact);
      
      // Create a custom signer that uses WASM
      const wasmSigner = {
        signPayload: async (payload: any) => {
          // Convert payload to bytes
          const payloadBytes = payload.toU8a();
          
          // Sign with WASM
          const signatureHex = await signMessage(keypairJson, payloadBytes);
          
          // Return hex string signature (SignerResult expects 0x${string})
          return {
            id: payload.id,
            signature: signatureHex as `0x${string}`,
          };
        },
        signRaw: async (raw: { data: string }) => {
          // Convert hex data to bytes
          const { hexToU8a } = await import('@polkadot/util');
          const dataBytes = hexToU8a(raw.data);
          
          // Sign with WASM
          const signatureHex = await signMessage(keypairJson, dataBytes);
          
          // Return hex string signature (SignerResult expects 0x${string} and id)
          return {
            id: 0, // SignerResult requires id
            signature: signatureHex as `0x${string}`,
          };
        }
      };
      
      // Sign and submit transaction using custom signer
      return new Promise((resolve, reject) => {
        transfer.signAndSend(
          fromAddress,
          { signer: wasmSigner },
          ({ status, txHash }) => {
            if (status.isInBlock || status.isFinalized) {
              resolve(txHash.toString());
            } else if ((status as any).isError) {
              reject(new Error('Transaction failed'));
            }
          }
        ).catch(reject);
      });
    } catch (error) {
      console.error('Failed to transfer CGT with WASM:', error);
      throw error;
    }
  }

  /**
   * Get user's DRC-369 assets
   * 
   * @param address User's account address
   * @returns Array of DRC-369 asset metadata
   */
  async getUserAssets(address: string): Promise<any[]> {
    if (!this.api) {
      await this.connect();
    }

    try {
      // Convert address to AccountId
      const accountId = this.api!.createType('AccountId32', address);
      
      // Query owner_items from pallet-drc369
      // Returns a BTreeMap<AccountId, BTreeSet<Uuid>> or similar structure
      const ownerItems = await this.api!.query.drc369.ownerItems(accountId);
      
      const assets: any[] = [];
      
      if (ownerItems) {
        // Convert to human-readable format for easier parsing
        const human = ownerItems.toHuman();
        
        if (Array.isArray(human)) {
          // If it's an array of UUIDs
          human.forEach((uuid: any) => {
            assets.push({
              uuid: uuid?.toString() || uuid,
            });
          });
        } else if (typeof human === 'object' && human !== null) {
          // If it's an object/map, extract values
          Object.values(human).forEach((uuid: any) => {
            assets.push({
              uuid: uuid?.toString() || uuid,
            });
          });
        }
        
        // Fetch full metadata for each asset
        for (const asset of assets) {
          try {
            const assetMetadata = await this.getAssetMetadata(asset.uuid);
            Object.assign(asset, assetMetadata);
          } catch (error) {
            // If metadata fetch fails, keep UUID only
            console.warn(`Failed to fetch metadata for asset ${asset.uuid}:`, error);
          }
        }
      }
      
      return assets;
    } catch (error) {
      console.error('Failed to query user assets:', error);
      return [];
    }
  }

  /**
   * Get full metadata for a DRC-369 asset
   * 
   * @param uuid Asset UUID (32-byte hash as hex string)
   * @returns Full asset metadata including state, resources, and nesting info
   */
  async getAssetMetadata(uuid: string): Promise<any> {
    if (!this.api) {
      await this.connect();
    }

    try {
      // Query asset metadata from pallet-drc369
      const assetData = await this.api!.query.drc369.assets(uuid);
      
      if (!assetData || assetData.isEmpty) {
        throw new Error(`Asset ${uuid} not found`);
      }

      // Convert to human-readable format
      const assetRaw = assetData.toHuman() as any;
      
      // Format the response
      return {
        uuid: uuid,
        name: (assetRaw?.name as string) || 'Unknown',
        creatorQorId: (assetRaw?.creator_qor_id as string) || '',
        creatorAccount: (assetRaw?.creator_account as string) || '',
        owner: (assetRaw?.owner as string) || '',
        assetType: 'virtual', // Can be determined from metadata
        xpLevel: (assetRaw?.level as number) || 0,
        experiencePoints: (assetRaw?.experience_points as number) || 0,
        durability: (assetRaw?.durability as number) || 100,
        killCount: (assetRaw?.kill_count as number) || 0,
        classId: (assetRaw?.class_id as number) || 0,
        isSoulbound: (assetRaw?.is_soulbound as boolean) || false,
        royaltyFeePercent: (assetRaw?.royalty_fee_percent as number) || 0,
        mintedAt: (assetRaw?.minted_at as number) || 0,
        metadata: {
          description: (assetRaw?.description as string) || '',
          image: (assetRaw?.image as string) || '',
          attributes: (assetRaw?.attributes as Record<string, any>) || {},
          resources: (assetRaw?.resources as any[]) || [],
          parentUuid: (assetRaw?.parent_uuid as string) || null,
          childrenUuids: (assetRaw?.children_uuids as string[]) || [],
          equipmentSlots: (assetRaw?.equipment_slots as any[]) || [],
          delegation: (assetRaw?.delegation as any) || null,
          customState: (assetRaw?.custom_state as Record<string, any>) || {}
        }
      };
    } catch (error) {
      console.error('Failed to query asset metadata:', error);
      throw error;
    }
  }

  /**
   * Mint avatar as DRC-369 NFT asset
   * 
   * @param fromPair Keyring pair of the creator
   * @param qorId User's QOR ID (e.g., "username#0001")
   * @param avatarUrl URL/IPFS hash of the avatar image
   * @param imageData Base64 or URL of the image (for metadata)
   * @returns Transaction hash
   */
  async mintAvatarAsset(
    fromPair: KeyringPair,
    qorId: string,
    avatarUrl: string,
    imageData?: string
  ): Promise<string> {
    if (!this.api) {
      await this.connect();
    }

    try {
      // Prepare DRC-369 metadata
      // For avatars, we use a special UE5 asset path that represents a 2D image
      // The avatar will be soulbound (is_soulbound = true) and have 0% royalty
      const name = `${qorId} Avatar`;
      const creatorQorId = qorId;
      
      // Use a placeholder UE5 asset path for 2D avatars
      // In the future, this could be a 3D representation of the avatar
      const ue5AssetPath = `Texture2D'/Game/UI/Avatars/${qorId.replace('#', '_')}_Avatar'`;
      const glassMaterial = `MaterialInstance'/Game/Glass/MI_Avatar.MI_Avatar'`;
      const vfxSocket = '';
      const isSoulbound = true; // Avatars are soulbound to the user
      const royaltyFeePercent = 0; // No royalties on avatars

      // Convert strings to Vec<u8>
      const nameBytes = Array.from(new TextEncoder().encode(name));
      const qorIdBytes = Array.from(new TextEncoder().encode(creatorQorId));
      const ue5PathBytes = Array.from(new TextEncoder().encode(ue5AssetPath));
      const glassMatBytes = Array.from(new TextEncoder().encode(glassMaterial));
      const vfxBytes = Array.from(new TextEncoder().encode(vfxSocket));

      // Build mint_item extrinsic
      const mintExtrinsic = this.api!.tx.drc369.mintItem(
        nameBytes,
        qorIdBytes,
        ue5PathBytes,
        glassMatBytes,
        vfxBytes,
        isSoulbound,
        royaltyFeePercent
      );

      // Sign and submit transaction
      return new Promise((resolve, reject) => {
        mintExtrinsic.signAndSend(fromPair, ({ status, txHash }) => {
          if (status.isInBlock || status.isFinalized) {
            resolve(txHash.toString());
          } else if ((status as any).isError) {
            reject(new Error('Transaction failed'));
          }
        }).catch(reject);
      });
    } catch (error) {
      console.error('Failed to mint avatar asset:', error);
      throw error;
    }
  }

  /**
   * Get transaction history for an account
   * 
   * Scans the last 100 blocks for transfer events related to the address.
   * @param address User's account address
   * @returns Array of formatted transaction objects
   */
  async getTransactions(address: string): Promise<any[]> {
    if (!this.api) {
      await this.connect();
      if (!this.api) return [];
    }

    try {
      const latestHeader = await this.api.rpc.chain.getHeader();
      const latestBlock = latestHeader.number.toNumber();
      const startBlock = Math.max(0, latestBlock - 100); // Scan last 100 blocks

      const allTransactions: any[] = [];

      for (let i = latestBlock; i > startBlock; i--) {
        const blockHash = await this.api.rpc.chain.getBlockHash(i);
        const blockEvents = await this.api.query.system.events.at(blockHash);

        (blockEvents as any).forEach((record: any) => {
          const { event } = record;
          if (this.api!.events.balances.Transfer.is(event)) {
            const [from, to, amount] = event.data;
            if (from.toString() === address || to.toString() === address) {
              allTransactions.push({
                hash: event.hash.toString(),
                from: from.toString(),
                to: to.toString(),
                amount: (amount as any).toString(),
                blockNumber: i,
              });
            }
          }
        });
      }

      return allTransactions;
    } catch (error) {
      console.error('Failed to fetch transaction history:', error);
      return [];
    }
  }

  /**
   * Get connection status
   */
  isConnected(): boolean {
    return this.api !== null && this.api.isConnected;
  }

  /**
   * Get API instance (for advanced operations)
   */
  getApi(): ApiPromise | null {
    return this.api;
  }

  /**
   * Authorize a session key for temporary authorization
   * 
   * @param fromPair Keyring pair of the primary account
   * @param sessionKeyAddress Address of the session key to authorize
   * @param duration Duration in blocks (max 7 days = 100,800 blocks)
   * @returns Transaction hash
   */
  async authorizeSessionKey(
    fromPair: KeyringPair,
    sessionKeyAddress: string,
    duration: number
  ): Promise<string> {
    if (!this.api) {
      await this.connect();
    }

    if (!this.api) {
      throw new Error('Blockchain not connected');
    }

    try {
      // Convert sessionKeyAddress to AccountId
      const sessionKeyAccountId = this.api.createType('AccountId32', sessionKeyAddress);
      
      // Convert duration to BlockNumber
      const durationBlockNumber = this.api.createType('u32', duration);
      
      // Build authorize_session_key extrinsic
      const authorizeExtrinsic = this.api.tx.sessionKeys.authorizeSessionKey(
        sessionKeyAccountId,
        durationBlockNumber
      );

      // Sign and submit transaction
      return new Promise((resolve, reject) => {
        authorizeExtrinsic.signAndSend(fromPair, ({ status, txHash }) => {
          if (status.isInBlock || status.isFinalized) {
            resolve(txHash.toString());
          } else if ((status as any).isError) {
            reject(new Error('Transaction failed'));
          }
        }).catch(reject);
      });
    } catch (error) {
      console.error('Failed to authorize session key:', error);
      throw error;
    }
  }

  /**
   * Revoke a session key
   * 
   * @param fromPair Keyring pair of the primary account
   * @param sessionKeyAddress Address of the session key to revoke
   * @returns Transaction hash
   */
  async revokeSessionKey(
    fromPair: KeyringPair,
    sessionKeyAddress: string
  ): Promise<string> {
    if (!this.api) {
      await this.connect();
    }

    if (!this.api) {
      throw new Error('Blockchain not connected');
    }

    try {
      // Convert sessionKeyAddress to AccountId
      const sessionKeyAccountId = this.api.createType('AccountId32', sessionKeyAddress);
      
      // Build revoke_session_key extrinsic
      const revokeExtrinsic = this.api.tx.sessionKeys.revokeSessionKey(sessionKeyAccountId);

      // Sign and submit transaction
      return new Promise((resolve, reject) => {
        revokeExtrinsic.signAndSend(fromPair, ({ status, txHash }) => {
          if (status.isInBlock || status.isFinalized) {
            resolve(txHash.toString());
          } else if ((status as any).isError) {
            reject(new Error('Transaction failed'));
          }
        }).catch(reject);
      });
    } catch (error) {
      console.error('Failed to revoke session key:', error);
      throw error;
    }
  }

  /**
   * Get active session keys for an account
   * 
   * Uses runtime API if available, otherwise falls back to storage queries.
   * 
   * @param primaryAddress Primary account address
   * @returns Array of { sessionKey, expiryBlock } objects
   */
  async getActiveSessionKeys(primaryAddress: string): Promise<Array<{ sessionKey: string; expiryBlock: number }>> {
    if (!this.api) {
      await this.connect();
    }

    if (!this.api) {
      return [];
    }

    try {
      // Convert address to AccountId
      const accountId = this.api.createType('AccountId32', primaryAddress);
      
      // Try to use runtime API first (more efficient)
      try {
        const runtimeApi = this.api.call.runtimeApi;
        if (runtimeApi && (runtimeApi as any).sessionKeysApi) {
          const keys = await (runtimeApi as any).sessionKeysApi.get_active_session_keys(accountId);
          const currentHeader = await this.api.rpc.chain.getHeader();
          const currentBlock = currentHeader.number.toNumber();
          
          return keys.map(([sessionKey, expiryBlock]: [any, any]) => ({
            sessionKey: sessionKey.toString(),
            expiryBlock: expiryBlock.toNumber(),
          })).filter((key: { sessionKey: string; expiryBlock: number }) => key.expiryBlock > currentBlock);
        }
      } catch (apiError) {
        // Runtime API not available, fall back to storage queries
        console.warn('Runtime API not available, using storage queries:', apiError);
      }
      
      // Fallback: Use storage iteration (more efficient than event scanning)
      // Query all session keys for this account using storage prefix
      const sessionKeys: Array<{ sessionKey: string; expiryBlock: number }> = [];
      
      // Get storage key prefix for SessionKeys double map with primary account
      const storageKey = this.api.query.sessionKeys.sessionKeys.key(accountId);
      const prefix = storageKey.slice(0, storageKey.length - 32); // Remove the second key part
      
      // Query all keys with this prefix
      const keys = await this.api.rpc.state.getKeysPaged(prefix, 1000);
      
      // Get current block number
      const currentHeader = await this.api.rpc.chain.getHeader();
      const currentBlock = currentHeader.number.toNumber();
      
      // Decode and filter active keys
      for (const key of keys) {
        try {
          // Decode the storage key to get session key address
          const storageEntry = await this.api.rpc.state.getStorage(key);
          if (storageEntry) {
            const expiryBlock = this.api.createType('u32', storageEntry);
            const expiry = expiryBlock.toNumber();
            
            // Extract session key from storage key
            // The storage key format is: prefix + primary_account + session_key
            const keyBytes = key.slice(prefix.length);
            // Skip primary account (32 bytes) to get session key
            const sessionKeyBytes = keyBytes.slice(32);
            const sessionKey = this.api.createType('AccountId32', sessionKeyBytes);
            
            // Only include if not expired
            if (expiry > currentBlock) {
              sessionKeys.push({
                sessionKey: sessionKey.toString(),
                expiryBlock: expiry,
              });
            }
          }
        } catch (error) {
          // Skip keys that fail to decode
          continue;
        }
      }

      return sessionKeys;
    } catch (error) {
      console.error('Failed to query active session keys:', error);
      return [];
    }
  }

  /**
   * Disconnect from node
   */
  async disconnect(): Promise<void> {
    if (this.api) {
      await this.api.disconnect();
      this.api = null;
    }
  }
}

// Export singleton instance
export const blockchainClient = new BlockchainClient();
