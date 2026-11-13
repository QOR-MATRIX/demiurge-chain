// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
// Demiurge Wallet Extension - Background Service Worker
// Handles all wallet operations and dApp communication

import { payloads } from '@demiurge/sdk';
import { keyring } from './keyring';
import { rpcHandler } from './rpc-handler';
import type { 
  Message, 
  MessageResponse,
  WalletCreatePayload,
  WalletImportPayload,
  WalletUnlockPayload,
  SignMessagePayload,
  SendTransactionPayload,
  WalletStateResponse,
  SophiaQueryPayload,
  AuthLoginPayload,
  AuthKeypairLoginPayload,
  AuthSessionResponse,
  SaveNotePayload,
  SaveMediaPayload,
  AnalyzePagePayload,
  MintSummaryPayload,
} from '../shared/messages';
import { createResponse } from '../shared/messages';
import type { Account, AuthState, EncryptedKeystore, PendingRequest, SavedNote, SavedMedia, WalletState } from '../shared/types';

// Wallet state
let walletState: WalletState = {
  isLocked: true,
  isInitialized: false,
  accounts: [],
  activeAccount: null,
  network: 'mainnet',
  pendingRequests: [],
  auth: {
    isAuthenticated: false,
    token: null,
    user: null,
  },
};

// Storage keys
const STORAGE_KEYS = {
  KEYSTORES: 'demiurge_keystores',
  ACCOUNTS: 'demiurge_accounts',
  SETTINGS: 'demiurge_settings',
  CONNECTED_SITES: 'demiurge_connected_sites',
  AUTH_TOKEN: 'demiurge_auth_token',
  AUTH_USER: 'demiurge_auth_user',
  NOTES: 'demiurge_notes',
  MEDIA: 'demiurge_media',
  SOPHIA_CONVERSATIONS: 'demiurge_sophia_conversations',
};

// Hub URL resolver
function getHubUrl(): string {
  return walletState.network === 'mainnet'
    ? 'https://qorsync.dev'
    : 'http://localhost:3000';
}

// Deterministic address derivation from QOR ID
// Mirrors the hub's generateSimpleAddress() logic for compatibility
function deriveAddressFromQorId(qorId: string): string {
  const encoder = new TextEncoder();
  const data = encoder.encode(`DEMIURGE_QOR_ID:${qorId}`);

  let hash1 = 2166136261;
  let hash2 = 3671253287;

  for (let i = 0; i < data.length; i++) {
    hash1 ^= data[i];
    hash1 = Math.imul(hash1, 16777619);
    hash2 ^= data[data.length - 1 - i];
    hash2 = Math.imul(hash2, 16777619);
  }

  const hex1 = (hash1 >>> 0).toString(16).padStart(8, '0');
  const hex2 = (hash2 >>> 0).toString(16).padStart(8, '0');
  const hex3 = ((hash1 ^ hash2) >>> 0).toString(16).padStart(8, '0');
  const hex4 = ((hash1 + hash2) >>> 0).toString(16).padStart(8, '0');
  const hex5 = ((hash1 * hash2) >>> 0).toString(16).padStart(8, '0');
  const hex6 = (Math.abs(hash1 - hash2) >>> 0).toString(16).padStart(8, '0');

  return `5${hex1}${hex2}${hex3}${hex4}${hex5}${hex6}`.slice(0, 48);
}

// Transfer activity tracking
import { evaluateSendPolicy, getAccountLimits, type AccountActivity } from '../shared/transfer-policy';

const STORAGE_TRANSFER_LOG = 'demiurge_transfer_log';

interface TransferLogEntry {
  to: string;
  amount: string;
  timestamp: number;
}

async function getTransferActivity(address: string): Promise<AccountActivity> {
  const stored = await chrome.storage.local.get([STORAGE_TRANSFER_LOG, STORAGE_KEYS.ACCOUNTS]);
  const log: TransferLogEntry[] = stored[STORAGE_TRANSFER_LOG] || [];
  const accounts: Account[] = stored[STORAGE_KEYS.ACCOUNTS] || [];

  const now = Date.now();
  const oneDayAgo = now - 24 * 60 * 60 * 1000;
  const oneHourAgo = now - 60 * 60 * 1000;
  const sevenDaysAgo = now - 7 * 24 * 60 * 60 * 1000;

  // Find account creation date
  const account = accounts.find(a => a.address === address);
  const createdAt = account?.createdAt || now;

  const todayTransfers = log.filter(e => e.timestamp > oneDayAgo);
  const hourTransfers = log.filter(e => e.timestamp > oneHourAgo);
  const weekRecipients = log
    .filter(e => e.timestamp > sevenDaysAgo)
    .map(e => e.to.toLowerCase());

  return {
    createdAt,
    totalTransfersSent: log.length,
    totalTransfersReceived: 0,
    transfersToday: todayTransfers.length,
    transfersThisHour: hourTransfers.length,
    lastTransferAt: log.length > 0 ? log[log.length - 1].timestamp : 0,
    uniqueRecipients: [...new Set(weekRecipients)],
    flagged: false,
  };
}

async function recordTransfer(to: string, amount: string): Promise<void> {
  const stored = await chrome.storage.local.get(STORAGE_TRANSFER_LOG);
  const log: TransferLogEntry[] = stored[STORAGE_TRANSFER_LOG] || [];

  log.push({ to: to.toLowerCase(), amount, timestamp: Date.now() });

  // Keep only last 30 days of entries
  const cutoff = Date.now() - 30 * 24 * 60 * 60 * 1000;
  const trimmed = log.filter(e => e.timestamp > cutoff);

  await chrome.storage.local.set({ [STORAGE_TRANSFER_LOG]: trimmed });
}

// Initialize wallet state from storage
async function initializeState(): Promise<void> {
  const stored = await chrome.storage.local.get([
    STORAGE_KEYS.KEYSTORES,
    STORAGE_KEYS.ACCOUNTS,
    STORAGE_KEYS.SETTINGS,
    STORAGE_KEYS.AUTH_TOKEN,
    STORAGE_KEYS.AUTH_USER,
  ]);

  const keystores = stored[STORAGE_KEYS.KEYSTORES] as EncryptedKeystore[] | undefined;
  const accounts = stored[STORAGE_KEYS.ACCOUNTS] as Account[] | undefined;
  const settings = stored[STORAGE_KEYS.SETTINGS] as { network?: string } | undefined;
  const authToken = stored[STORAGE_KEYS.AUTH_TOKEN] as string | undefined;
  const authUser = stored[STORAGE_KEYS.AUTH_USER] as { qorId: string; address?: string; displayName?: string; registeredAt?: number } | undefined;

  walletState.isInitialized = (keystores && keystores.length > 0) || false;
  walletState.accounts = accounts || [];
  walletState.network = settings?.network || 'mainnet';
  walletState.isLocked = true;

  // Restore auth session
  if (authToken && authUser) {
    // Ensure address is present (derive if missing)
    if (!authUser.address && authUser.qorId) {
      authUser.address = deriveAddressFromQorId(authUser.qorId);
    }

    walletState.auth = {
      isAuthenticated: true,
      token: authToken,
      user: authUser,
    };

    // Ensure account and activeAccount are set
    if (authUser.address) {
      const existingAcct = walletState.accounts.find(a => a.address === authUser.address);
      if (!existingAcct) {
        walletState.accounts.push({
          address: authUser.address!,
          publicKey: authUser.address!,
          name: authUser.displayName || 'QOR Account',
          createdAt: authUser.registeredAt || Date.now(),
        });
      } else if (authUser.registeredAt && existingAcct.createdAt > authUser.registeredAt) {
        // Correct createdAt if server registration is older
        existingAcct.createdAt = authUser.registeredAt;
      }
      walletState.activeAccount = authUser.address;
    }
  }

  if (settings?.network) {
    rpcHandler.setNetwork(settings.network);
  }
}

// Save accounts to storage
async function saveAccounts(): Promise<void> {
  await chrome.storage.local.set({
    [STORAGE_KEYS.ACCOUNTS]: walletState.accounts,
  });
}

// Save keystores to storage
async function saveKeystores(keystores: EncryptedKeystore[]): Promise<void> {
  await chrome.storage.local.set({
    [STORAGE_KEYS.KEYSTORES]: keystores,
  });
}

// Save settings to storage
async function saveSettings(): Promise<void> {
  await chrome.storage.local.set({
    [STORAGE_KEYS.SETTINGS]: {
      network: walletState.network,
    },
  });
}

// Get stored keystores
async function getKeystores(): Promise<EncryptedKeystore[]> {
  const stored = await chrome.storage.local.get(STORAGE_KEYS.KEYSTORES);
  return stored[STORAGE_KEYS.KEYSTORES] || [];
}

// Message handler
async function handleMessage(
  message: Message,
  sender: chrome.runtime.MessageSender
): Promise<MessageResponse> {
  const { type, payload, requestId, origin } = message;

  try {
    switch (type) {
      // Wallet lifecycle
      case 'WALLET_GET_STATE':
        return createResponse(true, getWalletState());

      case 'WALLET_CREATE':
        return await handleWalletCreate(payload as WalletCreatePayload);

      case 'WALLET_IMPORT':
        return await handleWalletImport(payload as WalletImportPayload);

      case 'WALLET_UNLOCK':
        return await handleWalletUnlock(payload as WalletUnlockPayload);

      case 'WALLET_LOCK':
        return handleWalletLock();

      // Account management
      case 'ACCOUNT_CREATE':
        return await handleAccountCreate(payload as { name: string });

      case 'ACCOUNT_SELECT':
        return handleAccountSelect(payload as { address: string });

      case 'ACCOUNT_GET_ALL':
        return createResponse(true, walletState.accounts);

      // Network
      case 'NETWORK_SWITCH':
        return handleNetworkSwitch(payload as { network: string });

      case 'NETWORK_GET_CURRENT':
        return createResponse(true, { network: walletState.network });

      // Transactions & signing
      case 'GET_BALANCE':
        return await handleGetBalance(payload as { address?: string });

      case 'SIGN_MESSAGE':
        return await handleSignMessage(payload as SignMessagePayload, origin);

      case 'SIGN_TRANSACTION':
        return await handleSignTransaction(payload as SendTransactionPayload, origin);

      case 'SEND_TRANSACTION':
        return await handleSendTransaction(payload as SendTransactionPayload, origin);

      // dApp connection
      case 'DAPP_CONNECT':
        return await handleDAppConnect(origin || sender.origin || '');

      case 'DAPP_GET_ACCOUNTS':
        return handleDAppGetAccounts(origin || sender.origin || '');

      // Request handling
      case 'REQUEST_GET_PENDING':
      case 'GET_PENDING_REQUESTS':
        return createResponse(true, walletState.pendingRequests);

      case 'REQUEST_APPROVE':
      case 'APPROVE_REQUEST':
        return await handleRequestApprove(payload as { requestId: string });

      case 'REQUEST_REJECT':
      case 'REJECT_REQUEST':
        return handleRequestReject(payload as { requestId: string; reason?: string });

      // QOR ID Auth
      case 'AUTH_LOGIN':
        return await handleAuthLogin(payload as AuthLoginPayload);

      case 'AUTH_KEYPAIR_LOGIN':
        return await handleAuthKeypairLogin(payload as AuthKeypairLoginPayload);

      case 'AUTH_LOGOUT':
        return await handleAuthLogout();

      case 'DETACH_WALLET':
        return await handleDetachWallet();

      case 'AUTH_GET_SESSION':
        return createResponse(true, getAuthSession());

      // Transfer policy
      case 'CHECK_TRANSFER_POLICY':
        return await handleCheckTransferPolicy(payload as { to: string; amount: string });

      case 'GET_ACCOUNT_LIMITS':
        return await handleGetAccountLimits();

      // Missing handlers
      case 'CLAIM_STARTER_TOKENS':
        return await handleClaimStarterTokens(payload as { address: string });

      case 'EXPORT_PRIVATE_KEY':
        return await handleExportPrivateKey(payload as { password: string });

      // Sophia AI
      case 'SOPHIA_QUERY':
        return await handleSophiaQuery(payload as SophiaQueryPayload);

      // Content capture
      case 'SAVE_NOTE':
        return await handleSaveNote(payload as SaveNotePayload);

      case 'GET_NOTES':
        return await handleGetNotes();

      case 'DELETE_NOTE':
        return await handleDeleteNote(payload as { id: string });

      case 'SAVE_MEDIA':
        return await handleSaveMedia(payload as SaveMediaPayload);

      case 'GET_MEDIA':
        return await handleGetMedia();

      // Phase 4 (Page Analysis) removed - replaced by VYB Chat integration

      // Page context (from content script)
      case 'GET_PAGE_CONTEXT':
        return createResponse(true, { supported: true });

      default:
        return createResponse(false, undefined, `Unknown message type: ${type}`);
    }
  } catch (error) {
    console.error('Message handler error:', error);
    return createResponse(false, undefined, (error as Error).message);
  }
}

// Get wallet state for UI
function getWalletState(): WalletStateResponse {
  return {
    isLocked: walletState.isLocked,
    isInitialized: walletState.isInitialized,
    accounts: walletState.accounts,
    activeAccount: walletState.activeAccount,
    network: walletState.network,
    pendingRequestCount: walletState.pendingRequests.length,
    auth: walletState.auth,
  };
}

// Get auth session
function getAuthSession(): AuthSessionResponse {
  return {
    isAuthenticated: walletState.auth.isAuthenticated,
    token: walletState.auth.token || undefined,
    user: walletState.auth.user || undefined,
  };
}

// Create new wallet
async function handleWalletCreate(payload: WalletCreatePayload): Promise<MessageResponse> {
  const { password, mnemonic: providedMnemonic } = payload;

  const mnemonic = providedMnemonic || keyring.generateMnemonic();
  
  if (!keyring.validateMnemonic(mnemonic)) {
    return createResponse(false, undefined, 'Invalid mnemonic phrase');
  }

  // Derive first keypair
  const keyPair = keyring.deriveKeyPair(mnemonic, 0);
  
  // Encrypt private key
  const keystore = await keyring.encryptPrivateKey(keyPair.privateKey, password);
  
  // Create account
  const account: Account = {
    address: keyPair.address,
    publicKey: keyPair.address,
    name: 'Account 1',
    createdAt: Date.now(),
  };

  // Save to storage
  await saveKeystores([keystore]);
  walletState.accounts = [account];
  walletState.activeAccount = account.address;
  walletState.isInitialized = true;
  await saveAccounts();

  // Unlock immediately
  await keyring.unlock([keystore], password);
  walletState.isLocked = false;

  return createResponse(true, { mnemonic, address: account.address });
}

// Import wallet from mnemonic
async function handleWalletImport(payload: WalletImportPayload): Promise<MessageResponse> {
  const { password, mnemonic } = payload;

  if (!keyring.validateMnemonic(mnemonic)) {
    return createResponse(false, undefined, 'Invalid mnemonic phrase');
  }

  // Derive keypair
  const keyPair = keyring.deriveKeyPair(mnemonic, 0);
  
  // Encrypt private key
  const keystore = await keyring.encryptPrivateKey(keyPair.privateKey, password);
  
  // Create account
  const account: Account = {
    address: keyPair.address,
    publicKey: keyPair.address,
    name: 'Imported Account',
    createdAt: Date.now(),
  };

  // Save to storage
  await saveKeystores([keystore]);
  walletState.accounts = [account];
  walletState.activeAccount = account.address;
  walletState.isInitialized = true;
  await saveAccounts();

  // Unlock immediately
  await keyring.unlock([keystore], password);
  walletState.isLocked = false;

  return createResponse(true, { address: account.address });
}

// Unlock wallet
async function handleWalletUnlock(payload: WalletUnlockPayload): Promise<MessageResponse> {
  const { password } = payload;
  
  const keystores = await getKeystores();
  if (keystores.length === 0) {
    return createResponse(false, undefined, 'Wallet not initialized');
  }

  try {
    await keyring.unlock(keystores, password);
    walletState.isLocked = false;
    
    if (walletState.accounts.length > 0 && !walletState.activeAccount) {
      walletState.activeAccount = walletState.accounts[0].address;
    }

    return createResponse(true, getWalletState());
  } catch (error) {
    return createResponse(false, undefined, 'Invalid password');
  }
}

// Lock wallet
function handleWalletLock(): MessageResponse {
  keyring.lock();
  walletState.isLocked = true;
  return createResponse(true);
}

// Create new account
async function handleAccountCreate(payload: { name: string }): Promise<MessageResponse> {
  if (walletState.isLocked) {
    return createResponse(false, undefined, 'Wallet is locked');
  }

  // For now, we don't support multiple accounts from the same mnemonic in this version
  // This would require storing the mnemonic (encrypted) or deriving from master key
  return createResponse(false, undefined, 'Multi-account not yet supported');
}

// Select active account
function handleAccountSelect(payload: { address: string }): MessageResponse {
  const account = walletState.accounts.find(a => a.address === payload.address);
  if (!account) {
    return createResponse(false, undefined, 'Account not found');
  }

  walletState.activeAccount = account.address;
  return createResponse(true, { address: account.address });
}

// Switch network
function handleNetworkSwitch(payload: { network: string }): MessageResponse {
  try {
    rpcHandler.setNetwork(payload.network);
    walletState.network = payload.network;
    saveSettings();
    return createResponse(true, { network: payload.network });
  } catch (error) {
    return createResponse(false, undefined, (error as Error).message);
  }
}

// Get balance
async function handleGetBalance(payload: { address?: string }): Promise<MessageResponse> {
  const address = payload.address || walletState.activeAccount;
  if (!address) {
    return createResponse(false, undefined, 'No account selected');
  }

  try {
    const balance = await rpcHandler.getBalance(address);
    return createResponse(true, balance);
  } catch (error) {
    return createResponse(false, undefined, (error as Error).message);
  }
}

// Sign message
async function handleSignMessage(payload: SignMessagePayload, origin?: string): Promise<MessageResponse> {
  if (walletState.isLocked) {
    return createResponse(false, undefined, 'Wallet is locked');
  }

  const address = payload.account || walletState.activeAccount;
  if (!address) {
    return createResponse(false, undefined, 'No account selected');
  }

  const messageBytes = new TextEncoder().encode(payload.message);
  const signature = await keyring.signMessage(address, messageBytes);

  return createResponse(true, {
    signature: Array.from(signature).map(b => b.toString(16).padStart(2, '0')).join(''),
  });
}

// Sign transaction (without sending)
async function handleSignTransaction(payload: SendTransactionPayload, origin?: string): Promise<MessageResponse> {
  if (walletState.isLocked) {
    return createResponse(false, undefined, 'Wallet is locked');
  }

  const { transaction } = payload;
  const from = payload.account || walletState.activeAccount;

  if (!from) {
    return createResponse(false, undefined, 'No account selected');
  }

  const messageBytes = new TextEncoder().encode(
    `${from}${transaction.to}${transaction.value}`
  );
  const signature = await keyring.signMessage(from, messageBytes);
  const signatureHex = Array.from(signature).map(b => b.toString(16).padStart(2, '0')).join('');

  const txData = JSON.stringify({ from, to: transaction.to, value: transaction.value, data: transaction.data });
  const txHex = Array.from(new TextEncoder().encode(txData))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
  return createResponse(true, {
    tx: txHex,
    signature: signatureHex,
  });
}

// Send transaction
async function handleSendTransaction(payload: SendTransactionPayload, origin?: string): Promise<MessageResponse> {
  // QOR-authenticated users can send even if local wallet is locked
  const isQorAuth = walletState.auth.isAuthenticated;
  if (walletState.isLocked && !isQorAuth) {
    return createResponse(false, undefined, 'Wallet is locked');
  }

  const { transaction } = payload;
  const from = payload.account || walletState.activeAccount;
  
  if (!from) {
    return createResponse(false, undefined, 'No account selected');
  }

  // --- Enforce transfer policy ---
  try {
    const balanceResult = await rpcHandler.getBalance(from);
    const activity = await getTransferActivity(from);
    const policyResult = evaluateSendPolicy(
      from,
      transaction.to,
      transaction.value,
      balanceResult.balance,
      activity,
    );

    if (!policyResult.allowed) {
      return createResponse(false, undefined, policyResult.reason || 'Transfer blocked by policy.');
    }
  } catch (policyError) {
    // If we can't check policy (e.g., RPC down), still allow but log
    console.warn('Transfer policy check failed, proceeding:', policyError);
  }

  // Bind the request to a fresh nonce, then sign the canonical payload the node
  // rebuilds and verifies. Concatenating the fields by hand would be rejected:
  // the payload is domain-separated and length-prefixed.
  let nonce: number;
  try {
    nonce = await rpcHandler.getAccountNonce(from);
  } catch (nonceError) {
    return createResponse(false, undefined, `Could not fetch account nonce: ${(nonceError as Error).message}`);
  }

  const messageBytes = payloads.transfer(from, transaction.to, transaction.value, nonce);

  // Sign the transaction
  const signature = await keyring.signMessage(from, messageBytes);
  const signatureHex = Array.from(signature).map(b => b.toString(16).padStart(2, '0')).join('');

  // Submit to network
  try {
    const result = await rpcHandler.submitTransaction(
      from,
      transaction.to,
      transaction.value,
      nonce,
      signatureHex
    );

    // Record successful transfer for rate limiting
    await recordTransfer(transaction.to, transaction.value);

    return createResponse(true, result);
  } catch (error) {
    return createResponse(false, undefined, (error as Error).message);
  }
}

// Check transfer policy without sending
async function handleCheckTransferPolicy(payload: { to: string; amount: string }): Promise<MessageResponse> {
  const from = walletState.activeAccount;
  if (!from) {
    return createResponse(false, undefined, 'No account selected');
  }

  try {
    const balanceResult = await rpcHandler.getBalance(from);
    const activity = await getTransferActivity(from);
    const result = evaluateSendPolicy(from, payload.to, payload.amount, balanceResult.balance, activity);
    return createResponse(true, result);
  } catch (error) {
    return createResponse(false, undefined, (error as Error).message);
  }
}

// Get current account limits
async function handleGetAccountLimits(): Promise<MessageResponse> {
  const from = walletState.activeAccount;
  if (!from) {
    return createResponse(true, {
      tier: 'new',
      dailyLimit: 3,
      dailyUsed: 0,
      hourlyLimit: 5,
      hourlyUsed: 0,
      maxSingleCGT: '100',
      canSend: false,
      accountAgeHours: 0,
    });
  }

  try {
    const activity = await getTransferActivity(from);
    const limits = getAccountLimits(activity);
    return createResponse(true, limits);
  } catch (error) {
    return createResponse(false, undefined, (error as Error).message);
  }
}

// dApp connect
async function handleDAppConnect(origin: string): Promise<MessageResponse> {
  if (!origin) {
    return createResponse(false, undefined, 'No origin specified');
  }

  // For now, auto-approve if unlocked
  // In production, this should create a pending request
  if (walletState.isLocked) {
    return createResponse(false, undefined, 'Wallet is locked');
  }

  // Store connected site
  const stored = await chrome.storage.local.get(STORAGE_KEYS.CONNECTED_SITES);
  const connectedSites = stored[STORAGE_KEYS.CONNECTED_SITES] || [];
  
  if (!connectedSites.includes(origin)) {
    connectedSites.push(origin);
    await chrome.storage.local.set({
      [STORAGE_KEYS.CONNECTED_SITES]: connectedSites,
    });
  }

  return createResponse(true, {
    accounts: walletState.accounts.map(a => a.address),
  });
}

// Get accounts for dApp
function handleDAppGetAccounts(origin: string): MessageResponse {
  if (walletState.isLocked) {
    return createResponse(true, { accounts: [] });
  }

  // Return only public information
  return createResponse(true, {
    accounts: walletState.accounts.map(a => a.address),
  });
}

// Approve pending request
async function handleRequestApprove(payload: { requestId: string }): Promise<MessageResponse> {
  const index = walletState.pendingRequests.findIndex(r => r.id === payload.requestId);
  if (index === -1) {
    return createResponse(false, undefined, 'Request not found');
  }

  const request = walletState.pendingRequests[index];
  walletState.pendingRequests.splice(index, 1);

  // Process the request based on type
  // This would trigger the actual operation
  return createResponse(true, { approved: true });
}

// Reject pending request
function handleRequestReject(payload: { requestId: string; reason?: string }): MessageResponse {
  const index = walletState.pendingRequests.findIndex(r => r.id === payload.requestId);
  if (index === -1) {
    return createResponse(false, undefined, 'Request not found');
  }

  walletState.pendingRequests.splice(index, 1);
  return createResponse(true, { rejected: true, reason: payload.reason });
}

// --- QOR ID Auth Handlers ---

async function handleAuthLogin(payload: AuthLoginPayload): Promise<MessageResponse> {
  try {
    const hubUrl = getHubUrl();
    const response = await fetch(`${hubUrl}/api/v1/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        identifier: payload.identifier,
        password: payload.password,
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      return createResponse(false, undefined, errorData.message || `Login failed (${response.status})`);
    }

    const data = await response.json();
    const token = data.token || data.access_token;
    const qorId = data.user?.qor_id || data.user?.qorId || payload.identifier;

    // Resolve on-chain address: use API response first, otherwise derive from QOR ID
    const apiAddress = data.user?.address || data.user?.on_chain_address || data.user?.on_chain?.address;
    const derivedAddress = deriveAddressFromQorId(qorId);
    const resolvedAddress = apiAddress || derivedAddress;

    // Fetch full profile from server to get accurate registration timestamp
    let registrationTime: number | null = null;
    let serverDisplayName: string | undefined;
    try {
      const profileResponse = await fetch(`${hubUrl}/api/v1/profile`, {
        headers: { 'Authorization': `Bearer ${token}` },
      });
      if (profileResponse.ok) {
        const profile = await profileResponse.json();
        const serverCreatedAt = profile.created_at || profile.createdAt;
        if (serverCreatedAt) {
          registrationTime = new Date(serverCreatedAt).getTime();
        }
        // Also pick up server-side address and display name if available
        if (!apiAddress && profile.on_chain?.address) {
          // Use profile's on-chain address if login didn't provide one
        }
        serverDisplayName = profile.display_name || profile.displayName;
      }
    } catch {
      // Profile fetch failed - not critical, fall back to login response data
    }

    // Fall back to login response if profile fetch didn't yield a timestamp
    if (!registrationTime) {
      const loginCreatedAt = data.user?.created_at || data.user?.createdAt;
      registrationTime = loginCreatedAt ? new Date(loginCreatedAt).getTime() : null;
    }

    const user = {
      qorId,
      address: resolvedAddress,
      displayName: serverDisplayName || data.user?.display_name || data.user?.displayName || payload.identifier.split('#')[0],
      registeredAt: registrationTime || undefined,
    };

    // Persist auth
    walletState.auth = { isAuthenticated: true, token, user };
    await chrome.storage.local.set({
      [STORAGE_KEYS.AUTH_TOKEN]: token,
      [STORAGE_KEYS.AUTH_USER]: user,
    });

    // Ensure account exists for this address
    const existingAccount = walletState.accounts.find(a => a.address === resolvedAddress);
    if (!existingAccount) {
      const account: Account = {
        address: resolvedAddress,
        publicKey: resolvedAddress,
        name: user.displayName || 'QOR Account',
        createdAt: registrationTime || Date.now(),
      };
      walletState.accounts.push(account);
      await saveAccounts();
    } else if (registrationTime && existingAccount.createdAt > registrationTime) {
      // Server says account is older than local record - fix it
      existingAccount.createdAt = registrationTime;
      await saveAccounts();
    }

    // Always set active account to the resolved address
    walletState.activeAccount = resolvedAddress;

    return createResponse(true, { token, user });
  } catch (error) {
    return createResponse(false, undefined, `Login failed: ${(error as Error).message}`);
  }
}

async function handleAuthKeypairLogin(payload: AuthKeypairLoginPayload): Promise<MessageResponse> {
  try {
    const hubUrl = getHubUrl();
    const response = await fetch(`${hubUrl}/api/v1/auth/keypair-login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        address: payload.address,
        signature: payload.signature,
        challenge: payload.challenge,
      }),
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      return createResponse(false, undefined, errorData.message || `Keypair login failed (${response.status})`);
    }

    const data = await response.json();
    const token = data.token || data.access_token;
    const user = {
      qorId: data.user?.qor_id || data.user?.qorId || payload.address,
      address: payload.address,
      displayName: data.user?.display_name || data.user?.displayName,
    };

    walletState.auth = { isAuthenticated: true, token, user };
    await chrome.storage.local.set({
      [STORAGE_KEYS.AUTH_TOKEN]: token,
      [STORAGE_KEYS.AUTH_USER]: user,
    });

    return createResponse(true, { token, user });
  } catch (error) {
    return createResponse(false, undefined, `Keypair login failed: ${(error as Error).message}`);
  }
}

async function handleAuthLogout(): Promise<MessageResponse> {
  // Clear auth state
  walletState.auth = { isAuthenticated: false, token: null, user: null };
  walletState.activeAccount = null;
  walletState.accounts = [];
  walletState.isInitialized = false;
  walletState.isLocked = true;
  walletState.pendingRequests = [];

  await chrome.storage.local.remove([
    STORAGE_KEYS.AUTH_TOKEN,
    STORAGE_KEYS.AUTH_USER,
    STORAGE_KEYS.ACCOUNTS,
    STORAGE_KEYS.KEYSTORES,
  ]);

  return createResponse(true);
}

async function handleDetachWallet(): Promise<MessageResponse> {
  // Full reset: clear ALL local data and wallet state
  walletState = {
    isLocked: true,
    isInitialized: false,
    accounts: [],
    activeAccount: null,
    network: walletState.network, // Preserve network preference
    pendingRequests: [],
    auth: { isAuthenticated: false, token: null, user: null },
  };

  // Clear all extension storage
  await chrome.storage.local.clear();

  return createResponse(true);
}

// --- Missing Handlers ---

async function handleClaimStarterTokens(payload: { address: string }): Promise<MessageResponse> {
  try {
    const result = await rpcHandler.claimStarter(payload.address);
    return createResponse(true, result);
  } catch (error) {
    return createResponse(false, undefined, `Failed to claim: ${(error as Error).message}`);
  }
}

async function handleExportPrivateKey(payload: { password: string }): Promise<MessageResponse> {
  try {
    const keystores = await getKeystores();
    if (keystores.length === 0) {
      return createResponse(false, undefined, 'No keystores found');
    }

    // Decrypt the active account's keystore
    const activeAddress = walletState.activeAccount;
    const targetKeystore = activeAddress
      ? keystores.find(k => k.address === activeAddress) || keystores[0]
      : keystores[0];

    const privateKey = await keyring.decryptPrivateKey(targetKeystore, payload.password);
    const privateKeyHex = Array.from(privateKey).map(b => b.toString(16).padStart(2, '0')).join('');

    // Wipe after returning
    setTimeout(() => privateKey.fill(0), 0);

    return createResponse(true, { privateKey: privateKeyHex });
  } catch (error) {
    return createResponse(false, undefined, 'Invalid password');
  }
}

// --- Content Capture Handlers ---

async function handleSaveNote(payload: SaveNotePayload): Promise<MessageResponse> {
  const stored = await chrome.storage.local.get(STORAGE_KEYS.NOTES);
  const notes: SavedNote[] = stored[STORAGE_KEYS.NOTES] || [];

  const note: SavedNote = {
    id: crypto.randomUUID(),
    title: payload.title,
    content: payload.content,
    url: payload.url,
    tags: payload.tags || [],
    createdAt: Date.now(),
    updatedAt: Date.now(),
  };

  notes.unshift(note);
  await chrome.storage.local.set({ [STORAGE_KEYS.NOTES]: notes });
  return createResponse(true, note);
}

async function handleGetNotes(): Promise<MessageResponse> {
  const stored = await chrome.storage.local.get(STORAGE_KEYS.NOTES);
  return createResponse(true, stored[STORAGE_KEYS.NOTES] || []);
}

async function handleDeleteNote(payload: { id: string }): Promise<MessageResponse> {
  const stored = await chrome.storage.local.get(STORAGE_KEYS.NOTES);
  const notes: SavedNote[] = stored[STORAGE_KEYS.NOTES] || [];
  const filtered = notes.filter(n => n.id !== payload.id);
  await chrome.storage.local.set({ [STORAGE_KEYS.NOTES]: filtered });
  return createResponse(true);
}

async function handleSaveMedia(payload: SaveMediaPayload): Promise<MessageResponse> {
  const stored = await chrome.storage.local.get(STORAGE_KEYS.MEDIA);
  const media: SavedMedia[] = stored[STORAGE_KEYS.MEDIA] || [];

  const item: SavedMedia = {
    id: crypto.randomUUID(),
    url: payload.url,
    title: payload.title,
    sourceUrl: payload.sourceUrl,
    type: payload.type,
    createdAt: Date.now(),
  };

  media.unshift(item);
  await chrome.storage.local.set({ [STORAGE_KEYS.MEDIA]: media });
  return createResponse(true, item);
}

async function handleGetMedia(): Promise<MessageResponse> {
  const stored = await chrome.storage.local.get(STORAGE_KEYS.MEDIA);
  return createResponse(true, stored[STORAGE_KEYS.MEDIA] || []);
}

// --- Page Analysis ---

// Phase 4 handlers (handleAnalyzePage, handleMintSummaryNFT) removed.
// VYB Chat is now handled directly via the side panel UI calling the hub API.

// --- Sophia AI query handler ---
async function handleSophiaQuery(payload: SophiaQueryPayload): Promise<MessageResponse> {
  try {
    const hubUrl = getHubUrl();
    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (walletState.auth.token) {
      headers['Authorization'] = `Bearer ${walletState.auth.token}`;
    }

    // Build messages array with conversation history
    const messages = payload.conversationHistory
      ? [...payload.conversationHistory, { role: 'user', content: payload.message }]
      : [{ role: 'user', content: payload.message }];

    const response = await fetch(`${hubUrl}/api/sophia/chat`, {
      method: 'POST',
      headers,
      body: JSON.stringify({
        messages,
        enableTools: true,
        walletContext: {
          activeAccount: walletState.activeAccount,
          network: walletState.network,
          isLocked: walletState.isLocked,
          ...(payload.context || {}),
        },
      }),
    });

    if (!response.ok) {
      return createResponse(false, undefined, `Sophia API returned ${response.status}`);
    }

    const data = await response.json();

    return createResponse(true, {
      text: data.text || 'Sophia did not return a response.',
      toolsUsed: data.toolsUsed || 0,
    });
  } catch (error) {
    return createResponse(false, undefined, `Failed to reach Sophia: ${(error as Error).message}`);
  }
}

// Listen for messages
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  handleMessage(message, sender).then(sendResponse);
  return true; // Keep channel open for async response
});

// Listen for external messages (from content scripts)
chrome.runtime.onMessageExternal.addListener((message, sender, sendResponse) => {
  handleMessage({ ...message, origin: sender.origin }, sender).then(sendResponse);
  return true;
});

// Initialize on load
initializeState().then(() => {
  console.log('Demiurge Wallet: Background service worker initialized');
});

// Handle extension install/update
chrome.runtime.onInstalled.addListener((details) => {
  if (details.reason === 'install') {
    console.log('Demiurge Wallet: Extension installed');
  } else if (details.reason === 'update') {
    console.log('Demiurge Wallet: Extension updated');
  }

  // Register context menus for content capture
  chrome.contextMenus?.create({
    id: 'demiurge-save-text',
    title: 'Save to Demiurge',
    contexts: ['selection'],
  });

  chrome.contextMenus?.create({
    id: 'demiurge-save-link',
    title: 'Save Link to Demiurge',
    contexts: ['link'],
  });

  chrome.contextMenus?.create({
    id: 'demiurge-save-image',
    title: 'Save Image to Demiurge',
    contexts: ['image'],
  });
});

// Handle context menu clicks
chrome.contextMenus?.onClicked.addListener(async (info, tab) => {
  const sourceUrl = tab?.url || info.pageUrl || '';
  const pageTitle = tab?.title || '';

  if (info.menuItemId === 'demiurge-save-text' && info.selectionText) {
    await handleSaveNote({
      title: `Note from ${pageTitle}`.slice(0, 100),
      content: info.selectionText,
      url: sourceUrl,
      tags: [],
    });
    console.log('Demiurge: Text saved');
  } else if (info.menuItemId === 'demiurge-save-link' && info.linkUrl) {
    await handleSaveMedia({
      url: info.linkUrl,
      title: info.linkUrl,
      sourceUrl,
      type: 'link',
    });
    console.log('Demiurge: Link saved');
  } else if (info.menuItemId === 'demiurge-save-image' && info.srcUrl) {
    await handleSaveMedia({
      url: info.srcUrl,
      title: pageTitle,
      sourceUrl,
      type: 'image',
    });
    console.log('Demiurge: Image saved');
  }
});
