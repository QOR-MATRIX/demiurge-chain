// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Transaction Builder - Create and sign blockchain transactions
 */

import { getKeypairForQorId } from './qor-wallet';
import { User } from '@demiurge/qor-sdk';
import { KeyringPair } from '@polkadot/keyring/types';
import { u8aToHex, hexToU8a } from '@polkadot/util';
import { blake2AsU8a } from '@polkadot/util-crypto';

export interface TransactionParams {
  from: string;
  nonce: number;
  data: TransactionData;
}

export type TransactionData =
  | { type: 'transfer'; to: string; amount: string }
  | { type: 'moduleCall'; module: string; call: Uint8Array }
  | { type: 'drc369Mint'; metadata: any }
  | { type: 'agentDeploy'; config: any };

export interface SignedTransaction {
  transaction: Uint8Array;
  signature: Uint8Array;
  hash: string;
}

/**
 * Build a transfer transaction
 */
export function buildTransferTransaction(
  from: string,
  to: string,
  amount: string,
  nonce: number
): Uint8Array {
  // Encode transaction as SCALE codec
  const encoder = new TextEncoder();
  
  // Simple encoding: [type][from][to][amount][nonce]
  // Type 0 = Transfer
  const txData: number[] = [0];
  
  // From address (32 bytes)
  const fromBytes = hexToU8a(from);
  txData.push(...Array.from(fromBytes).slice(0, 32));
  
  // To address (32 bytes)
  const toBytes = hexToU8a(to);
  txData.push(...Array.from(toBytes).slice(0, 32));
  
  // Amount (u128 as 16 bytes)
  const amountBigInt = BigInt(amount);
  const amountBytes = new Uint8Array(16);
  for (let i = 0; i < 16; i++) {
    amountBytes[i] = Number((amountBigInt >> BigInt(i * 8)) & BigInt(0xFF));
  }
  txData.push(...Array.from(amountBytes));
  
  // Nonce (u64 as 8 bytes)
  const nonceBytes = new Uint8Array(8);
  for (let i = 0; i < 8; i++) {
    nonceBytes[i] = Number((nonce >> (i * 8)) & 0xFF);
  }
  txData.push(...Array.from(nonceBytes));
  
  return new Uint8Array(txData);
}

/**
 * Build a DRC-369 mint transaction
 */
export function buildMintTransaction(
  from: string,
  metadata: any,
  nonce: number
): Uint8Array {
  // Type 1 = Module Call (DRC369)
  const txData: number[] = [1];
  
  // From address
  const fromBytes = hexToU8a(from);
  txData.push(...Array.from(fromBytes).slice(0, 32));
  
  // Module name: "drc369"
  const moduleBytes = new TextEncoder().encode('drc369');
  txData.push(moduleBytes.length, ...Array.from(moduleBytes));
  
  // Call data: mint function
  const callData = new TextEncoder().encode(JSON.stringify({
    function: 'mint',
    metadata,
  }));
  txData.push(callData.length, ...Array.from(callData));
  
  // Nonce
  const nonceBytes = new Uint8Array(8);
  for (let i = 0; i < 8; i++) {
    nonceBytes[i] = Number((nonce >> (i * 8)) & 0xFF);
  }
  txData.push(...Array.from(nonceBytes));
  
  return new Uint8Array(txData);
}

/**
 * Build an agent deployment transaction
 */
export function buildAgentDeployTransaction(
  from: string,
  agentConfig: any,
  nonce: number
): Uint8Array {
  // Type 1 = Module Call (Agentic)
  const txData: number[] = [1];
  
  // From address
  const fromBytes = hexToU8a(from);
  txData.push(...Array.from(fromBytes).slice(0, 32));
  
  // Module name: "agentic"
  const moduleBytes = new TextEncoder().encode('agentic');
  txData.push(moduleBytes.length, ...Array.from(moduleBytes));
  
  // Call data: deploy_agent function
  const callData = new TextEncoder().encode(JSON.stringify({
    function: 'deploy_agent',
    config: agentConfig,
  }));
  txData.push(callData.length, ...Array.from(callData));
  
  // Nonce
  const nonceBytes = new Uint8Array(8);
  for (let i = 0; i < 8; i++) {
    nonceBytes[i] = Number((nonce >> (i * 8)) & 0xFF);
  }
  txData.push(...Array.from(nonceBytes));
  
  return new Uint8Array(txData);
}

/**
 * Sign a transaction using QOR ID keypair
 */
export function signTransaction(
  user: User,
  txBytes: Uint8Array
): SignedTransaction {
  try {
    const keypair = getKeypairForQorId(user.qor_id);
    
    // Sign the transaction
    const signature = keypair.sign(txBytes);
    
    // Calculate transaction hash
    const hashInput = new Uint8Array(signature.length + txBytes.length);
    hashInput.set(signature);
    hashInput.set(txBytes, signature.length);
    
    const hash = blake2bHash(hashInput);
    
    return {
      transaction: txBytes,
      signature,
      hash: u8aToHex(hash),
    };
  } catch (error) {
    console.error('Transaction signing failed:', error);
    throw new Error('Failed to sign transaction. Please ensure you are logged in with a valid QOR ID.');
  }
}

/**
 * Blake2b hash function using Polkadot util-crypto
 */
function blake2bHash(data: Uint8Array): Uint8Array {
  return blake2AsU8a(data, 256);
}

/**
 * Encode signed transaction for RPC submission
 */
export function encodeSignedTransaction(signed: SignedTransaction): string {
  // Combine signature + transaction for submission
  const encoded = new Uint8Array(signed.signature.length + signed.transaction.length);
  encoded.set(signed.signature);
  encoded.set(signed.transaction, signed.signature.length);
  
  return u8aToHex(encoded);
}

/**
 * Get next nonce for an address
 * 
 * TODO: Query from blockchain
 * For now, use timestamp-based nonce
 */
export async function getNextNonce(address: string): Promise<number> {
  // In production, query: await client.getNonce(address)
  // For now, use timestamp as pseudo-nonce
  return Date.now() % 1000000;
}

/**
 * Estimate transaction fee
 */
export async function estimateFee(txData: TransactionData): Promise<{ energy: number; cgt: number }> {
  // Demiurge uses Energy system for feeless transactions
  // Most transactions cost 1-5 energy
  let energy = 1;
  
  switch (txData.type) {
    case 'transfer':
      energy = 1;
      break;
    case 'drc369Mint':
      energy = 5;
      break;
    case 'agentDeploy':
      energy = 10;
      break;
    case 'moduleCall':
      energy = 2;
      break;
  }
  
  return {
    energy,
    cgt: 0, // Feeless - only energy cost
  };
}

/**
 * Submit signed transaction to RPC
 */
export async function submitSignedTransaction(
  signedTx: string,
  rpcUrl: string = process.env.NEXT_PUBLIC_RPC_URL || 'http://127.0.0.1:9944'
): Promise<string> {
  const response = await fetch(rpcUrl, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      jsonrpc: '2.0',
      method: 'author_submitExtrinsic',
      params: [signedTx],
      id: Date.now(),
    }),
  });
  
  const data = await response.json();
  
  if (data.error) {
    throw new Error(data.error.message || 'Transaction submission failed');
  }
  
  return data.result; // Returns transaction hash
}

/**
 * High-level: Build, sign, and submit a transfer
 */
export async function sendCGT(
  user: User,
  toAddress: string,
  amountCGT: string,
  rpcUrl?: string
): Promise<string> {
  // Get address
  const { getOrCreateAddressForQorId } = await import('./qor-wallet');
  const fromAddress = await getOrCreateAddressForQorId(user, false);
  
  // Get nonce
  const nonce = await getNextNonce(fromAddress);
  
  // Convert CGT to Sparks (smallest unit)
  const amountSparks = (parseFloat(amountCGT) * 100).toString();
  
  // Build transaction
  const txBytes = buildTransferTransaction(fromAddress, toAddress, amountSparks, nonce);
  
  // Sign
  const signed = signTransaction(user, txBytes);
  
  // Encode for submission
  const encodedTx = encodeSignedTransaction(signed);
  
  // Submit to blockchain
  const txHash = await submitSignedTransaction(encodedTx, rpcUrl);
  
  return txHash;
}

/**
 * High-level: Mint DRC-369 NFT
 */
export async function mintDRC369(
  user: User,
  metadata: any,
  rpcUrl?: string
): Promise<string> {
  const { getOrCreateAddressForQorId } = await import('./qor-wallet');
  const fromAddress = await getOrCreateAddressForQorId(user, false);
  
  const nonce = await getNextNonce(fromAddress);
  
  const txBytes = buildMintTransaction(fromAddress, metadata, nonce);
  const signed = signTransaction(user, txBytes);
  const encodedTx = encodeSignedTransaction(signed);
  
  const txHash = await submitSignedTransaction(encodedTx, rpcUrl);
  
  return txHash;
}

/**
 * High-level: Deploy AI agent
 */
export async function deployAgent(
  user: User,
  agentConfig: any,
  rpcUrl?: string
): Promise<string> {
  const { getOrCreateAddressForQorId } = await import('./qor-wallet');
  const fromAddress = await getOrCreateAddressForQorId(user, false);
  
  const nonce = await getNextNonce(fromAddress);
  
  const txBytes = buildAgentDeployTransaction(fromAddress, agentConfig, nonce);
  const signed = signTransaction(user, txBytes);
  const encodedTx = encodeSignedTransaction(signed);
  
  const txHash = await submitSignedTransaction(encodedTx, rpcUrl);
  
  return txHash;
}
