// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
// Demiurge Wallet Extension - Shared Types

// Account types
export interface Account {
  address: string;
  publicKey: string;
  name: string;
  createdAt: number;
}

export interface EncryptedKeystore {
  version: 1;
  address: string;
  crypto: {
    cipher: 'aes-256-gcm';
    ciphertext: string;
    cipherparams: {
      iv: string;
    };
    kdf: 'pbkdf2';
    kdfparams: {
      iterations: number;
      salt: string;
    };
    mac: string;
  };
}

// Network configuration
export interface NetworkConfig {
  id: string;
  name: string;
  rpcUrl: string;
  chainId: string;
  explorerUrl?: string;
  isTestnet: boolean;
}

export const NETWORKS: Record<string, NetworkConfig> = {
  mainnet: {
    id: 'mainnet',
    name: 'Demiurge Mainnet',
    rpcUrl: 'http://127.0.0.1:9944',
    chainId: 'demiurge-mainnet',
    explorerUrl: 'https://qorsync.dev',
    isTestnet: false,
  },
  testnet: {
    id: 'testnet',
    name: 'Demiurge Testnet',
    rpcUrl: 'http://127.0.0.1:9944',
    chainId: 'demiurge-testnet',
    explorerUrl: 'https://qorsync.dev',
    isTestnet: true,
  },
  local: {
    id: 'local',
    name: 'Local Node',
    rpcUrl: 'http://localhost:9944',
    chainId: 'demiurge-local',
    isTestnet: true,
  },
};

// Transaction types
export interface TransactionRequest {
  to: string;
  value: string;
  data?: string;
  nonce?: number;
}

export interface SignedTransaction {
  hash: string;
  from: string;
  to: string;
  value: string;
  signature: string;
  nonce: number;
}

export interface TransactionResult {
  hash: string;
  status: 'pending' | 'confirmed' | 'failed';
  blockNumber?: number;
}

// Auth state
export interface AuthState {
  isAuthenticated: boolean;
  token: string | null;
  user: QorUser | null;
}

export interface QorUser {
  qorId: string;
  address?: string;
  displayName?: string;
}

// Wallet state
export interface WalletState {
  isLocked: boolean;
  isInitialized: boolean;
  accounts: Account[];
  activeAccount: string | null;
  network: string;
  pendingRequests: PendingRequest[];
  auth: AuthState;
}

// Saved note
export interface SavedNote {
  id: string;
  title: string;
  content: string;
  url?: string;
  tags: string[];
  createdAt: number;
  updatedAt: number;
}

// Saved media
export interface SavedMedia {
  id: string;
  url: string;
  title?: string;
  sourceUrl: string;
  type: 'image' | 'link';
  createdAt: number;
}

export interface PendingRequest {
  id: string;
  type: 'connect' | 'sign_message' | 'sign_transaction' | 'send_transaction';
  origin: string;
  data: any;
  createdAt: number;
}

// RPC request/response
export interface DemiurgeRpcRequest {
  method: string;
  params?: any[];
}

export interface DemiurgeRpcResponse {
  result?: any;
  error?: {
    code: number;
    message: string;
  };
}
